use std::{
    collections::{HashMap, HashSet},
    fs,
    path::Path,
};

use anyhow::{Context, Result};
use tree_sitter::{Node, Parser as TSParser};
use walkdir::WalkDir;

use crate::model::*;

pub struct Parser {
    parser: TSParser,
}

impl Parser {
    pub fn new() -> Result<Self> {
        let mut parser = TSParser::new();
        parser
            .set_language(&tree_sitter_c::LANGUAGE.into())
            .context("Failed to load C grammar")?;

        Ok(Self { parser })
    }

    /// Helper to create SourceLocation from a tree-sitter Node
    fn node_location(&self, node: Node) -> SourceLocation {
        SourceLocation::new(
            node.start_position().row + 1,
            node.start_position().column + 1,
            node.start_byte(),
            node.end_byte(),
        )
    }

    pub fn parse_directory(&mut self, path: &Path) -> Result<Project> {
        let mut project = Project::new();

        // Parse all files (.h and .c)
        for entry in WalkDir::new(path)
            .into_iter()
            .filter_map(|e| e.ok())
            .filter(|e| {
                e.path()
                    .extension()
                    .map_or(false, |ext| ext == "h" || ext == "c")
            })
        {
            self.parse_single_file(entry.path(), &mut project)?;
        }

        Ok(project)
    }

    pub fn parse_file(&mut self, path: &Path) -> Result<Project> {
        let mut project = Project::new();
        self.parse_single_file(path, &mut project)?;
        Ok(project)
    }

    fn parse_single_file(&mut self, path: &Path, project: &mut Project) -> Result<()> {
        let source = fs::read(path)?;
        let tree = self
            .parser
            .parse(&source, None)
            .context("Failed to parse file")?;

        let mut file_model = FileModel::new(path.to_path_buf());

        // Build macro map for export macros
        let macro_map = self.build_macro_map(tree.root_node(), &source);

        // Find static forward declarations (for .c files)
        let static_forwards = self.find_static_forward_declarations(tree.root_node(), &source);

        // Extract all content from this file
        self.extract_file_content(
            tree.root_node(),
            &source,
            &mut file_model,
            &macro_map,
            &static_forwards,
        );

        // Second pass: extract class structs for derivable types
        self.extract_class_structs(tree.root_node(), &source, &mut file_model);

        // Third pass: extract class structs from source text (for cases where
        // tree-sitter misparsed)
        self.extract_class_structs_from_text(&source, &mut file_model);

        // Store the source for detailed pattern matching by rules
        file_model.source = source;

        project.files.insert(path.to_path_buf(), file_model);
        Ok(())
    }

    fn extract_file_content<'a>(
        &self,
        node: Node,
        source: &'a [u8],
        file_model: &mut FileModel,
        macro_map: &HashMap<usize, Vec<&'a str>>,
        static_forwards: &HashSet<&'a str>,
    ) {
        self.visit_node(node, source, file_model, macro_map, static_forwards);
    }

    fn find_export_macros_in_declaration<'a>(
        &self,
        decl_node: Node,
        source: &'a [u8],
    ) -> Vec<&'a str> {
        let mut result = Vec::new();

        // The declaration node includes export macros when they're on the line before
        // Get the first line of the declaration
        let decl_start = decl_node.start_byte();
        let mut first_line_end = decl_start;
        while first_line_end < source.len() && source[first_line_end] != b'\n' {
            first_line_end += 1;
        }

        // Get the first line text
        if let Ok(first_line) = std::str::from_utf8(&source[decl_start..first_line_end]) {
            // Look for export macros in the first line
            for word in first_line.split_whitespace() {
                if word.ends_with("_EXPORT")
                    || word.starts_with("G_DEPRECATED")
                    || word == "G_GNUC_DEPRECATED"
                    || word == "G_GNUC_WARN_UNUSED_RESULT"
                {
                    result.push(word);
                    break; // Only take the first one
                }
            }
        }

        result
    }

    fn build_macro_map<'a>(&self, root: Node, source: &'a [u8]) -> HashMap<usize, Vec<&'a str>> {
        let mut map = HashMap::new();
        self.build_macro_map_recursive(root, source, &mut map);
        map
    }

    fn build_macro_map_recursive<'a>(
        &self,
        node: Node,
        source: &'a [u8],
        map: &mut HashMap<usize, Vec<&'a str>>,
    ) {
        // Check for preprocessor directives like #define
        if node.kind() == "preproc_call" {
            if let Some(directive) = node.child_by_field_name("directive") {
                let text = &source[directive.byte_range()];
                if let Ok(s) = std::str::from_utf8(text) {
                    if s.ends_with("_EXPORT")
                        || s.starts_with("G_DEPRECATED")
                        || s.starts_with("G_MODULE_")
                        || s == "G_GNUC_DEPRECATED"
                        || s == "G_GNUC_WARN_UNUSED_RESULT"
                    {
                        // Add to next line (the declaration)
                        map.entry(node.end_position().row + 1)
                            .or_insert_with(Vec::new)
                            .push(s);
                    }
                }
            }
        }
        // For declarations, check if they have export macros before them
        else if node.kind() == "declaration" {
            let decl_line = node.start_position().row;

            // Look for export macros by checking the source text before the declaration
            let export_macros = self.find_export_macros_in_declaration(node, source);

            if !export_macros.is_empty() {
                map.entry(decl_line)
                    .or_insert_with(Vec::new)
                    .extend(export_macros);
            }
        }

        // Recurse into children
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.build_macro_map_recursive(child, source, map);
        }
    }

    fn visit_node<'a>(
        &self,
        node: Node,
        source: &'a [u8],
        file_model: &mut FileModel,
        macro_map: &HashMap<usize, Vec<&'a str>>,
        static_forwards: &HashSet<&'a str>,
    ) {
        // Extract GObject type declarations (G_DECLARE_* macros) before skipping
        // preproc
        if node.kind() == "preproc_call" {
            if let Some(gobject_type) = self.extract_gobject_type_declaration(node, source) {
                file_model.gobject_types.push(gobject_type);
            }
        }

        // Extract includes before skipping preproc
        if node.kind() == "preproc_include" {
            if let Some(include) = self.extract_include(node, source) {
                file_model.includes.push(include);
            }
        }

        // Skip preprocessor macro definitions and includes, but traverse conditional
        // blocks
        if node.kind() == "preproc_def"
            || node.kind() == "preproc_function_def"
            || node.kind() == "preproc_call"
            || node.kind() == "preproc_include"
        {
            return;
        }

        // Extract typedefs (type_definition nodes)
        if node.kind() == "type_definition" {
            if let Some(typedef) = self.extract_typedef_from_type_definition(node, source) {
                file_model.typedefs.push(typedef);
            }
            // Also check for typedef enums
            if let Some(enum_info) = self.extract_enum(node, source) {
                file_model.enums.push(enum_info);
            }
        }

        // Extract GObject types from identifier pattern (handles ERROR nodes from
        // macros)
        if node.kind() == "identifier" {
            let text = std::str::from_utf8(&source[node.byte_range()]).unwrap_or("");
            if text.starts_with("G_DECLARE_") || text.starts_with("G_DEFINE_") {
                // Found a GObject type macro, look for parent to get arguments
                if let Some(parent) = node.parent() {
                    if let Some(gobject_type) =
                        self.extract_gobject_from_identifier(node, parent, source, text)
                    {
                        file_model.gobject_types.push(gobject_type);
                    }
                }
            }
        }

        // Extract structs directly from struct_specifier nodes
        if node.kind() == "struct_specifier" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(name) = std::str::from_utf8(&source[name_node.byte_range()]) {
                    let has_body = node.child_by_field_name("body").is_some();
                    file_model.structs.push(StructInfo {
                        name: name.to_owned(),
                        line: node.start_position().row + 1,
                        fields: Vec::new(),
                        is_opaque: !has_body,
                    });
                }
            }
        }

        // Look for declarations and definitions
        if node.kind() == "declaration" || node.kind() == "expression_statement" {
            // Get export macros for this line from the macro map
            let export_macros = macro_map
                .get(&node.start_position().row)
                .cloned()
                .unwrap_or_default();

            // Extract structs (this may find some, but struct_specifier above catches more)
            if let Some(struct_info) = self.extract_struct(node, source) {
                file_model.structs.push(struct_info);
            }

            // Extract enums
            if let Some(enum_info) = self.extract_enum(node, source) {
                file_model.enums.push(enum_info);
            }

            // Extract function declarations
            let mut func_names = Vec::new();
            self.find_all_function_names(node, source, &mut func_names);

            // Check if this declaration has 'static' storage class
            let is_static = self.has_static_storage_class(node, source);

            for func_name in func_names {
                if !is_macro_identifier(&func_name) && !is_gobject_type_macro(&func_name) {
                    file_model.functions.push(FunctionInfo {
                        name: func_name.to_owned(),
                        line: node.start_position().row + 1,
                        is_static,
                        export_macros: export_macros.iter().map(|s| s.to_string()).collect(),
                        has_static_forward_decl: static_forwards.contains(func_name),
                        is_definition: false,
                        return_type: None,
                        parameters: Vec::new(),
                        start_byte: None,
                        end_byte: None,
                        body_start_byte: None,
                        body_end_byte: None,
                        body_statements: Vec::new(),
                    });
                }
            }
        }

        // Extract function definitions
        if node.kind() == "function_definition" {
            // Check if this is a G_DECLARE macro that tree-sitter misparsed
            let func_info = self.extract_function_from_definition(node, source);
            let is_g_declare = func_info
                .as_ref()
                .map_or(false, |(name, _)| name.starts_with("G_DECLARE_"));

            // Only recurse into the declarator/type, NOT into the function body
            // This prevents picking up function calls inside function bodies as
            // declarations
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                // Skip compound_statement (function body) to avoid false declarations
                if child.kind() != "compound_statement" {
                    self.visit_node(child, source, file_model, macro_map, static_forwards);
                }
            }

            // Don't add G_DECLARE as a function
            if !is_g_declare {
                if let Some((name, is_static)) = func_info {
                    if !is_gobject_type_macro(&name) {
                        // Find the function body (compound_statement)
                        let body = node.child_by_field_name("body");
                        let (body_start, body_end) = body
                            .map(|b| (Some(b.start_byte()), Some(b.end_byte())))
                            .unwrap_or((None, None));

                        // Parse body statements
                        let body_statements = body
                            .map(|b| self.parse_function_body(b, source))
                            .unwrap_or_default();

                        file_model.functions.push(FunctionInfo {
                            name: name.to_owned(),
                            line: node.start_position().row + 1,
                            is_static: is_static || static_forwards.contains(name),
                            export_macros: Vec::new(),
                            has_static_forward_decl: static_forwards.contains(name),
                            is_definition: true,
                            return_type: None,
                            parameters: Vec::new(),
                            start_byte: Some(node.start_byte()),
                            end_byte: Some(node.end_byte()),
                            body_start_byte: body_start,
                            body_end_byte: body_end,
                            body_statements,
                        });
                    }
                }
            }
            // Don't recurse again at the bottom
            return;
        }

        // Recurse
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_node(child, source, file_model, macro_map, static_forwards);
        }
    }

    fn extract_include(&self, node: Node, source: &[u8]) -> Option<Include> {
        let path_node = node.child_by_field_name("path")?;
        let path_text = std::str::from_utf8(&source[path_node.byte_range()]).ok()?;

        // Check if system include (<>) or local ("")
        let is_system = path_text.starts_with('<');
        let path = path_text.trim_matches(&['<', '>', '"'][..]);

        Some(Include {
            path: path.to_owned(),
            is_system,
            line: node.start_position().row + 1,
        })
    }

    fn collect_identifiers<'a>(&self, node: Node, source: &'a [u8], result: &mut Vec<&'a str>) {
        if node.kind() == "identifier" || node.kind() == "type_identifier" {
            if let Ok(text) = std::str::from_utf8(&source[node.byte_range()]) {
                result.push(text);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.collect_identifiers(child, source, result);
        }
    }

    fn extract_gobject_from_identifier(
        &self,
        _id_node: Node,
        parent: Node,
        source: &[u8],
        macro_name: &str,
    ) -> Option<GObjectType> {
        // Recursively find all identifiers in parent node
        let mut arg_values = Vec::new();
        self.collect_identifiers(parent, source, &mut arg_values);

        // Remove the macro name itself from the list
        arg_values.retain(|name| *name != macro_name);

        // G_DECLARE_*_TYPE needs 5 args
        if macro_name.starts_with("G_DECLARE_") && arg_values.len() >= 5 {
            let type_name = arg_values[0];
            let function_prefix = arg_values[1];
            let module_prefix = arg_values[2];
            let type_prefix = arg_values[3];
            let parent_type = arg_values[4];

            let type_macro = format!("{}_TYPE_{}", module_prefix, type_prefix);

            let kind = match macro_name {
                "G_DECLARE_FINAL_TYPE" => GObjectTypeKind::DeclareFinal {
                    function_prefix: function_prefix.to_owned(),
                    module_prefix: module_prefix.to_owned(),
                    type_prefix: type_prefix.to_owned(),
                    parent_type: parent_type.to_owned(),
                },
                "G_DECLARE_DERIVABLE_TYPE" => GObjectTypeKind::DeclareDerivable {
                    function_prefix: function_prefix.to_owned(),
                    module_prefix: module_prefix.to_owned(),
                    type_prefix: type_prefix.to_owned(),
                    parent_type: parent_type.to_owned(),
                },
                "G_DECLARE_INTERFACE" => GObjectTypeKind::DeclareInterface {
                    function_prefix: function_prefix.to_owned(),
                    module_prefix: module_prefix.to_owned(),
                    type_prefix: type_prefix.to_owned(),
                    prerequisite_type: parent_type.to_owned(),
                },
                _ => return None,
            };

            return Some(GObjectType {
                type_name: type_name.to_owned(),
                type_macro,
                kind,
                class_struct: None,
                line: parent.start_position().row + 1,
            });
        }

        // G_DEFINE_* needs 3 args
        if macro_name.starts_with("G_DEFINE_") && arg_values.len() >= 3 {
            let type_name = arg_values[0];
            let function_prefix = arg_values[1];
            let parent_type = arg_values[2];

            let type_macro = format!("TYPE_{}", type_name.to_uppercase());

            let kind = match macro_name {
                "G_DEFINE_TYPE" => GObjectTypeKind::DefineType {
                    function_prefix: function_prefix.to_owned(),
                    parent_type: parent_type.to_owned(),
                },
                "G_DEFINE_TYPE_WITH_PRIVATE" => GObjectTypeKind::DefineTypeWithPrivate {
                    function_prefix: function_prefix.to_owned(),
                    parent_type: parent_type.to_owned(),
                },
                "G_DEFINE_ABSTRACT_TYPE" => GObjectTypeKind::DefineAbstractType {
                    function_prefix: function_prefix.to_owned(),
                    parent_type: parent_type.to_owned(),
                },
                _ => return None,
            };

            return Some(GObjectType {
                type_name: type_name.to_owned(),
                type_macro,
                kind,
                class_struct: None,
                line: parent.start_position().row + 1,
            });
        }

        None
    }

    fn extract_gobject_type_declaration(&self, node: Node, source: &[u8]) -> Option<GObjectType> {
        let directive = node.child_by_field_name("directive")?;
        let directive_text = std::str::from_utf8(&source[directive.byte_range()]).ok()?;

        // Check if it's a G_DECLARE_* or G_DEFINE_* macro
        match directive_text {
            "G_DECLARE_FINAL_TYPE" | "G_DECLARE_DERIVABLE_TYPE" | "G_DECLARE_INTERFACE" => {
                self.extract_g_declare(node, source, directive_text)
            }
            "G_DEFINE_TYPE" | "G_DEFINE_TYPE_WITH_PRIVATE" | "G_DEFINE_ABSTRACT_TYPE" => {
                self.extract_g_define(node, source, directive_text)
            }
            _ => None,
        }
    }

    fn extract_g_declare(
        &self,
        node: Node,
        source: &[u8],
        macro_name: &str,
    ) -> Option<GObjectType> {
        // G_DECLARE_*_TYPE (TypeName, function_prefix, MODULE, TYPE_NAME, ParentType)
        let args = node.child_by_field_name("arguments")?;
        let mut cursor = args.walk();
        let mut arg_values = Vec::new();

        for child in args.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "type_identifier" {
                let text = std::str::from_utf8(&source[child.byte_range()]).ok()?;
                arg_values.push(text);
            }
        }

        if arg_values.len() < 5 {
            return None;
        }

        let type_name = arg_values[0];
        let function_prefix = arg_values[1];
        let module_prefix = arg_values[2];
        let type_prefix = arg_values[3];
        let parent_type = arg_values[4];

        let type_macro = format!("{}_TYPE_{}", module_prefix, type_prefix);

        let kind = match macro_name {
            "G_DECLARE_FINAL_TYPE" => GObjectTypeKind::DeclareFinal {
                function_prefix: function_prefix.to_owned(),
                module_prefix: module_prefix.to_owned(),
                type_prefix: type_prefix.to_owned(),
                parent_type: parent_type.to_owned(),
            },
            "G_DECLARE_DERIVABLE_TYPE" => GObjectTypeKind::DeclareDerivable {
                function_prefix: function_prefix.to_owned(),
                module_prefix: module_prefix.to_owned(),
                type_prefix: type_prefix.to_owned(),
                parent_type: parent_type.to_owned(),
            },
            "G_DECLARE_INTERFACE" => GObjectTypeKind::DeclareInterface {
                function_prefix: function_prefix.to_owned(),
                module_prefix: module_prefix.to_owned(),
                type_prefix: type_prefix.to_owned(),
                prerequisite_type: parent_type.to_owned(),
            },
            _ => return None,
        };

        Some(GObjectType {
            type_name: type_name.to_owned(),
            type_macro,
            kind,
            class_struct: None,
            line: node.start_position().row + 1,
        })
    }

    fn extract_g_define(&self, node: Node, source: &[u8], macro_name: &str) -> Option<GObjectType> {
        // G_DEFINE_TYPE (TypeName, function_prefix, PARENT_TYPE)
        let args = node.child_by_field_name("arguments")?;
        let mut cursor = args.walk();
        let mut arg_values = Vec::new();

        for child in args.children(&mut cursor) {
            if child.kind() == "identifier" || child.kind() == "type_identifier" {
                let text = std::str::from_utf8(&source[child.byte_range()]).ok()?;
                arg_values.push(text);
            }
        }

        if arg_values.len() < 3 {
            return None;
        }

        let type_name = arg_values[0];
        let function_prefix = arg_values[1];
        let parent_type = arg_values[2];

        // Generate type macro from type name
        let type_macro = format!("TYPE_{}", type_name.to_uppercase());

        let kind = match macro_name {
            "G_DEFINE_TYPE" => GObjectTypeKind::DefineType {
                function_prefix: function_prefix.to_owned(),
                parent_type: parent_type.to_owned(),
            },
            "G_DEFINE_TYPE_WITH_PRIVATE" => GObjectTypeKind::DefineTypeWithPrivate {
                function_prefix: function_prefix.to_owned(),
                parent_type: parent_type.to_owned(),
            },
            "G_DEFINE_ABSTRACT_TYPE" => GObjectTypeKind::DefineAbstractType {
                function_prefix: function_prefix.to_owned(),
                parent_type: parent_type.to_owned(),
            },
            _ => return None,
        };

        Some(GObjectType {
            type_name: type_name.to_owned(),
            type_macro,
            kind,
            class_struct: None,
            line: node.start_position().row + 1,
        })
    }

    fn extract_typedef_from_type_definition(
        &self,
        node: Node,
        source: &[u8],
    ) -> Option<TypedefInfo> {
        // type_definition has "declarator" for the typedef name and "type" for what
        // it's typedef'ing
        let declarator_node = node.child_by_field_name("declarator")?;
        let name = std::str::from_utf8(&source[declarator_node.byte_range()]).ok()?;

        let type_node = node.child_by_field_name("type")?;
        let target_type = std::str::from_utf8(&source[type_node.byte_range()]).ok()?;

        Some(TypedefInfo {
            name: name.to_owned(),
            line: node.start_position().row + 1,
            target_type: target_type.to_owned(),
        })
    }

    fn extract_struct(&self, node: Node, source: &[u8]) -> Option<StructInfo> {
        // Look for struct definitions or declarations
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "struct_specifier" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = std::str::from_utf8(&source[name_node.byte_range()]).ok()?;
                    let has_body = child.child_by_field_name("body").is_some();
                    return Some(StructInfo {
                        name: name.to_owned(),
                        line: child.start_position().row + 1,
                        fields: Vec::new(), // TODO: extract fields
                        is_opaque: !has_body,
                    });
                }
            }
        }
        None
    }

    fn extract_enum(&self, node: Node, source: &[u8]) -> Option<EnumInfo> {
        // Handle typedef enum { ... } Name;
        if node.kind() == "type_definition" {
            if let Some(type_node) = node.child_by_field_name("type") {
                if type_node.kind() == "enum_specifier" {
                    if let Some(declarator_node) = node.child_by_field_name("declarator") {
                        let name =
                            std::str::from_utf8(&source[declarator_node.byte_range()]).ok()?;
                        if let Some(body) = type_node.child_by_field_name("body") {
                            let values = self.extract_enum_values(body, source);
                            return Some(EnumInfo {
                                name: name.to_owned(),
                                line: node.start_position().row + 1,
                                values,
                                body_start_byte: body.start_byte(),
                                body_end_byte: body.end_byte(),
                            });
                        }
                    }
                }
            }
        }

        // Handle standalone enum Name { ... };
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "enum_specifier" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = std::str::from_utf8(&source[name_node.byte_range()]).ok()?;
                    if let Some(body) = child.child_by_field_name("body") {
                        let values = self.extract_enum_values(body, source);
                        return Some(EnumInfo {
                            name: name.to_owned(),
                            line: child.start_position().row + 1,
                            values,
                            body_start_byte: body.start_byte(),
                            body_end_byte: body.end_byte(),
                        });
                    }
                }
            }
        }
        None
    }

    fn extract_enum_values(&self, body_node: Node, source: &[u8]) -> Vec<EnumValue> {
        let mut values = Vec::new();

        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if child.kind() == "enumerator" {
                if let Some(name_node) = child.child_by_field_name("name") {
                    let name = std::str::from_utf8(&source[name_node.byte_range()])
                        .unwrap_or("")
                        .to_owned();

                    let (value, value_start, value_end) =
                        if let Some(value_node) = child.child_by_field_name("value") {
                            // Try to parse the value as an integer
                            let value_str = std::str::from_utf8(&source[value_node.byte_range()])
                                .unwrap_or("")
                                .trim();
                            (
                                value_str.parse::<i64>().ok(),
                                Some(value_node.start_byte()),
                                Some(value_node.end_byte()),
                            )
                        } else {
                            (None, None, None)
                        };

                    values.push(EnumValue {
                        name,
                        value,
                        start_byte: child.start_byte(),
                        end_byte: child.end_byte(),
                        name_start_byte: name_node.start_byte(),
                        name_end_byte: name_node.end_byte(),
                        value_start_byte: value_start,
                        value_end_byte: value_end,
                    });
                }
            }
        }

        values
    }

    fn find_static_forward_declarations<'a>(
        &self,
        node: Node,
        source: &'a [u8],
    ) -> HashSet<&'a str> {
        let mut static_decls = HashSet::new();
        self.visit_for_static_decls(node, source, &mut static_decls);
        static_decls
    }

    fn visit_for_static_decls<'a>(
        &self,
        node: Node,
        source: &'a [u8],
        static_decls: &mut HashSet<&'a str>,
    ) {
        if node.kind() == "declaration" {
            let mut is_static = false;
            let mut cursor = node.walk();
            for child in node.children(&mut cursor) {
                if child.kind() == "storage_class_specifier" {
                    let text = &source[child.byte_range()];
                    if std::str::from_utf8(text).unwrap_or("") == "static" {
                        is_static = true;
                        break;
                    }
                }
            }

            if is_static {
                let mut names = Vec::new();
                self.find_all_function_names(node, source, &mut names);
                for name in names {
                    static_decls.insert(name);
                }
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_static_decls(child, source, static_decls);
        }
    }

    fn extract_function_from_definition<'a>(
        &self,
        node: Node,
        source: &'a [u8],
    ) -> Option<(&'a str, bool)> {
        let mut is_static = false;
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "storage_class_specifier" {
                let text = &source[child.byte_range()];
                if std::str::from_utf8(text).unwrap_or("") == "static" {
                    is_static = true;
                }
            }
        }

        let declarator = node.child_by_field_name("declarator")?;
        let name = self.extract_declarator_name(declarator, source)?;

        Some((name, is_static))
    }

    fn has_static_storage_class(&self, node: Node, source: &[u8]) -> bool {
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.kind() == "storage_class_specifier" {
                let text = &source[child.byte_range()];
                if std::str::from_utf8(text).unwrap_or("") == "static" {
                    return true;
                }
            }
        }
        false
    }

    fn find_all_function_names<'a>(&self, node: Node, source: &'a [u8], result: &mut Vec<&'a str>) {
        if node.kind() == "function_declarator" {
            if let Some(name) = self.extract_declarator_name(node, source) {
                result.push(name);
            }
        } else if node.kind() == "expression_statement" {
            // Handle call_expression pattern (CLUTTER_EXPORT cases)
            if let Some(name) = self.extract_from_call_expression(node, source) {
                result.push(name);
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.find_all_function_names(child, source, result);
        }
    }

    fn extract_declarator_name<'a>(&self, declarator: Node, source: &'a [u8]) -> Option<&'a str> {
        if let Some(inner) = declarator.child_by_field_name("declarator") {
            if inner.kind() == "identifier" {
                let name = &source[inner.byte_range()];
                return Some(std::str::from_utf8(name).ok()?);
            }
            return self.extract_declarator_name(inner, source);
        }

        if declarator.kind() == "identifier" {
            let name = &source[declarator.byte_range()];
            return Some(std::str::from_utf8(name).ok()?);
        }

        // Handle parenthesized declarators like (function_name) used to prevent macro
        // expansion
        if declarator.kind() == "parenthesized_declarator" {
            let mut cursor = declarator.walk();
            for child in declarator.children(&mut cursor) {
                if child.kind() == "identifier" {
                    let name = &source[child.byte_range()];
                    return Some(std::str::from_utf8(name).ok()?);
                }
            }
        }

        None
    }

    fn extract_from_call_expression<'a>(&self, node: Node, source: &'a [u8]) -> Option<&'a str> {
        let call_expr = self.find_call_expression(node)?;
        let func_node = call_expr.child_by_field_name("function")?;
        if func_node.kind() == "identifier" {
            let name = &source[func_node.byte_range()];
            return Some(std::str::from_utf8(name).ok()?);
        }
        None
    }

    fn find_call_expression<'a>(&self, node: Node<'a>) -> Option<Node<'a>> {
        if node.kind() == "call_expression" {
            return Some(node);
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(found) = self.find_call_expression(child) {
                return Some(found);
            }
        }

        None
    }

    fn extract_class_structs(&self, node: Node, source: &[u8], file_model: &mut FileModel) {
        self.visit_for_class_structs(node, source, file_model);
    }

    fn visit_for_class_structs(&self, node: Node, source: &[u8], file_model: &mut FileModel) {
        // Look for struct_specifier nodes
        if node.kind() == "struct_specifier" {
            if let Some(name_node) = node.child_by_field_name("name") {
                if let Ok(struct_name) = std::str::from_utf8(&source[name_node.byte_range()]) {
                    // Check if this is a class struct (ends with "Class" and starts with "_")
                    if struct_name.starts_with("_") && struct_name.ends_with("Class") {
                        // Extract the type name: _CoglWinsysClass -> CoglWinsys
                        let type_name = &struct_name[1..struct_name.len() - 5]; // Remove leading "_" and trailing "Class"

                        // Find matching GObjectType
                        if let Some(gobject_idx) = file_model
                            .gobject_types
                            .iter()
                            .position(|gt| gt.type_name == type_name)
                        {
                            // Extract virtual functions from this struct
                            if let Some(body) = node.child_by_field_name("body") {
                                let vfuncs = self.extract_vfuncs(body, source);

                                let class_struct = ClassStruct {
                                    name: struct_name.to_owned(),
                                    vfuncs,
                                };

                                // Update the GObjectType with the class struct
                                file_model.gobject_types[gobject_idx].class_struct =
                                    Some(class_struct);
                            }
                        }
                    }
                }
            }
        }

        // Recurse
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.visit_for_class_structs(child, source, file_model);
        }
    }

    fn extract_vfuncs(&self, body_node: Node, source: &[u8]) -> Vec<VirtualFunction> {
        let mut vfuncs = Vec::new();

        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if child.kind() == "field_declaration" {
                // Look for function pointer fields
                if let Some(vfunc) = self.extract_vfunc_from_field(child, source) {
                    vfuncs.push(vfunc);
                }
            }
        }

        vfuncs
    }

    fn extract_vfunc_from_field(&self, field_node: Node, source: &[u8]) -> Option<VirtualFunction> {
        // A function pointer field looks like:
        // return_type (*name) (params);
        // In tree-sitter this is a field_declaration with a function_declarator

        let mut cursor = field_node.walk();
        for child in field_node.children(&mut cursor) {
            if child.kind() == "function_declarator" {
                // This is a function pointer
                return self.extract_function_pointer(child, field_node, source);
            }
        }

        None
    }

    fn extract_function_pointer(
        &self,
        func_decl: Node,
        field_node: Node,
        source: &[u8],
    ) -> Option<VirtualFunction> {
        // Get the function name from the declarator
        let declarator = func_decl.child_by_field_name("declarator")?;
        let name = self.extract_pointer_declarator_name(declarator, source)?;

        // Get return type from the field_declaration type
        let return_type = field_node
            .child_by_field_name("type")
            .and_then(|t| std::str::from_utf8(&source[t.byte_range()]).ok());

        // Extract parameters
        let mut parameters = Vec::new();
        if let Some(params_node) = func_decl.child_by_field_name("parameters") {
            parameters = self.extract_parameters(params_node, source);
        }

        Some(VirtualFunction {
            name: name.to_owned(),
            return_type: return_type.map(ToOwned::to_owned),
            parameters,
        })
    }

    fn extract_pointer_declarator_name<'a>(
        &self,
        declarator: Node,
        source: &'a [u8],
    ) -> Option<&'a str> {
        // For function pointers, the declarator can be:
        // - parenthesized_declarator containing pointer_declarator
        // - pointer_declarator containing identifier or field_identifier

        if declarator.kind() == "parenthesized_declarator" {
            // Look for pointer_declarator inside
            let mut cursor = declarator.walk();
            for child in declarator.children(&mut cursor) {
                if child.kind() == "pointer_declarator" {
                    return self.extract_pointer_declarator_name(child, source);
                } else if child.kind() == "identifier" || child.kind() == "field_identifier" {
                    return std::str::from_utf8(&source[child.byte_range()]).ok();
                }
            }
        } else if declarator.kind() == "pointer_declarator" {
            if let Some(inner) = declarator.child_by_field_name("declarator") {
                if inner.kind() == "identifier" || inner.kind() == "field_identifier" {
                    return std::str::from_utf8(&source[inner.byte_range()]).ok();
                }
                return self.extract_pointer_declarator_name(inner, source);
            }
        } else if declarator.kind() == "identifier" || declarator.kind() == "field_identifier" {
            return std::str::from_utf8(&source[declarator.byte_range()]).ok();
        }

        None
    }

    fn extract_parameters(&self, params_node: Node, source: &[u8]) -> Vec<Parameter> {
        let mut parameters = Vec::new();

        let mut cursor = params_node.walk();
        for child in params_node.children(&mut cursor) {
            if child.kind() == "parameter_declaration" {
                let type_node = child.child_by_field_name("type");
                let type_name = type_node
                    .and_then(|t| std::str::from_utf8(&source[t.byte_range()]).ok())
                    .unwrap_or_default();

                let declarator = child.child_by_field_name("declarator");
                let name = declarator.and_then(|d| self.extract_declarator_name(d, source));

                parameters.push(Parameter {
                    name: name.map(ToOwned::to_owned),
                    type_name: type_name.to_owned(),
                });
            }
        }

        parameters
    }

    fn extract_class_structs_from_text(&self, source: &[u8], file_model: &mut FileModel) {
        // For derivable types without class_struct, try to find it in the source text
        let source_str = std::str::from_utf8(source).unwrap_or("");

        for gobject_type in &mut file_model.gobject_types {
            // Only process derivable types that don't have a class_struct yet
            if let GObjectTypeKind::DeclareDerivable { .. } = &gobject_type.kind {
                if gobject_type.class_struct.is_some() {
                    continue;
                }

                // Look for "struct _TypeNameClass"
                let struct_name = format!("_{}", gobject_type.type_name) + "Class";
                let pattern = format!("struct {}", struct_name);

                if let Some(start_idx) = source_str.find(&pattern) {
                    // Found the struct definition - extract it and re-parse
                    let struct_start = start_idx;
                    // Find the matching closing brace
                    if let Some(open_brace) = source_str[struct_start..].find('{') {
                        let _body_start = struct_start + open_brace + 1;
                        if let Some(struct_end) =
                            self.find_matching_brace(source_str, struct_start + open_brace)
                        {
                            // Extract the struct text and re-parse it with tree-sitter
                            let struct_text = &source_str[struct_start..struct_end + 1];

                            // Create a new parser for re-parsing this snippet
                            let mut temp_parser = TSParser::new();
                            if temp_parser
                                .set_language(&tree_sitter_c::LANGUAGE.into())
                                .is_ok()
                            {
                                if let Some(tree) = temp_parser.parse(struct_text.as_bytes(), None)
                                {
                                    let mut vfuncs = Vec::new();
                                    // Look for struct_specifier in the parsed tree
                                    self.extract_vfuncs_from_tree(
                                        tree.root_node(),
                                        struct_text.as_bytes(),
                                        &mut vfuncs,
                                    );

                                    gobject_type.class_struct = Some(ClassStruct {
                                        name: struct_name,
                                        vfuncs,
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }
    }

    fn find_matching_brace(&self, text: &str, open_pos: usize) -> Option<usize> {
        let mut depth = 1;
        let bytes = text.as_bytes();

        for (i, &ch) in bytes.iter().enumerate().skip(open_pos + 1) {
            match ch {
                b'{' => depth += 1,
                b'}' => {
                    depth -= 1;
                    if depth == 0 {
                        return Some(i);
                    }
                }
                _ => {}
            }
        }
        None
    }

    fn extract_vfuncs_from_tree(
        &self,
        node: Node,
        source: &[u8],
        vfuncs: &mut Vec<VirtualFunction>,
    ) {
        // Recursively look for struct_specifier with a body
        if node.kind() == "struct_specifier" {
            if let Some(body) = node.child_by_field_name("body") {
                *vfuncs = self.extract_vfuncs(body, source);
                return;
            }
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            self.extract_vfuncs_from_tree(child, source, vfuncs);
        }
    }

    // ========================================================================
    // Statement and Expression Parsing
    // ========================================================================

    fn parse_function_body(&self, body_node: Node, source: &[u8]) -> Vec<Statement> {
        let mut statements = Vec::new();

        let mut cursor = body_node.walk();
        for child in body_node.children(&mut cursor) {
            if let Some(stmt) = self.parse_statement(child, source) {
                statements.push(stmt);
            }
        }

        statements
    }

    fn parse_statement(&self, node: Node, source: &[u8]) -> Option<Statement> {
        use crate::model::*;

        match node.kind() {
            "declaration" => {
                // Variable declaration
                self.parse_variable_decl(node, source)
                    .map(Statement::Declaration)
            }
            "expression_statement" => {
                // Expression like function call, assignment, etc.
                self.parse_expression_stmt(node, source)
                    .map(Statement::Expression)
            }
            "if_statement" => self.parse_if_statement(node, source).map(Statement::If),
            "return_statement" => self
                .parse_return_statement(node, source)
                .map(Statement::Return),
            "goto_statement" => self.parse_goto_statement(node, source).map(Statement::Goto),
            "labeled_statement" => self
                .parse_labeled_statement(node, source)
                .map(Statement::Labeled),
            "compound_statement" => self
                .parse_compound_statement(node, source)
                .map(Statement::Compound),
            _ => None,
        }
    }

    fn parse_variable_decl(&self, node: Node, source: &[u8]) -> Option<VariableDecl> {
        // declaration contains declarator and optionally type_specifier
        let mut type_name = String::new();
        let mut declarator = None;

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            match child.kind() {
                "type_qualifier"
                | "storage_class_specifier"
                | "type_specifier"
                | "type_identifier"
                | "primitive_type"
                | "sized_type_specifier"
                | "struct_specifier" => {
                    if !type_name.is_empty() {
                        type_name.push(' ');
                    }
                    type_name.push_str(std::str::from_utf8(&source[child.byte_range()]).ok()?);
                }
                // Declarations with initializer: int x = 5;
                "init_declarator" => {
                    declarator = Some(child);
                }
                // Declarations without initializer: int x;  or  int *x;
                "pointer_declarator" | "identifier" | "array_declarator" => {
                    if declarator.is_none() {
                        declarator = Some(child);
                    }
                }
                _ => {}
            }
        }

        let declarator = declarator?;

        // Get variable name from declarator
        let mut var_name = None;
        let mut initializer = None;

        // For pointer types like "GError *error", check if this is a pointer declarator
        let declarator_text = std::str::from_utf8(&source[declarator.byte_range()]).ok()?;
        if declarator_text.contains('*') && !type_name.contains('*') {
            type_name.push('*');
        }

        let mut dec_cursor = declarator.walk();
        let mut has_equals = false;
        for child in declarator.children(&mut dec_cursor) {
            match child.kind() {
                "pointer_declarator" | "identifier" => {
                    // Extract identifier from declarator
                    if let Some(id) = self.find_identifier(child, source) {
                        var_name = Some(id);
                    }
                }
                "=" => {
                    has_equals = true;
                }
                _ => {
                    // Only treat as initializer if we've seen an "=" sign
                    if has_equals {
                        initializer = self.parse_expression(child, source);
                    }
                }
            }
        }

        Some(VariableDecl {
            type_name,
            name: var_name?.to_owned(),
            initializer,
            location: self.node_location(node),
        })
    }

    fn find_identifier<'a>(&self, node: Node, source: &'a [u8]) -> Option<&'a str> {
        if node.kind() == "identifier" {
            return std::str::from_utf8(&source[node.byte_range()]).ok();
        }

        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if let Some(id) = self.find_identifier(child, source) {
                return Some(id);
            }
        }

        None
    }

    fn parse_expression_stmt(&self, node: Node, source: &[u8]) -> Option<ExpressionStmt> {
        // Get the actual expression inside the statement
        let mut cursor = node.walk();
        for child in node.children(&mut cursor) {
            if child.is_named() && child.kind() != ";" {
                if let Some(expr) = self.parse_expression(child, source) {
                    return Some(ExpressionStmt {
                        expr,
                        location: self.node_location(node),
                    });
                }
            }
        }
        None
    }

    fn parse_expression(&self, node: Node, source: &[u8]) -> Option<Expression> {
        use crate::model::Expression;

        match node.kind() {
            "call_expression" => self
                .parse_call_expression(node, source)
                .map(Expression::Call),
            "assignment_expression" => self
                .parse_assignment(node, source)
                .map(Expression::Assignment),
            "binary_expression" => self
                .parse_binary_expression(node, source)
                .map(Expression::Binary),
            "unary_expression" | "pointer_expression" => self
                .parse_unary_expression(node, source)
                .map(Expression::Unary),
            "parenthesized_expression" => {
                // Unwrap the parentheses and parse the inner expression
                let mut cursor = node.walk();
                for child in node.children(&mut cursor) {
                    if child.is_named() && child.kind() != "(" && child.kind() != ")" {
                        return self.parse_expression(child, source);
                    }
                }
                None
            }
            "identifier" => {
                let name = std::str::from_utf8(&source[node.byte_range()])
                    .ok()?
                    .to_owned();
                Some(Expression::Identifier(crate::model::IdentifierExpression {
                    name,
                    location: self.node_location(node),
                }))
            }
            "field_expression" => {
                let text = std::str::from_utf8(&source[node.byte_range()])
                    .ok()?
                    .to_owned();
                Some(Expression::FieldAccess(
                    crate::model::FieldAccessExpression {
                        text,
                        location: self.node_location(node),
                    },
                ))
            }
            "string_literal" => {
                let value = std::str::from_utf8(&source[node.byte_range()])
                    .ok()?
                    .to_owned();
                Some(Expression::StringLiteral(
                    crate::model::StringLiteralExpression {
                        value,
                        location: self.node_location(node),
                    },
                ))
            }
            "number_literal" => {
                let value = std::str::from_utf8(&source[node.byte_range()])
                    .ok()?
                    .to_owned();
                Some(Expression::NumberLiteral(
                    crate::model::NumberLiteralExpression {
                        value,
                        location: self.node_location(node),
                    },
                ))
            }
            "null" | "NULL" => Some(Expression::Null(crate::model::NullExpression {
                location: self.node_location(node),
            })),
            "true" | "TRUE" => Some(Expression::Boolean(crate::model::BooleanExpression {
                value: true,
                location: self.node_location(node),
            })),
            "false" | "FALSE" => Some(Expression::Boolean(crate::model::BooleanExpression {
                value: false,
                location: self.node_location(node),
            })),
            "cast_expression" => self.parse_cast_expression(node, source),
            "conditional_expression" => self.parse_conditional_expression(node, source),
            "sizeof_expression" => {
                let text = std::str::from_utf8(&source[node.byte_range()])
                    .ok()?
                    .to_owned();
                Some(Expression::Sizeof(crate::model::SizeofExpression {
                    text,
                    location: self.node_location(node),
                }))
            }
            "subscript_expression" => self.parse_subscript_expression(node, source),
            "initializer_list" => {
                let text = std::str::from_utf8(&source[node.byte_range()])
                    .ok()?
                    .to_owned();
                Some(Expression::InitializerList(
                    crate::model::InitializerListExpression {
                        text,
                        location: self.node_location(node),
                    },
                ))
            }
            "char_literal" => {
                let value = std::str::from_utf8(&source[node.byte_range()])
                    .ok()?
                    .to_owned();
                Some(Expression::CharLiteral(
                    crate::model::CharLiteralExpression {
                        value,
                        location: self.node_location(node),
                    },
                ))
            }
            "update_expression" => self.parse_update_expression(node, source),
            _ => {
                // Unknown expression type - fail loudly so we implement it immediately
                todo!(
                    "Unimplemented expression type: {} at {}:{}",
                    node.kind(),
                    node.start_position().row + 1,
                    node.start_position().column + 1
                )
            }
        }
    }

    fn parse_call_expression(&self, node: Node, source: &[u8]) -> Option<CallExpression> {
        let function_node = node.child_by_field_name("function")?;
        let function = std::str::from_utf8(&source[function_node.byte_range()])
            .ok()?
            .to_owned();

        let mut arguments = Vec::new();
        if let Some(args_node) = node.child_by_field_name("arguments") {
            let mut cursor = args_node.walk();
            for child in args_node.children(&mut cursor) {
                if child.is_named() && child.kind() != "," {
                    if let Some(expr) = self.parse_expression(child, source) {
                        arguments.push(Argument::Expression(Box::new(expr)));
                    }
                }
            }
        }

        Some(CallExpression {
            function,
            arguments,
            location: self.node_location(node),
        })
    }

    fn parse_assignment(&self, node: Node, source: &[u8]) -> Option<Assignment> {
        let left_node = node.child_by_field_name("left")?;
        let lhs = std::str::from_utf8(&source[left_node.byte_range()])
            .ok()?
            .to_owned();

        let operator_node = node.child_by_field_name("operator")?;
        let operator = std::str::from_utf8(&source[operator_node.byte_range()])
            .ok()?
            .to_owned();

        let right_node = node.child_by_field_name("right")?;
        let rhs = self.parse_expression(right_node, source)?;

        Some(Assignment {
            lhs,
            operator,
            rhs: Box::new(rhs),
            location: self.node_location(node),
        })
    }

    fn parse_binary_expression(&self, node: Node, source: &[u8]) -> Option<BinaryExpression> {
        let left_node = node.child_by_field_name("left")?;
        let left = self.parse_expression(left_node, source)?;

        let operator_node = node.child_by_field_name("operator")?;
        let operator = std::str::from_utf8(&source[operator_node.byte_range()])
            .ok()?
            .to_owned();

        let right_node = node.child_by_field_name("right")?;
        let right = self.parse_expression(right_node, source)?;

        Some(BinaryExpression {
            left: Box::new(left),
            operator,
            right: Box::new(right),
            location: self.node_location(node),
        })
    }

    fn parse_unary_expression(&self, node: Node, source: &[u8]) -> Option<UnaryExpression> {
        let operator_node = node.child_by_field_name("operator")?;
        let operator = std::str::from_utf8(&source[operator_node.byte_range()])
            .ok()?
            .to_owned();

        let operand_node = node.child_by_field_name("argument")?;
        let operand = self.parse_expression(operand_node, source)?;

        Some(UnaryExpression {
            operator,
            operand: Box::new(operand),
            location: self.node_location(node),
        })
    }

    fn parse_cast_expression(&self, node: Node, source: &[u8]) -> Option<Expression> {
        use crate::model::{CastExpression, Expression};

        // Get the type node
        let type_node = node.child_by_field_name("type")?;
        let type_name = std::str::from_utf8(&source[type_node.byte_range()])
            .ok()?
            .to_owned();

        // Get the value node
        let value_node = node.child_by_field_name("value")?;
        let operand = self.parse_expression(value_node, source)?;

        Some(Expression::Cast(CastExpression {
            type_name,
            operand: Box::new(operand),
            location: self.node_location(node),
        }))
    }

    fn parse_conditional_expression(&self, node: Node, source: &[u8]) -> Option<Expression> {
        use crate::model::{ConditionalExpression, Expression};

        let condition_node = node.child_by_field_name("condition")?;
        let condition = self.parse_expression(condition_node, source)?;

        let consequence_node = node.child_by_field_name("consequence")?;
        let then_expr = self.parse_expression(consequence_node, source)?;

        let alternative_node = node.child_by_field_name("alternative")?;
        let else_expr = self.parse_expression(alternative_node, source)?;

        Some(Expression::Conditional(ConditionalExpression {
            condition: Box::new(condition),
            then_expr: Box::new(then_expr),
            else_expr: Box::new(else_expr),
            location: self.node_location(node),
        }))
    }

    fn parse_subscript_expression(&self, node: Node, source: &[u8]) -> Option<Expression> {
        use crate::model::{Expression, SubscriptExpression};

        let argument_node = node.child_by_field_name("argument")?;
        let array = self.parse_expression(argument_node, source)?;

        let index_node = node.child_by_field_name("index")?;
        let index = self.parse_expression(index_node, source)?;

        Some(Expression::Subscript(SubscriptExpression {
            array: Box::new(array),
            index: Box::new(index),
            location: self.node_location(node),
        }))
    }

    fn parse_update_expression(&self, node: Node, source: &[u8]) -> Option<Expression> {
        use crate::model::{Expression, UpdateExpression};

        let operator_node = node.child_by_field_name("operator")?;
        let operator = std::str::from_utf8(&source[operator_node.byte_range()])
            .ok()?
            .to_owned();

        let argument_node = node.child_by_field_name("argument")?;
        let operand = self.parse_expression(argument_node, source)?;

        // Determine if prefix or postfix based on node positions
        let is_prefix = operator_node.start_byte() < argument_node.start_byte();

        Some(Expression::Update(UpdateExpression {
            operator,
            operand: Box::new(operand),
            is_prefix,
            location: self.node_location(node),
        }))
    }

    fn parse_if_statement(&self, node: Node, source: &[u8]) -> Option<IfStatement> {
        let condition_node = node.child_by_field_name("condition")?;
        let condition = self.parse_expression(condition_node, source)?;

        let consequence_node = node.child_by_field_name("consequence")?;
        let then_has_braces = consequence_node.kind() == "compound_statement";
        let then_body = if then_has_braces {
            self.parse_function_body(consequence_node, source)
        } else {
            // Single statement
            self.parse_statement(consequence_node, source)
                .map(|s| vec![s])
                .unwrap_or_default()
        };

        let else_body = node.child_by_field_name("alternative").map(|alt_node| {
            if alt_node.kind() == "compound_statement" {
                self.parse_function_body(alt_node, source)
            } else {
                self.parse_statement(alt_node, source)
                    .map(|s| vec![s])
                    .unwrap_or_default()
            }
        });

        Some(IfStatement {
            condition,
            then_body,
            then_has_braces,
            else_body,
            location: self.node_location(node),
        })
    }

    fn parse_return_statement(&self, node: Node, source: &[u8]) -> Option<ReturnStatement> {
        let value = node.child(1).and_then(|v| {
            // Check if it's actually an expression (not a semicolon)
            if v.is_named() && v.kind() != ";" {
                self.parse_expression(v, source)
            } else {
                None
            }
        });

        Some(ReturnStatement {
            value,
            location: self.node_location(node),
        })
    }

    fn parse_goto_statement(&self, node: Node, source: &[u8]) -> Option<GotoStatement> {
        let label_node = node.child_by_field_name("label")?;
        let label = std::str::from_utf8(&source[label_node.byte_range()])
            .ok()?
            .to_owned();

        Some(GotoStatement {
            label,
            location: self.node_location(node),
        })
    }

    fn parse_labeled_statement(&self, node: Node, source: &[u8]) -> Option<LabeledStatement> {
        let label_node = node.child_by_field_name("label")?;
        let label = std::str::from_utf8(&source[label_node.byte_range()])
            .ok()?
            .to_owned();

        // Get the statement after the label
        let mut cursor = node.walk();
        let mut statement = None;
        for child in node.children(&mut cursor) {
            if child.kind() != "label" && child.kind() != ":" && child.is_named() {
                statement = self.parse_statement(child, source);
                break;
            }
        }

        Some(LabeledStatement {
            label,
            statement: Box::new(statement?),
            location: self.node_location(node),
        })
    }

    fn parse_compound_statement(&self, node: Node, source: &[u8]) -> Option<CompoundStatement> {
        let statements = self.parse_function_body(node, source);

        Some(CompoundStatement {
            statements,
            location: self.node_location(node),
        })
    }
}

impl Default for Parser {
    fn default() -> Self {
        Self::new().expect("Failed to create parser")
    }
}

fn is_gobject_type_macro(name: &str) -> bool {
    name.starts_with("G_DECLARE_") || name.starts_with("G_DEFINE_")
}

fn is_macro_identifier(name: &str) -> bool {
    // Specific known macros and keywords
    if name == "G_DECLARE_FINAL_TYPE"
        || name == "G_DECLARE_DERIVABLE_TYPE"
        || name == "G_DECLARE_INTERFACE"
        || name == "void"
        || name == "int"
        || name.starts_with("META_TYPE_")
        || name.starts_with("CLUTTER_TYPE_")
        || name.starts_with("COGL_TYPE_")
        || name.starts_with("GTK_TYPE_")
        || name.starts_with("G_TYPE_")
        || name == "COGL_PRIVATE"
        || name.ends_with("_get_type")
        || name.ends_with("_error_quark")
        || name.ends_with("_END")
        || name == "main"
    {
        return true;
    }

    // Heuristic: if the identifier is ALL_CAPS (with underscores), it's likely a
    // macro Exception: single words like NULL, TRUE, FALSE are constants, not
    // macro calls
    if name
        .chars()
        .all(|c| c.is_uppercase() || c == '_' || c.is_numeric())
    {
        // But allow common constants/types that are legitimately all-caps
        if name == "NULL" || name == "TRUE" || name == "FALSE" {
            return false;
        }
        // If it contains an underscore or is longer than 4 chars, likely a macro
        if name.contains('_') || name.len() > 4 {
            return true;
        }
    }

    false
}
