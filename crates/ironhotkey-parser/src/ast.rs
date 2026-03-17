use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct Script {
    pub directives: Vec<Directive>,
    pub auto_exec: Vec<Statement>,
    pub hotkeys: Vec<Hotkey>,
    pub hotstrings: Vec<Hotstring>,
    pub functions: Vec<Function>,
    pub classes: Vec<Class>,
    pub labels: Vec<Label>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Statement {
    Assignment {
        target: Expr,
        op: AssignOp,
        value: Expr,
    },
    Command {
        name: String,
        args: Vec<CommandArg>,
    },
    ExprStatement(Expr),
    If {
        condition: Expr,
        body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    IfLegacy {
        variant: LegacyIfVariant,
        var: String,
        values: Vec<Expr>,
        body: Vec<Statement>,
        else_body: Option<Vec<Statement>>,
    },
    Loop {
        variant: LoopVariant,
        body: Vec<Statement>,
    },
    While {
        condition: Expr,
        body: Vec<Statement>,
    },
    For {
        key: String,
        value: Option<String>,
        iterable: Expr,
        body: Vec<Statement>,
    },
    Break {
        label: Option<String>,
    },
    Continue {
        label: Option<String>,
    },
    Until(Expr),
    Return(Option<Expr>),
    Goto(String),
    Gosub(String),
    Block(Vec<Statement>),
    Switch {
        value: Expr,
        cases: Vec<SwitchCase>,
    },
    Try {
        body: Vec<Statement>,
        catch: Option<CatchClause>,
        finally: Option<Vec<Statement>>,
    },
    Throw(Expr),
    VarDecl {
        scope: VarScope,
        declarations: Vec<(String, Option<Expr>)>,
    },
    Empty,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Expr {
    StringLiteral(String),
    NumberLiteral(f64),
    Variable(String),
    Deref(String),
    DoubleDeref(Vec<DerefPart>),
    UnaryOp {
        op: UnaryOp,
        operand: Box<Expr>,
    },
    BinaryOp {
        left: Box<Expr>,
        op: BinaryOp,
        right: Box<Expr>,
    },
    Ternary {
        condition: Box<Expr>,
        then_branch: Box<Expr>,
        else_branch: Box<Expr>,
    },
    FunctionCall {
        name: Box<Expr>,
        args: Vec<Expr>,
    },
    MethodCall {
        object: Box<Expr>,
        method: String,
        args: Vec<Expr>,
    },
    PropertyAccess {
        object: Box<Expr>,
        property: String,
    },
    IndexAccess {
        object: Box<Expr>,
        indices: Vec<Expr>,
    },
    ObjectLiteral(Vec<(Expr, Expr)>),
    ArrayLiteral(Vec<Expr>),
    NewObject {
        class: Box<Expr>,
        args: Vec<Expr>,
    },
    Concatenation(Vec<Expr>),
    RegexMatch {
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Comma(Vec<Expr>),
    Base,
    This,
    Variadic(Box<Expr>),
    True,
    False,
    Null,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Function {
    pub name: String,
    pub params: Vec<Param>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Param {
    pub name: String,
    pub is_byref: bool,
    pub default: Option<Expr>,
    pub is_variadic: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Class {
    pub name: String,
    pub extends: Option<String>,
    pub body: Vec<ClassMember>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ClassMember {
    Method(Function),
    Property {
        name: String,
        getter: Vec<Statement>,
        setter: Vec<Statement>,
    },
    InstanceVar {
        name: String,
        value: Option<Expr>,
    },
    StaticVar {
        name: String,
        value: Option<Expr>,
    },
    NestedClass(Class),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotkey {
    pub modifiers: Vec<Modifier>,
    pub key: String,
    pub custom_combo: Option<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum Modifier {
    Win,
    Alt,
    Ctrl,
    Shift,
    Left,
    Right,
    Wildcard,
    PassThrough,
    Hook,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Hotstring {
    pub options: String,
    pub trigger: String,
    pub replacement: HotstringAction,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HotstringAction {
    Text(String),
    Command(Vec<Statement>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Directive {
    pub name: String,
    pub args: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Label {
    pub name: String,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CommandArg {
    Literal(String),
    Expression(Expr),
    OutputVar(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AssignOp {
    Legacy,
    Expr,
    Add,
    Sub,
    Mul,
    Div,
    FloorDiv,
    Concat,
    BitOr,
    BitAnd,
    BitXor,
    ShiftRight,
    ShiftLeft,
    ShiftRightLogical,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BinaryOp {
    Pow,
    Mul,
    Div,
    FloorDiv,
    Add,
    Sub,
    ShiftLeft,
    ShiftRight,
    ShiftRightLogical,
    BitAnd,
    BitXor,
    BitOr,
    RegexMatch,
    Concat,
    Eq,
    StrictEq,
    Neq,
    StrictNeq,
    Lt,
    Lte,
    Gt,
    Gte,
    And,
    Or,
    Comma,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnaryOp {
    Neg,
    Not,
    BitNot,
    LogicalNotKeyword,
    AddressOf,
    Deref,
    PreInc,
    PreDec,
    PostInc,
    PostDec,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VarScope {
    Global,
    Local,
    Static,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LoopVariant {
    Count(Option<Expr>),
    Parse {
        string: Expr,
        delimiters: Option<Expr>,
    },
    File {
        pattern: Expr,
        mode: Option<Expr>,
    },
    Read {
        file: Expr,
    },
    Reg {
        root_key: Expr,
        key: Option<Expr>,
        mode: Option<Expr>,
    },
    Infinite,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LegacyIfVariant {
    Equal,
    NotEqual,
    Between,
    NotBetween,
    In,
    NotIn,
    Contains,
    NotContains,
    Is,
    IsNot,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SwitchCase {
    pub values: Vec<Expr>,
    pub body: Vec<Statement>,
    pub is_default: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatchClause {
    pub var: Option<String>,
    pub body: Vec<Statement>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DerefPart {
    Literal(String),
    Variable(String),
}
