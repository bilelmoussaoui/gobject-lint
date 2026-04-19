use super::{Fix, Rule};
use crate::{ast_context::AstContext, config::Config, rules::Violation};

pub struct UsePragmaOnce;

impl Rule for UsePragmaOnce {
    fn name(&self) -> &'static str {
        "use_pragma_once"
    }

    fn description(&self) -> &'static str {
        "Suggest #pragma once instead of traditional include guards"
    }

    fn category(&self) -> super::Category {
        super::Category::Style
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
        use gobject_ast::top_level::{PreprocessorDirective, TopLevelItem};

        for (path, file) in ast_context.iter_header_files() {
            // Check if the file already uses #pragma once
            let has_pragma_once = file.top_level_items.iter().any(|item| {
                matches!(
                    item,
                    TopLevelItem::Preprocessor(PreprocessorDirective::Pragma {
                        kind: gobject_ast::top_level::PragmaKind::Once,
                        ..
                    })
                )
            });

            if has_pragma_once {
                continue; // Already using #pragma once
            }

            // Look for traditional include guard pattern
            if let Some((conditional_loc, define_loc, guard_name)) =
                self.find_include_guard(&file.top_level_items)
            {
                // Build fixes:
                // 1. Replace #ifndef and #define lines with #pragma once
                // 2. Remove the #endif line (including any comment) at the end
                let mut fixes = Vec::new();

                // Fix 1: Replace from start of #ifndef to end of #define line
                // Note: find_line_bounds includes preceding blank lines, but we want
                // to preserve them, so find the actual start of the #ifndef line
                let ifndef_actual_start =
                    self.find_actual_line_start(&file.source, conditional_loc.start_byte);
                let (_, define_line_end) = define_loc.find_line_bounds(&file.source);

                fixes.push(Fix::new(
                    ifndef_actual_start,
                    define_line_end,
                    "#pragma once\n".to_string(),
                ));

                // Fix 2: Remove the entire #endif line including any trailing comment
                let (endif_line_start, endif_line_end) =
                    self.find_endif_line_with_comment(&file.source, conditional_loc.end_byte);
                fixes.push(Fix::new(endif_line_start, endif_line_end, String::new()));

                violations.push(self.violation_with_fixes(
                    path,
                    conditional_loc.line,
                    conditional_loc.column,
                    format!("Use #pragma once instead of include guard '{}'", guard_name),
                    fixes,
                ));
            }
        }
    }
}

impl UsePragmaOnce {
    /// Find traditional include guard pattern
    /// Returns (conditional_location, define_location, guard_name)
    fn find_include_guard(
        &self,
        items: &[gobject_ast::top_level::TopLevelItem],
    ) -> Option<(
        gobject_ast::SourceLocation,
        gobject_ast::SourceLocation,
        String,
    )> {
        use gobject_ast::top_level::{ConditionalKind, PreprocessorDirective, TopLevelItem};

        // The first item should be #ifndef (traditional include guard)
        items.first().and_then(|item| match item {
            TopLevelItem::Preprocessor(PreprocessorDirective::Conditional {
                kind: ConditionalKind::Ifndef,
                condition: Some(name),
                body,
                location,
            }) => {
                // Found #ifndef - check it contains matching #define as first item
                let define_loc = self.find_matching_define(body, name)?;
                Some((*location, define_loc, name.clone()))
            }
            _ => None, // First item is not #ifndef
        })
    }

    /// Find matching #define inside the #ifndef body
    /// Returns the define location only if it's a guard (no value) and there's
    /// content after it
    fn find_matching_define(
        &self,
        body: &[gobject_ast::top_level::TopLevelItem],
        guard_name: &str,
    ) -> Option<gobject_ast::SourceLocation> {
        use gobject_ast::top_level::{PreprocessorDirective, TopLevelItem};

        // Need at least 2 items: the #define and some actual content
        if body.len() < 2 {
            return None;
        }

        // First item in body should be #define with matching name and NO value
        // (include guards are #define GUARD_NAME with no value)
        body.first().and_then(|item| match item {
            TopLevelItem::Preprocessor(PreprocessorDirective::Define {
                name,
                value,
                location,
            }) => {
                if name == guard_name && value.is_none() {
                    Some(*location)
                } else {
                    None
                }
            }
            _ => None,
        })
    }

    /// Find the actual start of a line without including preceding blank lines
    /// This is different from find_line_bounds which includes them
    fn find_actual_line_start(&self, source: &[u8], pos: usize) -> usize {
        let mut line_start = pos;
        // Go back to the start of this line (after the previous \n)
        while line_start > 0 && source[line_start - 1] != b'\n' {
            line_start -= 1;
        }
        line_start
    }

    /// Find the #endif line including any trailing comment and preceding blank
    /// lines Returns (line_start, line_end) byte positions
    fn find_endif_line_with_comment(
        &self,
        source: &[u8],
        conditional_end: usize,
    ) -> (usize, usize) {
        // The conditional_end is right after the #endif word
        // Search backwards to find the start of the #endif line
        let mut line_start = conditional_end;
        while line_start > 0 && source[line_start - 1] != b'\n' {
            line_start -= 1;
        }

        // Also remove any blank lines before #endif
        while line_start >= 2 && source[line_start - 1] == b'\n' && source[line_start - 2] == b'\n'
        {
            line_start -= 1;
        }

        // Search forward from conditional_end to find the actual end of line
        // (to include any trailing comment like /* GUARD_NAME */)
        let mut line_end = conditional_end;
        while line_end < source.len() && source[line_end] != b'\n' {
            line_end += 1;
        }

        // Include the newline
        if line_end < source.len() && source[line_end] == b'\n' {
            line_end += 1;
        }

        (line_start, line_end)
    }
}
