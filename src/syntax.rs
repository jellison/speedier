#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TokenKind {
    Whitespace,
    Number,
    Operator,
    Function,
    Constant,
    Identifier,
    Paren,
    Comma,
    Unknown,
}

#[derive(Clone, Debug)]
pub struct Token {
    pub kind: TokenKind,
    pub text: String,
}

pub fn tokenize(input: &str) -> Vec<Token> {
    if input.is_empty() {
        return Vec::new();
    }

    let runes: Vec<char> = input.chars().collect();
    let mut out = Vec::new();
    let mut i = 0;
    while i < runes.len() {
        let r = runes[i];
        if r.is_whitespace() {
            let start = i;
            while i < runes.len() && runes[i].is_whitespace() {
                i += 1;
            }
            out.push(Token {
                kind: TokenKind::Whitespace,
                text: runes[start..i].iter().collect(),
            });
            continue;
        }
        if is_number_start(&runes, i) {
            let start = i;
            i = consume_number(&runes, i);
            out.push(Token {
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
            let ident: String = runes[start..i].iter().collect();
            out.push(Token {
                kind: classify_identifier(&ident, &runes, i),
                text: ident,
            });
            continue;
        }
        match r {
            '(' | ')' | '[' | ']' | '{' | '}' => {
                out.push(Token {
                    kind: TokenKind::Paren,
                    text: r.to_string(),
                });
                i += 1;
            }
            ',' => {
                out.push(Token {
                    kind: TokenKind::Comma,
                    text: r.to_string(),
                });
                i += 1;
            }
            _ if is_operator_rune(r) => {
                out.push(Token {
                    kind: TokenKind::Operator,
                    text: r.to_string(),
                });
                i += 1;
            }
            _ => {
                out.push(Token {
                    kind: TokenKind::Unknown,
                    text: r.to_string(),
                });
                i += 1;
            }
        }
    }

    out
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
        if r == 'e' || r == 'E' {
            i += 1;
            if i < runes.len() && (runes[i] == '+' || runes[i] == '-') {
                i += 1;
            }
            continue;
        }
        if !r.is_ascii_digit() {
            break;
        }
        i += 1;
    }
    i
}

fn is_identifier_start(r: char) -> bool {
    r.is_alphabetic() || r == '_'
}

fn is_identifier_part(r: char) -> bool {
    r.is_alphanumeric() || r == '_'
}

fn is_operator_rune(r: char) -> bool {
    matches!(r, '+' | '-' | '*' | '/' | '%' | '^' | '!' | '=')
}

fn classify_identifier(ident: &str, runes: &[char], idx: usize) -> TokenKind {
    let lower = ident.to_lowercase();
    match lower.as_str() {
        "ans" | "pi" | "e" | "nan" | "inf" | "infinity" => return TokenKind::Constant,
        _ => {}
    }

    if is_function_name(&lower) {
        return TokenKind::Function;
    }

    if next_non_space_is(runes, idx, '(') {
        return TokenKind::Function;
    }

    TokenKind::Identifier
}

fn is_function_name(name: &str) -> bool {
    matches!(
        name,
        "sin" | "cos" | "tan" | "sqrt" | "pow" | "log" | "ln" | "abs" | "ceil" | "floor"
    )
}

fn next_non_space_is(runes: &[char], idx: usize, target: char) -> bool {
    let mut i = idx;
    while i < runes.len() {
        if runes[i].is_whitespace() {
            i += 1;
            continue;
        }
        return runes[i] == target;
    }
    false
}
