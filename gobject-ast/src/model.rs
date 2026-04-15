use std::{collections::HashMap, path::PathBuf};

use serde::{Deserialize, Serialize};

/// Source location information for AST nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SourceLocation {
    pub line: usize,
    pub column: usize,
    pub start_byte: usize,
    pub end_byte: usize,
}

impl SourceLocation {
    pub fn new(line: usize, column: usize, start_byte: usize, end_byte: usize) -> Self {
        Self {
            line,
            column,
            start_byte,
            end_byte,
        }
    }

    /// Extract the source text for this location
    pub fn as_str<'a>(&self, source: &'a [u8]) -> Option<&'a str> {
        std::str::from_utf8(&source[self.start_byte..self.end_byte]).ok()
    }
}

/// The complete project model - a map of files to their content
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Project {
    pub files: HashMap<PathBuf, FileModel>,
}

impl Project {
    pub fn new() -> Self {
        Self {
            files: HashMap::new(),
        }
    }

    /// Get a file's model
    pub fn get_file(&self, path: &PathBuf) -> Option<&FileModel> {
        self.files.get(path)
    }

    /// Find a function by name across all files
    pub fn find_function(&self, name: &str) -> Option<&FunctionInfo> {
        for file in self.files.values() {
            if let Some(func) = file.functions.iter().find(|f| f.name == name) {
                return Some(func);
            }
        }
        None
    }

    /// Check if a function is declared in any header
    pub fn is_function_declared_in_header(&self, name: &str) -> bool {
        for file in self.files.values() {
            if file.path.extension().map_or(false, |ext| ext == "h") {
                if file.functions.iter().any(|f| f.name == name) {
                    return true;
                }
            }
        }
        false
    }

    /// Check if a function has export macros (truly public API)
    pub fn is_function_exported(&self, name: &str) -> bool {
        for file in self.files.values() {
            if let Some(func) = file.functions.iter().find(|f| f.name == name) {
                if !func.export_macros.is_empty() {
                    return true;
                }
            }
        }
        false
    }
}

/// Model of a single file (header or C file)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FileModel {
    pub path: PathBuf,
    pub includes: Vec<Include>,
    pub typedefs: Vec<TypedefInfo>,
    pub structs: Vec<StructInfo>,
    pub enums: Vec<EnumInfo>,
    pub functions: Vec<FunctionInfo>,
    pub gobject_types: Vec<GObjectType>,
    /// The raw source code of this file - available for detailed pattern
    /// matching
    #[serde(skip)]
    pub source: Vec<u8>,
}

impl FileModel {
    pub fn new(path: PathBuf) -> Self {
        Self {
            path,
            includes: Vec::new(),
            typedefs: Vec::new(),
            structs: Vec::new(),
            enums: Vec::new(),
            functions: Vec::new(),
            gobject_types: Vec::new(),
            source: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GObjectType {
    pub type_name: String,  // e.g., "ClutterInputDeviceTool"
    pub type_macro: String, // e.g., "CLUTTER_TYPE_INPUT_DEVICE_TOOL"
    pub kind: GObjectTypeKind,
    pub class_struct: Option<ClassStruct>, // For derivable types
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassStruct {
    pub name: String, // e.g., "CoglWinsysClass"
    pub vfuncs: Vec<VirtualFunction>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VirtualFunction {
    pub name: String,
    pub return_type: Option<String>,
    pub parameters: Vec<Parameter>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum GObjectTypeKind {
    DeclareFinal {
        function_prefix: String, // e.g., "clutter_input_device_tool"
        module_prefix: String,   // e.g., "CLUTTER"
        type_prefix: String,     // e.g., "INPUT_DEVICE_TOOL"
        parent_type: String,     // e.g., "GObject"
    },
    DeclareDerivable {
        function_prefix: String,
        module_prefix: String,
        type_prefix: String,
        parent_type: String,
    },
    DeclareInterface {
        function_prefix: String,
        module_prefix: String,
        type_prefix: String,
        prerequisite_type: String,
    },
    DefineType {
        function_prefix: String,
        parent_type: String,
    },
    DefineTypeWithPrivate {
        function_prefix: String,
        parent_type: String,
    },
    DefineAbstractType {
        function_prefix: String,
        parent_type: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Include {
    pub path: String,
    pub is_system: bool, // <> vs ""
    pub line: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FunctionInfo {
    pub name: String,
    pub line: usize,
    pub is_static: bool,
    pub export_macros: Vec<String>, // CLUTTER_EXPORT, G_MODULE_EXPORT, G_DEPRECATED_FOR, etc.
    pub has_static_forward_decl: bool, // Has a static forward declaration in the same file
    pub is_definition: bool,        // true = definition, false = declaration
    pub return_type: Option<String>,
    pub parameters: Vec<Parameter>,
    /// Byte range of the entire function (for definitions) - use with
    /// FileModel.source
    pub start_byte: Option<usize>,
    pub end_byte: Option<usize>,
    /// Byte range of just the function body (for definitions) - use with
    /// FileModel.source
    pub body_start_byte: Option<usize>,
    pub body_end_byte: Option<usize>,
    /// Parsed body statements (for definitions) - ordered list
    pub body_statements: Vec<Statement>,
}

impl FunctionInfo {
    /// Find all calls to specific functions in the body
    /// Returns references to all CallExpression nodes that match any of the
    /// given function names
    pub fn find_calls<'a>(&'a self, function_names: &[&str]) -> Vec<&'a CallExpression> {
        let mut calls = Vec::new();
        self.find_calls_recursive(&self.body_statements, function_names, &mut calls);
        calls
    }

    fn find_calls_recursive<'a>(
        &'a self,
        statements: &'a [Statement],
        function_names: &[&str],
        calls: &mut Vec<&'a CallExpression>,
    ) {
        for stmt in statements {
            match stmt {
                Statement::Expression(expr_stmt) => {
                    self.find_calls_in_expr(&expr_stmt.expr, function_names, calls);
                }
                Statement::Return(ret) => {
                    if let Some(expr) = &ret.value {
                        self.find_calls_in_expr(expr, function_names, calls);
                    }
                }
                Statement::Declaration(decl) => {
                    if let Some(expr) = &decl.initializer {
                        self.find_calls_in_expr(expr, function_names, calls);
                    }
                }
                Statement::If(if_stmt) => {
                    self.find_calls_in_expr(&if_stmt.condition, function_names, calls);
                    self.find_calls_recursive(&if_stmt.then_body, function_names, calls);
                    if let Some(else_body) = &if_stmt.else_body {
                        self.find_calls_recursive(else_body, function_names, calls);
                    }
                }
                Statement::Compound(compound) => {
                    self.find_calls_recursive(&compound.statements, function_names, calls);
                }
                Statement::Labeled(labeled) => {
                    self.find_calls_recursive(
                        std::slice::from_ref(&labeled.statement),
                        function_names,
                        calls,
                    );
                }
                _ => {}
            }
        }
    }

    fn find_calls_in_expr<'a>(
        &'a self,
        expr: &'a Expression,
        function_names: &[&str],
        calls: &mut Vec<&'a CallExpression>,
    ) {
        match expr {
            Expression::Call(call) => {
                if function_names.contains(&call.function.as_str()) {
                    calls.push(call);
                }
                // Also check arguments
                for arg in &call.arguments {
                    let Argument::Expression(e) = arg;
                    self.find_calls_in_expr(e, function_names, calls);
                }
            }
            Expression::Assignment(assign) => {
                self.find_calls_in_expr(&assign.rhs, function_names, calls);
            }
            Expression::Binary(binary) => {
                self.find_calls_in_expr(&binary.left, function_names, calls);
                self.find_calls_in_expr(&binary.right, function_names, calls);
            }
            Expression::Unary(unary) => {
                self.find_calls_in_expr(&unary.operand, function_names, calls);
            }
            _ => {}
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Parameter {
    pub name: Option<String>,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructInfo {
    pub name: String,
    pub line: usize,
    pub fields: Vec<Field>,
    pub is_opaque: bool, // Only declared, not defined
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Field {
    pub name: String,
    pub type_name: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumInfo {
    pub name: String,
    pub line: usize,
    pub values: Vec<EnumValue>,
    /// Byte range of the enum body for inserting fixes
    pub body_start_byte: usize,
    pub body_end_byte: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnumValue {
    pub name: String,
    pub value: Option<i64>,
    /// Byte range of this enumerator node
    pub start_byte: usize,
    pub end_byte: usize,
    /// Byte range of just the name
    pub name_start_byte: usize,
    pub name_end_byte: usize,
    /// Byte range of the value (if present)
    pub value_start_byte: Option<usize>,
    pub value_end_byte: Option<usize>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TypedefInfo {
    pub name: String,
    pub line: usize,
    pub target_type: String,
}

// ============================================================================
// Statement and Expression AST
// ============================================================================

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Declaration(VariableDecl),
    Expression(ExpressionStmt),
    If(IfStatement),
    Return(ReturnStatement),
    Goto(GotoStatement),
    Labeled(LabeledStatement),
    Compound(CompoundStatement),
}

impl Statement {
    /// Recursively visit all nested statements
    pub fn walk<F>(&self, f: &mut F)
    where
        F: FnMut(&Statement),
    {
        f(self);
        match self {
            Statement::If(if_stmt) => {
                for stmt in &if_stmt.then_body {
                    stmt.walk(f);
                }
                if let Some(else_body) = &if_stmt.else_body {
                    for stmt in else_body {
                        stmt.walk(f);
                    }
                }
            }
            Statement::Compound(compound) => {
                for stmt in &compound.statements {
                    stmt.walk(f);
                }
            }
            Statement::Labeled(labeled) => {
                labeled.statement.walk(f);
            }
            _ => {}
        }
    }

    /// Get all direct expressions contained in this statement (non-recursive)
    pub fn expressions(&self) -> Vec<&Expression> {
        match self {
            Statement::Expression(expr_stmt) => vec![&expr_stmt.expr],
            Statement::Return(ret) => ret.value.as_ref().into_iter().collect(),
            Statement::Declaration(decl) => decl.initializer.as_ref().into_iter().collect(),
            _ => vec![],
        }
    }

    pub fn location(&self) -> &SourceLocation {
        match self {
            Statement::Declaration(d) => &d.location,
            Statement::Expression(e) => &e.location,
            Statement::If(i) => &i.location,
            Statement::Return(r) => &r.location,
            Statement::Goto(g) => &g.location,
            Statement::Labeled(l) => &l.location,
            Statement::Compound(c) => &c.location,
        }
    }

    /// Recursively walk all expressions in this statement tree
    /// Visits each expression once, including nested expressions within other
    /// expressions
    pub fn walk_expressions<F>(&self, f: &mut F)
    where
        F: FnMut(&Expression),
    {
        // Visit direct expressions in this statement
        for expr in self.expressions() {
            f(expr);
        }

        // Recurse into nested statements
        match self {
            Statement::If(if_stmt) => {
                f(&if_stmt.condition);
                for stmt in &if_stmt.then_body {
                    stmt.walk_expressions(f);
                }
                if let Some(else_body) = &if_stmt.else_body {
                    for stmt in else_body {
                        stmt.walk_expressions(f);
                    }
                }
            }
            Statement::Compound(compound) => {
                for stmt in &compound.statements {
                    stmt.walk_expressions(f);
                }
            }
            Statement::Labeled(labeled) => {
                labeled.statement.walk_expressions(f);
            }
            _ => {}
        }
    }

    /// Extract the call expression if this is an expression statement with a
    /// call
    pub fn extract_call(&self) -> Option<&CallExpression> {
        if let Statement::Expression(expr_stmt) = self {
            if let Expression::Call(call) = &expr_stmt.expr {
                return Some(call);
            }
        }
        None
    }

    /// Check if this statement assigns a value matching the predicate to the
    /// target variable
    pub fn is_assignment_to<F>(&self, target_var: &str, value_check: F) -> bool
    where
        F: Fn(&Expression) -> bool,
    {
        if let Statement::Expression(expr_stmt) = self {
            if let Expression::Assignment(assign) = &expr_stmt.expr {
                return assign.lhs.trim() == target_var.trim() && value_check(&assign.rhs);
            }
        }
        false
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpressionStmt {
    pub expr: Expression,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentifierExpression {
    pub name: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldAccessExpression {
    pub text: String, // Full text like "self->field" or "obj.field"
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringLiteralExpression {
    pub value: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NumberLiteralExpression {
    pub value: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NullExpression {
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BooleanExpression {
    pub value: bool,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CastExpression {
    pub type_name: String,
    pub operand: Box<Expression>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConditionalExpression {
    pub condition: Box<Expression>,
    pub then_expr: Box<Expression>,
    pub else_expr: Box<Expression>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SizeofExpression {
    pub text: String, // Full text like "sizeof(int)" or "sizeof x"
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SubscriptExpression {
    pub array: Box<Expression>,
    pub index: Box<Expression>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InitializerListExpression {
    pub text: String, // Full text like "{1, 2, 3}" or "{.x = 1, .y = 2}"
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharLiteralExpression {
    pub value: String, // Like "'a'" or "'\\n'"
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UpdateExpression {
    pub operator: String, // "++", "--"
    pub operand: Box<Expression>,
    pub is_prefix: bool, // true for ++x, false for x++
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expression {
    Call(CallExpression),
    Assignment(Assignment),
    Binary(BinaryExpression),
    Unary(UnaryExpression),
    Identifier(IdentifierExpression),
    FieldAccess(FieldAccessExpression),
    StringLiteral(StringLiteralExpression),
    NumberLiteral(NumberLiteralExpression),
    Null(NullExpression),
    Boolean(BooleanExpression),
    Cast(CastExpression),
    Conditional(ConditionalExpression),
    Sizeof(SizeofExpression),
    Subscript(SubscriptExpression),
    InitializerList(InitializerListExpression),
    CharLiteral(CharLiteralExpression),
    Update(UpdateExpression),
}

impl Expression {
    /// Get the byte range of this expression
    pub fn byte_range(&self) -> (usize, usize) {
        match self {
            Expression::Call(c) => (c.location.start_byte, c.location.end_byte),
            Expression::Assignment(a) => (a.location.start_byte, a.location.end_byte),
            Expression::Binary(b) => (b.location.start_byte, b.location.end_byte),
            Expression::Unary(u) => (u.location.start_byte, u.location.end_byte),
            Expression::Identifier(i) => (i.location.start_byte, i.location.end_byte),
            Expression::FieldAccess(f) => (f.location.start_byte, f.location.end_byte),
            Expression::StringLiteral(s) => (s.location.start_byte, s.location.end_byte),
            Expression::NumberLiteral(n) => (n.location.start_byte, n.location.end_byte),
            Expression::Null(n) => (n.location.start_byte, n.location.end_byte),
            Expression::Boolean(b) => (b.location.start_byte, b.location.end_byte),
            Expression::Cast(c) => (c.location.start_byte, c.location.end_byte),
            Expression::Conditional(c) => (c.location.start_byte, c.location.end_byte),
            Expression::Sizeof(s) => (s.location.start_byte, s.location.end_byte),
            Expression::Subscript(s) => (s.location.start_byte, s.location.end_byte),
            Expression::InitializerList(i) => (i.location.start_byte, i.location.end_byte),
            Expression::CharLiteral(c) => (c.location.start_byte, c.location.end_byte),
            Expression::Update(u) => (u.location.start_byte, u.location.end_byte),
        }
    }

    pub fn location(&self) -> &SourceLocation {
        match self {
            Expression::Call(c) => &c.location,
            Expression::Assignment(a) => &a.location,
            Expression::Binary(b) => &b.location,
            Expression::Unary(u) => &u.location,
            Expression::Identifier(i) => &i.location,
            Expression::FieldAccess(f) => &f.location,
            Expression::StringLiteral(s) => &s.location,
            Expression::NumberLiteral(n) => &n.location,
            Expression::Null(n) => &n.location,
            Expression::Boolean(b) => &b.location,
            Expression::Cast(c) => &c.location,
            Expression::Conditional(c) => &c.location,
            Expression::Sizeof(s) => &s.location,
            Expression::Subscript(s) => &s.location,
            Expression::InitializerList(i) => &i.location,
            Expression::CharLiteral(c) => &c.location,
            Expression::Update(u) => &u.location,
        }
    }

    /// Convert this expression back to source text
    pub fn to_source_string(&self, source: &[u8]) -> Option<String> {
        let (start, end) = self.byte_range();
        std::str::from_utf8(&source[start..end])
            .ok()
            .map(ToOwned::to_owned)
    }

    /// Recursively walk all nested expressions
    pub fn walk<F>(&self, f: &mut F)
    where
        F: FnMut(&Expression),
    {
        f(self);
        match self {
            Expression::Call(call) => {
                for arg in &call.arguments {
                    let Argument::Expression(e) = arg;
                    e.walk(f);
                }
            }
            Expression::Assignment(assign) => {
                assign.rhs.walk(f);
            }
            Expression::Unary(unary) => {
                unary.operand.walk(f);
            }
            Expression::Binary(binary) => {
                binary.left.walk(f);
                binary.right.walk(f);
            }
            Expression::Cast(cast) => {
                cast.operand.walk(f);
            }
            Expression::Conditional(cond) => {
                cond.condition.walk(f);
                cond.then_expr.walk(f);
                cond.else_expr.walk(f);
            }
            Expression::Subscript(subscript) => {
                subscript.array.walk(f);
                subscript.index.walk(f);
            }
            Expression::Update(update) => {
                update.operand.walk(f);
            }
            _ => {}
        }
    }

    /// Extract variable name from simple expressions (Identifier or
    /// FieldAccess)
    pub fn extract_variable_name(&self) -> Option<String> {
        match self {
            Expression::Identifier(id) => Some(id.name.clone()),
            Expression::FieldAccess(f) => Some(f.text.clone()),
            _ => None,
        }
    }

    /// Check if this expression is NULL
    pub fn is_null(&self) -> bool {
        matches!(self, Expression::Null(_))
    }

    /// Check if this expression is the number 0
    pub fn is_zero(&self) -> bool {
        matches!(self, Expression::NumberLiteral(n) if n.value.trim() == "0")
    }

    /// Check if this expression is a string literal
    pub fn is_string_literal(&self) -> bool {
        matches!(self, Expression::StringLiteral(_))
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallExpression {
    pub function: String,
    pub arguments: Vec<Argument>,
    pub location: SourceLocation,
}

impl CallExpression {
    /// Get argument as source text
    pub fn get_arg_text(&self, index: usize, source: &[u8]) -> Option<String> {
        self.arguments.get(index)?.to_source_string(source)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Argument {
    Expression(Box<Expression>),
    // Add more specific types as needed
}

impl Argument {
    /// Convert this argument back to source text
    pub fn to_source_string(&self, source: &[u8]) -> Option<String> {
        match self {
            Argument::Expression(expr) => expr.to_source_string(source),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub lhs: String,      // Keep simple for now - just variable name
    pub operator: String, // "=", "+=", etc.
    pub rhs: Box<Expression>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BinaryExpression {
    pub left: Box<Expression>,
    pub operator: String,
    pub right: Box<Expression>,
    pub location: SourceLocation,
}

impl BinaryExpression {
    /// Check if this is a NULL comparison (x != NULL, x == NULL, etc.)
    pub fn is_null_check(&self) -> bool {
        (self.operator == "==" || self.operator == "!=")
            && (self.left.is_null() || self.right.is_null())
    }

    /// Extract the variable being compared in expressions like `x != 0`, `x >
    /// 0`, `0 < x`
    pub fn extract_compared_variable(&self) -> Option<String> {
        let left_is_zero = self.left.is_zero();
        let right_is_zero = self.right.is_zero();

        match self.operator.as_str() {
            "!=" | "==" | ">" | ">=" => {
                if right_is_zero {
                    self.left.extract_variable_name()
                } else if left_is_zero {
                    self.right.extract_variable_name()
                } else {
                    None
                }
            }
            "<" | "<=" => {
                if left_is_zero {
                    self.right.extract_variable_name()
                } else if right_is_zero {
                    self.left.extract_variable_name()
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnaryExpression {
    pub operator: String,
    pub operand: Box<Expression>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableDecl {
    pub type_name: String,
    pub name: String,
    pub initializer: Option<Expression>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IfStatement {
    pub condition: Expression,
    pub then_body: Vec<Statement>,
    pub then_has_braces: bool,
    pub else_body: Option<Vec<Statement>>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ReturnStatement {
    pub value: Option<Expression>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GotoStatement {
    pub label: String,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LabeledStatement {
    pub label: String,
    pub statement: Box<Statement>,
    pub location: SourceLocation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompoundStatement {
    pub statements: Vec<Statement>,
    pub location: SourceLocation,
}
