use std::{slice::Iter, collections::VecDeque, fmt::Debug, iter::Peekable };

use super::lexer::TokenType;

#[derive(Clone)]
pub struct Variable {
    pub name: String,
    pub var_type: Type,
}

impl PartialEq<Variable> for Variable {
    fn eq(&self, other: &Variable) -> bool {
        return self.name == other.name && self.var_type == other.var_type;
    }
}

#[derive(Clone)]
pub struct Type {
    pub name: &'static str,
    pub size: u64,
    pub fields: Option<Vec<Variable>>,
}

const STR_TYPE: Type = Type{
    name: "str",
    size: 8,
    fields: None,
};

const INT_TYPE: Type = Type{
    name: "int",
    size: 8,
    fields: None,
};

const FLOAT_TYPE: Type = Type{
    name: "float",
    size: 8,
    fields: None,
};

impl PartialEq<Type> for Type {
    fn eq(&self, other: &Type) -> bool {
        if self.name == other.name && self.size == other.size {
            match &self.fields {
                None => {
                    return match other.fields {
                        None => true,
                        Some(_) => false,
                    };
                },
                Some(fields) => {
                    return match &other.fields {
                        None => false,
                        Some(other_fields) => fields == other_fields,
                    }
                }
            }
        } else {
            return false;
        }
    }
}

#[derive(Clone)]
pub enum Ast {
    Global(Vec<Ast>),
    FunctionDeclaration{
        name: String,
        children: Vec<Ast>,
        parameters: Vec<Variable>,
    },
    FunctionCall{
        name: String,
        children: Vec<Ast>
    },
    Int(i64),
    Float(f64),
    Str(String),
    Assignement{
        variable: String,
        expression: Box<Ast>,
    },
    Condition {
        condition: Box<Ast>,
        valid_branch: Vec<Ast>,
        invalid_branch: Vec<Ast>,
    },
    Symbol(String),
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
            Self::Addition { left, right } => write!(f, "({:?} + {:?})", left, right),
            Self::Substraction { left, right } => write!(f, "({:?} - {:?})", left, right),
            Self::Multiplication { left, right } => write!(f, "({:?} * {:?})", left, right),
            Self::Division { left, right } => write!(f, "({:?} / {:?})", left, right),
            Self::UnaryPlus { child } => write!(f, "(+{:?})", child),
            Self::UnaryMinus { child } => write!(f, "(-{:?})", child),
            Self::Symbol(var) => write!(f, "Symbol({})", var),
            Self::FunctionCall { name, children } => write!(f, "<FunctionCall name={:?}, params={:?} />", name, children),
            Self::Assignement { variable, expression } => write!(f, "<Assignement variable={:?}, expression={:?} />", variable, expression),
            Self::EqualTo { left, right } => write!(f, "({:?} == {:?})", left, right),
            Self::NotEqualTo { left, right } => write!(f, "({:?} != {:?})", left, right),
            Self::GreaterThan { left, right } => write!(f, "({:?} > {:?})", left, right),
            Self::LowerThan { left, right } => write!(f, "({:?} < {:?})", left, right),
            Self::Condition { condition, valid_branch, invalid_branch } =>
                write!(f, "<Condition condition={:?} then={:?} else={:?} />", condition, valid_branch, invalid_branch),

            _ => todo!("ast Debug::fmt not implemented"),
        };
    }
}

pub fn load_ast(tokens: &Vec<TokenType>) -> Result<Ast, String> {

    let mut token_iter = tokens.iter().peekable();
    let mut children = Vec::<Ast>::new();
    while let Some(_) = token_iter.peek() {
        if let Some(child) = build_ast(&mut token_iter) {
            match child {
                Err(e) => return Err(e),
                Ok(child) => children.push(child),
            };
        }
    }

    return Ok(Ast::Global(children));
}

fn build_conditional_ast(tokens: &mut Peekable<Iter<TokenType>>, nested_if: bool) -> Result<Ast, String> {

    let condition = Box::new(match build_expression_ast(tokens) {
        Err(e) => return Err(e),
        Ok(condition) => condition,
    });

    let mut has_else_statement = false;
    let mut valid_branch_children = Vec::<Ast>::new();

    loop {
        let token = match tokens.peek() {
            Some(token) => token,
            None => return Err(String::from("parser: unfinished if statement")),
        };

        match token {
            TokenType::Keyword(val) if val == "else" => {
                tokens.next();
                has_else_statement = true;
                break;
            },
            TokenType::Keyword(val) if val == "end" => {
                if !nested_if {
                    tokens.next();
                }
                break;
            },
            _ => {
                match build_ast(tokens) {
                    None => (),
                    Some(result) => {
                        valid_branch_children.push(match result {
                            Ok(child) => child,
                            Err(e) => return Err(e),
                        });
                    },
                };
            }
        }
    };

    let mut invalid_branch_children = Vec::<Ast>::new();
    while has_else_statement {
        let token = match tokens.peek() {
            Some(token) => token,
            None => return Err(String::from("parser: unfinished if-else statement")),
        };

        match token {
            TokenType::Keyword(val) if val == "end" => {
                if !nested_if {
                    tokens.next();
                }
                break;
            },
            TokenType::EndLine => {
                tokens.next();
            },
            TokenType::Keyword(val) if val == "if" => {
                tokens.next();
                invalid_branch_children.push(match build_conditional_ast(tokens, true) {
                    Ok(child) => child,
                    Err(e) => return Err(e),
                });
            }
            _ => {
                match build_ast(tokens) {
                    Some(result) => invalid_branch_children.push(match result {
                        Ok(child) => child,
                        Err(e) => return Err(e),
                    }),
                    None => (),
                };
            }
        }
    }

    return Ok(Ast::Condition {
        condition,
        valid_branch: valid_branch_children,
        invalid_branch: invalid_branch_children,
    });
}

fn get_operator_precedency(operator: &TokenType) -> i64 {

    return match operator {
        TokenType::UnaryOperator(_) => 4,
        TokenType::BinaryOperator(val) => {
            match val.as_str() {
                "+" | "-"   => 1,
                "*" | "/"   => 3,
                "%"         => 2,
                "<-"        => 0,
                _ => -1,
            }
        },
        _ => -1,
    };
}

fn build_assignement_ast(buffer: &Vec<TokenType>, tokens: &mut Iter<TokenType>) -> Result<Ast, String> {

    return Err(String::from("TODO: implement assignement ast"));
}

fn can_cast_type(origin_type: &Type, target_type: &Type) -> Result<Type, String> {
    if origin_type == target_type {
        return Ok(target_type.clone());
    }

    if origin_type == &INT_TYPE && (target_type == &INT_TYPE || target_type == &FLOAT_TYPE) {
        return Ok(target_type.clone());
    }

    if origin_type == &FLOAT_TYPE && (target_type == &INT_TYPE || target_type == &FLOAT_TYPE) {
        return Ok(origin_type.clone());
    }

    return Err(format!("cannot auto cast '{}' into '{}'", origin_type.name, target_type.name));
}

fn create_binary_operator_ast(operator_str: &str, output_stack: &mut Vec<Ast>) -> Result<(), String> {
    if output_stack.len() < 2 {
        return Err(format!("invalid expression in create_binary_operator_ast, missing value for operator {}", operator_str));
    }
    let el1 = output_stack.pop().unwrap();
    let el2 = output_stack.pop().unwrap();
    let left = Box::new(el2);
    let right = Box::new(el1);
    output_stack.push(match operator_str {
        "+" => Ast::Addition { left, right },
        "-" => Ast::Substraction { left, right },
        "*" => Ast::Multiplication { left, right },
        "/" => Ast::Division { left, right },
        "<-" => {
            match *left {
                Ast::Symbol(name) => Ast::Assignement {
                    variable: name,
                    expression: right,
                },
                _ => return Err(format!("parser: can only assign value to variable")),
            }
        },
        "%" => Ast::Modulo { left, right },
        "==" => Ast::EqualTo { left, right },
        "!=" => Ast::NotEqualTo { left, right },
        ">" => Ast::GreaterThan { left, right },
        "<" => Ast::LowerThan { left, right },
        op => return Err(format!("parser: missing implementation for operator '{}'", op)),
    });

    return Ok(());
}

fn create_function_ast(function_name: &str, output_stack: &mut Vec<Ast>) -> Result<(), String> {
    let mut children = Vec::<Ast>::new();
    loop {
        let child = match output_stack.pop() {
            Some(c) => c,
            None => {
                break;
            },
        };

        match child {
            Ast::FunctionCall { name: _name, children: _children } => {
                children.reverse();
                output_stack.push(Ast::FunctionCall {
                    name: function_name.to_string(),
                    children: children.clone(),
                });
                return Ok(());
            },
            val => {
                children.push(val.clone());
            },
        };
    }
    return Err(String::from("missing function call."));
}

fn create_unary_operator_ast(operator_str: &str, output_stack: &mut Vec<Ast>) -> Result<(), String> {
    let el1 = Box::new(match output_stack.pop() {
        Some(o) => o,
        None => return Err(String::from("invalid expression in create_unary_operator_ast")),
    });
    output_stack.push(match operator_str {
        "+" => Ast::UnaryPlus {
            child: el1,
        },
        "-" | _ => Ast::UnaryMinus {
            child: el1,
        },
    });

    return Ok(());
}
pub fn build_expression_ast(tokens: &mut Peekable<Iter<TokenType>>) -> Result<Ast, String> {

    let mut output_stack = Vec::<Ast>::new();
    let mut operator_stack = Vec::<TokenType>::new();

    loop {
        let token = match tokens.next() {
            Some(token) => token,
            None => {
                return Err(format!("missing token"));
            },
        };

        match token {
            TokenType::Int(val) => {
                output_stack.push(Ast::Int(val.clone()));
            },
            TokenType::Float(val) => {
                output_stack.push(Ast::Float(val.clone()));
            },
            TokenType::Variable(name) => {
                output_stack.push(Ast::Symbol(name.clone()));
            },
            TokenType::FunctionCall(val) => {
                operator_stack.push(token.clone());
                output_stack.push(Ast::FunctionCall {
                    name: val.clone(),
                    children: Vec::new(),
                });
            },
            TokenType::UnaryOperator(_) | TokenType::BinaryOperator(_) => {
                let precedency = get_operator_precedency(&token.clone());
                loop {
                    let operator = match operator_stack.last() {
                        None => {
                            break
                        },
                        Some(operator) => operator,
                    };

                    match operator.clone() {
                        TokenType::BinaryOperator(val) if get_operator_precedency(&operator) >= precedency => {
                            operator_stack.pop();
                            if let Err(e) = create_binary_operator_ast(val.as_str(), &mut output_stack) {
                                return Err(e);
                            }
                        },
                        TokenType::UnaryOperator(val) if get_operator_precedency(&operator) > precedency => {
                            operator_stack.pop();
                            if let Err(e) = create_unary_operator_ast(val.as_str(), &mut output_stack) {
                                return Err(e);
                            }
                        },
                        _ => {
                            break;
                        },
                    };
                }
                operator_stack.push(token.clone());
            },
            TokenType::Comma => {
                loop {
                    let operator = match operator_stack.last() {
                        Some(o) => o,
                        None => return Err(String::from("missing left parenthesis")),
                    };
                    match operator {
                        TokenType::BinaryOperator(val) => {
                            if let Err(e) = create_binary_operator_ast(val.as_str(), &mut output_stack) {
                                return Err(e);
                            }
                            operator_stack.pop();
                        },
                        TokenType::UnaryOperator(val) => {
                            if let Err(e) = create_unary_operator_ast(val.as_str(), &mut output_stack) {
                                return Err(e);
                            }
                            operator_stack.pop();
                        },
                        TokenType::OpeningParenthesis | _ => {
                            break;
                        }
                    }
                }
            },
            TokenType::OpeningParenthesis => {
                operator_stack.push(token.clone());
            },
            TokenType::ClosingParenthesis => {
                loop {
                    let operator = match operator_stack.pop() {
                        Some(o) => o,
                        None => return Err(String::from("invalid expression parsing ')' in build_expression_ast")),
                    };

                    match operator {
                        TokenType::UnaryOperator(val) => {
                            if let Err(e) = create_unary_operator_ast(val.as_str(), &mut output_stack) {
                                return Err(e);
                            }
                        },
                        TokenType::BinaryOperator(val) => {
                            if let Err(e) = create_binary_operator_ast(val.as_str(), &mut output_stack) {
                                return Err(e);
                            }
                        },
                        TokenType::OpeningParenthesis | _ => {
                            break;
                        },
                    };
                };

                if let Some(last_token) = operator_stack.last_mut() {
                    if let TokenType::FunctionCall(func_call) = last_token {
                        if let Err(e) = create_function_ast(func_call.as_str(), &mut output_stack) {
                            return Err(e);
                        }
                        operator_stack.pop();
                    }
                }
            },
            TokenType::EndLine => {
                break;
            },
            _ => return Err(format!("invalid token {}", token)),
        }
    }

    while let Some(operator) = operator_stack.pop() {
        match operator {
            TokenType::UnaryOperator(operator_str) => {
                if let Err(e) = create_unary_operator_ast(&operator_str, &mut output_stack) {
                    return Err(e);
                }
            },
            TokenType::BinaryOperator(operator_str) => {
                if let Err(e) = create_binary_operator_ast(&operator_str, &mut output_stack) {
                    return Err(e);
                }
            },
            TokenType::FunctionCall(func_name) => {
                if let Err(e) = create_function_ast(&func_name, &mut output_stack) {
                    return Err(e);
                }
            },
            token => return Err(format!("invalid token {} in build_expression_ast", token)),
        };
    }


    if output_stack.len() != 1 {
        println!("{:?}", output_stack);
        return Err(format!("invalid expression, parsing items in build_expression_ast, expected length of 1, got {}", output_stack.len()));
    }

    return Ok(output_stack.pop().unwrap());
}

fn build_ast(tokens: &mut Peekable<Iter<TokenType>>) -> Option<Result<Ast, String>> {
    let next_token = match tokens.peek() {
        Some(token) => token,
        None => return Some(Err(String::from("missing token"))),
    };
    match next_token {
        TokenType::EndLine => {
            tokens.next();
            return None;
        },
        TokenType::Keyword(val) if val == "if" => {
            tokens.next();
            return Some(build_conditional_ast(tokens, false));
        },
        _ => return Some(build_expression_ast(tokens)),
    };
}


trait Visitor<T> {
    fn visit(&mut self, visited: &Ast) -> T;
}

struct ASTVisitor;

impl Visitor<()> for ASTVisitor {
    fn visit(&mut self, visited: &Ast) {
        match visited {
            Ast::Int(val) => println!("{}", val),
            Ast::Statement{
                children,
            }=> println!("{}", children.len()),
            _ => (),
        };
    }
}

pub fn test() {
    let ast = Ast::Int(3);
    let mut visitor = ASTVisitor{};
    visitor.visit(&ast);
}
