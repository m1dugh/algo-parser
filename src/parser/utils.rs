use super::super::lexer::TokenType;

pub fn get_operator_precedency(operator: &TokenType) -> i64 {

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

