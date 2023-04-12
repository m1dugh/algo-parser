use std::{slice::Iter, collections::VecDeque, fmt::Debug };

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
        variable: Variable,
        expression: Vec<Ast>,
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
}

impl Debug for Ast {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        return match self {
            Self::Global(children) => write!(f, "<Global {:?}>", children),
            Self::Int(val) => write!(f, "{}", val),
            Self::Addition { left, right } => write!(f, "({:?} + {:?})", left, right),
            Self::Substraction { left, right } => write!(f, "({:?} - {:?})", left, right),
            Self::Multiplication { left, right } => write!(f, "({:?} * {:?})", left, right),
            Self::Division { left, right } => write!(f, "({:?} / {:?})", left, right),
            Self::UnaryPlus { child } => write!(f, "(+{:?})", child),
            Self::UnaryMinus { child } => write!(f, "(-{:?})", child),
            Self::Symbol(var) => write!(f, "Symbol({})", var),
            Self::FunctionCall { name, children } => write!(f, "<FunctionCall name={:?}, params={:?} />", name, children),

            _ => todo!(),
        };
    }
}

pub fn load_ast(tokens: &Vec<TokenType>) -> Result<Ast, String> {


    return match build_ast(&mut tokens.iter()) {
        Err(e) => Err(e),
        Ok(children) => Ok(Ast::Global(children))
    };
}

fn build_conditional_ast(tokens: &mut Iter<TokenType>) -> Result<Ast, String> {

    return Err(String::new());
}

fn get_operator_precedency(operator: &TokenType) -> i64 {

    return match operator {
        TokenType::UnaryOperator(_) => 4,
        TokenType::BinaryOperator(val) => {
            match val.as_str() {
                "+" | "-"   => 1,
                "*" | "/"   => 3,
                "%"         => 2,
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
        return Err(format!("invalid expression in create_binary_operator_ast"));
    }
    let el1 = Box::new(output_stack.pop().unwrap());
    let el2 = Box::new(output_stack.pop().unwrap());
    output_stack.push(match operator_str {
        "+" => Ast::Addition {
            left: el2,
            right: el1,
        },
        "-" => Ast::Substraction {
            left: el2,
            right: el1,
        },
        "*" => Ast::Multiplication {
            left: el2,
            right: el1,
        },
        "/" => Ast::Division {
            left: el2,
            right: el1,
        },
        "%" | _ => Ast::Modulo {
            left: el2,
            right: el1,
        },
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
pub fn build_expression_ast(tokens: &mut Iter<TokenType>) -> Result<Ast, String> {

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

fn build_ast(tokens: &mut Iter<TokenType>) -> Result<Vec<Ast>, String> {
    let mut children: Vec<Ast> = Vec::new();

    for token in &tokens.next() {
        match token {
            TokenType::Keyword(val) if val == "if" => {
                match build_conditional_ast(tokens) {
                    Ok(child) => children.push(child),
                    Err(e) => return Err(e),
                };
            },
            TokenType::EndLine => {
                break
            },

            _ => (),
        };
    }

    return Ok(children);
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
