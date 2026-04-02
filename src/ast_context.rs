use anyhow::Result;
use globset::GlobSet;
use gobject_ast::{FunctionInfo, GObjectTypeKind, Parser, Project};
use indicatif::ProgressBar;
use std::path::Path;
use walkdir::WalkDir;

/// AST-based project context that replaces the old tree-sitter based ProjectContext
pub struct AstContext {
    pub project: Project,
}

impl AstContext {
    /// Build with ignore patterns
    pub fn build_with_ignore(
        directory: &Path,
        ignore_matcher: &GlobSet,
        spinner: Option<&ProgressBar>,
    ) -> Result<Self> {
        let mut parser = Parser::new()?;
        let mut project = Project::new();

        // Collect all files first to get count
        let files: Vec<_> = WalkDir::new(directory)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .is_some_and(|ext| ext == "h" || ext == "c")
            })
            .filter(|e| {
                let path = e.path();
                let relative_path = path.strip_prefix(directory).unwrap_or(path);
                !ignore_matcher.is_match(relative_path)
            })
            .collect();

        let total_files = files.len();

        // Parse each file
        for (i, entry) in files.iter().enumerate() {
            let path = entry.path();

            if let Some(sp) = spinner {
                sp.set_message(format!("Parsing files... {}/{}", i + 1, total_files));
            }

            // Parse this file
            if let Ok(file_project) = parser.parse_file(path) {
                // Merge into main project
                for (file_path, file_model) in file_project.files {
                    project.files.insert(file_path, file_model);
                }
            }
        }

        Ok(Self { project })
    }

    /// Get the source text for an entire function
    pub fn get_function_source<'a>(
        &'a self,
        file_path: &Path,
        func: &FunctionInfo,
    ) -> Option<&'a [u8]> {
        let file = self.project.files.get(file_path)?;

        if let (Some(start), Some(end)) = (func.start_byte, func.end_byte) {
            Some(&file.source[start..end])
        } else {
            None
        }
    }

    /// Check if a function is declared in any header file
    pub fn is_declared_in_header(&self, func_name: &str) -> bool {
        for (path, file) in &self.project.files {
            if path.extension().is_some_and(|ext| ext == "h")
                && file.functions.iter().any(|f| f.name == func_name)
            {
                return true;
            }
        }
        false
    }

    /// Find all non-static functions that are NOT declared in headers and don't have export macros
    /// Returns (file_path, function_info) tuples
    /// These should either be:
    /// 1. Made static (if only used in one file)
    /// 2. Given an export macro (if part of public API)
    /// 3. Declared in a header (if part of internal API)
    pub fn find_undeclared_non_static_functions(&self) -> Vec<(&Path, &FunctionInfo)> {
        self.project
            .files
            .iter()
            .filter(|(path, _)| path.extension().is_some_and(|ext| ext == "c"))
            .flat_map(|(path, file)| {
                file.functions
                    .iter()
                    .filter(|f| f.is_definition)
                    .filter(|f| !f.is_static)
                    .filter(|f| f.export_macros.is_empty())
                    .filter(|f| !self.is_declared_in_header(&f.name))
                    .filter(|f| !is_special_function(&f.name))
                    .map(move |f| (path.as_path(), f))
            })
            .collect()
    }

    /// Check if a function is part of a GObject type definition
    /// (e.g., type registration functions, vfuncs, etc.)
    pub fn is_gobject_internal(&self, func_name: &str) -> bool {
        // Check if it's a _get_type function
        if func_name.ends_with("_get_type") {
            return true;
        }

        // Check if it's part of a GObject type's class_init, instance_init, etc.
        for file in self.project.files.values() {
            for gtype in &file.gobject_types {
                let prefix = match &gtype.kind {
                    GObjectTypeKind::DeclareFinal {
                        function_prefix, ..
                    }
                    | GObjectTypeKind::DeclareDerivable {
                        function_prefix, ..
                    }
                    | GObjectTypeKind::DeclareInterface {
                        function_prefix, ..
                    }
                    | GObjectTypeKind::DefineType {
                        function_prefix, ..
                    }
                    | GObjectTypeKind::DefineTypeWithPrivate {
                        function_prefix, ..
                    }
                    | GObjectTypeKind::DefineAbstractType {
                        function_prefix, ..
                    } => function_prefix,
                };

                // GObject generates functions like: prefix_class_init, prefix_init, prefix_finalize
                if func_name.starts_with(prefix) {
                    let suffix = &func_name[prefix.len()..];
                    if matches!(
                        suffix,
                        "_class_init"
                            | "_init"
                            | "_finalize"
                            | "_dispose"
                            | "_constructed"
                            | "_set_property"
                            | "_get_property"
                    ) {
                        return true;
                    }
                }
            }
        }

        false
    }
}

/// Check if a function name indicates it's a special function that shouldn't be linted
fn is_special_function(name: &str) -> bool {
    // main function
    if name == "main" {
        return true;
    }

    // GObject type registration functions
    if name.ends_with("_get_type") || name.ends_with("_error_quark") {
        return true;
    }

    // Common GObject lifecycle functions
    if name.ends_with("_class_init")
        || name.ends_with("_init")
        || name.ends_with("_finalize")
        || name.ends_with("_dispose")
        || name.ends_with("_constructed")
        || name.ends_with("_set_property")
        || name.ends_with("_get_property")
    {
        return true;
    }

    false
}
