use crate::ast::{Binop, Expr, FunctionDecl, Parameter, Return, Stmt, VariableDecl};
use crate::ast::Expr::{FunctionCall, Identifier};
use crate::lexer::*;

pub struct Parser<'a> {
    lexer: Lexer<'a>,
    current: Option<Token>,
}

impl<'a> Parser<'a> {
    pub fn new(mut lexer: Lexer<'a>) -> Self {
        let current = lexer.next();
        Self { lexer, current }
    }

    fn advance(&mut self) {
        self.current = self.lexer.next();
    }

    fn check(&self, target_type: &TokenType) -> bool {
        if let Some(ref token) = self.current {
            token.token_type == *target_type
        } else {
            *target_type == TokenType::EOF
        }
    }

    fn consume(&mut self, expected: TokenType) -> Result<Token, String> {
        if self.check(&expected) {
            let token = self.current.clone();
            self.advance();

            token.ok_or_else(|| "[twee::error] unexpected end of input".to_string())
        } else {
            Err(format!(
                "[twee::error] expected {:?} but got {:?}",
                expected,
                self.current.as_ref().map(|t| &t.token_type)
            ))
        }
    }

    fn peek(&self) -> Option<&Token> {
        self.current.as_ref()
    }

    pub fn parse(&mut self) -> Result<Vec<Stmt>, String> {
        let mut stmts = Vec::new();

        while !self.check(&TokenType::EOF) {
            stmts.push(self.parse_stmt()?);
            if self.check(&TokenType::Semi) {
                self.advance();
            }
        }

        Ok(stmts)
    }

    fn parse_stmt(&mut self) -> Result<Stmt, String> {
        let stmt = match self.peek() {
            Some(token) => match token.token_type {
                TokenType::DataType => self.parse_variable_declaration()?,
                TokenType::Return => self.parse_return_stmt()?,
                _ => Stmt::Expression(self.parse_expr()?),
            },
            None => return Err("[twee::error] unexpected end of input".to_string()),
        };

        if self.check(&TokenType::Semi) {
            self.advance();
        }

        Ok(stmt)
    }

    /*
        Parse a return statement.
        Syntax:
            return value<Expr>;
        Example:
            return 42;
     */
    fn parse_return_stmt(&mut self) -> Result<Stmt, String> {
        self.consume(TokenType::Return)?;
        let value = self.parse_expr()?;
        Ok(Stmt::Return(Return { value }))
    }

    /*
        Parse a local variable declaration.
        Syntax:
            data_type<Ident> ident = value<Expr>;<Optional>
        Example:
            int number = 24;
    */
    fn parse_variable_declaration(&mut self) -> Result<Stmt, String> {
        /* Expect a data type token */
        let data_type = if self.check(&TokenType::DataType) {
            // self.advance();

            let data_type_str = self.consume(TokenType::DataType)?.lexeme;
            data_type_str
        } else {
            "auto".to_string()
        };

        /* Expect and consume an identifier, this is the variabels identifier. */
        let name = self.consume(TokenType::Identifier)?.lexeme;

        if self.check(&TokenType::LParen) {
            return self.parse_function_declaration(data_type, name);
        }

        /* Expect and consume an equals symbol. */
        self.consume(TokenType::Equals)?;

        /* Parse an expression for the value of the variable. */
        let value = self.parse_expr()?;

        Ok(Stmt::VariableDecl(VariableDecl {
            data_type,
            name,
            value,
        }))
    }

    /*
        Parse a function declaration.
        Syntax:
            int main() { ... }
    */
    fn parse_function_declaration(&mut self, data_type: String, name: String) -> Result<Stmt, String> {
        self.consume(TokenType::LParen)?;

        let mut params: Vec<Parameter> = vec![];

        /* empty fn params */
        if self.check(&TokenType::RParen) {
            self.advance();
        } else {
            loop {
                let param_type = self.consume(TokenType::DataType)?.lexeme;
                let param_name = self.consume(TokenType::Identifier)?.lexeme;

                params.push(Parameter {
                    data_type: param_type,
                    name: param_name
                });

                if self.check(&TokenType::Comma) {
                    self.advance();
                } else if self.check(&TokenType::RParen) {
                    self.advance(); // eat da ')'
                    break;
                } else {
                    return Err("Expected ',' or ')' in function parameters".to_string());
                }
            }
        }

        let mut body = vec![];
        self.consume(TokenType::LBrace)?;

        loop {
            if self.check(&TokenType::RBrace) { break; }
            body.push(self.parse_stmt()?);
        }

        self.consume(TokenType::RBrace)?;

        Ok(Stmt::FunctionDecl(FunctionDecl {
            data_type,
            name,
            body,
            params
        }))
    }

    /*
        Parse an ordinary expression.
    */
    fn parse_expr(&mut self) -> Result<Expr, String> {
        self.parse_precedence(0) /* Start with parsing by preceden */
    }

    fn parse_precedence(&mut self, min: u8) -> Result<Expr, String> {
        let mut left = self.parse_primary()?;

        while let Some(op) = self.binop() {
            let precedence = op.precedence();

            if precedence < min {
                break;
            }

            self.advance();

            let right_min = if op.is_left_linked() {
                precedence + 1
            } else {
                precedence
            };

            let right = self.parse_precedence(right_min)?;

            left = Expr::BinaryOp {
                left: Box::new(left),
                op,
                right: Box::new(right),
            };
        }

        Ok(left)
    }

    /*
        Parse primary expressions (literals, identifiers, and grouped expressions).
    */
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(token) => match token.token_type.clone() {
                /* Parse a numeric literal. */
                TokenType::Number => {
                    let value = token.lexeme.parse::<f64>().map_err(|e| e.to_string())?;
                    self.advance();

                    Ok(Expr::Number(value))
                }

                /* Parse a reference to an identifier */
                TokenType::Identifier => {
                    let value = token.lexeme.clone();
                    self.advance();

                    if self.check(&TokenType::LParen) {
                        return Ok(self.parse_function_call(value)?)
                    }

                    Ok(Expr::Identifier(value))
                }

                /* Parse a string literal. */
                TokenType::String => {
                    let value = token.lexeme.clone();
                    self.advance();

                    Ok(Expr::String(value))
                }

                /* Parse parenthesized expressions */
                TokenType::LParen => {
                    self.advance(); // consume '('
                    let expr = self.parse_expr()?;
                    self.consume(TokenType::RParen)?; // consume ')'
                    Ok(expr)
                }

                _ => Err(format!(
                    "[twee::error] unexpected token {:?}",
                    token.token_type
                )),
            },

            None => Err("[twee::error] unexpected end of input".to_string()),
        }
    }

    fn parse_function_call(&mut self, callee: String) -> Result<Expr, String> {
        self.advance();

        let mut args: Vec<Expr> = vec![];

        /* empty fn call args */
        if self.check(&TokenType::RParen) {
            self.advance();
            return Ok(FunctionCall { callee, args })
        }

        loop {
            args.push(self.parse_expr()?);

            if self.check(&TokenType::Comma) {
                self.advance();
            } else if self.check(&TokenType::RParen) {
                break;
            } else {
                return Err("Expected ',' or ')' in function call arguments list.".to_string());
            }
        }

        self.consume(TokenType::RParen)?;

        Ok(FunctionCall {
            callee,
            args
        })
    }

    /*
        Is the current token a binary operator? (add, sub, mul, div) if so return it as a binop.
    */
    fn binop(&self) -> Option<Binop> {
        match self.peek() {
            Some(tok) => match tok.token_type {
                TokenType::Add => Some(Binop::Add),
                TokenType::Sub => Some(Binop::Sub),
                TokenType::Mul => Some(Binop::Mul),
                TokenType::Div => Some(Binop::Div),
                _ => None,
            },

            None => None,
        }
    }
}
