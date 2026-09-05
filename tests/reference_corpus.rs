use std::fs;
use std::path::{Path, PathBuf};

use softbrush_ls::analysis::{Analysis, Severity, analyze};
use softbrush_ls::catalog::Dialect;

const FIXTURES: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/fixtures");

fn fixture_files(directory: &Path, extension: Option<&str>) -> Vec<PathBuf> {
    let mut pending = vec![directory.to_owned()];
    let mut files = Vec::new();
    while let Some(current) = pending.pop() {
        for entry in fs::read_dir(&current).expect("fixture corpus directory") {
            let entry = entry.expect("fixture corpus entry");
            let path = entry.path();
            if path.is_dir() {
                pending.push(path);
            } else if extension.is_none_or(|expected| {
                path.extension().and_then(|value| value.to_str()) == Some(expected)
            }) {
                files.push(path);
            }
        }
    }
    files.sort();
    assert!(
        !files.is_empty(),
        "expected fixtures below {}",
        directory.display()
    );
    files
}

fn analyze_fixture(path: &Path, dialect: Dialect) -> Analysis {
    let source = fs::read_to_string(path).expect("UTF-8 fixture source");
    analyze(&source, dialect)
}

fn assert_no_structural_errors(path: &Path, analysis: &Analysis) {
    assert!(
        analysis.syntax.errors.is_empty(),
        "tolerant scanner rejected {}: {:?}",
        path.display(),
        analysis.syntax.errors
    );
    assert_eq!(
        analysis.syntax.antlr_syntax_errors,
        0,
        "ANTLR parser rejected {}",
        path.display()
    );
}

fn assert_no_error_diagnostics(path: &Path, analysis: &Analysis) {
    assert!(
        analysis
            .diagnostics
            .iter()
            .all(|diagnostic| diagnostic.severity != Severity::Error),
        "{} produced an error diagnostic: {:?}",
        path.display(),
        analysis.diagnostics
    );
}

fn assert_accepted_corpus(directory: &Path, extension: Option<&str>, dialect: Dialect) {
    for path in fixture_files(directory, extension) {
        let analysis = analyze_fixture(&path, dialect);
        assert_no_structural_errors(&path, &analysis);
        assert_no_error_diagnostics(&path, &analysis);
    }
}

#[test]
fn accepts_supplied_valid_sdc_corpus() {
    assert_accepted_corpus(
        &Path::new(FIXTURES).join("sdc/valid"),
        Some("sdc"),
        Dialect::Sdc,
    );
}

#[test]
fn accepts_supplied_positive_xdc_corpus() {
    assert_accepted_corpus(
        &Path::new(FIXTURES).join("xdc/positive"),
        Some("xdc"),
        Dialect::Xdc,
    );
}

#[test]
fn accepts_representative_tcl_86_library_files() {
    assert_accepted_corpus(&Path::new(FIXTURES).join("tcl"), Some("tcl"), Dialect::Tcl);
}

#[test]
fn keeps_sdc_portability_edges_non_blocking() {
    assert_accepted_corpus(
        &Path::new(FIXTURES).join("sdc/edge"),
        Some("sdc"),
        Dialect::Sdc,
    );
}

#[test]
fn keeps_context_dependent_and_unsupported_xdc_non_blocking() {
    for category in ["context_dependent", "unsupported_sdc"] {
        assert_accepted_corpus(
            &Path::new(FIXTURES).join("xdc").join(category),
            Some("xdc"),
            Dialect::Xdc,
        );
    }
}

#[derive(Clone, Copy)]
struct ExpectedDiagnostic {
    relative_path: &'static str,
    code: &'static str,
    severity: Severity,
}

const SUPPORTED_NEGATIVE_DIAGNOSTICS: &[ExpectedDiagnostic] = &[
    ExpectedDiagnostic {
        relative_path: "sdc/invalid/continuation_has_trailing_space.sdc",
        code: "tcl-continuation-whitespace",
        severity: Severity::Warning,
    },
    ExpectedDiagnostic {
        relative_path: "sdc/invalid/multicycle_bad_option_spelling.sdc",
        code: "constraint-unknown-command",
        severity: Severity::Hint,
    },
    ExpectedDiagnostic {
        relative_path: "sdc/invalid/non_constraint_runtime_command_in_sdc.sdc",
        code: "constraint-unknown-command",
        severity: Severity::Hint,
    },
    ExpectedDiagnostic {
        relative_path: "sdc/invalid/unclosed_collection_bracket.sdc",
        code: "tcl-syntax",
        severity: Severity::Error,
    },
    ExpectedDiagnostic {
        relative_path: "sdc/invalid/unknown_command_create_clock_typo.sdc",
        code: "constraint-unknown-command",
        severity: Severity::Hint,
    },
    ExpectedDiagnostic {
        relative_path: "xdc/negative/create_clock_missing_period.xdc",
        code: "constraint-missing-option",
        severity: Severity::Warning,
    },
    ExpectedDiagnostic {
        relative_path: "xdc/negative/create_clock_zero_period.xdc",
        code: "constraint-invalid-value",
        severity: Severity::Error,
    },
    ExpectedDiagnostic {
        relative_path: "xdc/negative/extra_close_brace.xdc",
        code: "tcl-syntax",
        severity: Severity::Error,
    },
    ExpectedDiagnostic {
        relative_path: "xdc/negative/unclosed_brace.xdc",
        code: "tcl-syntax",
        severity: Severity::Error,
    },
    ExpectedDiagnostic {
        relative_path: "xdc/negative/unclosed_command_substitution.xdc",
        code: "tcl-syntax",
        severity: Severity::Error,
    },
    ExpectedDiagnostic {
        relative_path: "xdc/negative/unclosed_quote.xdc",
        code: "tcl-syntax",
        severity: Severity::Error,
    },
    ExpectedDiagnostic {
        relative_path: "xdc/negative/unknown_command.xdc",
        code: "constraint-unknown-command",
        severity: Severity::Hint,
    },
];

fn assert_negative_corpus(directory: &Path, extension: &str, dialect: Dialect) {
    for path in fixture_files(directory, Some(extension)) {
        let relative = path
            .strip_prefix(FIXTURES)
            .expect("fixture path is below fixture root")
            .to_string_lossy();
        let analysis = analyze_fixture(&path, dialect);
        let expected = SUPPORTED_NEGATIVE_DIAGNOSTICS
            .iter()
            .find(|expected| expected.relative_path == relative);

        if let Some(expected) = expected {
            assert!(
                analysis.diagnostics.iter().any(|diagnostic| {
                    diagnostic.code == expected.code && diagnostic.severity == expected.severity
                }),
                "{} did not produce {} at {:?}: {:?}",
                path.display(),
                expected.code,
                expected.severity,
                analysis.diagnostics
            );
        }

        if expected.is_none_or(|expected| expected.code != "tcl-syntax") {
            assert_no_structural_errors(&path, &analysis);
        }

        // The remaining fixture comments describe vendor checks outside the current lint scope.
        // Keep those constructs tolerant until the analyzer supports a specific diagnostic.
        if expected.is_none_or(|expected| expected.severity != Severity::Error) {
            assert_no_error_diagnostics(&path, &analysis);
        }
    }
}

#[test]
fn exercises_all_negative_constraint_fixtures_at_supported_scope() {
    assert_negative_corpus(
        &Path::new(FIXTURES).join("sdc/invalid"),
        "sdc",
        Dialect::Sdc,
    );
    assert_negative_corpus(
        &Path::new(FIXTURES).join("xdc/negative"),
        "xdc",
        Dialect::Xdc,
    );
}
