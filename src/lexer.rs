use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq)]
pub enum TokenType {
    Identifier,
    Number,
    String,
    Equals,
    DataType,
    Colon,
    Semi,
    Add,
    Sub,
    Mul,
    Div,
    LParen,
    RParen,
    LBrace,
    RBrace,
    Comma,
    EOF,
}

#[derive(Debug, Clone)]
pub struct Token {
    pub token_type: TokenType,
    pub lexeme: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone)]
pub struct Lexer<'a> {
    source: &'a str,
    chars: std::str::Chars<'a>,
    current: Option<char>,
    pos: usize,
    line: usize,
    column: usize,
    keywords: HashMap<&'a str, TokenType>,
}

impl<'a> Lexer<'a> {
    pub fn new(source: &'a str) -> Self {
        let mut keywords = HashMap::new();

        keywords.insert("int", TokenType::DataType);
        keywords.insert("char", TokenType::DataType);
        keywords.insert("char*", TokenType::DataType);

        let mut chars = source.chars();
        let current = chars.next();

        Self {
            source,
            chars,
            current,
            pos: 0,
            line: 1,
            column: 1,
            keywords,
        }
    }

    /*
        Main processor for returning the
        next token in the source code.
    */
    fn next_token(&mut self) -> Result<Token, String> {
        loop {
            self.skip_whitespace();

            let ch = match self.current {
                Some(c) => c,
                None => return Ok(self.make(TokenType::EOF, String::new())),
            };

            /* Skip comments */
            if ch == '-' && self.peek() == Some('-') {
                self.skip_comment();
                continue;
            }

            let token = match ch {
                /* Process string literals start with " or ' */
                '"' | '\'' => {
                    let value = self.process_string()?;
                    self.make(TokenType::String, value)
                }

                /* Process numeric literals */
                c if c.is_ascii_digit() => {
                    let value = self.process_numeric();
                    self.make(TokenType::Number, value)
                }

                /* Process identifiers and keywords (if they exist) */
                c if c.is_ascii_alphabetic() || c == '_' => {
                    let value = self.process_identifier();
                    let typ = self
                        .keywords
                        .get(value.as_str())
                        .cloned()
                        .unwrap_or(TokenType::Identifier);
                    self.make(typ, value)
                }

                '=' => {
                    self.advance();
                    self.make(TokenType::Equals, ch.to_string())
                }

                ';' => {
                    self.advance();
                    self.make(TokenType::Semi, ch.to_string())
                }

                ':' => {
                    self.advance();
                    self.make(TokenType::Colon, ch.to_string())
                }

                ',' => {
                    self.advance();
                    self.make(TokenType::Comma, ch.to_string())
                }

                '+' => {
                    self.advance();
                    self.make(TokenType::Add, ch.to_string())
                }

                '-' => {
                    self.advance();
                    self.make(TokenType::Sub, ch.to_string())
                }

                '/' => {
                    self.advance();
                    self.make(TokenType::Div, ch.to_string())
                }

                '*' => {
                    self.advance();
                    self.make(TokenType::Mul, ch.to_string())
                }

                '(' => {
                    self.advance();
                    self.make(TokenType::LParen, ch.to_string())
                }

                ')' => {
                    self.advance();
                    self.make(TokenType::RParen, ch.to_string())
                }

                '{' => {
                    self.advance();
                    self.make(TokenType::LBrace, ch.to_string())
                }

                '}' => {
                    self.advance();
                    self.make(TokenType::RBrace, ch.to_string())
                }

                _ => {
                    self.advance();
                    return Err(format!("[twee::error] unknown character '{}'", ch));
                }
            };

            return Ok(token);
        }
    }

    /*
        Wrapper for next_token to return a token, or an error token.
    */
    pub fn next(&mut self) -> Option<Token> {
        match self.next_token() {
            Ok(t) => Some(t),
            // Err(e) => Some(),
            Err(_) => None,
        }
    }

    /*
        Skip if the current character is a whitespace.
    */
    fn skip_whitespace(&mut self) {
        while let Some(c) = self.current() {
            if c.is_whitespace() {
                self.advance();
            } else {
                break;
            }
        }
    }

    /*
        Skip any characters trailing a comment
    */
    fn skip_comment(&mut self) {
        /* Does current == - and next == - ?? */
        if self.current == Some('-') && self.peek() == Some('-') {
            while let Some(ch) = self.current() {
                if ch == '\n' {
                    break;
                }

                self.advance();
            }
        }
    }

    /*
        Helper for initiating a new token.
    */
    fn make(&mut self, token_type: TokenType, lexeme: String) -> Token {
        Token {
            line: self.line,
            column: self.column,
            token_type,
            lexeme,
        }
    }

    /*
        Returns the character one position ahead of the current character.
    */
    fn peek(&self) -> Option<char> {
        let mut chars = self.chars.clone();
        chars.next()
    }

    /*
        Returns a character depending on the position.
    */
    fn current(&mut self) -> Option<char> {
        self.source.chars().nth(self.pos)
    }

    /*
        Advances to the next character by incrementing the position.
    */
    fn advance(&mut self) {
        self.pos += 1;
        if self.current() == Some('\n') {
            self.line += 1;
            self.column = 1;
        } else {
            self.column += 1;
        }

        self.current = self.chars.next();
    }

    /*
        This function is responsible for processing a string literal.
    */
    fn process_string(&mut self) -> Result<String, String> {
        /* Track the opening quote so we can properly terminate the string. */
        let opening_quote = self.current();
        self.advance();

        let mut value = String::new();
        while let Some(ch) = self.current() {
            if ch == opening_quote.unwrap() {
                self.advance();
                return Ok(value);
            }

            /*
                Process escape characters.
                When we encounter a \ expect another character for an escape char
            */
            if ch == '\\' {
                self.advance();
                match self.current {
                    Some('n') => value.push('\n'),
                    Some('t') => value.push('\t'),
                    Some('r') => value.push('\r'),
                    Some('\\') => value.push('\\'),
                    Some('\'') => value.push('\''),
                    Some('"') => value.push('"'),
                    Some('0') => value.push('\0'),
                    Some(c) => {
                        value.push('\\');
                        value.push(c);
                    }

                    None => return Err("Unterminated string literal".to_string()),
                }
            } else {
                value.push(ch);
            }

            self.advance();
        }

        /*
            Assume the string is unterminated, 9/10 it is.
        */
        Err("Unterminated string literal".to_string())
    }

    /*
        This function is responsible for processing a numeric literal.
    */
    fn process_numeric(&mut self) -> String {
        let mut value = String::new();
        /* Track if this is a floating point numeric literal */
        let mut floating = false;

        while let Some(ch) = self.current() {
            /*
                If the current character is a number, advance.
                However, if it's a '.' character, and floating flag isn't true,
                and if the next character after '.' is a number, then set the flag to true,
                and advance.
                if all fails, break.
            */
            match ch {
                c if c.is_numeric() => {
                    self.advance();
                    value.push(c);
                }
                '.' if !floating && self.peek().map_or(false, |n| n.is_numeric()) => {
                    floating = true;
                    value.push(ch);
                    self.advance();
                }
                _ => break,
            }
        }

        value
    }

    /*
        This function is responsible for processing an identifier.
    */
    fn process_identifier(&mut self) -> String {
        let mut value = String::new();

        while let Some(ch) = self.current() {
            if ch.is_ascii_alphanumeric() || ch == '_' || ch == '*' {
                value.push(ch);
                self.advance();
            } else {
                break;
            }
        }

        /* Just return the identifier as a string, will be handled elsewhere */
        value
    }
}
