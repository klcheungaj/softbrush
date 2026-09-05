use std::collections::HashSet;

use softbrush_ls::analysis::{SemanticKind, Severity, SymbolKind, analyze};
use softbrush_ls::catalog::Dialect;

#[test]
fn emits_every_semantic_highlight_category() {
    let source = concat!(
        "# model setup\n",
        "namespace eval chip {}\n",
        "proc scale {value {factor 2} args} {return value}\n",
        "set result 42\n",
        "puts \"computed\"\n",
        "custom_command -mode $result\n",
    );

    let analysis = analyze(source, Dialect::Tcl);
    let kinds = analysis
        .semantic_spans
        .iter()
        .map(|semantic| semantic.kind)
        .collect::<HashSet<_>>();

    assert_eq!(
        kinds,
        HashSet::from([
            SemanticKind::Comment,
            SemanticKind::String,
            SemanticKind::Number,
            SemanticKind::Variable,
            SemanticKind::Function,
            SemanticKind::Keyword,
            SemanticKind::Operator,
            SemanticKind::Parameter,
            SemanticKind::Namespace,
        ])
    );
    let parameters = analysis
        .semantic_spans
        .iter()
        .filter(|semantic| semantic.kind == SemanticKind::Parameter)
        .map(|semantic| &source[semantic.span.clone()])
        .collect::<Vec<_>>();
    assert_eq!(parameters, ["value", "factor", "args"]);
}

#[test]
fn preserves_quoted_strings_around_variable_substitutions_without_overlap() {
    let source = "puts \"computed $name\"\n";

    let analysis = analyze(source, Dialect::Tcl);
    let strings = analysis
        .semantic_spans
        .iter()
        .filter(|semantic| semantic.kind == SemanticKind::String)
        .map(|semantic| &source[semantic.span.clone()])
        .collect::<Vec<_>>();
    let variables = analysis
        .semantic_spans
        .iter()
        .filter(|semantic| semantic.kind == SemanticKind::Variable)
        .map(|semantic| &source[semantic.span.clone()])
        .collect::<Vec<_>>();

    assert_eq!(strings, ["\"computed ", "\""]);
    assert_eq!(variables, ["$name"]);
    assert!(
        analysis
            .semantic_spans
            .windows(2)
            .all(|pair| pair[0].span.end <= pair[1].span.start)
    );
}

#[test]
fn extracts_every_supported_symbol_category_without_treating_reads_as_declarations() {
    let source = concat!(
        "namespace eval constraints {}\n",
        "proc period_ns {mhz} {return 10}\n",
        "set assigned 10\n",
        "set assigned\n",
        "create_clock -name core_clk -period 10 [get_ports clk]\n",
        "create_pblock compute_region\n",
    );

    let analysis = analyze(source, Dialect::Xdc);
    let symbols = analysis
        .symbols
        .iter()
        .map(|symbol| (symbol.name.as_str(), symbol.kind))
        .collect::<Vec<_>>();

    assert!(symbols.contains(&("constraints", SymbolKind::Namespace)));
    assert!(symbols.contains(&("period_ns", SymbolKind::Function)));
    assert!(symbols.contains(&("assigned", SymbolKind::Variable)));
    assert!(symbols.contains(&("core_clk", SymbolKind::Clock)));
    assert!(symbols.contains(&("compute_region", SymbolKind::Object)));
    assert_eq!(
        symbols
            .iter()
            .filter(|(name, _)| *name == "assigned")
            .count(),
        1
    );
}

#[test]
fn reports_high_confidence_lints_and_keeps_catalog_misses_non_blocking() {
    let source = concat!(
        "proc incomplete {arg}\n",
        "create_clock -period 0 [get_ports clk]\n",
        "create_generated_clock -name divided [get_pins q]\n",
        "set_input_delay 2.0 [get_ports data]\n",
        "set_multicycle_path -setup\n",
        "vendor_extension -custom value\n",
    );

    let analysis = analyze(source, Dialect::Sdc);
    let codes = analysis
        .diagnostics
        .iter()
        .map(|diagnostic| diagnostic.code)
        .collect::<HashSet<_>>();

    assert!(codes.contains("tcl-arity"));
    assert!(codes.contains("constraint-invalid-value"));
    assert!(codes.contains("constraint-missing-option"));
    assert!(codes.contains("constraint-missing-value"));
    assert!(analysis.diagnostics.iter().any(|diagnostic| {
        diagnostic.code == "constraint-unknown-command" && diagnostic.severity == Severity::Hint
    }));
}

#[test]
fn rejects_non_positive_and_non_finite_clock_periods() {
    for period in ["0", "-0.0", "-1", "NaN", "inf", "-inf", "1e999"] {
        assert!(
            period.parse::<f64>().is_ok(),
            "test period must be accepted by Rust's numeric parser: {period}"
        );
        let analysis = analyze(
            &format!("create_clock -period {period} [get_ports clk]\n"),
            Dialect::Sdc,
        );

        assert!(
            analysis
                .diagnostics
                .iter()
                .any(|diagnostic| diagnostic.code == "constraint-invalid-value"),
            "expected `{period}` to be rejected"
        );
    }

    let analysis = analyze("create_clock -period 0.001 [get_ports clk]\n", Dialect::Sdc);
    assert!(
        analysis
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.code != "constraint-invalid-value")
    );
}

#[test]
fn selects_dialect_case_insensitively_from_file_extension() {
    assert_eq!(Dialect::from_uri_path("/work/top.TCL"), Dialect::Tcl);
    assert_eq!(Dialect::from_uri_path("/work/top.SDC"), Dialect::Sdc);
    assert_eq!(Dialect::from_uri_path("/work/top.XDC"), Dialect::Xdc);
}

#[test]
fn highlights_constraint_options_and_signed_numeric_values() {
    let source = concat!(
        "set_input_delay -clock_fall -clock sys_clk -max -1.25 [get_ports din]\n",
        "set_output_delay -clock sys_clk -min +0.5 [get_ports dout]\n",
        "vendor_constraint -vendor_option -2e-3\n",
    );

    for dialect in [Dialect::Sdc, Dialect::Xdc] {
        let analysis = analyze(source, dialect);
        let options = analysis
            .semantic_spans
            .iter()
            .filter(|semantic| semantic.kind == SemanticKind::Keyword && !semantic.declaration)
            .map(|semantic| &source[semantic.span.clone()])
            .collect::<Vec<_>>();
        let numbers = analysis
            .semantic_spans
            .iter()
            .filter(|semantic| semantic.kind == SemanticKind::Number)
            .map(|semantic| &source[semantic.span.clone()])
            .collect::<Vec<_>>();

        assert_eq!(
            options,
            [
                "-clock_fall",
                "-clock",
                "-max",
                "-clock",
                "-min",
                "-vendor_option"
            ]
        );
        assert_eq!(numbers, ["-1.25", "+0.5", "-2e-3"]);
    }
}

#[test]
fn highlights_only_clock_references_defined_earlier_in_the_document() {
    let source = concat!(
        "set_input_delay -clock future_clk -max 1 din\n",
        "create_clock -name base_clk -period 10\n",
        "set_input_delay -clock base_clk -min -0.5 din\n",
        "set_input_delay -clock { base_clk } -max 1 din\n",
        "create_generated_clock -name divided_clk -source pin -master_clock base_clk out\n",
        "set_output_delay -clock divided_clk -max 2 dout\n",
        "create_clock -period 8 direct_clk\n",
        "set_output_delay -clock direct_clk -min 1 dout\n",
        "create_clock -period 6 [get_ports inferred_clk]\n",
        "set_input_delay -clock inferred_clk -max 1 din\n",
        "set_input_delay -clock missing_clk -max 1 din\n",
    );

    for dialect in [Dialect::Sdc, Dialect::Xdc] {
        let analysis = analyze(source, dialect);
        let clock_tokens = analysis
            .semantic_spans
            .iter()
            .filter(|semantic| semantic.kind == SemanticKind::Variable)
            .map(|semantic| (&source[semantic.span.clone()], semantic.declaration))
            .collect::<Vec<_>>();

        assert_eq!(
            clock_tokens,
            [
                ("base_clk", true),
                ("base_clk", false),
                ("base_clk", false),
                ("divided_clk", true),
                ("base_clk", false),
                ("divided_clk", false),
                ("direct_clk", true),
                ("direct_clk", false),
                ("inferred_clk", true),
                ("inferred_clk", false),
            ]
        );
        assert!(
            !clock_tokens
                .iter()
                .any(|(text, _)| { matches!(*text, "future_clk" | "missing_clk") })
        );
    }
}

#[test]
fn does_not_invent_clock_definitions_or_color_unresolved_literal_names() {
    let source = concat!(
        "vendor_command -name fake\n",
        "create_clock -period 10\n",
        "create_clock -name -period 10\n",
        "create_clock -period 10 [get_ports]\n",
        "create_clock -period 10 [get_ports clk*]\n",
        "create_clock -period 10 [get_ports -filter fake]\n",
        "create_clock -vendor_option fake -period 10\n",
        "create_clock -name self -period 10 [get_clocks self]\n",
        "set_input_delay -clock fake -max -1 din\n",
        "set_input_delay -clock {missing} -min -2 din\n",
        "set_input_delay -clock \"missing\" -min -3 din\n",
        "set_input_delay -clock 10 -min -4 din\n",
        "set_input_delay -clock get_ports -min -5 din\n",
        "set_input_delay -clock clk* -min -6 din\n",
    );
    for dialect in [Dialect::Sdc, Dialect::Xdc] {
        let analysis = analyze(source, dialect);
        let clocks = analysis
            .symbols
            .iter()
            .filter(|symbol| symbol.kind == SymbolKind::Clock)
            .map(|symbol| symbol.name.as_str())
            .collect::<Vec<_>>();
        assert_eq!(clocks, ["self"]);
        assert!(
            !analysis
                .semantic_spans
                .iter()
                .any(|span| span.kind == SemanticKind::Variable && !span.declaration)
        );
        for name in ["{missing}", "\"missing\""] {
            let start = source.find(name).expect("fixture name");
            assert!(
                !analysis
                    .semantic_spans
                    .iter()
                    .any(|span| span.span.start < start + name.len() - 1
                        && span.span.end > start + 1)
            );
        }
    }
}

#[test]
fn distinguishes_clock_arguments_from_query_options_and_timing_values() {
    let source = concat!(
        "create_clock -name base -period 10\n",
        "get_clocks -filter base\n",
        "set_input_jitter base 0.25\n",
        "set_clock_latency -source -min -0.5 base\n",
        "set_clock_uncertainty -setup 0.125 base\n",
    );
    let analysis = analyze(source, Dialect::Xdc);
    let numbers = analysis
        .semantic_spans
        .iter()
        .filter(|span| span.kind == SemanticKind::Number)
        .map(|span| &source[span.span.clone()])
        .collect::<Vec<_>>();
    assert_eq!(numbers, ["10", "0.25", "-0.5", "0.125"]);
    let filter_start = source.find("-filter base").expect("filter") + "-filter ".len();
    assert!(
        !analysis
            .semantic_spans
            .iter()
            .any(|span| span.span.contains(&filter_start))
    );
    assert_eq!(
        analysis
            .semantic_spans
            .iter()
            .filter(|span| span.kind == SemanticKind::Variable && !span.declaration)
            .count(),
        3
    );
}

#[test]
fn separates_constraint_braces_from_contents_without_coloring_escaped_braces() {
    let source = "get_ports {din {nested} escaped\\{brace\\}}\r\nget_pins {😀\r\nq}\r\nget_ports {";
    for dialect in [Dialect::Sdc, Dialect::Xdc] {
        let analysis = analyze(source, dialect);
        let braces = analysis
            .semantic_spans
            .iter()
            .filter(|token| token.kind == SemanticKind::Operator)
            .map(|token| &source[token.span.clone()])
            .collect::<Vec<_>>();
        assert_eq!(braces, ["{", "{", "}", "}", "{", "}", "{"]);
        for text in ["din ", "nested", " escaped\\{brace\\}", "😀\r\nq"] {
            assert!(
                analysis
                    .semantic_spans
                    .iter()
                    .any(|token| token.kind == SemanticKind::String
                        && &source[token.span.clone()] == text)
            );
        }
        assert!(
            analysis
                .semantic_spans
                .windows(2)
                .all(|pair| pair[0].span.end <= pair[1].span.start)
        );
    }
}
