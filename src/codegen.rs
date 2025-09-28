use std::collections::HashMap;
use std::fmt::format;
use crate::ast::{Binop, Expr, FunctionDecl, Stmt, VariableDecl};

pub struct CodeGen {
    output: String,
    strings: HashMap<String, usize>,
    variable_offsets: HashMap<String, usize>,
    string_sect: String,
    label_count: usize,
    rbp_offset: usize,
    isize: usize,   /* indent size */
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            variable_offsets: HashMap::new(),
            string_sect: String::new(),
            strings: HashMap::new(),
            label_count: 0,
            rbp_offset: 0,
            isize: 0,
        }
    }

    pub fn generate(&mut self, stmts: &[Stmt]) -> Result<String, String> {
        /* collect all string s */
        let mut t_output = String::new();
        std::mem::swap(&mut self.output, &mut t_output);

        for stmt in stmts {
            self.generate_stmt(stmt)?;
        }

        let code_sect = self.output.clone();
        self.output = t_output;

        if !self.string_sect.is_empty() {
            self.emit_line(".section .rodata");
            self.emit(&self.string_sect.clone());
        }

        self.emit_line(".section .text");
        self.emit_line("    .globl main");
        self.emit_line("    .type main, @function");
        self.emit(&code_sect);

        Ok(self.output.clone())
    }

    fn generate_stmt(&mut self, stmt: &Stmt) -> Result<(), String> {
        match stmt {
            Stmt::VariableDecl(vdecl) => self.generate_var_decl(vdecl),
            Stmt::FunctionDecl(fdecl) => self.generate_fn_decl(fdecl),
            Stmt::Expression(expr) => self.generate_expr_stmt(expr)
        }
    }

    fn generate_var_decl(&mut self, var_decl: &VariableDecl) -> Result<(), String> {
        let value = match var_decl.value {
            Expr::Number(n) => n,
            _ => 0.0
        };

        let size_offset = match var_decl.data_type.as_str() {
            "int" => 4,
            "char" => 1,
            "char*" => 8,
            _ => 0,
        };

        self.rbp_offset += size_offset;
        self.variable_offsets.insert(var_decl.name.clone(), self.rbp_offset);

        match var_decl.value.clone() {
            Expr::Number(n) => Ok(self.emit(&format!("    movl ${}, -{}(%rbp)\n", n, self.rbp_offset))),
            Expr::String(str) => {
                /* only process chars */
                if str.len() == 1 {
                    Ok(self.emit(&format!("    movl ${}, -{}(%rbp)\n", str.as_bytes()[0], self.rbp_offset)))
                } else {
                    self.generate_string(&*str)?;
                    let label = self.strings.get(&str).unwrap();
                    self.emit(format!("    leaq .LC{}(%rip), %rax\n", label).as_str());
                    self.emit(format!("    movq %rax, -{}(%rbp)\n", self.rbp_offset).as_str());
                    Ok(())
                }
            }
            _ => Ok(())
        }
        // Ok(())
    }

    fn generate_fn_decl(&mut self, func_decl: &FunctionDecl) -> Result<(), String> {
        self.rbp_offset = 0;
        self.emit(format!("{}:\n", func_decl.name).as_str());
        self.emit_line("    pushq %rbp");
        self.emit_line("    movq %rsp, %rbp");

        if self.rbp_offset > 0 {
            let stk_size = ((self.rbp_offset + 15) / 16) * 16;
            self.emit_line(&format!("    subq ${}, %rsp", stk_size));
        }

        for stmt in func_decl.body.iter() {
            self.generate_stmt(&stmt)?;
        }

        self.emit_line("    leave");
        self.emit_line("    ret");
        Ok(())
    }

    fn generate_function_call(&mut self, callee: &String, args: &[Expr]) -> Result<(), String> {
        let arg_regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];

        for (i, arg) in args.iter().enumerate() {
            if i < 6 {
                match arg {
                    Expr::Identifier(ident ) => {
                        /* var from stk */
                        let offset = self.get_variable_offset(ident)?;
                        self.emit_line(&format!("    movq -{}(%rbp), {}", offset, arg_regs[i]));
                    },

                    Expr::String(st) => {
                        /* load the string addr */
                        self.generate_string(st)?;
                        let label = self.strings.get(st).unwrap();
                        self.emit_line(&format!("    leaq .LC{}(%rip), {}", label, arg_regs[i]));
                    },

                    Expr::Number(n) => {
                        self.emit_line(&format!("    movq ${}, {}", *n as i64, arg_regs[i]));
                    },

                    _ => return Err("unsupported arg type".to_string())
                }
            } else {
                /* if greater than 6, just push to the stack */
                /* stack args */
            }
        }

        /* variadic functions */
        if callee == "printf" {
            self.emit_line("    movl $0, %eax");
        }

        self.emit_line(&format!("    call {}", callee));

        Ok(())
    }

    fn generate_expr_stmt(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::FunctionCall { callee, args } => {
                self.generate_function_call(callee, args)
            },
            _ => Ok(())
        }
    }

    fn generate_number(&mut self, n: f64) -> Result<(), String> {
        todo!("impl number generation")
    }

    fn generate_string(&mut self, s: &str) -> Result<(), String> {
        if !self.strings.contains_key(s) {
            let lc = self.label_count.to_owned();
            self.label_count += 1;
            self.strings.insert(s.parse().unwrap(), lc);

            self.string_sect.push_str(&format!(".LC{}:\n", lc));
            self.string_sect.push_str(&format!("    .string \"{}\"\n", s));
        }

        Ok(())
    }

    fn generate_identifier(&mut self, ident: &str) -> Result<(), String> {
        todo!("impl identifier ref")
    }

    fn generate_binary_op(&mut self, left: &Expr, op: &Binop, right: &Expr) -> Result<(), String> {
        todo!("impl me in gen binop!")
    }

    fn get_variable_offset(&self, variable_name: &str) -> Result<usize, String> {
        self.variable_offsets.get(variable_name).copied().ok_or_else(|| format!("undefined variable: {}", variable_name))
    }

    fn emit(&mut self, code: &str) {
        self.output.push_str(code);
    }

    fn emit_label_with_code(&mut self, label: &str, code: &str) {
        self.output.insert_str(0, format!(".{}: {}\n", label, code).as_str());
    }

    fn emit_line(&mut self, code: &str) {
        self.emit_indent();
        self.output.push_str(code);
        self.output.push('\n');
    }

    fn emit_indent(&mut self) {
        for _ in 0..self.isize {
            self.output.push_str("    ");
        }
    }

    fn inc_indent(&mut self) {
        self.isize += 1;
    }

    fn dec_ident(&mut self) {
        if self.isize > 0 {
            self.isize -= 1;
        }
    }

    pub fn get_output(&self) -> &str {
        &self.output
    }

    pub fn clear(&mut self) {
        self.output.clear();
        self.isize = 0;
    }
}