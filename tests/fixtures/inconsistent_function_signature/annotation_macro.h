#pragma once

/* tree-sitter parses `bool SOME_ANNOTATION func()` as two items:
 *   1. declaration: `bool SOME_ANNOTATION;`
 *   2. function_declaration with return type `SOME_ANNOTATION`
 * The rule should recognize that SOME_ANNOTATION is not a real type
 * and skip the false mismatch. */
bool SOME_ANNOTATION anno_func (void);
