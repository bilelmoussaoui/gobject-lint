/**
 * QEMU C grammar, extending the GObject grammar without changing its default
 * generated parser.
 */

/// <reference types="tree-sitter-cli/dsl" />
// @ts-check

const GObject = require('../grammar');

function qemuPointerDeclarator($, declarator) {
  return prec.dynamic(1, prec.right(seq(
    optional($.ms_based_modifier),
    '*',
    repeat($.ms_pointer_modifier),
    repeat($.type_qualifier),
    repeat($.macro_modifier),
    field('declarator', declarator),
  )));
}

module.exports = grammar(GObject, {
  // Keep the language symbol stable. Cargo selects exactly one generated
  // parser, so gobject-ast does not need a runtime dialect switch.
  name: 'c_gobject',

  externals: ($, original) => [
    ...original,
    $._qemu_function_like_modifier,
  ],

  rules: {
    // TSA_* modifiers with arguments need a distinct external token. Without
    // it, `(lock)` is ambiguous with the parenthesized declarator that may
    // follow a pointer modifier.
    macro_modifier: $ => prec.left(2, choice(
      seq($._qemu_function_like_modifier, $.argument_list),
      seq($._macro_modifier_name, optional($.argument_list)),
    )),

    // QEMU annotations are declaration specifiers, including in callback
    // typedefs such as `typedef void coroutine_fn (*Callback)(void)`.
    _type_definition_type: $ => seq(
      repeat(choice($.type_qualifier, $.macro_modifier)),
      field('type', $.type_specifier),
      repeat(choice($.type_qualifier, $.macro_modifier)),
    ),

    struct_specifier: $ => prec.right(seq(
      'struct',
      optional($.attribute_specifier),
      optional($.ms_declspec_modifier),
      repeat($.macro_modifier),
      choice(
        seq(
          field('name', $._type_identifier),
          field('body', optional($.field_declaration_list)),
        ),
        field('body', $.field_declaration_list),
      ),
      optional($.attribute_specifier),
    )),

    union_specifier: $ => prec.right(seq(
      'union',
      optional($.ms_declspec_modifier),
      repeat($.macro_modifier),
      choice(
        seq(
          field('name', $._type_identifier),
          field('body', optional($.field_declaration_list)),
        ),
        field('body', $.field_declaration_list),
      ),
      optional($.attribute_specifier),
    )),

    // QEMU permits declaration modifiers between a pointer star and the
    // declarator, for example `QMPRequest * coroutine_fn handle()`.
    pointer_declarator: ($, _original) => qemuPointerDeclarator($, $._declarator),
    pointer_field_declarator: ($, _original) => qemuPointerDeclarator($, $._field_declarator),
    pointer_type_declarator: ($, _original) => qemuPointerDeclarator($, $._type_declarator),
  },
});
