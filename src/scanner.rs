use crate::token::*;
use crate::Result;
use std::collections::HashMap;
use std::error;
use std::fmt;
use std::ops::Range;

pub struct Scanner {
    source: Vec<char>,
    tokens: Vec<Token>,
    start: usize,
    current: usize,
    line: usize,
}

lazy_static! {
    static ref RESERVED_WORDS: HashMap<&'static str, TokenType> = {
        let mut m = HashMap::new();
        m.insert("and", TokenType::And);
        m.insert("class", TokenType::Class);
        m.insert("else", TokenType::Else);
        m.insert("false", TokenType::False);
        m.insert("for", TokenType::For);
        m.insert("fun", TokenType::Fun);
        m.insert("if", TokenType::If);
        m.insert("nil", TokenType::Nil);
        m.insert("or", TokenType::Or);
        m.insert("print", TokenType::Print);
        m.insert("return", TokenType::Return);
        m.insert("super", TokenType::Super);
        m.insert("this", TokenType::This);
        m.insert("true", TokenType::True);
        m.insert("var", TokenType::Var);
        m.insert("while", TokenType::While);
        m
    };
}

impl Scanner {
    pub fn new(source: &str) -> Self {
        let chars = source.chars().collect();
        Scanner {
            source: chars,
            tokens: Vec::new(),
            start: 0,
            current: 0,
            line: 1,
        }
    }

    pub fn scan_tokens(&mut self) -> Result<Vec<Token>> {
        while !self.is_at_end() {
            self.start = self.current;
            self.scan_token()?;
        }

        self.tokens
            .push(Token::new(TokenType::EOF, String::new(), None, self.line));
        Ok(self.tokens.clone())
    }

    fn scan_token(&mut self) -> Result<()> {
        match self.advance() {
            '(' => self.add_token(TokenType::LeftParen),
            ')' => self.add_token(TokenType::RightParen),
            '{' => self.add_token(TokenType::LeftBrace),
            '}' => self.add_token(TokenType::RightBrace),
            ',' => self.add_token(TokenType::Comma),
            '.' => self.add_token(TokenType::Dot),
            '-' => self.add_token(TokenType::Minus),
            '+' => self.add_token(TokenType::Plus),
            ';' => self.add_token(TokenType::Semicolon),
            '*' => self.add_token(TokenType::Star),
            '!' => {
                let typ = if self.matches('=') {
                    TokenType::BangEqual
                } else {
                    TokenType::Bang
                };
                self.add_token(typ);
            }
            '=' => {
                let typ = if self.matches('=') {
                    TokenType::EqualEqual
                } else {
                    TokenType::Equal
                };
                self.add_token(typ);
            }
            '<' => {
                let typ = if self.matches('=') {
                    TokenType::LessEqual
                } else {
                    TokenType::Less
                };
                self.add_token(typ);
            }
            '>' => {
                let typ = if self.matches('=') {
                    TokenType::GreaterEqual
                } else {
                    TokenType::Greater
                };
                self.add_token(typ);
            }
            '/' => {
                if self.matches('/') {
                    while self.peek() != Some('\n') && !self.is_at_end() {
                        self.advance();
                    }
                } else if self.matches('*') {
                    self.multiline_comment()?;
                } else {
                    self.add_token(TokenType::Slash);
                }
            }
            ' ' | '\r' | '\t' => {}
            '\n' => self.line += 1,
            '"' => self.string()?,
            '0'..='9' => self.number()?,
            'a'..='z' | 'A'..='Z' | '_' => self.identifier(),
            _ => {
                return Err(
                    ScannerError::new(self.line, String::from("Unexpected character.")).into(),
                )
            }
        }
        Ok(())
    }

    fn add_token(&mut self, typ: TokenType) {
        self.add_token_literal(typ, None);
    }

    fn add_token_literal(&mut self, typ: TokenType, literal: Option<Literal>) {
        let text = self.value_for(self.start..self.current);
        self.tokens.push(Token::new(typ, text, literal, self.line));
    }

    fn advance(&mut self) -> char {
        let char = self.source[self.current];
        self.current += 1;
        char
    }

    fn matches(&mut self, expected: char) -> bool {
        if self.is_at_end() || self.source[self.current] != expected {
            false
        } else {
            self.current += 1;
            true
        }
    }

    fn peek(&mut self) -> Option<char> {
        if self.is_at_end() {
            None
        } else {
            Some(self.source[self.current])
        }
    }

    fn peek_next(&mut self) -> Option<char> {
        if self.current + 1 >= self.source.len() {
            None
        } else {
            Some(self.source[self.current + 1])
        }
    }

    fn is_at_end(&self) -> bool {
        self.current >= self.source.len()
    }

    fn string(&mut self) -> Result<()> {
        while self.peek() != Some('"') && !self.is_at_end() {
            if self.advance() == '\n' {
                self.line += 1;
            }
        }

        if self.is_at_end() {
            return Err(ScannerError::new(self.line, String::from("Unterminated string.")).into());
        }

        self.advance();

        let value = self.value_for(self.start + 1..self.current - 1);
        self.add_token_literal(TokenType::String, Some(Literal::String(value)));

        Ok(())
    }

    fn number(&mut self) -> Result<()> {
        while let Some('0'..='9') = self.peek() {
            self.advance();
        }

        // Look for a fractional part
        if self.peek() == Some('.') {
            if let Some('0'..='9') = self.peek_next() {
                // Consume the "."
                self.advance();

                while let Some('0'..='9') = self.peek() {
                    self.advance();
                }
            }
        }

        let value = self.value_for(self.start..self.current);
        self.add_token_literal(TokenType::Number, Some(Literal::Number(value.parse()?)));

        Ok(())
    }

    fn identifier(&mut self) {
        while let Some('a'..='z' | 'A'..='Z' | '0'..='9' | '_') = self.peek() {
            self.advance();
        }

        let text = self.value_for(self.start..self.current);
        let typ = match RESERVED_WORDS.get(text.as_str()) {
            Some(&typ) => typ,
            None => TokenType::Identifier,
        };

        match typ {
            TokenType::True => {
                self.add_token_literal(typ, Some(Literal::Boolean(true)));
            }
            TokenType::False => {
                self.add_token_literal(typ, Some(Literal::Boolean(false)));
            }
            TokenType::Nil => {
                self.add_token_literal(typ, Some(Literal::Nil));
            }
            typ => self.add_token(typ),
        }
    }

    fn multiline_comment(&mut self) -> Result<()> {
        // Search for matching "*/"
        loop {
            while self.peek() != Some('*') && !self.is_at_end() {
                if self.advance() == '\n' {
                    self.line += 1;
                }
            }

            if self.is_at_end() {
                return Err(
                    ScannerError::new(self.line, String::from("Unterminated comment.")).into(),
                );
            }

            // Consume '*'
            self.advance();

            if self.matches('/') {
                break;
            }
        }
        Ok(())
    }

    fn value_for(&self, index: Range<usize>) -> String {
        self.source.get(index).unwrap().iter().collect()
    }
}

#[derive(Debug, Clone)]
pub struct ScannerError {
    line: usize,
    description: String,
}

impl ScannerError {
    pub fn new(line: usize, description: String) -> Self {
        ScannerError { line, description }
    }
}

impl error::Error for ScannerError {}

impl fmt::Display for ScannerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[line {}] Error: {}", self.line, self.description)
    }
}
