use std::fmt;
use std::{io::{BufReader, BufRead}, fs::File};

enum TokenType {
    OpeningParenthesis,
    ClosingParenthesis,
    Comma,
    Assignement,
    Colon,
    EndLine,
    Value(String),
    QuotedValue(String),
    Operator(String),
    Name(String),
    Keyword(String),
    TypeDef(String),
    EndToken,
}

impl fmt::Debug for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return fmt::Display::fmt(&self, f);
    }
}

impl fmt::Display for TokenType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        return match self {
            Self::OpeningParenthesis => write!(f, "("),
            Self::ClosingParenthesis => write!(f, ")"),
            Self::EndLine => write!(f, "<\\n>"),
            Self::Comma => write!(f, ","),
            Self::Assignement => write!(f, "(<-)"),
            Self::Colon => write!(f, ":"),
            Self::Operator(val) => write!(f, "<Operator ({})>", val),
            Self::Name(val) => write!(f, "<Name ({})>", val),
            Self::Keyword(val) => write!(f, "<Keyword ({})>", val),
            Self::TypeDef(val) => write!(f, "<TypeDef ({})>", val),
            Self::Value(val) => write!(f, "<Value ({})>", val),
            Self::QuotedValue(val) => write!(f, "<QuotedValue ('{}')>", val),
            Self::EndToken => write!(f, "<End>"),
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
static SEPARATORS: &str = "(){}:";
static START_NAME_CHARACTERS: &str = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ_";
static NUMERIC_CHARACTERS: &str = "0123456789";

fn read_lines(filename: String) -> Vec<String> {
    let file = File::open(filename);
    if let Ok(buffer) = file {
        let lines = BufReader::new(buffer).lines();
        return lines.map(|l| l.unwrap()).collect::<Vec<String>>();
    } else if let Err(e) = file {
        println!("{}", e);
    }

    return Vec::new();

}

static TYPES: [&str; 5] = ["int", "float", "string", "char", "function"];
static OPERATORS: [&str; 13] = [">", "<", ">=", "<=", "+", "-", "<-", "/", "%", "*", "==", "!=", "!"];

fn create_token(token_value: String, context: TokenizerContext) -> Result<TokenType, String> {

    match context {
        TokenizerContext::Name => {
            if TYPES.iter().any(|&s| s == token_value) {
                return Ok(TokenType::TypeDef(token_value));
            } else if token_value == "return" {
                return Ok(TokenType::Keyword(token_value));
            } else if token_value == "end" {
                return Ok(TokenType::EndToken);
            } else {
                return Ok(TokenType::Name(token_value));
            }
        },
        TokenizerContext::Operator => {
            if OPERATORS.iter().any(|&s| s == token_value) {
                return Ok(TokenType::Operator(token_value));
            } else {
                return Err(format!("invalid operator '{}'", token_value));
            }
        },
        TokenizerContext::Value => {
            return Ok(TokenType::Value(token_value));
        },
        TokenizerContext::QuotedValue => {
            return Ok(TokenType::QuotedValue(token_value));
        },
        TokenizerContext::Separator => {
            return Ok(TokenType::QuotedValue(token_value));
        }
        TokenizerContext::None => {
            return Err(String::new());
        },
    };

}

fn tokenize(filename: String) -> Result<Vec<TokenType>, String> {

    let lines = read_lines(filename);
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
                        TokenizerContext::Name => {
                            if !START_NAME_CHARACTERS.contains(c) && !NUMERIC_CHARACTERS.contains(c) {
                                push_context = Some(context);
                                next_char = false;
                            }
                        },
                        TokenizerContext::Separator => {
                            push_context = Some(context);
                            next_char = false;
                        },
                        TokenizerContext::Operator => {
                            if !OPERATOR_STRING.contains(c) {
                                push_context = Some(context);
                                next_char = false;
                            }
                        },
                        TokenizerContext::Value => {
                            if !NUMERIC_CHARACTERS.contains(c) {
                                return Err(format!("invalid characted '{}' at {}:{}", c, line_index, char_index));
                            }
                        },
                        TokenizerContext::QuotedValue => {
                            if c == '\"' {
                                push_context = Some(context);
                                should_push = false;
                            }
                        },
                    }
                }

                match push_context {
                    Some(val) => {
                        let token_value = current_token.iter().collect::<String>();
                        match create_token(token_value, val) {
                            Ok(token) => result.push(token),
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
                match create_token(token_value, context) {
                    Ok(token) => result.push(token),
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

fn main() {
    let filename = "./examples/procedure.algo".to_string();
    let tokenized = tokenize(filename);
    if let Ok(tokens) = tokenized {
        for token in tokens {
            println!("{}", token);
        }
    } else if let Err(e) = tokenized {
        println!("error {}", e);
    }
}
