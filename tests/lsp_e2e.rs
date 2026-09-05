use std::collections::VecDeque;
use std::io::{BufRead, BufReader, Read, Write};
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::mpsc::{self, Receiver};
use std::thread::{self, JoinHandle};
use std::time::{Duration, Instant};

use serde_json::{Value, json};

const TIMEOUT: Duration = Duration::from_secs(10);
const TCL_URI: &str = "file:///tmp/softbrush_e2e.tcl";
const SDC_URI: &str = "file:///tmp/softbrush_e2e.sdc";
const XDC_URI: &str = "file:///tmp/softbrush_e2e.xdc";
const SEMANTIC_SDC_URI: &str = "file:///tmp/softbrush_semantic.sdc";

const TCL_SOURCE: &str = concat!(
    "# 😀 heading\r\n",
    "namespace eval chip {}\r\n",
    "proc scale {value} {return value}\r\n",
    "set count 42\r\n",
    "puts \"😀 first\r\n",
    "second\"\r\n",
    "vendor_cmd -mode $count\r\n",
);
const SDC_SOURCE: &str = concat!(
    "create_generated_clock -name divided\r\n",
    "vendor_extension -custom value\r\n",
    "puts 😀 \"unterminated",
);
const VALID_SDC_SOURCE: &str = concat!(
    "create_generated_clock -source master -name divided\r\n",
    "create_clock -period 10\r\n",
);
const XDC_SOURCE: &str = concat!(
    "create_pblock region_😀\r\n",
    "create_clock -name sys_clk -period 10 [get_ports clk]\r\n",
);
const SEMANTIC_SDC_SOURCE: &str = concat!(
    "create_clock -name sys_clk -period 10\r\n",
    "set_input_delay -clock_fall -clock sys_clk -max -1.25 [get_ports din]\r\n",
    "set_output_delay -clock missing_clk -min +0.5 [get_ports dout]\r\n",
);

struct LspClient {
    child: Child,
    stdin: Option<ChildStdin>,
    messages: Receiver<Result<Value, String>>,
    pending: VecDeque<Value>,
    stderr_thread: Option<JoinHandle<Vec<u8>>>,
    next_id: u64,
}

impl LspClient {
    fn start() -> Self {
        let executable = std::env::var_os("SOFTBRUSH_LS_BIN")
            .unwrap_or_else(|| env!("CARGO_BIN_EXE_softbrush_ls").into());
        let mut child = Command::new(executable)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()
            .expect("start softbrush_ls");
        let stdout = child.stdout.take().expect("server stdout");
        let stderr = child.stderr.take().expect("server stderr");
        let (sender, messages) = mpsc::channel();
        thread::spawn(move || {
            let mut reader = BufReader::new(stdout);
            loop {
                match read_frame(&mut reader) {
                    Ok(Some(message)) => {
                        if sender.send(Ok(message)).is_err() {
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(error) => {
                        let _ = sender.send(Err(error));
                        break;
                    }
                }
            }
        });
        let stderr_thread = thread::spawn(move || {
            let mut output = Vec::new();
            let _ = BufReader::new(stderr).read_to_end(&mut output);
            output
        });
        Self {
            stdin: child.stdin.take(),
            child,
            messages,
            pending: VecDeque::new(),
            stderr_thread: Some(stderr_thread),
            next_id: 1,
        }
    }

    fn notify(&mut self, method: &str, params: &Value) {
        self.send(&json!({"jsonrpc": "2.0", "method": method, "params": params}));
    }

    fn request(&mut self, method: &str, params: &Value) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method, "params": params}));
        self.wait_for(|message| message.get("id").and_then(Value::as_u64) == Some(id))
    }

    fn request_without_params(&mut self, method: &str) -> Value {
        let id = self.next_id;
        self.next_id += 1;
        self.send(&json!({"jsonrpc": "2.0", "id": id, "method": method}));
        self.wait_for(|message| message.get("id").and_then(Value::as_u64) == Some(id))
    }

    fn wait_for_notification(&mut self, method: &str, uri: &str) -> Value {
        self.wait_for(|message| {
            message.get("method").and_then(Value::as_str) == Some(method)
                && message.pointer("/params/uri").and_then(Value::as_str) == Some(uri)
        })
    }

    fn wait_for(&mut self, predicate: impl Fn(&Value) -> bool) -> Value {
        if let Some(index) = self.pending.iter().position(&predicate) {
            return self.pending.remove(index).expect("pending message index");
        }
        let deadline = Instant::now() + TIMEOUT;
        loop {
            let remaining = deadline.saturating_duration_since(Instant::now());
            let message = self
                .messages
                .recv_timeout(remaining)
                .expect("timed out waiting for server message")
                .expect("read valid server frame");
            if predicate(&message) {
                return message;
            }
            self.pending.push_back(message);
        }
    }

    fn send(&mut self, message: &Value) {
        let body = serde_json::to_vec(message).expect("serialize request");
        let stdin = self.stdin.as_mut().expect("server stdin is open");
        write!(stdin, "Content-Length: {}\r\n\r\n", body.len()).expect("write frame header");
        stdin.write_all(&body).expect("write frame body");
        stdin.flush().expect("flush frame");
    }

    fn shutdown(mut self) {
        let response = self.request_without_params("shutdown");
        assert_eq!(response.get("result"), Some(&Value::Null));
        self.send(&json!({"jsonrpc": "2.0", "method": "exit"}));
        drop(self.stdin.take());

        let deadline = Instant::now() + TIMEOUT;
        loop {
            if let Some(status) = self.child.try_wait().expect("query server status") {
                let stderr = self.join_stderr();
                assert!(
                    status.success(),
                    "softbrush_ls exited with {status}; stderr: {stderr}"
                );
                return;
            }
            assert!(Instant::now() < deadline, "softbrush_ls did not exit");
            thread::sleep(Duration::from_millis(10));
        }
    }

    fn join_stderr(&mut self) -> String {
        let output = self
            .stderr_thread
            .take()
            .expect("stderr drain thread")
            .join()
            .expect("join stderr drain thread");
        String::from_utf8_lossy(&output).into_owned()
    }
}

impl Drop for LspClient {
    fn drop(&mut self) {
        if self.child.try_wait().ok().flatten().is_none() {
            let _ = self.child.kill();
            let _ = self.child.wait();
        }
        if let Some(stderr_thread) = self.stderr_thread.take() {
            let _ = stderr_thread.join();
        }
    }
}

fn read_frame(reader: &mut impl BufRead) -> Result<Option<Value>, String> {
    let mut content_length = None;
    let mut line = String::new();
    loop {
        line.clear();
        let count = reader
            .read_line(&mut line)
            .map_err(|error| error.to_string())?;
        if count == 0 {
            return Ok(None);
        }
        if line == "\r\n" {
            break;
        }
        if let Some(value) = line.strip_prefix("Content-Length:") {
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|error| error.to_string())?,
            );
        }
    }
    let length = content_length.ok_or("missing Content-Length header")?;
    let mut body = vec![0; length];
    reader
        .read_exact(&mut body)
        .map_err(|error| error.to_string())?;
    serde_json::from_slice(&body)
        .map(Some)
        .map_err(|error| error.to_string())
}

fn initialize(client: &mut LspClient) -> Value {
    let response = client.request(
        "initialize",
        &json!({
            "processId": null,
            "rootUri": null,
            "capabilities": {"general": {"positionEncodings": ["utf-16"]}}
        }),
    );
    client.notify("initialized", &json!({}));
    response
}

fn assert_initialize_capabilities(initialized: &Value) {
    let capabilities = initialized
        .pointer("/result/capabilities")
        .expect("initialize capabilities");
    assert_eq!(capabilities.get("positionEncoding"), Some(&json!("utf-16")));
    assert_eq!(capabilities.get("textDocumentSync"), Some(&json!(1)));
    assert_eq!(
        capabilities.get("semanticTokensProvider"),
        Some(&json!({
            "legend": {
                "tokenTypes": [
                    "comment", "string", "number", "variable", "function", "keyword",
                    "operator", "parameter", "namespace"
                ],
                "tokenModifiers": ["declaration"]
            },
            "full": true
        }))
    );
    assert_eq!(
        capabilities.get("documentSymbolProvider"),
        Some(&json!(true))
    );
    assert_eq!(
        capabilities.get("workspaceSymbolProvider"),
        Some(&json!(true))
    );
    assert_eq!(capabilities.get("hoverProvider"), Some(&json!(true)));
    assert_eq!(
        capabilities.get("completionProvider"),
        Some(&json!({"triggerCharacters": ["-"]}))
    );
    assert_eq!(
        initialized.pointer("/result/serverInfo/name"),
        Some(&json!("softbrush_ls"))
    );
}

fn open_document(client: &mut LspClient, uri: &str, language_id: &str, text: &str) -> Value {
    client.notify(
        "textDocument/didOpen",
        &json!({
            "textDocument": {
                "uri": uri,
                "languageId": language_id,
                "version": 1,
                "text": text
            }
        }),
    );
    client.wait_for_notification("textDocument/publishDiagnostics", uri)
}

fn diagnostics(message: &Value) -> &[Value] {
    message
        .pointer("/params/diagnostics")
        .and_then(Value::as_array)
        .map(Vec::as_slice)
        .expect("diagnostic array")
}

fn assert_diagnostic_protocol(client: &mut LspClient) {
    let published = open_document(client, SDC_URI, "sdc", SDC_SOURCE);
    let summaries = diagnostics(&published)
        .iter()
        .map(|diagnostic| {
            json!({
                "code": diagnostic.get("code"),
                "severity": diagnostic.get("severity"),
                "range": diagnostic.get("range"),
                "source": diagnostic.get("source")
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(
        summaries,
        vec![
            json!({
                "code": "constraint-missing-option",
                "severity": 2,
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 36}
                },
                "source": "softbrush_ls"
            }),
            json!({
                "code": "constraint-unknown-command",
                "severity": 4,
                "range": {
                    "start": {"line": 1, "character": 0},
                    "end": {"line": 1, "character": 16}
                },
                "source": "softbrush_ls"
            }),
            json!({
                "code": "tcl-syntax",
                "severity": 1,
                "range": {
                    "start": {"line": 2, "character": 8},
                    "end": {"line": 2, "character": 9}
                },
                "source": "softbrush_ls"
            }),
        ]
    );
}

#[derive(Debug, Eq, PartialEq)]
struct SemanticToken {
    line: u32,
    start: u32,
    length: u32,
    token_type: u32,
    modifiers: u32,
}

fn decode_semantic_tokens(response: &Value) -> Vec<SemanticToken> {
    let data = response
        .pointer("/result/data")
        .and_then(Value::as_array)
        .expect("semantic token data");
    assert_eq!(data.len() % 5, 0, "semantic tokens use five integers");

    let mut line = 0;
    let mut start = 0;
    data.as_chunks::<5>()
        .0
        .iter()
        .map(|encoded| {
            let delta_line = semantic_token_u32(&encoded[0], "delta line");
            let delta_start = semantic_token_u32(&encoded[1], "delta start");
            line += delta_line;
            start = if delta_line == 0 {
                start + delta_start
            } else {
                delta_start
            };
            SemanticToken {
                line,
                start,
                length: semantic_token_u32(&encoded[2], "token length"),
                token_type: semantic_token_u32(&encoded[3], "token type"),
                modifiers: semantic_token_u32(&encoded[4], "token modifiers"),
            }
        })
        .collect()
}

fn semantic_token_u32(value: &Value, field: &str) -> u32 {
    u32::try_from(value.as_u64().unwrap_or_else(|| panic!("missing {field}")))
        .unwrap_or_else(|_| panic!("{field} exceeds u32"))
}

fn utf16_byte_offset(text: &str, target: u32) -> usize {
    let mut units = 0;
    for (offset, character) in text.char_indices() {
        if units == target {
            return offset;
        }
        units += u32::try_from(character.len_utf16()).expect("character UTF-16 length fits u32");
        assert!(units <= target, "UTF-16 position splits a character");
    }
    assert_eq!(units, target, "UTF-16 position exceeds line length");
    text.len()
}

fn token_text(source: &str, token: &SemanticToken) -> String {
    let line_with_cr = source
        .split('\n')
        .nth(token.line as usize)
        .expect("semantic token line");
    let line = line_with_cr.strip_suffix('\r').unwrap_or(line_with_cr);
    let start = utf16_byte_offset(line, token.start);
    let end = utf16_byte_offset(line, token.start + token.length);
    line[start..end].to_owned()
}

fn assert_semantic_tokens(client: &mut LspClient) {
    let published = open_document(client, TCL_URI, "tcl", TCL_SOURCE);
    assert!(diagnostics(&published).is_empty());
    let response = client.request(
        "textDocument/semanticTokens/full",
        &json!({"textDocument": {"uri": TCL_URI}}),
    );
    let tokens = decode_semantic_tokens(&response);
    let observed = tokens
        .iter()
        .map(|token| {
            (
                token.line,
                token.start,
                token.length,
                token.token_type,
                token.modifiers,
                token_text(TCL_SOURCE, token),
            )
        })
        .collect::<Vec<_>>();
    assert_eq!(
        observed,
        vec![
            (0, 0, 12, 0, 0, "# 😀 heading".to_owned()),
            (1, 0, 9, 5, 0, "namespace".to_owned()),
            (1, 15, 4, 8, 1, "chip".to_owned()),
            (1, 20, 2, 1, 0, "{}".to_owned()),
            (2, 0, 4, 5, 0, "proc".to_owned()),
            (2, 5, 5, 4, 1, "scale".to_owned()),
            (2, 12, 5, 7, 1, "value".to_owned()),
            (2, 19, 14, 1, 0, "{return value}".to_owned()),
            (3, 0, 3, 4, 0, "set".to_owned()),
            (3, 4, 5, 3, 1, "count".to_owned()),
            (3, 10, 2, 2, 0, "42".to_owned()),
            (4, 0, 4, 4, 0, "puts".to_owned()),
            (4, 5, 9, 1, 0, "\"😀 first".to_owned()),
            (5, 0, 7, 1, 0, "second\"".to_owned()),
            (6, 0, 10, 4, 0, "vendor_cmd".to_owned()),
            (6, 11, 5, 6, 0, "-mode".to_owned()),
            (6, 17, 6, 3, 0, "$count".to_owned()),
        ]
    );
}

fn assert_constraint_semantic_tokens(client: &mut LspClient) {
    let published = open_document(client, SEMANTIC_SDC_URI, "sdc", SEMANTIC_SDC_SOURCE);
    assert!(diagnostics(&published).is_empty());
    let response = client.request(
        "textDocument/semanticTokens/full",
        &json!({"textDocument": {"uri": SEMANTIC_SDC_URI}}),
    );
    let observed = decode_semantic_tokens(&response)
        .iter()
        .map(|token| {
            (
                token.line,
                token.token_type,
                token.modifiers,
                token_text(SEMANTIC_SDC_SOURCE, token),
            )
        })
        .collect::<Vec<_>>();

    assert_eq!(
        observed,
        [
            (0, 4, 0, "create_clock".to_owned()),
            (0, 7, 0, "-name".to_owned()),
            (0, 3, 1, "sys_clk".to_owned()),
            (0, 7, 0, "-period".to_owned()),
            (0, 2, 0, "10".to_owned()),
            (1, 4, 0, "set_input_delay".to_owned()),
            (1, 7, 0, "-clock_fall".to_owned()),
            (1, 7, 0, "-clock".to_owned()),
            (1, 3, 0, "sys_clk".to_owned()),
            (1, 7, 0, "-max".to_owned()),
            (1, 2, 0, "-1.25".to_owned()),
            (1, 4, 0, "get_ports".to_owned()),
            (2, 4, 0, "set_output_delay".to_owned()),
            (2, 7, 0, "-clock".to_owned()),
            (2, 7, 0, "-min".to_owned()),
            (2, 2, 0, "+0.5".to_owned()),
            (2, 4, 0, "get_ports".to_owned()),
        ]
    );
    close_document(client, SEMANTIC_SDC_URI);
}

fn assert_document_symbols(client: &mut LspClient, uri: &str, expected: &[Value]) {
    let response = client.request(
        "textDocument/documentSymbol",
        &json!({"textDocument": {"uri": uri}}),
    );
    let observed = response
        .get("result")
        .and_then(Value::as_array)
        .expect("document symbols")
        .iter()
        .map(|symbol| {
            json!({
                "name": symbol.get("name"),
                "detail": symbol.get("detail"),
                "kind": symbol.get("kind"),
                "range": symbol.get("range"),
                "selectionRange": symbol.get("selectionRange")
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(observed, expected);
}

fn assert_symbol_providers(client: &mut LspClient) {
    assert_document_symbols(
        client,
        TCL_URI,
        &[
            json!({
                "name": "chip", "detail": "Tcl namespace", "kind": 3,
                "range": {
                    "start": {"line": 1, "character": 0},
                    "end": {"line": 1, "character": 22}
                },
                "selectionRange": {
                    "start": {"line": 1, "character": 15},
                    "end": {"line": 1, "character": 19}
                }
            }),
            json!({
                "name": "scale", "detail": "Tcl procedure", "kind": 12,
                "range": {
                    "start": {"line": 2, "character": 0},
                    "end": {"line": 2, "character": 33}
                },
                "selectionRange": {
                    "start": {"line": 2, "character": 5},
                    "end": {"line": 2, "character": 10}
                }
            }),
            json!({
                "name": "count", "detail": "Tcl variable", "kind": 13,
                "range": {
                    "start": {"line": 3, "character": 0},
                    "end": {"line": 3, "character": 12}
                },
                "selectionRange": {
                    "start": {"line": 3, "character": 4},
                    "end": {"line": 3, "character": 9}
                }
            }),
        ],
    );

    let published = open_document(client, XDC_URI, "xdc", XDC_SOURCE);
    assert!(diagnostics(&published).is_empty());
    assert_document_symbols(
        client,
        XDC_URI,
        &[
            json!({
                "name": "region_😀", "detail": "XDC object", "kind": 19,
                "range": {
                    "start": {"line": 0, "character": 0},
                    "end": {"line": 0, "character": 23}
                },
                "selectionRange": {
                    "start": {"line": 0, "character": 14},
                    "end": {"line": 0, "character": 23}
                }
            }),
            json!({
                "name": "sys_clk", "detail": "Timing clock", "kind": 19,
                "range": {
                    "start": {"line": 1, "character": 0},
                    "end": {"line": 1, "character": 53}
                },
                "selectionRange": {
                    "start": {"line": 1, "character": 19},
                    "end": {"line": 1, "character": 26}
                }
            }),
        ],
    );

    let workspace_symbols = client.request("workspace/symbol", &json!({"query": "region_"}));
    assert_eq!(
        workspace_symbols.get("result"),
        Some(&json!([{
            "name": "region_😀",
            "kind": 19,
            "location": {
                "uri": XDC_URI,
                "range": {
                    "start": {"line": 0, "character": 14},
                    "end": {"line": 0, "character": 23}
                }
            },
            "containerName": "XDC object"
        }]))
    );
}

fn assert_hover_and_completion(client: &mut LspClient) {
    let hover = client.request(
        "textDocument/hover",
        &json!({
            "textDocument": {"uri": XDC_URI},
            "position": {"line": 1, "character": 2}
        }),
    );
    assert_eq!(
        hover.pointer("/result/contents"),
        Some(&json!({
            "kind": "markdown",
            "value": "`create_clock`\n\nCreate a primary or virtual timing clock."
        }))
    );
    assert_eq!(
        hover.pointer("/result/range"),
        Some(&json!({
            "start": {"line": 1, "character": 0},
            "end": {"line": 1, "character": 12}
        }))
    );

    let completion = client.request(
        "textDocument/completion",
        &json!({
            "textDocument": {"uri": XDC_URI},
            "position": {"line": 1, "character": 27}
        }),
    );
    let completion_labels = completion
        .get("result")
        .and_then(Value::as_array)
        .expect("completion items")
        .iter()
        .filter_map(|item| item.get("label").and_then(Value::as_str))
        .collect::<Vec<_>>();
    assert!(completion_labels.contains(&"create_clock"));
    assert!(completion_labels.contains(&"set_property"));
}

fn close_document(client: &mut LspClient, uri: &str) {
    client.notify(
        "textDocument/didClose",
        &json!({"textDocument": {"uri": uri}}),
    );
    let published = client.wait_for_notification("textDocument/publishDiagnostics", uri);
    assert!(diagnostics(&published).is_empty());
}

fn assert_change_and_close_lifecycle(client: &mut LspClient) {
    client.notify(
        "textDocument/didChange",
        &json!({
            "textDocument": {"uri": SDC_URI, "version": 2},
            "contentChanges": [{"text": VALID_SDC_SOURCE}]
        }),
    );
    let corrected = client.wait_for_notification("textDocument/publishDiagnostics", SDC_URI);
    assert!(diagnostics(&corrected).is_empty());

    close_document(client, XDC_URI);
    let workspace_symbols = client.request("workspace/symbol", &json!({"query": "region_"}));
    assert_eq!(workspace_symbols.get("result"), Some(&json!([])));

    close_document(client, TCL_URI);
    let after_close = client.request(
        "textDocument/semanticTokens/full",
        &json!({"textDocument": {"uri": TCL_URI}}),
    );
    assert_eq!(after_close.get("result"), Some(&Value::Null));
    close_document(client, SDC_URI);
}

#[test]
fn validates_every_advertised_lsp_feature_over_stdio() {
    let mut client = LspClient::start();
    let initialized = initialize(&mut client);
    assert_initialize_capabilities(&initialized);
    assert_semantic_tokens(&mut client);
    assert_constraint_semantic_tokens(&mut client);
    assert_diagnostic_protocol(&mut client);
    assert_symbol_providers(&mut client);
    assert_hover_and_completion(&mut client);
    assert_change_and_close_lifecycle(&mut client);
    client.shutdown();
}
