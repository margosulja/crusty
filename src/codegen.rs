use std::collections::HashMap;
use std::fmt::format;
use crate::ast::{Binop, Expr, FunctionDecl, Stmt, VariableDecl};

pub struct CodeGen {
    output: String,
    strings: HashMap<String, usize>,
    label_count: usize,
    rbp_offset: usize,
    isize: usize,   /* indent size */
}

impl CodeGen {
    pub fn new() -> Self {
        Self {
            output: String::new(),
            strings: HashMap::new(),
            label_count: 0,
            rbp_offset: 0,
            isize: 0,
        }
    }

    pub fn generate(&mut self, stmts: &[Stmt]) -> Result<String, String> {
        for stmt in stmts {
            self.generate_stmt(stmt)?;
        }

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

        match var_decl.value.clone() {
            Expr::Number(n) => Ok(self.emit(format!("    mov DWORD PTR [rbp-{}], {}\n", self.rbp_offset, n).as_str())),
            Expr::String(str) => {
                /* only process chars */
                if str.len() == 1 {
                    Ok(self.emit(format!("    mov BYTE PTR [rbp-{}], {}\n", self.rbp_offset, str.as_bytes()[0]).as_str()))
                } else {
                    self.generate_string(&*str)?;
                    Ok(self.emit(format!("    mov QWORD PTR [rbp-{}], OFFSET FLAT:.LC{}\n", self.rbp_offset, self.strings.get(&str).unwrap()).as_str()))
                }
            }
            _ => Ok(())
        }

        // Ok(())
    }

    fn generate_fn_decl(&mut self, func_decl: &FunctionDecl) -> Result<(), String> {
        self.emit(format!("{}:\n", func_decl.name).as_str());
        self.emit_line("    push rbp");
        self.emit_line("    mov rbp, rsp");

        for stmt in func_decl.body.iter() {
            self.generate_stmt(&stmt)?;
        }

        self.emit_line("    mov eax, 0");
        self.emit_line("    pop rbp");
        self.emit_line("    ret");
        Ok(())
    }

    fn generate_expr(&mut self, expr: &Expr) -> Result<(), String> {
        match expr {
            Expr::Number(n) => self.generate_number(*n),
            Expr::String(s) => self.generate_string(s),
            Expr::Identifier(ident) => self.generate_identifier(ident),
            Expr::BinaryOp { left, op, right } => self.generate_binary_op(left, op, right),
        }
    }

    fn generate_expr_stmt(&mut self, expr: &Expr) -> Result<(), String> {
        todo!("impl expr stmt generation")
    }

    fn generate_number(&mut self, n: f64) -> Result<(), String> {
        todo!("impl number generation")
    }

    fn generate_string(&mut self, s: &str) -> Result<(), String> {
        let lc = self.label_count.to_owned();
        self.label_count += 1;
        self.strings.insert(s.parse().unwrap(), lc);
        Ok(self.emit_label_with_code(format!("LC{}", lc).as_str(), format!(".string \"{}\"", s).as_str()))
    }

    fn generate_identifier(&mut self, ident: &str) -> Result<(), String> {
        todo!("impl identifier ref")
    }

    fn generate_binary_op(&mut self, left: &Expr, op: &Binop, right: &Expr) -> Result<(), String> {
        todo!("impl me in gen binop!")
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