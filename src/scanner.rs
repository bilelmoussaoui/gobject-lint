use crate::ast_context::AstContext;
use crate::config::{Config, RuleConfig};
use crate::rules::chainup::DisposeFinalizeChainsUp;
use crate::rules::deprecated_add_private::DeprecatedAddPrivate;
use crate::rules::g_param_spec::GParamSpecNullNickBlurb;
use crate::rules::gdeclare_semicolon::GDeclareSemicolon;
use crate::rules::gerror_init::GErrorInit;
use crate::rules::gtask_source_tag::GTaskSourceTag;
use crate::rules::missing_implementation::MissingImplementation;
use crate::rules::property_enum_zero::PropertyEnumZero;
use crate::rules::unnecessary_null_check::UnnecessaryNullCheck;
use crate::rules::use_clear_functions::UseClearFunctions;
use crate::rules::use_g_strcmp0::UseGStrcmp0;
use crate::rules::Violation;
use anyhow::Result;
use indicatif::ProgressBar;
use std::path::Path;

/// Filter violations based on per-rule ignore patterns
fn filter_violations(
    violations: Vec<Violation>,
    config: &Config,
    rule_config: &RuleConfig,
) -> Result<Vec<Violation>> {
    let ignore_matcher = config.build_rule_ignore_matcher(rule_config)?;

    Ok(violations
        .into_iter()
        .filter(|v| {
            let path = Path::new(&v.file);
            !ignore_matcher.is_match(path)
        })
        .collect())
}

/// New AST-based scanner - much simpler than the old one!
pub fn scan_with_ast(
    ast_context: &AstContext,
    config: &Config,
    spinner: Option<&ProgressBar>,
) -> Result<Vec<Violation>> {
    let mut violations = Vec::new();

    if let Some(sp) = spinner {
        sp.set_message("Running linter rules...");
    }

    // Run G_DECLARE semicolon checks
    if config.rules.gdeclare_semicolon.enabled {
        let rule = GDeclareSemicolon;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.gdeclare_semicolon,
        )?);
    }

    // Run missing implementation checks
    if config.rules.missing_implementation.enabled {
        let rule = MissingImplementation;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.missing_implementation,
        )?);
    }

    // Run deprecated API checks
    if config.rules.deprecated_add_private.enabled {
        let rule = DeprecatedAddPrivate;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.deprecated_add_private,
        )?);
    }

    // Run string comparison checks
    if config.rules.use_g_strcmp0.enabled {
        let rule = UseGStrcmp0;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.use_g_strcmp0,
        )?);
    }

    // Run g_param_spec checks
    if config.rules.g_param_spec_null_nick_blurb.enabled {
        let rule = GParamSpecNullNickBlurb;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.g_param_spec_null_nick_blurb,
        )?);
    }

    // Run GError initialization checks
    if config.rules.gerror_init.enabled {
        let rule = GErrorInit;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.gerror_init,
        )?);
    }

    // Run property enum checks
    if config.rules.property_enum_zero.enabled {
        let rule = PropertyEnumZero;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.property_enum_zero,
        )?);
    }

    // Run dispose/finalize chain-up checks
    if config.rules.dispose_finalize_chains_up.enabled {
        let rule = DisposeFinalizeChainsUp;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.dispose_finalize_chains_up,
        )?);
    }

    // Run GTask source tag checks
    if config.rules.gtask_source_tag.enabled {
        let rule = GTaskSourceTag;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.gtask_source_tag,
        )?);
    }

    // Run unnecessary NULL check detection
    if config.rules.unnecessary_null_check.enabled {
        let rule = UnnecessaryNullCheck;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.unnecessary_null_check,
        )?);
    }

    // Run use clear functions checks
    if config.rules.use_clear_functions.enabled {
        let rule = UseClearFunctions;
        let rule_violations = rule.check_all(ast_context, config);
        violations.extend(filter_violations(
            rule_violations,
            config,
            &config.rules.use_clear_functions,
        )?);
    }

    Ok(violations)
}
