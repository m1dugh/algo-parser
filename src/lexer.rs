use std::{fmt, str::Chars};

#[derive(Clone)]
pub enum TokenType {
    OpeningParenthesis,
    ClosingParenthesis,
    OpeningBracket,
    ClosingBracket,
    Comma,
    Colon,
    EndLine,
    Int(i64),
    Float(f64),
    String(String),
    ArrayTypeDef(String),
    BinaryOperator(String),
    UnaryOperator(String),
    Variable(String),
    FunctionCall(String),
    Keyword(String),
    TypeDef(String),
}

impl fmt::Debug for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return fmt::Display::fmt(&self, f);
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Self::OpeningParenthesis => write!(f, "<OpeningParenthesis '('>"),
            Self::ClosingParenthesis => write!(f, "<ClosingParenthesis ')'>"),
            Self::OpeningBracket => write!(f, "<OpeningBracket '['>"),
            Self::ClosingBracket => write!(f, "<ClosingBracket ']'>"),
            Self::EndLine => write!(f, "<EndLine>"),
            Self::Comma => write!(f, "<Comma ','>"),
            Self::Colon => write!(f, "<Colon ':'>"),
            Self::BinaryOperator(val) => write!(f, "<BinaryOperator ({})>", val),
            Self::UnaryOperator(val) => write!(f, "<UnaryOperator ({})>", val),
            Self::Variable(val) => write!(f, "<Variable ({})>", val),
            Self::FunctionCall(val) => write!(f, "<FunctionCall ({})>", val),
            Self::Keyword(val) => write!(f, "<Keyword ({})>", val),
            Self::TypeDef(val) => write!(f, "<TypeDef ({})>", val),
            Self::Int(val) => write!(f, "<Int ({})>", val),
            Self::Float(val) => write!(f, "<Float ({})>", val),
            Self::String(val) => write!(f, "<String ({})>", val),
            Self::ArrayTypeDef(val) => write!(f, "<Array ({})>", val),
        };
    }

}

#[derive(Copy, Clone)]
enum TokenizerContext {
    None,
    Name,
    Operator,
    Separator,
    Value,
    QuotedValue,
}

static OPERATOR_STRING: &str = "+-%/-*<>=!";
static SEPARATORS: &str = "()[]:,";
static START_NAME_CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
static NUMERIC_CHARACTERS: &str = ".0123456789";

static TYPES: [&str; 4] = ["int", "float", "string", "char"];
static BINARY_OPERATORS: [&str; 13] = [">", "<", ">=", "<=", "+", "-", "<-", "/", "%", "*", "==", "!=", "!"];
static UNARY_OPERATORS: [&str; 2] = ["-", "+"];
static KEYWORDS: [&str; 7] = ["end", "return", "function", "while", "for", "if", "else"];


static NUMBER_STRING: &str = "0123456789";

fn to_float(token_value: &String) -> Option<f64> {

    let mut upper_part: f64 = 0.0;
    let mut lower_part: f64 = 0.0;
    let mut chars = token_value.chars();

    while let Some(c) = chars.next() {
        if let Some(val) = c.to_digit(10) {
            upper_part = upper_part * 10.0 + val as f64;
        } else if c == '.' {
            break;
        } else {
            return None;
        }
    }

    for c in chars.rev() {
        if let Some(val) = c.to_digit(10) {
            lower_part = lower_part / 10.0 + val as f64;
        } else {
            return None;
        }
    }

    lower_part /= 10.0;

    return Some(upper_part + lower_part);
}

fn to_int(token_value: &String) -> Option<i64> {
    let mut result: i64 = 0;
    for c in token_value.chars() {
        if let Some(val) = c.to_digit(10) {
            result = result * 10 + val as i64;
        } else {
            return None;
        }
    }

    return Some(result);
}

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

fn create_token(token_value: String, context: TokenizerContext, old_tokens: Vec<TokenType>) -> Result<Vec<TokenType>, String> {

    let mut tokens: Vec<TokenType> = Vec::with_capacity(old_tokens.len());
    for element in old_tokens.iter() {
        tokens.push(element.clone());
    }

    match context {
        TokenizerContext::Name => {
            if TYPES.iter().any(|&s| s == token_value) {
                tokens.push(TokenType::TypeDef(token_value));
            } else if KEYWORDS.iter().any(|&s| s == token_value) {
                tokens.push(TokenType::Keyword(token_value));
            } else if let Some(last_token) = tokens.last() {
                tokens.push(match last_token {
                    TokenType::Colon => TokenType::TypeDef(token_value),
                    _ => TokenType::Variable(token_value)
                });
            } else {
                tokens.push(TokenType::Variable(token_value));
            }
        },
        TokenizerContext::Operator => {
            match lex_operators(token_value.clone(), old_tokens.last()) {
                Ok(operators) => {
                    for token in operators {
                        tokens.push(token);
                    }
                },
                Err(e) => return Err(e),
            };
        },
        TokenizerContext::Value => {
            if let Some(val) = to_int(&token_value) {
                tokens.push(TokenType::Int(val));
            } else if let Some(val) = to_float(&token_value) {
                tokens.push(TokenType::Float(val));
            } else {
                // TODO: implement proper errors
                return Err(format!("invalid number '{}'", token_value));
            }
        },
        TokenizerContext::QuotedValue => {
            tokens.push(TokenType::String(token_value));
        },
        TokenizerContext::Separator => {
            match token_value.to_string().as_str() {
                "(" => {
                    if let Some(last_token) = old_tokens.last() {
                        if let TokenType::Variable(val) = last_token {
                            if let Some(before_last_token) = old_tokens.get(old_tokens.len() - 2) {
                                match before_last_token {
                                    TokenType::Keyword(val) if val == "function" => (),
                                    _ => {
                                        tokens.pop();
                                        tokens.push(TokenType::FunctionCall(val.clone()));
                                    }
                                }
                            } else {
                                tokens.pop();
                                tokens.push(TokenType::FunctionCall(val.clone()));
                            }
                        }
                    }
                    tokens.push(TokenType::OpeningParenthesis);
                },
                ")" => tokens.push(TokenType::ClosingParenthesis),
                "[" => tokens.push(TokenType::OpeningBracket),
                "]" => {

                    let tokens_len = tokens.len();
                    if tokens_len >= 2 {
                        match old_tokens.get(tokens_len - 1).unwrap() {
                            TokenType::OpeningBracket => {
                                match old_tokens.get(tokens_len - 2).unwrap() {
                                    TokenType::TypeDef(val) => {
                                        tokens.pop();
                                        tokens.pop();
                                        tokens.push(TokenType::ArrayTypeDef(val.clone()));
                                    },
                                    _ => {
                                        tokens.push(TokenType::ClosingBracket);
                                    }
                                }
                            },
                            _ => {
                                tokens.push(TokenType::ClosingBracket);
                            }
                        }
                    }
                },
                ":" => tokens.push(TokenType::Colon),
                "," => tokens.push(TokenType::Comma),
                _   => return Err(format!("invalid separator '{}'", token_value))
            };

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
