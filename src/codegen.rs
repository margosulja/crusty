use std::collections::HashMap;
use std::fmt::format;
use crate::ast::{Binop, Expr, FunctionDecl, Parameter, Return, Stmt, VariableDecl};

pub struct CodeGen {
    output: String,
    strings: HashMap<String, usize>,
    variable_offsets: HashMap<String, usize>,
    variable_types: HashMap<String, String>,
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
            variable_types: HashMap::new(),
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
            Stmt::Expression(expr) => self.generate_expr_stmt(expr),
            Stmt::Return(ret) => self.generate_return_stmt(ret),
        }
    }

    fn generate_return_stmt(&mut self, ret: &Return) -> Result<(), String> {
        match &ret.value {
            Expr::Number(n) => { self.emit_line(&format!("    movl ${}, %eax", *n as i32)) },
            Expr::Identifier(ident) => {
                let offset = self.get_variable_offset(ident)?;
                let data_type = self.variable_types.get(ident).ok_or_else(|| format!("tried to get data type for variable {}", ident))?;

                match data_type.as_str() {
                    "int" => self.emit_line(&format!("    movl -{}(%rbp), %eax", offset)),
                    "char" => self.emit_line(&format!("    movzbl -{}(%rbp), %eax", offset)),
                    _ => return Err("unable to return this data type".to_string())
                }
            },

            _ => return Err("unsupported return expression".to_string())
        }

        Ok(())
    }

    fn generate_var_decl(&mut self, var_decl: &VariableDecl) -> Result<(), String> {
        let size_offset = self.get_type_size(&var_decl.data_type);

        self.rbp_offset += size_offset;
        if self.rbp_offset % 8 != 0 {
            self.rbp_offset += 8 - (self.rbp_offset % 8);
        }

        self.variable_offsets.insert(var_decl.name.clone(), self.rbp_offset);
        self.variable_types.insert(var_decl.name.clone(), var_decl.data_type.clone());

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
            },
            Expr::FunctionCall { callee, args } => {
                self.generate_function_call(&callee, &args)?;
                match var_decl.data_type.as_str() {
                    "int" => self.emit_line(&format!("    movl %eax, -{}(%rbp)", self.rbp_offset)),
                    "char" => self.emit_line(&format!("    movb %al, -{}(%rbp)", self.rbp_offset)),
                    _ => return Err(format!("unable to store return value for type: {}", var_decl.data_type))
                }

                Ok(())
            }
            _ => Ok(())
        }
        // Ok(())
    }

    fn generate_fn_decl(&mut self, func_decl: &FunctionDecl) -> Result<(), String> {
        self.rbp_offset = 0;

        let param_regs = ["%edi", "%esi", "%edx", "%ecx", "%r8d", "%r9d"];
        for (i, param) in func_decl.params.iter().enumerate() {
            let size = self.get_type_size(&param.data_type);
            self.rbp_offset += size;
            self.variable_offsets.insert(param.name.clone(), self.rbp_offset);
        }

        self.emit(format!("{}:\n", func_decl.name).as_str());
        self.emit_line("    pushq %rbp");
        self.emit_line("    movq %rsp, %rbp");

        if self.rbp_offset > 0 {
            let stk_size = ((self.rbp_offset + 15) / 16) * 16;
            self.emit_line(&format!("    subq ${}, %rsp", stk_size));
        }

        for (i, param) in func_decl.params.iter().enumerate() {
            if i < 6 {
                self.save_param_to_stk(param, i)?;
            } else {
                return Err("tried to do stack parameters, not implemented".to_string())
            }
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
                    Expr::Identifier(ident) => {
                        let offset = self.get_variable_offset(ident)?;
                        let var_type = self.variable_types.get(ident)
                            .ok_or_else(|| format!("unknown variable type: {}", ident))?;

                        match var_type.as_str() {
                            "int" => {
                                let reg_32 = ["%edi", "%esi", "%edx", "%ecx", "%r8d", "%r9d"][i];
                                self.emit_line(&format!("    movl -{}(%rbp), {}", offset, reg_32));
                            },
                            "char*" => {
                                self.emit_line(&format!("    movq -{}(%rbp), {}", offset, arg_regs[i]));
                            },
                            "char" => {
                                let reg_32 = ["%edi", "%esi", "%edx", "%ecx", "%r8d", "%r9d"][i];
                                self.emit_line(&format!("    movzbl -{}(%rbp), {}", offset, reg_32));
                            },
                            _ => return Err(format!("Unsupported variable type: {}", var_type))
                        }
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
            self.string_sect.push_str(&format!("    .string \"{}\"\n", self.get_escaped_string(s)));
        }

        Ok(())
    }

    fn save_param_to_stk(&mut self, param: &Parameter, reg_idx: usize) -> Result<(), String> {
        let offset = self.variable_offsets.get(&param.name).ok_or_else(|| format!("failed to find an offset for parameter '{}'", param.name))?;
        let (reg, inst) = match param.data_type.as_str() {
            "char*" => (self.get_64bit_reg(reg_idx)?, "movq"),
            "int"   => (self.get_32bit_reg(reg_idx)?, "movl"),
            "char"  => (self.get_8bit_reg(reg_idx)?, "movb"),
            _       => return Err(format!("unknown data type tried in save_param_to_stk. data type: {}", param.data_type))
        };

        self.emit_line(&format!("    {} {}, -{}(%rbp)", inst, reg, offset));
        Ok(())
    }

    fn get_64bit_reg(&self, idx: usize) -> Result<&'static str, String> {
        let regs = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
        regs.get(idx)
            .copied()
            .ok_or_else(|| "too many params for registers".to_string())
    }

    fn get_32bit_reg(&self, idx: usize) -> Result<&'static str, String> {
        let regs = ["%edi", "%esi", "%edx", "%ecx", "%r8d", "%r9d"];
        regs.get(idx)
            .copied()
            .ok_or_else(|| "too many params for registers".to_string())
    }

    fn get_8bit_reg(&self, idx: usize) -> Result<&'static str, String> {
        let regs = ["%dil", "%sil", "%dl", "%cl", "%r8b", "%r9b"];
        regs.get(idx)
            .copied()
            .ok_or_else(|| "too many params for registers".to_string())
    }

    fn get_param_register(&self, param_type: &str, index: usize) -> &'static str {
        let regs_64bit = ["%rdi", "%rsi", "%rdx", "%rcx", "%r8", "%r9"];
        let regs_32bit = ["%edi", "%esi", "%edx", "%ecx", "%r8d", "%r9d"];

        if param_type == "char*" {
            regs_64bit[index]
        } else {
            regs_32bit[index]
        }
    }

    fn get_type_size(&self, data_type: &str) -> usize {
        match data_type {
            "int" => 4,
            "char" => 1,
            "char*" => 8,
            _ => 0,
        }
    }

    fn get_escaped_string(&self, s: &str) -> String {
        s.chars()
            .map(|c| match c {
                '\n' => "\\n".to_string(),
                '\t' => "\\t".to_string(),
                '\r' => "\\r".to_string(),
                '\\' => "\\\\".to_string(),
                '"' => "\\\"".to_string(),
                '\0' => "\\0".to_string(),
                c if c.is_control() => format!("\\x{:02x}", c as u8),
                c => c.to_string(),
            })
            .collect()
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

    fn align_offset(&self, offset: usize, size: usize) -> usize {
        let alg = size.min(8);
        ((offset + alg - 1) / alg) * alg
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