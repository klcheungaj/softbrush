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
