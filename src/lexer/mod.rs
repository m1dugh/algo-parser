

mod types;
pub use types::TokenType;
use types::TokenizerContext;

mod utils;
use utils::*;

mod contants;
use contants::*;

fn lex_operators(token_value: String, last_token: Option<&TokenType>) -> Result<Vec<TokenType>, String> {
    let mut op_string = token_value.clone();
    let mut token_index = 0;
    let mut op_string_index = op_string.len();
    let mut result = match last_token {
        Some(token) => vec![token.clone()],
        None => Vec::<TokenType>::new(),
    };

    while op_string_index > 0 {
        op_string = op_string[..op_string_index].to_string();
        if let Some(last_token) = result.last() {
            match last_token {
                TokenType::BinaryOperator(_)
                    | TokenType::UnaryOperator(_)
                    | TokenType::Keyword(_)
                    | TokenType::Comma
                    if UNARY_OPERATORS.iter().any(|&s| s == op_string) => {
                        result.push(TokenType::UnaryOperator(op_string));
                        token_index += op_string_index;
                        op_string = token_value[token_index..].to_string();
                        op_string_index = op_string.len();
                    },
                    _ if BINARY_OPERATORS.iter().any(|&s| s == op_string) => {
                        result.push(TokenType::BinaryOperator(op_string));
                        token_index += op_string_index;
                        op_string = token_value[token_index..].to_string();
                        op_string_index = op_string.len();
                    },
                    _ if op_string_index > 0 => {
                        op_string_index -= 1;
                    },
                    _ => return Err(format!("invalid operator '{}'", token_value)),
            };
        } else if UNARY_OPERATORS.iter().any(|&s| s == op_string) {
            result.push(TokenType::UnaryOperator(op_string[..op_string_index].to_string()));
            token_index += op_string_index;
            op_string = token_value[token_index..].to_string();
            op_string_index = op_string.len();
        } else {
            op_string_index -= 1;
        }
    }

    if token_value.len() == token_index {
        return Ok(match last_token {
            Some(_) => {
                result.remove(0);
                result
            },
            None => result,
        });
    } else {
        return Err(format!("invalid operator '{}'", token_value));
    }
}

fn lex_name_token(token_value: String, result: &mut Vec<TokenType>) {
    if TYPES.iter().any(|&s| s == token_value) {
        result.push(TokenType::TypeDef(token_value));
    } else if KEYWORDS.iter().any(|&s| s == token_value) {
        result.push(TokenType::Keyword(token_value));
    } else if token_value == "true" {
        result.push(TokenType::Bool(true));
    } else if token_value == "false" {
        result.push(TokenType::Bool(false));
    } else if let Some(last_token) = result.last() {
        result.push(match last_token {
            TokenType::Colon => TokenType::TypeDef(token_value),
            _ => TokenType::Variable(token_value)
        });
    } else {
        result.push(TokenType::Variable(token_value));
    }
}

fn lex_value_token(token_value: &String, result: &mut Vec<TokenType>) -> Result<(), String> {
    if let Some(val) = to_int(&token_value) {
        result.push(TokenType::Int(val));
    } else if let Some(val) = to_float(&token_value) {
        result.push(TokenType::Float(val));
    } else {
        // TODO: implement proper errors
        return Err(format!("invalid number '{}'", token_value));
    }

    return Ok(());
}

fn lex_closing_brackets(old_tokens: &Vec<TokenType>, result: &mut Vec<TokenType>) {

    let tokens_len = result.len();
    if tokens_len >= 2 {
        match old_tokens.get(tokens_len - 1).unwrap() {
            TokenType::OpeningBracket => {
                match old_tokens.get(tokens_len - 2).unwrap() {
                    TokenType::TypeDef(val) => {
                        result.pop();
                        result.pop();
                        result.push(TokenType::ArrayTypeDef(val.clone()));
                    },
                    _ => {
                        result.push(TokenType::ClosingBracket);
                    }
                }
            },
            _ => {
                result.push(TokenType::ClosingBracket);
            }
        }
    }
}

fn lex_opening_parenthesis(old_tokens: &Vec<TokenType>, result: &mut Vec<TokenType>) {

    if let Some(last_token) = old_tokens.last() {
        if let TokenType::Variable(val) = last_token {
            if let Some(before_last_token) = old_tokens.get(old_tokens.len() - 2) {
                match before_last_token {
                    TokenType::Keyword(val) if val == "function" => (),
                    _ => {
                        result.pop();
                        result.push(TokenType::FunctionCall(val.clone()));
                    }
                }
            } else {
                result.pop();
                result.push(TokenType::FunctionCall(val.clone()));
            }
        }
    }
    result.push(TokenType::OpeningParenthesis);
}

fn lex_separator(token_value: &String, old_tokens: &Vec<TokenType>, result: &mut Vec<TokenType>) -> Result<(), String> {
    match token_value.to_string().as_str() {
        "(" => lex_opening_parenthesis(&old_tokens, result),
        ")" => result.push(TokenType::ClosingParenthesis),
        "[" => result.push(TokenType::OpeningBracket),
        "]" => lex_closing_brackets(&old_tokens, result),
        ":" => result.push(TokenType::Colon),
        "," => result.push(TokenType::Comma),
        _   => return Err(format!("invalid separator '{}'", token_value))
    };

    return Ok(());
}

fn create_token(token_value: String, context: TokenizerContext, old_tokens: Vec<TokenType>) -> Result<Vec<TokenType>, String> {

    let mut tokens: Vec<TokenType> = Vec::with_capacity(old_tokens.len());
    for element in old_tokens.iter() {
        tokens.push(element.clone());
    }

    match context {
        TokenizerContext::Name => lex_name_token(token_value, &mut tokens),
        TokenizerContext::Operator => {
            match lex_operators(token_value.clone(), old_tokens.last()) {
                Ok(operators) =>
                    operators.iter().for_each(|token| tokens.push(token.clone())),
                Err(e) => return Err(e),
            };
        },
        TokenizerContext::Value => {
            if let Err(e) = lex_value_token(&token_value, &mut tokens) {
                return Err(e);
            }
        },
        TokenizerContext::QuotedValue => {
            tokens.push(TokenType::String(token_value));
        },
        TokenizerContext::Separator => {

            if let Err(e) = lex_separator(&token_value, &old_tokens, &mut tokens) {
                return Err(e);
            }
        }
        TokenizerContext::None => {
            return Err(format!("invalid token '{}' in context None", token_value))
        },
    };

    return Ok(tokens);
}

pub fn tokenize(lines: &Vec<String>) -> Result<Vec<TokenType>, String> {

    let mut context = TokenizerContext::None;
    let mut current_token = Vec::<char>::new();
    let mut result = Vec::<TokenType>::new();

    for (line_index, l) in lines.iter().enumerate() {
        let mut chars = l.chars().enumerate();
        if let Some((mut char_index, mut c)) = chars.next() {
            loop {
                let mut push_context: Option<TokenizerContext> = None;
                let mut next_char = true;
                let mut should_push = true;
                if c == ' ' && !matches!(context, TokenizerContext::QuotedValue) {
                    should_push = false;
                    match context {
                        TokenizerContext::None => (),
                        _ => {
                            push_context = Some(context);
                        },
                    }
                } else {
                    match context {
                        TokenizerContext::None => {
                            if OPERATOR_STRING.contains(c) {
                                context = TokenizerContext::Operator;
                            } else if SEPARATORS.contains(c) {
                                context = TokenizerContext::Separator;
                            } else if START_NAME_CHARACTERS.contains(c) {
                                context = TokenizerContext::Name;
                            } else if NUMERIC_CHARACTERS.contains(c) {
                                context = TokenizerContext::Value;
                            } else if c == '"' {
                                context = TokenizerContext::QuotedValue;
                                should_push = false;
                            } else {
                                return Err(format!("invalid character '{}' at {}:{}", c, line_index, char_index));
                            }
                        },
                        TokenizerContext::Name if !START_NAME_CHARACTERS.contains(c) && !NUMERIC_CHARACTERS.contains(c) => {
                            push_context = Some(context);
                            next_char = false;
                        },
                        TokenizerContext::Separator => {
                            push_context = Some(context);
                            next_char = false;
                        },
                        TokenizerContext::Operator if !OPERATOR_STRING.contains(c) => {
                            push_context = Some(context);
                            next_char = false;
                        },
                        TokenizerContext::Value if !NUMERIC_CHARACTERS.contains(c) => {
                            push_context = Some(context);
                            next_char = false;
                        },
                        TokenizerContext::QuotedValue if c == '\"' => {
                            push_context = Some(context);
                            should_push = false;
                        },

                        _ => (),
                    }
                }

                match push_context {
                    Some(_) => {
                        let token_value = current_token.iter().collect::<String>();
                        match create_token(token_value, context, result) {
                            Ok(val) => result = val,
                            Err(e) => return Err(e),
                        };
                        context = TokenizerContext::None;
                        current_token.clear();
                    },
                    None => (),
                };

                if next_char && should_push {
                    current_token.push(c);
                }

                if next_char {
                    if let Some((new_char_index, new_char)) = chars.next() {
                        char_index = new_char_index;
                        c = new_char;
                    } else {
                        break;
                    }
                }

            }
        }
        match context {
            TokenizerContext::None => (),
            _ => {
                let token_value = current_token.iter().collect::<String>();
                match create_token(token_value, context, result) {
                    Ok(val) => result = val,
                    Err(e) => return Err(e),
                };
                current_token.clear();
                context = TokenizerContext::None;
            },
        };
        result.push(TokenType::EndLine);
    }
    return Ok(result);
}
