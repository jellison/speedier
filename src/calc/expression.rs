use crate::calc::EvaluatorError;
use std::fmt;

#[derive(Clone, Debug)]
pub enum Expr {
    Number(f64),
    Ident(String),
    Unary {
        op: char,
        expr: Box<Expr>,
    },
    Postfix {
        op: char,
        expr: Box<Expr>,
    },
    Binary {
        op: char,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        name: String,
        args: Vec<Expr>,
    },
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum TokenKind {
    Eof,
    Number,
    Identifier,
    Operator,
    LParen,
    RParen,
    Comma,
}

#[derive(Clone, Debug)]
struct Token {
    kind: TokenKind,
    text: String,
}

pub fn parse_expression(input: &str) -> Result<Expr, EvaluatorError> {
    let mut parser = Parser::new(input)?;
    let expr = parser.parse_expression()?;
    if parser.peek().kind != TokenKind::Eof {
        return Err(EvaluatorError::new(format!(
            "unexpected token '{}'",
            parser.peek().text
        )));
    }
    Ok(expr)
}

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(input: &str) -> Result<Self, EvaluatorError> {
        let tokens = tokenize(input)?;
        Ok(Self { tokens, pos: 0 })
    }

    fn parse_expression(&mut self) -> Result<Expr, EvaluatorError> {
        self.parse_add_sub()
    }

    fn parse_add_sub(&mut self) -> Result<Expr, EvaluatorError> {
        let mut left = self.parse_mul_div()?;
        loop {
            if self.match_operator('+') {
                let right = self.parse_mul_div()?;
                left = Expr::Binary {
                    op: '+',
                    left: Box::new(left),
                    right: Box::new(right),
                };
                continue;
            }
            if self.match_operator('-') {
                let right = self.parse_mul_div()?;
                left = Expr::Binary {
                    op: '-',
                    left: Box::new(left),
                    right: Box::new(right),
                };
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_mul_div(&mut self) -> Result<Expr, EvaluatorError> {
        let mut left = self.parse_unary()?;
        loop {
            if self.match_operator('*') {
                let right = self.parse_unary()?;
                left = Expr::Binary {
                    op: '*',
                    left: Box::new(left),
                    right: Box::new(right),
                };
                continue;
            }
            if self.match_operator('/') {
                let right = self.parse_unary()?;
                left = Expr::Binary {
                    op: '/',
                    left: Box::new(left),
                    right: Box::new(right),
                };
                continue;
            }
            if self.match_operator('%') {
                let right = self.parse_unary()?;
                left = Expr::Binary {
                    op: '%',
                    left: Box::new(left),
                    right: Box::new(right),
                };
                continue;
            }
            break;
        }
        Ok(left)
    }

    fn parse_unary(&mut self) -> Result<Expr, EvaluatorError> {
        if self.match_operator('+') {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: '+',
                expr: Box::new(expr),
            });
        }
        if self.match_operator('-') {
            let expr = self.parse_unary()?;
            return Ok(Expr::Unary {
                op: '-',
                expr: Box::new(expr),
            });
        }
        self.parse_power()
    }

    fn parse_power(&mut self) -> Result<Expr, EvaluatorError> {
        let left = self.parse_postfix()?;
        if self.match_operator('^') {
            let right = self.parse_unary()?;
            return Ok(Expr::Binary {
                op: '^',
                left: Box::new(left),
                right: Box::new(right),
            });
        }
        Ok(left)
    }

    fn parse_postfix(&mut self) -> Result<Expr, EvaluatorError> {
        let mut expr = self.parse_primary()?;
        while self.match_operator('!') {
            expr = Expr::Postfix {
                op: '!',
                expr: Box::new(expr),
            };
        }
        Ok(expr)
    }

    fn parse_primary(&mut self) -> Result<Expr, EvaluatorError> {
        match self.peek().kind {
            TokenKind::Number => {
                let tok = self.next();
                let value: f64 = tok
                    .text
                    .parse()
                    .map_err(|_| EvaluatorError::new(format!("invalid number '{}'", tok.text)))?;
                Ok(Expr::Number(value))
            }
            TokenKind::Identifier => {
                let tok = self.next();
                if self.match_kind(TokenKind::LParen) {
                    let args = self.parse_args()?;
                    Ok(Expr::Call {
                        name: tok.text,
                        args,
                    })
                } else {
                    Ok(Expr::Ident(tok.text))
                }
            }
            TokenKind::LParen => {
                self.next();
                let expr = self.parse_expression()?;
                if !self.match_kind(TokenKind::RParen) {
                    return Err(EvaluatorError::new("expected ')'"));
                }
                Ok(expr)
            }
            TokenKind::Eof => Err(EvaluatorError::new("unexpected end of expression")),
            _ => Err(EvaluatorError::new(format!(
                "unexpected token '{}'",
                self.peek().text
            ))),
        }
    }

    fn parse_args(&mut self) -> Result<Vec<Expr>, EvaluatorError> {
        if self.match_kind(TokenKind::RParen) {
            return Ok(Vec::new());
        }

        let mut args = Vec::new();
        loop {
            let expr = self.parse_expression()?;
            args.push(expr);
            if self.match_kind(TokenKind::Comma) {
                continue;
            }
            if self.match_kind(TokenKind::RParen) {
                break;
            }
            return Err(EvaluatorError::new("expected ',' or ')'"));
        }
        Ok(args)
    }

    fn match_kind(&mut self, kind: TokenKind) -> bool {
        if self.peek().kind == kind {
            self.pos += 1;
            return true;
        }
        false
    }

    fn match_operator(&mut self, op: char) -> bool {
        let tok = self.peek();
        if tok.kind == TokenKind::Operator && tok.text == op.to_string() {
            self.pos += 1;
            return true;
        }
        false
    }

    fn peek(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&EOF_TOKEN)
    }

    fn next(&mut self) -> Token {
        let tok = self.peek().clone();
        if tok.kind != TokenKind::Eof {
            self.pos += 1;
        }
        tok
    }
}

static EOF_TOKEN: Token = Token {
    kind: TokenKind::Eof,
    text: String::new(),
};

fn tokenize(input: &str) -> Result<Vec<Token>, EvaluatorError> {
    let runes: Vec<char> = input.chars().collect();
    let mut tokens = Vec::with_capacity(runes.len());
    let mut i = 0;
    while i < runes.len() {
        let r = runes[i];
        if r.is_whitespace() {
            i += 1;
            continue;
        }
        match r {
            '(' => {
                tokens.push(Token {
                    kind: TokenKind::LParen,
                    text: "(".to_string(),
                });
                i += 1;
                continue;
            }
            ')' => {
                tokens.push(Token {
                    kind: TokenKind::RParen,
                    text: ")".to_string(),
                });
                i += 1;
                continue;
            }
            ',' => {
                tokens.push(Token {
                    kind: TokenKind::Comma,
                    text: ",".to_string(),
                });
                i += 1;
                continue;
            }
            '+' | '-' | '*' | '/' | '%' | '^' | '!' => {
                tokens.push(Token {
                    kind: TokenKind::Operator,
                    text: r.to_string(),
                });
                i += 1;
                continue;
            }
            _ => {}
        }

        if is_number_start(&runes, i) {
            let start = i;
            i = consume_number(&runes, i);
            tokens.push(Token {
                kind: TokenKind::Number,
                text: runes[start..i].iter().collect(),
            });
            continue;
        }

        if is_identifier_start(r) {
            let start = i;
            i += 1;
            while i < runes.len() && is_identifier_part(runes[i]) {
                i += 1;
            }
            tokens.push(Token {
                kind: TokenKind::Identifier,
                text: runes[start..i].iter().collect(),
            });
            continue;
        }

        return Err(EvaluatorError::new(format!("unexpected character '{}'", r)));
    }

    tokens.push(Token {
        kind: TokenKind::Eof,
        text: String::new(),
    });

    Ok(tokens)
}

fn is_number_start(runes: &[char], idx: usize) -> bool {
    let r = runes[idx];
    if r.is_ascii_digit() {
        return true;
    }
    r == '.' && idx + 1 < runes.len() && runes[idx + 1].is_ascii_digit()
}

fn consume_number(runes: &[char], idx: usize) -> usize {
    let mut i = idx;
    let mut seen_dot = false;
    while i < runes.len() {
        let r = runes[i];
        if r == '.' {
            if seen_dot {
                break;
            }
            seen_dot = true;
            i += 1;
            continue;
        }
        if !r.is_ascii_digit() {
            break;
        }
        i += 1;
    }

    if i < runes.len() && (runes[i] == 'e' || runes[i] == 'E') {
        let mut j = i + 1;
        if j < runes.len() && (runes[j] == '+' || runes[j] == '-') {
            j += 1;
        }
        if j < runes.len() && runes[j].is_ascii_digit() {
            i = j + 1;
            while i < runes.len() && runes[i].is_ascii_digit() {
                i += 1;
            }
        }
    }

    i
}

fn is_identifier_start(r: char) -> bool {
    r.is_alphabetic() || r == '_'
}

fn is_identifier_part(r: char) -> bool {
    r.is_alphanumeric() || r == '_'
}

impl fmt::Display for Expr {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Expr::Number(value) => write!(f, "{}", value),
            Expr::Ident(name) => write!(f, "{}", name),
            Expr::Unary { op, expr } => write!(f, "{}({})", op, expr),
            Expr::Postfix { op, expr } => write!(f, "{}{}", expr, op),
            Expr::Binary { op, left, right } => {
                if *op == '^' {
                    write!(f, "pow({}, {})", left, right)
                } else {
                    write!(f, "({}{}{})", left, op, right)
                }
            }
            Expr::Call { name, args } => {
                let parts: Vec<String> = args.iter().map(|arg| arg.to_string()).collect();
                write!(f, "{}({})", name, parts.join(","))
            }
        }
    }
}

pub fn prepend_ans_if_leading_operator(expr: &str) -> String {
    let trimmed = expr.trim();
    if trimmed.is_empty() {
        return expr.to_string();
    }
    let first = trimmed.chars().next().unwrap();
    match first {
        '+' | '-' | '*' | '/' | '%' | '^' => format!("ans{}", trimmed),
        _ => trimmed.to_string(),
    }
}
