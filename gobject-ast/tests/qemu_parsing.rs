#![cfg(feature = "qemu")]

use std::path::Path;

use gobject_ast::{
    Parser,
    model::{TopLevelItem, TypeDefItem, TypedefTarget},
};

#[test]
fn preserves_qemu_modifiers_on_callable_declarations() {
    let path = Path::new("tests/fixtures/qemu_annotations.c");
    let mut parser = Parser::new().expect("create parser");
    let project = parser.parse_file(path).expect("parse QEMU fixture");
    let file = project.files.values().next().expect("fixture model");

    let declared = file
        .iter_function_declarations()
        .find(|function| function.name == "declared")
        .expect("declared function");
    assert_eq!(declared.macro_modifiers, ["coroutine_fn"]);

    let mixed = file
        .iter_function_declarations()
        .find(|function| function.name == "mixed_only")
        .expect("stacked modifier declaration");
    assert_eq!(
        mixed.macro_modifiers,
        ["coroutine_mixed_fn", "no_coroutine_fn"]
    );

    let pointer_return = file
        .iter_function_declarations()
        .find(|function| function.name == "pointer_return")
        .expect("pointer-return declaration");
    assert_eq!(pointer_return.macro_modifiers, ["coroutine_fn"]);

    let defined = file
        .iter_function_definitions()
        .find(|function| function.name == "defined")
        .expect("annotated definition");
    assert_eq!(defined.macro_modifiers, ["coroutine_fn"]);

    let typedefs = file.top_level_items.iter().filter_map(|item| match item {
        TopLevelItem::TypeDefinition(item) => Some(item),
        _ => None,
    });
    let mut saw_entry = false;
    let mut saw_blocking = false;
    let mut saw_fields = false;
    for typedef in typedefs {
        match typedef {
            TypeDefItem::Typedef {
                name,
                target:
                    TypedefTarget::Callback {
                        macro_modifiers, ..
                    },
                ..
            } if name == "CoroutineEntry" => {
                assert_eq!(macro_modifiers, &["coroutine_fn"]);
                saw_entry = true;
            }
            TypeDefItem::Typedef {
                name,
                target:
                    TypedefTarget::Callback {
                        macro_modifiers, ..
                    },
                ..
            } if name == "BlockingCallback" => {
                assert_eq!(macro_modifiers, &["no_coroutine_fn"]);
                saw_blocking = true;
            }
            TypeDefItem::Typedef {
                name,
                struct_fields,
                ..
            } if name == "Ops" => {
                let run = struct_fields
                    .iter()
                    .find(|field| field.field_name.as_deref() == Some("run"))
                    .expect("run callback field");
                assert_eq!(
                    run.callable
                        .as_ref()
                        .expect("callable shape")
                        .macro_modifiers,
                    ["coroutine_fn"]
                );
                let dispatch = struct_fields
                    .iter()
                    .find(|field| field.field_name.as_deref() == Some("dispatch"))
                    .expect("dispatch callback field");
                assert_eq!(
                    dispatch
                        .callable
                        .as_ref()
                        .expect("callable shape")
                        .macro_modifiers,
                    ["co_wrapper_mixed"]
                );
                saw_fields = true;
            }
            _ => {}
        }
    }
    assert!(saw_entry && saw_blocking && saw_fields);
}
