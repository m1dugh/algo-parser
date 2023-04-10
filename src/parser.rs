
struct AST {
    token: Option<TokenType>,
    children: Vec<Box<AST>>
}

impl AST {
    fn evaluate(&self) {
    }
}

fn parse_scope(tokens: Vec<TokenType>) -> AST {
    
}
