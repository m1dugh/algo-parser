use std::fs::File;
use std::process::exit;
use std::io::{BufRead, BufReader};

pub mod lexer;
pub mod parser;
pub mod compiler;

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


fn lex(filename: String) -> Result<Vec<lexer::TokenType>, String> {
    let lines = read_lines(filename);
    let tokens = match lexer::tokenize(&lines) {
        Err(e) => return Err(e),
        Ok(tokens) => tokens,
    };

    return Ok(tokens);

}

fn main() {
    let filename = "./examples/test_functions.algo".to_string();
    let tokens = match lex(filename) {
        Err(e) => {
            println!("{}", e);
            exit(-1);
        },
        Ok(tokens) => tokens,
    };

    let ast = match parser::load_ast(&tokens) {
        Err(e) => panic!("{}", e),
        Ok(ast) => ast,
    };

    compiler::test(&ast);
}
