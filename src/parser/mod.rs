use std::{slice::Iter, iter::Peekable };

use super::lexer::TokenType;
mod types;
pub use types::{Ast, Variable, Type};

mod utils;
use utils::get_operator_precedency;

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
        "<-" => match *left {
                Ast::Variable(..) | Ast::ArrayAccess { .. } => Ast::Assignement { variable: left, expression: right },
                _ => return Err(format!("parser: can only assign value to variable")),
        },
        "%" => Ast::Modulo { left, right },
        "==" => Ast::EqualTo { left, right },
        "!=" => Ast::NotEqualTo { left, right },
        ">" => Ast::GreaterThan { left, right },
        "<" => Ast::LowerThan { left, right },
        "<=" => Ast::LowerOrEqual { left, right },
        ">=" => Ast::GreaterOrEqual { left, right },
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

fn parse_function_header(tokens: &mut Peekable<Iter<TokenType>>) -> Result<(String, Vec<Variable>, Option<String>), String> {
    let name: String;
    let mut params = Vec::<Variable>::new();
    let return_type: Option<String>;

    let token = match tokens.next() {
        Some(token) => token,
        None => return Err(String::from("missing name for function")),
    };

    match token {
        TokenType::Variable(func_name) => name = func_name.clone(),
        _ => return Err(format!("invalid token {} for function name", token)),
    };

    let token = match tokens.next() {
        Some(token) => token,
        None => return Err(format!("parser: missing '(' after function declaration ('{}').", name)),
    };

    match token {
        TokenType::OpeningParenthesis => (),
        _ => return Err(format!("parser: expected '(', got {} for function '{}'", token, name)),
    };

    while let Some(token) = tokens.peek() {
        match token {
            TokenType::ClosingParenthesis => {
                tokens.next();
                break;
            },
            TokenType::Comma => {
                tokens.next();
            },
            _ => {
                params.push(match parse_variable(tokens, true) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                });
            },
        }
    }

    let token = match tokens.peek() {
        None => return Err(format!("invalid function declaration for '{}'", name)),
        Some(token) => token,
    };

    match token {
        TokenType::EndLine => return Ok((name, params, None)),
        TokenType::Colon => {
            tokens.next();
        },
        _ => return Err(format!("parser: unexpected token {} in function '{}' declaration", token, name)),
    };

    let token = match tokens.next() {
        None => return Err(format!("parser: unexpected end of document in function declaration '{}'", name)),
        Some(token) => token,
    };

    return_type = Some(match token {
        TokenType::TypeDef(return_type) => return_type.clone(),
        _ => return Err(format!("unexpected token {} in function declaration '{}', expected TypeDef", token, name)),
    });

    let token = match tokens.next() {
        None => return Err(format!("parser: unexpected end of document in function declaration '{}'", name)),
        Some(token) => token,
    };

    return match token {
        TokenType::EndLine => Ok((name, params, return_type)),
        _ => Err(format!("parser: expected end of line, got {} in function declaration '{}'", token, name)),
    };
}

fn build_return_ast(tokens: &mut Peekable<Iter<TokenType>>) -> Result<Ast, String> {
    match tokens.peek() {
        None => return Ok(Ast::ReturnStatement(None)),
        Some(_) => (),
    };

    return match build_expression_ast(tokens) {
        Err(e) => Err(e),
        Ok(ast) => return Ok(Ast::ReturnStatement(Some(Box::new(ast)))),
    };
}

fn build_declaration_ast(tokens: &mut Peekable<Iter<TokenType>>) -> Result<Ast, String> {
    let token = match tokens.next() {
        None => return Err(format!("unexpected end of document after declare keyword")),
        Some(val) => val,
    };

    return match token {
        TokenType::Keyword(val) if val == "function" => build_function_declaration_ast(tokens),
        val => Err(format!("unexpected token {}, after declare keyword", val)),
    };
}

fn build_function_declaration_ast(tokens: &mut Peekable<Iter<TokenType>>) -> Result<Ast, String> {
    let (name, parameters, return_type) = match parse_function_header(tokens) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };
    return Ok(Ast::FunctionHeader { name, parameters, return_type });
}

fn build_function_ast(tokens: &mut Peekable<Iter<TokenType>>) -> Result<Ast, String> {


    let (name, parameters, return_type) = match parse_function_header(tokens) {
        Ok(v) => v,
        Err(e) => return Err(e),
    };

    let mut children = Vec::<Ast>::new();

    loop {
        let token = match tokens.peek() {
            Some(token) => token,
            None => return Err(format!("parser: unexpected end of document parsing function '{}'", name)),
        };
        match token {
            TokenType::Keyword(val) if val == "end" => {
                tokens.next();
                break;
            },
            _ => {
                match build_ast(tokens) {
                    Some(ast) => {
                        children.push(match ast {
                            Ok(ast) => ast,
                            Err(e) => return Err(e),
                        });
                    },
                    None => (),
                };
            },
        };
    };

    return Ok(Ast::FunctionDeclaration {
        name,
        children ,
        parameters,
        return_type,
    });
}

fn parse_variable(tokens: &mut Peekable<Iter<TokenType>>, require_type: bool) -> Result<Variable, String> {
    let mut token = match tokens.next() {
        None => return Err(String::from("missing token for variable")),
        Some(val) => val,
    };

    let var_name: String;

    match token {
        TokenType::Variable(name) => var_name = name.to_string(),
        _ => return Err(format!("parser: invalid token {} for variable declaration.", token)),
    };

    token = match tokens.peek() {
        None => return Ok(Variable { name: var_name, typename: None }),
        Some(token) => token,
    };

    match token {
        TokenType::Colon => tokens.next(),
        _ if !require_type => return Ok(Variable{ name: var_name, typename: None }),
        _ => return Err(format!("missing typedef for variable '{}'", var_name)),
    };

    token = match tokens.next() {
        None => return Err(format!("missing type declaration for variable {}", var_name)),
        Some(token) => token,
    };

    let var_type: Type;
    match token {
        TokenType::TypeDef(name) => var_type = Type {
            name: name.clone(),
            is_array: false,
        },
        TokenType::ArrayTypeDef(name) => var_type = Type{
            name: name.clone(),
            is_array: true,
        },
        _ => return Err(format!("parser: invalid type token {} for variable '{}'", token, var_name)),
    };

    return Ok(Variable { name: var_name, typename: Some(var_type) });
}

fn build_array_value_ast(tokens: &mut Peekable<Iter<TokenType>>) -> Result<Ast, String> {

    let mut buffer = Vec::<TokenType>::new();
    let mut result = Vec::<Ast>::new();

    loop {
        let token = match tokens.peek() {
            Some(token) => token,
            None => return Err(String::from("parser: unexpected end of document in build_array_value_ast")),
        };

        match *token {
            TokenType::Comma => {
                tokens.next();
                buffer.push(TokenType::EndLine);
                match build_expression_ast(&mut buffer.iter().peekable()) {
                    Ok(child) => result.push(child),
                    Err(e) => return Err(e),
                };
                buffer.clear();
            },
            TokenType::ClosingBracket => {
                tokens.next();
                buffer.push(TokenType::EndLine);
                match build_expression_ast(&mut buffer.iter().peekable()) {
                    Ok(child) => result.push(child),
                    Err(e) => return Err(e),
                };
                break;
            },
            TokenType::EndLine => return Err(format!("parser: unexpected token {} while parsing array value.", TokenType::EndLine)),
            val => {
                tokens.next();
                buffer.push(val.clone());
            },
        };
    };
    
    return Ok(Ast::ArrayValue(result));
}

fn build_expression_ast(tokens: &mut Peekable<Iter<TokenType>>) -> Result<Ast, String> {

    let mut output_stack = Vec::<Ast>::new();
    let mut operator_stack = Vec::<TokenType>::new();

    loop {
        let token = match tokens.peek_mut() {
            Some(token) => token,
            None => {
                return Err(format!("missing token"));
            },
        };

        match token {
            TokenType::Bool(val) => {
                output_stack.push(Ast::Bool(val.clone()));
                tokens.next();
            },
            TokenType::Int(val) => {
                output_stack.push(Ast::Int(val.clone()));
                tokens.next();
            },
            TokenType::Float(val) => {
                output_stack.push(Ast::Float(val.clone()));
                tokens.next();
            },
            TokenType::String(val) => {
                output_stack.push(Ast::Str(val.clone()));
                tokens.next();
            },
            TokenType::Variable(_) if operator_stack.len() == 0 => {
                output_stack.push(match parse_variable(tokens, false) {
                    Ok(var) => Ast::Variable(var),
                    Err(e) => return Err(e),
                });
            },
            TokenType::Variable(name) => {
                output_stack.push(Ast::Variable(Variable { name: name.clone(), typename: None }));
                tokens.next();
            },
            TokenType::FunctionCall(val) => {
                operator_stack.push(token.clone());
                output_stack.push(Ast::FunctionCall {
                    name: val.clone(),
                    children: Vec::new(),
                });
                tokens.next();
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
                tokens.next();
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
                tokens.next();
            },
            TokenType::OpeningParenthesis => {
                operator_stack.push(token.clone());
                tokens.next();
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
                tokens.next();
            },
            TokenType::EndLine => {
                tokens.next();
                break;
            },
            TokenType::OpeningBracket => {
                tokens.next();
                let array_token = match build_array_value_ast(tokens) {
                    Ok(val) => val,
                    Err(e) => return Err(e),
                };
                let children = match &array_token {
                    Ast::ArrayValue(val) => val,
                    _ => return Err(String::new()),
                };
                if children.len() != 1 {
                    output_stack.push(array_token);
                    continue;
                }
                let offset = match children.get(0).unwrap() {
                    Ast::Int(val) => *val as u64,
                    _ => {
                        output_stack.push(array_token);
                        continue;
                    },
                };
                let last_token = match output_stack.pop() {
                    Some(val) => val,
                    None => {
                        output_stack.push(array_token);
                        continue;
                    },
                };
                let last_token_name = match last_token {
                    Ast::Variable(var) if var.typename == None => var.name.clone(),
                    val => {
                        output_stack.push(val.clone());
                        output_stack.push(array_token);
                        continue;
                    },
                };
                output_stack.push(Ast::ArrayAccess { variable: last_token_name, offset });
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
        TokenType::Keyword(val) if val == "function" => {
            tokens.next();
            return Some(build_function_ast(tokens));
        },
        TokenType::Keyword(val) if val == "declare" => {
            tokens.next();
            return Some(build_declaration_ast(tokens));
        },
        TokenType::Keyword(val) if val == "while" => {
            tokens.next();
            return Some(build_while_loop_ast(tokens));
        },
        TokenType::Keyword(val) if val == "return" => {
            tokens.next();
            return Some(build_return_ast(tokens));
        },
        _ => return Some(build_expression_ast(tokens)),
    };
}

fn build_while_loop_ast(tokens: &mut Peekable<Iter<TokenType>>) -> Result<Ast, String> {
    let condition = match build_expression_ast(tokens) {
        Ok(ast) => Box::new(ast),
        Err(e) => return Err(e),
    };

    let mut children = Vec::<Ast>::new();

    loop {
        let token = match tokens.peek() {
            Some(token) => token,
            None => return Err(format!("parser: error in while loop, unexpected end of document")),
        };
        match token {
            TokenType::Keyword(val) if val == "end" => {
                tokens.next();
                break;
            },
            _ => {
                match build_ast(tokens) {
                    Some(ast) => {
                        children.push(match ast {
                            Ok(ast) => ast,
                            Err(e) => return Err(e),
                        });
                    },
                    None => (),
                };
            },
        };
    };

    return Ok(Ast::WhileLoop { condition, children });
}

pub trait Visitor<T> {
    fn visit(&self, current: T, element: &Ast) -> Result<T, String>;
    fn visit_global(&self, current: T, children: &Vec<Ast>) -> Result<T, String>;
    fn visit_function(&self, current: T, name: &String, children: &Vec<Ast>, parameters: &Vec<Variable>, return_type: &Option<Type>) -> Result<T, String>;
    fn visit_value(&self, current: T, value: &Ast) -> Result<T, String>;
    fn visit_binary_operator(&self, current: T, value: &Ast) -> Result<T, String>;
    fn visit_unary_operator(&self, current: T, value: &Ast) -> Result<T, String>;
}

