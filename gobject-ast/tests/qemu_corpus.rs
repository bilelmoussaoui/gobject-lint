#![cfg(feature = "qemu")]

use std::{collections::HashMap, env, fs, path::Path};

use gobject_ast::{
    Parser,
    model::{StructField, TopLevelItem, TypeDefItem, TypedefTarget},
};
use ignore::WalkBuilder;
use tree_sitter::Node;

const QEMU_MODIFIERS: &[&str] = &[
    "coroutine_fn",
    "coroutine_mixed_fn",
    "no_coroutine_fn",
    "co_wrapper",
    "co_wrapper_mixed",
    "co_wrapper_bdrv_rdlock",
    "co_wrapper_mixed_bdrv_rdlock",
    "no_co_wrapper",
    "no_co_wrapper_bdrv_rdlock",
    "no_co_wrapper_bdrv_wrlock",
];

fn contains_qemu_modifier(source: &str) -> bool {
    source
        .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
        .any(|word| {
            QEMU_MODIFIERS.contains(&word) || word.starts_with("GRAPH_") || word.starts_with("TSA_")
        })
}

fn walk(node: Node<'_>, visit: &mut impl FnMut(Node<'_>)) {
    visit(node);
    let mut cursor = node.walk();
    for child in node.children(&mut cursor) {
        walk(child, visit);
    }
}

fn modifier_counts(modifiers: &[String], counts: &mut HashMap<String, usize>) {
    for modifier in modifiers {
        if QEMU_MODIFIERS.contains(&modifier.as_str()) {
            *counts.entry(modifier.clone()).or_default() += 1;
        }
    }
}

fn field_modifier_counts(fields: &[StructField], counts: &mut HashMap<String, usize>) {
    for field in fields {
        if let Some(callable) = &field.callable {
            modifier_counts(&callable.macro_modifiers, counts);
        }
        field_modifier_counts(&field.inner_fields, counts);
    }
}

/// Corpus gate for the QEMU syntax extension. The script in `../scripts`
/// supplies a pinned checkout by default; developers can point QEMU_SRC at an
/// existing checkout for a faster local run.
#[test]
#[ignore = "requires a QEMU source checkout; run scripts/check-qemu-corpus.sh"]
fn qemu_annotations_parse_without_recovery_and_reach_the_ast() {
    let source_root = env::var_os("QEMU_SRC").expect("QEMU_SRC must name a QEMU checkout");
    let source_root = Path::new(&source_root);
    assert!(source_root.join("include/qemu/osdep.h").is_file());

    let mut parser = tree_sitter::Parser::new();
    parser
        .set_language(&tree_sitter_c_gobject::LANGUAGE.into())
        .expect("load QEMU grammar");
    let mut ast_parser = Parser::new().expect("load QEMU AST parser");

    let mut files_checked = 0usize;
    let mut raw_counts = HashMap::<String, usize>::new();
    let mut ast_counts = HashMap::<String, usize>::new();
    let mut failures = Vec::new();

    for entry in WalkBuilder::new(source_root)
        .hidden(false)
        .git_ignore(true)
        .git_exclude(true)
        .build()
        .filter_map(Result::ok)
        .filter(|entry| {
            entry
                .path()
                .extension()
                .is_some_and(|extension| extension == "c" || extension == "h")
        })
    {
        let path = entry.path();
        let Ok(source) = fs::read(path) else { continue };
        let Ok(source_text) = std::str::from_utf8(&source) else {
            continue;
        };
        if !contains_qemu_modifier(source_text) {
            continue;
        }
        files_checked += 1;

        let tree = parser.parse(&source, None).expect("tree-sitter parse tree");
        walk(tree.root_node(), &mut |node| {
            if node.kind() == "macro_modifier"
                && let Ok(text) = std::str::from_utf8(&source[node.byte_range()])
                && QEMU_MODIFIERS.contains(&text)
            {
                *raw_counts.entry(text.to_owned()).or_default() += 1;
            }

            if node.is_error() || node.is_missing() {
                let diagnostic_node = if node.is_missing() {
                    node.parent().unwrap_or(node)
                } else {
                    node
                };
                let snippet =
                    std::str::from_utf8(&source[diagnostic_node.byte_range()]).unwrap_or_default();
                // QEMU also uses unrelated extensions that can place a large
                // part of a translation unit under one recovery node. Only
                // attribute local recovery constructs to this extension.
                if diagnostic_node.byte_range().len() <= 4_096 && contains_qemu_modifier(snippet) {
                    failures.push(format!(
                        "{}:{}:{}: QEMU annotation intersects {} node: {}",
                        path.strip_prefix(source_root).unwrap_or(path).display(),
                        node.start_position().row + 1,
                        node.start_position().column + 1,
                        node.kind(),
                        snippet.lines().next().unwrap_or_default().trim(),
                    ));
                }
            }
        });

        let (_, model) = ast_parser
            .parse_file_to_model(path)
            .unwrap_or_else(|error| panic!("{}: {error:#}", path.display()));
        for function in model.iter_function_declarations() {
            modifier_counts(&function.macro_modifiers, &mut ast_counts);
        }
        for function in model.iter_function_definitions() {
            modifier_counts(&function.macro_modifiers, &mut ast_counts);
        }
        for item in model.iter_all_items() {
            let TopLevelItem::TypeDefinition(type_definition) = item else {
                continue;
            };
            match type_definition {
                TypeDefItem::Typedef {
                    target:
                        TypedefTarget::Callback {
                            macro_modifiers, ..
                        },
                    struct_fields,
                    ..
                } => {
                    modifier_counts(macro_modifiers, &mut ast_counts);
                    field_modifier_counts(struct_fields, &mut ast_counts);
                }
                TypeDefItem::Typedef { struct_fields, .. } => {
                    field_modifier_counts(struct_fields, &mut ast_counts);
                }
                TypeDefItem::Struct { fields, .. } => {
                    field_modifier_counts(fields, &mut ast_counts);
                }
                TypeDefItem::Enum(_) => {}
            }
        }
    }

    assert!(
        files_checked > 100,
        "only checked {files_checked} QEMU files"
    );
    assert!(
        raw_counts.get("coroutine_fn").copied().unwrap_or_default() > 1_000,
        "unexpected QEMU annotation counts: {raw_counts:?}"
    );
    for modifier in QEMU_MODIFIERS {
        let raw = raw_counts.get(*modifier).copied().unwrap_or_default();
        let ast = ast_counts.get(*modifier).copied().unwrap_or_default();
        if raw > 0 && ast == 0 {
            failures.push(format!(
                "modifier `{modifier}` parsed {raw} times but was never preserved in callable AST metadata"
            ));
        }
    }
    assert!(
        failures.is_empty(),
        "{} QEMU corpus failures:\n{}",
        failures.len(),
        failures.into_iter().take(50).collect::<Vec<_>>().join("\n")
    );
}
