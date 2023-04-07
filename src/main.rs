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
    Operator(char),
    Name(String),
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
            Self::OpeningParenthesis => write!(f, "("),
            Self::ClosingParenthesis => write!(f, ")"),
            Self::EndLine => write!(f, "<\\n>"),
            Self::Comma => write!(f, ","),
            Self::Assignement => write!(f, "(<-)"),
            Self::Colon => write!(f, ":"),
            Self::Operator(val) => write!(f, "({})", val),
            Self::Name(val) | Self::Keyword(val) | Self::TypeDef(val) => write!(f, "<Name ({})>", val),
            Self::Value(val) => write!(f, "<Value ({})>", val),
            Self::QuotedValue(val) => write!(f, "<QuotedValue ('{}')>", val),
        };
    }

}

enum TokenizerContext {
    None,
    Name,
    Operator,
    Separator,
    Value,
    QuotedValue,
}

static OPERATORS: &str = "+-%/-*<>=";
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

fn create_token(token_value: String, context: TokenizerContext) -> TokenType {

    if ["int", "float", "string", "char", "function"].iter().any(|&s| s == token_value) {
        return TokenType::TypeDef(token_value);
    } else {
        return TokenType::Name(token_value);
    }
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
                let mut next_char = true;
                let mut should_push = true;
                if c == ' ' && !matches!(context, TokenizerContext::QuotedValue) {
                    should_push = false;
                    match context {
                        TokenizerContext::None => (),
                        _ => {
                            let token_value = current_token.iter().collect::<String>();
                            result.push(create_token(token_value, context));
                            context = TokenizerContext::None;
                            current_token.clear();
                        },
                    }
                } else {
                    match context {
                        TokenizerContext::None => {
                            if OPERATORS.contains(c) {
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
                                let token_value = current_token.iter().collect::<String>();
                                result.push(create_token(token_value, context));
                                current_token.clear();
                                context = TokenizerContext::None;
                                next_char = false;
                            }
                        },
                        TokenizerContext::Separator => {
                            let token_value = current_token.iter().collect::<String>();
                            result.push(create_token(token_value, context));
                            current_token.clear();
                            context = TokenizerContext::None;
                            next_char = false;
                        },
                        TokenizerContext::Operator => {
                            if !OPERATORS.contains(c) {
                                let token_value = current_token.iter().collect::<String>();
                                result.push(create_token(token_value, context));
                                current_token.clear();
                                context = TokenizerContext::None;
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
                                let token_value = current_token.iter().collect::<String>();
                                result.push(create_token(token_value, context));
                                current_token.clear();
                                context = TokenizerContext::None;
                                should_push = false;
                            }
                        },
                    }
                }

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
        println!("{}", e);
    }
}
