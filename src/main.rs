use colored::Colorize;
use std::fs::{read_to_string, write};
use std::process::Command;
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

        write("out.s", &asm).unwrap();
        println!("{} Compiled!", "[crusty]".bold().truecolor(252, 88, 88));

        let output = Command::new("gcc")
            .args(["-no-pie", "out.s", "-o", "out"])
            .output()
            .expect("Failed to execute gcc");

        if !output.status.success() {
            println!("{} An error occurred while compiling: {}", "[crusty]".bold().truecolor(252, 88, 88), String::from_utf8_lossy(&output.stderr).bold().underline());
        }
    }
}