use std::fmt::Debug;

#[derive(Clone)]
pub struct Type {
    pub name: String,
    pub is_array: bool,
}

impl PartialEq<Type> for Type {
    fn eq(&self, other: &Type) -> bool {
        return self.name == other.name && self.is_array == other.is_array;
    }
}

impl Debug for Type {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self.is_array {
            true => write!(f, "{}[]", self.name),
            false => write!(f, "{}", self.name),
        }
    }
}

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub typename: Option<Type>,
}

impl Debug for Variable {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match &self.typename {
            Some(val) => write!(f, "{}: {:?}", self.name, val),
            None => write!(f, "{}", self.name),
        };
    }
}

impl PartialEq<Variable> for Variable {
    fn eq(&self, other: &Variable) -> bool {
        return self.name == other.name && self.typename == other.typename;
    }
}

#[derive(Clone)]
pub enum Ast {
    Global(Vec<Ast>),
    FunctionHeader{
        name: String,
        parameters: Vec<Variable>,
        return_type: Option<String>,
    },
    FunctionDeclaration{
        name: String,
        children: Vec<Ast>,
        parameters: Vec<Variable>,
        return_type: Option<String>,
    },
    FunctionCall{
        name: String,
        children: Vec<Ast>
    },
    Int(i64),
    Float(f64),
    Str(String),
    Bool(bool),
    ArrayValue(Vec<Ast>),
    Assignement{
        variable: Box<Ast>,
        expression: Box<Ast>,
    },
    Condition {
        condition: Box<Ast>,
        valid_branch: Vec<Ast>,
        invalid_branch: Vec<Ast>,
    },
    WhileLoop {
        condition: Box<Ast>,
        children: Vec<Ast>,
    },
    Variable(Variable),
    Statement {
        children: Vec<Ast>
    },
    Addition {
        left: Box<Ast>,
        right: Box<Ast>
    },
    UnaryPlus {
        child: Box<Ast>
    },
    UnaryMinus {
        child: Box<Ast>
    },
    Substraction{
        left: Box<Ast>,
        right: Box<Ast>
    },
    Multiplication{
        left: Box<Ast>,
        right: Box<Ast>
    },
    Division{
        left: Box<Ast>,
        right: Box<Ast>
    },
    Modulo{
        left: Box<Ast>,
        right: Box<Ast>
    },
    GreaterThan {
        left: Box<Ast>,
        right: Box<Ast>,
    },
    LowerThan {
        left: Box<Ast>,
        right: Box<Ast>,
    },
    GreaterOrEqual {
        left: Box<Ast>,
        right: Box<Ast>,
    },
    LowerOrEqual {
        left: Box<Ast>,
        right: Box<Ast>,
    },
    EqualTo {
        left: Box<Ast>,
        right: Box<Ast>,
    },
    NotEqualTo {
        left: Box<Ast>,
        right: Box<Ast>,
    },
    ReturnStatement(Option<Box<Ast>>),
    ArrayAccess {
        variable: String,
        offset: u64,
    },
}

impl Debug for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Global(children) => {
                for tree in children {
                    match writeln!(f, "{:?}", tree) {
                        Ok(()) => (),
                        Err(e) => return Err(e),
                    };
                }
                return Ok(());
            },
            Self::Int(val) => write!(f, "{}", val),
            Self::Float(val) => write!(f, "{}", val),
            Self::Str(val) => write!(f, "{}", val),
            Self::Bool(val) => write!(f, "{}", val),
            Self::ArrayValue(children) => write!(f, "{:?}", children),
            Self::ArrayAccess { variable, offset } => write!(f, "{}[{}]", variable, offset),
            Self::Addition { left, right } => write!(f, "({:?} + {:?})", left, right),
            Self::Substraction { left, right } => write!(f, "({:?} - {:?})", left, right),
            Self::Multiplication { left, right } => write!(f, "({:?} * {:?})", left, right),
            Self::Division { left, right } => write!(f, "({:?} / {:?})", left, right),
            Self::UnaryPlus { child } => write!(f, "(+{:?})", child),
            Self::UnaryMinus { child } => write!(f, "(-{:?})", child),
            Self::Variable(var)  => write!(f, "{:?}", var),
            Self::FunctionCall { name, children } => write!(f, "<FunctionCall name={:?}, params={:?} />", name, children),
            Self::Assignement { variable, expression } => write!(f, "<Assignement variable={:?}, expression={:?} />", variable, expression),
            Self::EqualTo { left, right } => write!(f, "({:?} == {:?})", left, right),
            Self::NotEqualTo { left, right } => write!(f, "({:?} != {:?})", left, right),
            Self::GreaterThan { left, right } => write!(f, "({:?} > {:?})", left, right),
            Self::LowerThan { left, right } => write!(f, "({:?} < {:?})", left, right),
            Self::GreaterOrEqual { left, right } => write!(f, "({:?} >= {:?})", left, right),
            Self::LowerOrEqual { left, right } => write!(f, "({:?} <= {:?})", left, right),
            Self::Condition { condition, valid_branch, invalid_branch } =>
                write!(f, "<Condition condition={:?} then={:?} else={:?} />", condition, valid_branch, invalid_branch),
            Self::WhileLoop { condition, children } =>
                write!(f, "<While condition={:?} children={:?} />", condition, children),
            Self::ReturnStatement(ast) => write!(f, "<Return {:?} />", ast),
            Self::FunctionDeclaration { name, children, parameters, return_type } =>
                write!(f, "<Function name={:?} parameters={:?} return_type={:?} children={:?} />", name, parameters, return_type, children),
            Self::FunctionHeader { name, parameters, return_type } =>
                write!(f, "<FunctionHeader name={:?} parameters={:?} return_type={:?} />", name, parameters, return_type),
            _ => todo!("ast fmt::Debug not implemented"),
        };
    }
}

