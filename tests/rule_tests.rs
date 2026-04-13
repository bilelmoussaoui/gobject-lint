use std::{fs, path::Path};

use globset::GlobSetBuilder;
use goblint::{ast_context::AstContext, config::Config, fixer, rules::Rule};

/// Build an AstContext from a single C file copied into a temp directory.
/// Returns the TempDir (must stay alive for the duration of the test).
fn build_context_for_file(c_file: &Path) -> (AstContext, tempfile::TempDir) {
    let temp_dir = tempfile::tempdir().expect("failed to create temp dir");
    let dest = temp_dir.path().join(c_file.file_name().unwrap());
    fs::copy(c_file, &dest).expect("failed to copy fixture");

    let ignore = GlobSetBuilder::new().build().unwrap();
    let ctx = AstContext::build_with_ignore(temp_dir.path(), &ignore, None)
        .expect("failed to build AstContext");

    (ctx, temp_dir)
}

/// Format violations as `filename:line:col: rule: message`, sorted.
fn format_violations(violations: &[goblint::rules::Violation], strip_prefix: &Path) -> String {
    let lines: Vec<String> = violations
        .iter()
        .map(|v| {
            let relative = v.file.strip_prefix(strip_prefix).unwrap_or(&v.file);
            format!(
                "{}:{}:{}: {}: {}",
                relative.display(),
                v.line,
                v.column,
                v.rule,
                v.message
            )
        })
        .collect();
    lines.join("\n")
}

/// Core fixture runner for a single rule.
///
/// - Iterates all `*.c` files in `tests/fixtures/<rule_name>/`
/// - Runs the rule, compares violations against `<stem>.stderr`
/// - If `<stem>.fixed.c` exists, applies fixes and compares the result
/// - If `<stem>.stderr` doesn't exist or `BLESS=1` is set, writes/updates it
fn run_fixture_tests(rule_name: &str, rule: &dyn Rule) {
    let fixtures_dir = Path::new("tests/fixtures").join(rule_name);
    if !fixtures_dir.exists() {
        return;
    }

    let mut c_files: Vec<_> = fs::read_dir(&fixtures_dir)
        .unwrap()
        .filter_map(|e| e.ok())
        .filter(|e| e.path().extension().is_some_and(|ext| ext == "c"))
        .map(|e| e.path())
        .collect();
    c_files.sort();

    let bless = std::env::var("BLESS").is_ok();
    let mut failures: Vec<String> = Vec::new();

    for c_file in c_files {
        let stem = c_file.file_stem().unwrap().to_str().unwrap().to_owned();
        let stderr_file = fixtures_dir.join(format!("{stem}.stderr"));
        let fixed_file = fixtures_dir.join(format!("{stem}.fixed.c"));

        // --- violation check ---
        let (ctx, temp_dir) = build_context_for_file(&c_file);
        let config = Config::default();

        let mut violations = Vec::new();
        rule.check_all(&ctx, &config, &mut violations);
        violations.sort_by_key(|v| (v.line, v.column));

        let actual_stderr = format_violations(&violations, temp_dir.path());

        if bless || !stderr_file.exists() {
            fs::write(&stderr_file, format!("{actual_stderr}\n")).expect("failed to write .stderr");
            if bless {
                println!("blessed {}", stderr_file.display());
            }
        } else {
            let expected = fs::read_to_string(&stderr_file).unwrap_or_default();
            if actual_stderr.trim() != expected.trim() {
                failures.push(format!(
                    "fixture {rule_name}/{stem}: violations mismatch\n\
                     --- expected ---\n{}\n--- got ---\n{}",
                    expected.trim(),
                    actual_stderr.trim(),
                ));
            }
        }

        // --- fix check ---
        if fixed_file.exists() {
            fixer::apply_fixes(&violations).expect("failed to apply fixes");

            let temp_c = temp_dir.path().join(c_file.file_name().unwrap());
            let actual_fixed = fs::read_to_string(&temp_c).expect("failed to read fixed file");
            let expected_fixed = fs::read_to_string(&fixed_file).expect("failed to read .fixed.c");

            if actual_fixed != expected_fixed {
                failures.push(format!(
                    "fixture {rule_name}/{stem}: fix output mismatch\n\
                     --- expected ---\n{}\n--- got ---\n{}",
                    expected_fixed.trim(),
                    actual_fixed.trim(),
                ));
            }
        }
    }

    if !failures.is_empty() {
        panic!("\n{}", failures.join("\n\n"));
    }
}

macro_rules! rule_test {
    ($rule_name:ident, $rule:expr) => {
        #[test]
        fn $rule_name() {
            run_fixture_tests(stringify!($rule_name), &$rule);
        }
    };
}

rule_test!(deprecated_add_private, goblint::rules::DeprecatedAddPrivate);
rule_test!(g_error_init, goblint::rules::GErrorInit);
rule_test!(unnecessary_null_check, goblint::rules::UnnecessaryNullCheck);
rule_test!(use_g_strcmp0, goblint::rules::UseGStrcmp0);
