use tree_sitter::Node;

use super::{CheckContext, Fix, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct GParamSpecStaticStrings;

impl Rule for GParamSpecStaticStrings {
    fn name(&self) -> &'static str {
        "g_param_spec_static_strings"
    }

    fn description(&self) -> &'static str {
        "Ensure g_param_spec_* calls use G_PARAM_STATIC_STRINGS flag for string literals"
    }

    fn category(&self) -> super::Category {
        super::Category::Perf
    }

    fn fixable(&self) -> bool {
        true
    }

    fn check_all(
        &self,
        ast_context: &AstContext,
        _config: &Config,
        violations: &mut Vec<Violation>,
    ) {
        for (path, file) in ast_context.iter_c_files() {
            for func in &file.functions {
                if !func.is_definition {
                    continue;
                }

                if let Some(func_source) = ast_context.get_function_source(path, func)
                    && let Some(tree) = ast_context.parse_c_source(func_source)
                {
                    let ctx = CheckContext {
                        source: func_source,
                        file_path: path,
                        base_line: func.line,
                        base_byte: func.start_byte.unwrap_or(0),
                    };
                    self.check_node(ast_context, tree.root_node(), &ctx, violations);
                }
            }
        }
    }
}

impl GParamSpecStaticStrings {
    fn check_node(
        &self,
        ast_context: &AstContext,
        node: Node,
        ctx: &CheckContext,
        violations: &mut Vec<Violation>,
    ) {
        // Look for g_param_spec_* calls
        if node.kind() == "call_expression"
            && let Some(function) = node.child_by_field_name("function")
        {
            let func_name = ast_context.get_node_text(function, ctx.source);

            if func_name.starts_with("g_param_spec_")
                && func_name != "g_param_spec_override"
                && func_name != "g_param_spec_internal"
            {
                // Check if this g_param_spec call has string literals and missing
                // G_PARAM_STATIC_STRINGS
                if let Some((
                    flags_arg,
                    flags_arg_text,
                    is_satisfied,
                    nick_is_literal,
                    blurb_is_literal,
                )) = self.check_param_spec_flags(ast_context, node, ctx.source)
                    && !is_satisfied
                {
                    let needed = self.needed_flags(nick_is_literal, blurb_is_literal);
                    let fix = Fix::from_node(
                        flags_arg,
                        ctx,
                        self.build_fixed_flags(flags_arg_text, nick_is_literal, blurb_is_literal),
                    );

                    violations.push(self.violation_with_fix(
                        ctx.file_path,
                        ctx.base_line + node.start_position().row,
                        node.start_position().column + 1,
                        format!(
                            "Add {} to {} flags (saves memory for static strings)",
                            needed, func_name
                        ),
                        fix,
                    ));
                }
            }
        }

        // Recurse
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.check_node(ast_context, child, ctx, violations);
        }
    }

    /// Check if g_param_spec_* has string literals and whether it already has
    /// the minimal required static-string flags.
    ///
    /// Returns `(flags_node, flags_text, is_satisfied, nick_is_literal,
    /// blurb_is_literal)`.
    fn check_param_spec_flags<'a>(
        &self,
        ast_context: &AstContext,
        call_node: Node<'a>,
        source: &'a [u8],
    ) -> Option<(Node<'a>, &'a str, bool, bool, bool)> {
        let args = call_node.child_by_field_name("arguments")?;

        let mut cursor = args.walk();
        let arguments: Vec<Node> = args
            .children(&mut cursor)
            .filter(|c| c.kind() != "(" && c.kind() != ")" && c.kind() != ",")
            .collect();

        // g_param_spec_*(name, nick, blurb, ..., flags) — need at least 4 args
        if arguments.len() < 4 {
            return None;
        }

        let name_is_literal = arguments[0].kind() == "string_literal";
        let nick_is_literal = arguments[1].kind() == "string_literal";
        let blurb_is_literal = arguments[2].kind() == "string_literal";

        let nick_text = ast_context.get_node_text(arguments[1], source);
        let blurb_text = ast_context.get_node_text(arguments[2], source);

        // Only check when name is a string literal and nick/blurb are literals or NULL
        if !name_is_literal
            || (!nick_is_literal && !ast_context.is_null_literal(nick_text))
            || (!blurb_is_literal && !ast_context.is_null_literal(blurb_text))
        {
            return None;
        }

        let flags_arg = *arguments.last()?;
        let flags_text = ast_context.get_node_text(flags_arg, source);

        let has_static_strings = flags_text.contains("G_PARAM_STATIC_STRINGS");
        let has_static_name = flags_text.contains("G_PARAM_STATIC_NAME");
        let has_static_nick = flags_text.contains("G_PARAM_STATIC_NICK");
        let has_static_blurb = flags_text.contains("G_PARAM_STATIC_BLURB");

        // Is the minimal required set of static flags already present?
        let is_satisfied = if has_static_strings {
            // G_PARAM_STATIC_STRINGS covers everything — always satisfied
            true
        } else if nick_is_literal && blurb_is_literal {
            // All three strings are literals — need NAME + NICK + BLURB
            has_static_name && has_static_nick && has_static_blurb
        } else if nick_is_literal {
            has_static_name && has_static_nick
        } else if blurb_is_literal {
            has_static_name && has_static_blurb
        } else {
            // nick and blurb are NULL — only the name needs the static flag
            has_static_name
        };

        Some((
            flags_arg,
            flags_text,
            is_satisfied,
            nick_is_literal,
            blurb_is_literal,
        ))
    }

    /// Return the flag expression that should be added, given which args are
    /// literals.
    fn needed_flags(&self, nick_is_literal: bool, blurb_is_literal: bool) -> &'static str {
        match (nick_is_literal, blurb_is_literal) {
            (true, true) => "G_PARAM_STATIC_STRINGS",
            (true, false) => "G_PARAM_STATIC_NAME | G_PARAM_STATIC_NICK",
            (false, true) => "G_PARAM_STATIC_NAME | G_PARAM_STATIC_BLURB",
            (false, false) => "G_PARAM_STATIC_NAME",
        }
    }

    /// Build the replacement flags string: remove any individual static flags
    /// already present, then append the minimal required ones.
    fn build_fixed_flags(
        &self,
        flags_text: &str,
        nick_is_literal: bool,
        blurb_is_literal: bool,
    ) -> String {
        const INDIVIDUAL: &[&str] = &[
            "G_PARAM_STATIC_NAME",
            "G_PARAM_STATIC_NICK",
            "G_PARAM_STATIC_BLURB",
            "G_PARAM_STATIC_STRINGS",
        ];

        // Strip existing static flags; keep everything else.
        let mut parts: Vec<&str> = if flags_text.is_empty() || flags_text == "0" {
            Vec::new()
        } else {
            flags_text
                .split('|')
                .map(|s| s.trim())
                .filter(|s| !INDIVIDUAL.contains(s))
                .collect()
        };

        // Append the minimal needed flags.
        match (nick_is_literal, blurb_is_literal) {
            (true, true) => parts.push("G_PARAM_STATIC_STRINGS"),
            (true, false) => {
                parts.push("G_PARAM_STATIC_NAME");
                parts.push("G_PARAM_STATIC_NICK");
            }
            (false, true) => {
                parts.push("G_PARAM_STATIC_NAME");
                parts.push("G_PARAM_STATIC_BLURB");
            }
            (false, false) => parts.push("G_PARAM_STATIC_NAME"),
        }

        parts.join(" | ")
    }
}
