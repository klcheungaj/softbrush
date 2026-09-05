//! Process tests for debug token inspection and its release-build gate.

use std::process::{Command, Output};

fn run(arguments: &[&str]) -> Output {
    Command::new(env!("CARGO_BIN_EXE_softbrush_ls"))
        .args(arguments)
        .output()
        .expect("run softbrush_ls")
}

#[test]
fn rejects_invalid_arguments_without_starting_lsp() {
    for args in [
        vec!["--not-an-option"],
        vec!["--stdio", "--not-an-option"],
        vec!["--stdio", "--stdio"],
        vec![
            "--stdio",
            "--dump-tokens",
            "tests/fixtures/sdc/edge/token_dump.sdc",
        ],
        vec!["--dump-tokens", "--stdio"],
        vec!["--dump-tokens"],
        vec!["--dump-tokens", "--"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(!output.stderr.is_empty());
    }
}

#[cfg(not(debug_assertions))]
#[test]
fn release_build_rejects_non_transport_arguments() {
    for args in [
        vec!["--dump-tokens", "tests/fixtures/sdc/edge/token_dump.sdc"],
        vec!["--stdio", "--not-an-option"],
    ] {
        let output = run(&args);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8_lossy(&output.stderr).contains("unknown argument"));
    }
}

#[cfg(debug_assertions)]
mod debug {
    use super::run;
    use std::path::{Path, PathBuf};

    const FIXTURE: &str = "tests/fixtures/sdc/edge/token_dump.sdc";

    fn dump(paths: &[&str]) -> String {
        let mut args = vec!["--dump-tokens", "--"];
        args.extend_from_slice(paths);
        let output = run(&args);
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(output.stderr.is_empty());
        String::from_utf8(output.stdout).expect("UTF-8 report")
    }

    #[test]
    fn dumps_every_lexer_and_semantic_byte_with_stable_coordinates() {
        let source = std::fs::read_to_string(FIXTURE).expect("fixture");
        let first = dump(&[FIXTURE]);
        assert_eq!(first, dump(&[FIXTURE]));
        for layer in ["lexer", "semantic"] {
            let mut end = 0;
            for row in first
                .lines()
                .filter(|row| row.starts_with(&format!("{layer}\t")))
            {
                let columns = row.split('\t').collect::<Vec<_>>();
                assert_eq!(columns.len(), 6);
                let (start, stop) = columns[1].split_once("..").expect("byte range");
                let start = start.parse::<usize>().expect("start");
                let stop = stop.parse::<usize>().expect("end");
                assert_eq!(start, end, "gaps and overlap must be explicit: {row}");
                assert!(stop >= start);
                assert_eq!(columns[5], format!("{:?}", &source[start..stop]));
                end = stop;
            }
            assert_eq!(end, source.len());
        }
        assert!(first.contains("\t2:35-2:41\tvariable\t-\t\"clk_😀\""));
        assert!(first.contains("\tkeyword\t-\t\"-clock_fall\""));
        assert!(first.contains("\tnumber\t-\t\"-1.25\""));
        assert!(first.contains("\tnumber\t-\t\"+.5\""));
        assert!(first.contains("\tnumber\t-\t\"-2e-3\""));
        assert!(first.contains("\tEOF\tchannel=0\t\"\""));
        for row in first.lines().filter(|row| row.starts_with("semantic\t")) {
            if row.contains("impostor") || row.contains("missing") {
                assert!(row.contains("\tunclassified\t"), "{row}");
            }
        }
    }

    #[test]
    fn supports_multiple_files_and_reports_source_errors() {
        let broken = "tests/fixtures/xdc/negative/unclosed_brace.xdc";
        let report = dump(&[FIXTURE, broken]);
        assert_eq!(report.matches("# softbrush.tokenDump/v1").count(), 2);
        assert!(report.contains("dialect=Sdc"));
        assert!(report.contains("dialect=Xdc"));
        assert!(report.contains("\ttcl-syntax\tError:"));
    }

    #[test]
    fn reports_usage_and_file_errors_on_stderr() {
        let help = run(&["--help"]);
        assert!(help.status.success());
        assert!(String::from_utf8_lossy(&help.stdout).contains("--dump-tokens"));
        for file in ["missing.sdc", "README.md", "--bad-option"] {
            let output = run(&["--dump-tokens", file]);
            assert_eq!(output.status.code(), Some(2));
            assert!(output.stdout.is_empty());
            assert!(!output.stderr.is_empty());
        }
    }

    struct TempDirectory(PathBuf);

    impl TempDirectory {
        fn new() -> Self {
            let path = std::env::temp_dir().join(format!("softbrush-dump-{}", std::process::id()));
            std::fs::create_dir(&path).expect("unique test directory");
            Self(path)
        }

        fn write(&self, name: &str, source: &[u8]) -> PathBuf {
            let path = self.0.join(name);
            std::fs::write(&path, source).expect("write test input");
            path
        }
    }

    impl Drop for TempDirectory {
        fn drop(&mut self) {
            std::fs::remove_dir_all(&self.0).expect("remove owned test directory");
        }
    }

    #[test]
    fn handles_empty_files_crlf_tabs_unicode_and_invalid_utf8() {
        let temp = TempDirectory::new();
        let empty = temp.write("empty.tcl", b"");
        let mixed = temp.write(
            "mixed name.SDC",
            "# 😀\r\nset n\t-1.5\r\nputs \"first\r\nsecond\"".as_bytes(),
        );
        let invalid = temp.write("invalid.xdc", &[0xff]);
        let as_str = |path: &Path| path.to_str().expect("test path is UTF-8").to_owned();
        let report = dump(&[&as_str(&empty), &as_str(&mixed)]);
        assert!(report.contains("lexer\t0..0\t0:0-0:0\tEOF"));
        assert!(report.contains("\t1:6-1:10\tnumber\t-\t\"-1.5\""));
        assert!(report.contains("\t2:5-2:11\tstring\t-\t\"\\\"first\""));
        assert!(report.contains("\t3:0-3:7\tstring\t-\t\"second\\\"\""));
        let output = run(&["--dump-tokens", &as_str(&invalid)]);
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
}
