use std::fs::File;
use std::process::exit;
use std::io::{BufRead, BufReader};

pub mod lexer;
pub mod parser;

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
    let filename = "./examples/procedure.algo".to_string();
    let tokens = match lex(filename) {
        Err(e) => {
            println!("{}", e);
            exit(-1);
        },
        Ok(tokens) => tokens,
    };

    for token in &tokens {
        println!("{}", token);
    }

    match parser::load_ast(&tokens) {
        Err(e) => panic!("{}", e),
        Ok(ast) => println!("{:?}", ast),
    }

}
