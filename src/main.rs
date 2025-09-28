use std::fs::{read_to_string, write};
use crate::codegen::CodeGen;
use crate::lexer::Lexer;
use crate::parser::Parser;

mod ast;
mod lexer;
mod parser;
mod codegen;

fn main() {
    let mut args: Vec<String> = std::env::args().collect();

    if args.len() > 1 {
        args.remove(0);

        let file = args.join(" ");
        let input = read_to_string(file.clone()).unwrap();

        let lexer = Lexer::new(&*input);
        let mut parser = Parser::new(lexer);
        let mut codegen = CodeGen::new();
        let program = parser.parse().unwrap();

        let asm = codegen.generate(&*program).unwrap();

        println!("{asm}");
        write("out.s", asm).unwrap();
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn ascii_char_representation() {
        let result = "a".as_bytes();
        assert_eq!(result[0], 97);
    }
}