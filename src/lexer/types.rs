use std::fmt;

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
    Bool(bool),
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

#[derive(Copy, Clone)]
pub enum TokenizerContext {
    None,
    Name,
    Operator,
    Separator,
    Value,
    QuotedValue,
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
            Self::Bool(val) => write!(f, "<Bool ({})>", val),
        };
    }

}

