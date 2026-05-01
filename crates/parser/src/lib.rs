//! Expression parser: tokenizes a string and builds an AST.
//!
//! Supported grammar (precedence low → high):
//!   expr    = term   (('+' | '-') term)*
//!   term    = unary  (('*' | '/') unary)*
//!   unary   = '-' unary | power            ← unary minus < exponentiation
//!   power   = primary ('^' unary)?         (right-associative)
//!   primary = NUMBER | VAR | '(' expr ')' | FUNC '(' expr ')'
//!
//! Functions: sin, cos, tan, exp, ln, sqrt, abs
//! Constants: pi, e
//! Variable:  any identifier that is not a function or constant

// ────────────────────────────────────────────────────────────
// AST
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
pub enum Expr {
    Num(f64),
    Var(String),                      // any variable identifier
    Neg(Box<Expr>),
    Add(Box<Expr>, Box<Expr>),
    Sub(Box<Expr>, Box<Expr>),
    Mul(Box<Expr>, Box<Expr>),
    Div(Box<Expr>, Box<Expr>),
    Pow(Box<Expr>, Box<Expr>),
    Sin(Box<Expr>),
    Cos(Box<Expr>),
    Tan(Box<Expr>),
    Exp(Box<Expr>),
    Ln(Box<Expr>),
    Sqrt(Box<Expr>),
    Abs(Box<Expr>),
}

impl Expr {
    /// Evaluate the expression for a given value of the variable.
    pub fn eval(&self, x: f64) -> f64 {
        match self {
            Expr::Num(n)    => *n,
            Expr::Var(_)    => x,
            Expr::Neg(e)    => -e.eval(x),
            Expr::Add(a, b) => a.eval(x) + b.eval(x),
            Expr::Sub(a, b) => a.eval(x) - b.eval(x),
            Expr::Mul(a, b) => a.eval(x) * b.eval(x),
            Expr::Div(a, b) => a.eval(x) / b.eval(x),
            Expr::Pow(a, b) => a.eval(x).powf(b.eval(x)),
            Expr::Sin(e)    => e.eval(x).sin(),
            Expr::Cos(e)    => e.eval(x).cos(),
            Expr::Tan(e)    => e.eval(x).tan(),
            Expr::Exp(e)    => e.eval(x).exp(),
            Expr::Ln(e)     => e.eval(x).ln(),
            Expr::Sqrt(e)   => e.eval(x).sqrt(),
            Expr::Abs(e)    => e.eval(x).abs(),
        }
    }

    /// Pretty-print the expression with minimal parentheses.
    pub fn to_string_repr(&self) -> String {
        fmt_expr(self, 0)
    }
}

// Precedence levels: higher = tighter binding
// 1 = +/-,  2 = *//,  3 = unary -,  4 = ^,  5 = atom/function
fn prec(e: &Expr) -> u8 {
    match e {
        Expr::Add(_,_) | Expr::Sub(_,_) => 1,
        Expr::Mul(_,_) | Expr::Div(_,_) => 2,
        Expr::Neg(_)                     => 3,
        Expr::Pow(_,_)                   => 4,
        _                                => 5,
    }
}

fn needs_parens(e: &Expr, parent_prec: u8) -> bool {
    prec(e) < parent_prec
}

fn fmt_num(n: f64) -> String {
    // Recognise mathematical constants and simple multiples before generic formatting
    if (n - std::f64::consts::PI).abs() < 1e-12 { return "π".to_string(); }
    if (n - std::f64::consts::E).abs()  < 1e-12 { return "e".to_string(); }
    // n = k*π or n = (p/q)*π for small integers
    let pi = std::f64::consts::PI;
    let n_over_pi = n / pi;
    if n_over_pi.abs() > 1e-12 {
        // integer multiple: 2π, 3π, -π, …
        if (n_over_pi - n_over_pi.round()).abs() < 1e-9 {
            let k = n_over_pi.round() as i64;
            if k.abs() <= 100 && k != 0 && k != 1 {
                return if k == -1 { "-π".to_string() } else { format!("{}*π", k) };
            }
        }
        // simple fraction multiple: π/2, π/3, 2π/3, …
        for denom in 2i64..=12 {
            let numer = (n_over_pi * denom as f64).round() as i64;
            if numer != 0 && (numer as f64 / denom as f64 - n_over_pi).abs() < 1e-9 {
                let g = gcd(numer.unsigned_abs(), denom.unsigned_abs()) as i64;
                let p = numer / g;
                let q = denom / g;
                if q > 1 {
                    return if p == 1 { format!("π/{}", q) }
                           else if p == -1 { format!("-π/{}", q) }
                           else { format!("{}*π/{}", p, q) };
                }
            }
        }
    }
    if n == n.floor() && n.abs() < 1e15 {
        return format!("{}", n as i64);
    }
    // Try to express as a simple fraction p/q with small denominator
    for denom in 2i64..=99 {
        let numer = (n * denom as f64).round() as i64;
        if (numer as f64 / denom as f64 - n).abs() < 1e-12 {
            // Reduce by GCD
            let g = gcd(numer.unsigned_abs(), denom.unsigned_abs()) as i64;
            let p = numer / g;
            let q = denom / g;
            if q == 1 {
                return format!("{}", p);
            }
            return format!("{}/{}", p, q);
        }
    }
    format!("{}", n)
}

fn gcd(a: u64, b: u64) -> u64 {
    if b == 0 { a } else { gcd(b, a % b) }
}

fn paren(s: String, wrap: bool) -> String {
    if wrap { format!("({})", s) } else { s }
}

fn fmt_expr(e: &Expr, parent_prec: u8) -> String {
    let s = match e {
        Expr::Num(n)     => fmt_num(*n),
        Expr::Var(name)  => name.clone(),

        Expr::Neg(inner) => {
            let wrap = matches!(inner.as_ref(), Expr::Add(_,_) | Expr::Sub(_,_));
            format!("-{}", paren(fmt_expr(inner, 3), wrap))
        }

        Expr::Add(a, b) => {
            // a + (-x)  →  a - x
            if let Expr::Neg(inner) = b.as_ref() {
                let wrap_rhs = matches!(inner.as_ref(), Expr::Add(_,_) | Expr::Sub(_,_));
                return format!("{} - {}", fmt_expr(a, 1), paren(fmt_expr(inner, 1), wrap_rhs));
            }
            format!("{} + {}", fmt_expr(a, 1), fmt_expr(b, 1))
        }

        Expr::Sub(a, b) => {
            // wrap rhs if it is add/sub to preserve meaning
            let wrap_rhs = matches!(b.as_ref(), Expr::Add(_,_) | Expr::Sub(_,_));
            format!("{} - {}", fmt_expr(a, 1), paren(fmt_expr(b, 1), wrap_rhs))
        }

        Expr::Mul(a, b) => {
            let ls = paren(fmt_expr(a, 2), needs_parens(a, 2));
            let rs = paren(fmt_expr(b, 2), needs_parens(b, 2));
            format!("{}*{}", ls, rs)
        }

        Expr::Div(a, b) => {
            let wrap_num = matches!(a.as_ref(), Expr::Add(_,_) | Expr::Sub(_,_));
            let wrap_den = matches!(b.as_ref(),
                Expr::Add(_,_) | Expr::Sub(_,_) | Expr::Mul(_,_) | Expr::Div(_,_));
            format!("{}/{}", paren(fmt_expr(a, 2), wrap_num), paren(fmt_expr(b, 2), wrap_den))
        }

        Expr::Pow(base, exp) => {
            let wrap_base = matches!(base.as_ref(),
                Expr::Add(_,_) | Expr::Sub(_,_) | Expr::Mul(_,_) | Expr::Div(_,_) | Expr::Neg(_));
            let wrap_exp = matches!(exp.as_ref(), Expr::Add(_,_) | Expr::Sub(_,_));
            format!("{}^{}", paren(fmt_expr(base, 4), wrap_base), paren(fmt_expr(exp, 4), wrap_exp))
        }

        Expr::Sin(inner)  => format!("sin({})",  fmt_expr(inner, 0)),
        Expr::Cos(inner)  => format!("cos({})",  fmt_expr(inner, 0)),
        Expr::Tan(inner)  => format!("tan({})",  fmt_expr(inner, 0)),
        Expr::Exp(inner)  => format!("exp({})",  fmt_expr(inner, 0)),
        Expr::Ln(inner)   => format!("ln({})",   fmt_expr(inner, 0)),
        Expr::Sqrt(inner) => format!("sqrt({})", fmt_expr(inner, 0)),
        Expr::Abs(inner)  => format!("abs({})",  fmt_expr(inner, 0)),
    };
    paren(s, needs_parens(e, parent_prec))
}

// ────────────────────────────────────────────────────────────
// Tokens
// ────────────────────────────────────────────────────────────

#[derive(Debug, Clone, PartialEq)]
enum Token {
    Num(f64),
    Ident(String),
    Plus,
    Minus,
    Star,
    Slash,
    Caret,
    LParen,
    RParen,
}

fn tokenize(input: &str) -> Result<Vec<Token>, String> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => { chars.next(); }
            '+' => { tokens.push(Token::Plus);   chars.next(); }
            '-' => { tokens.push(Token::Minus);  chars.next(); }
            '*' => { tokens.push(Token::Star);   chars.next(); }
            '/' => { tokens.push(Token::Slash);  chars.next(); }
            '^' => { tokens.push(Token::Caret);  chars.next(); }
            '(' => { tokens.push(Token::LParen); chars.next(); }
            ')' => { tokens.push(Token::RParen); chars.next(); }
            '0'..='9' | '.' => {
                let mut num = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        num.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                // optional exponent
                if let Some(&'e') | Some(&'E') = chars.peek() {
                    num.push('e');
                    chars.next();
                    if let Some(&s) = chars.peek() {
                        if s == '+' || s == '-' {
                            num.push(s);
                            chars.next();
                        }
                    }
                    while let Some(&c) = chars.peek() {
                        if c.is_ascii_digit() {
                            num.push(c);
                            chars.next();
                        } else {
                            break;
                        }
                    }
                }
                let v: f64 = num.parse().expect("tokenizer built a non-f64 string — impossible");
                tokens.push(Token::Num(v));
            }
            'a'..='z' | 'A'..='Z' | '_' => {
                let mut ident = String::new();
                while let Some(&c) = chars.peek() {
                    if c.is_alphanumeric() || c == '_' {
                        ident.push(c);
                        chars.next();
                    } else {
                        break;
                    }
                }
                tokens.push(Token::Ident(ident));
            }
            'π' => { tokens.push(Token::Num(std::f64::consts::PI)); chars.next(); }
            'τ' => { tokens.push(Token::Num(std::f64::consts::TAU)); chars.next(); }
            other => return Err(format!("Unexpected character: '{}'", other)),
        }
    }
    Ok(tokens)
}

// ────────────────────────────────────────────────────────────
// Recursive-descent parser
// ────────────────────────────────────────────────────────────

struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    fn new(tokens: Vec<Token>) -> Self {
        Parser { tokens, pos: 0 }
    }

    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    fn consume(&mut self) -> Option<Token> {
        let t = self.tokens.get(self.pos).cloned();
        self.pos += 1;
        t
    }

    fn expect(&mut self, expected: &Token) -> Result<(), String> {
        match self.peek() {
            Some(t) if t == expected => { self.consume(); Ok(()) }
            Some(t) => Err(format!("Expected {:?}, found {:?}", expected, t)),
            None    => Err(format!("Expected {:?}, found end of input", expected)),
        }
    }

    // expr = term (('+' | '-') term)*
    fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_term()?;
        loop {
            match self.peek() {
                Some(Token::Plus) => {
                    self.consume();
                    let rhs = self.parse_term()?;
                    lhs = Expr::Add(Box::new(lhs), Box::new(rhs));
                }
                Some(Token::Minus) => {
                    self.consume();
                    let rhs = self.parse_term()?;
                    lhs = Expr::Sub(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    // term = unary (('*' | '/') unary)*
    fn parse_term(&mut self) -> Result<Expr, String> {
        let mut lhs = self.parse_unary()?;
        loop {
            match self.peek() {
                Some(Token::Star) => {
                    self.consume();
                    let rhs = self.parse_unary()?;
                    lhs = Expr::Mul(Box::new(lhs), Box::new(rhs));
                }
                Some(Token::Slash) => {
                    self.consume();
                    let rhs = self.parse_unary()?;
                    lhs = Expr::Div(Box::new(lhs), Box::new(rhs));
                }
                _ => break,
            }
        }
        Ok(lhs)
    }

    // unary = '-' unary | power
    // Unary minus has LOWER precedence than '^', so -x^2 = -(x^2).
    fn parse_unary(&mut self) -> Result<Expr, String> {
        if let Some(Token::Minus) = self.peek() {
            self.consume();
            let e = self.parse_unary()?;
            // fold negation of a numeric literal immediately
            Ok(match e {
                Expr::Num(n) => Expr::Num(-n),
                other        => Expr::Neg(Box::new(other)),
            })
        } else {
            self.parse_power()
        }
    }

    // power = primary ('^' unary)?   (right-associative)
    fn parse_power(&mut self) -> Result<Expr, String> {
        let base = self.parse_primary()?;
        if let Some(Token::Caret) = self.peek() {
            self.consume();
            let exp = self.parse_unary()?;   // right-recursive through unary
            Ok(Expr::Pow(Box::new(base), Box::new(exp)))
        } else {
            Ok(base)
        }
    }

    // primary = NUMBER | VAR | '(' expr ')' | FUNC '(' expr ')'
    //
    // Implicit multiplication is recognized when a number or non-function
    // identifier is immediately followed by an identifier or '('.
    // Examples: 3x → 3*x,  2t^3 → 2*(t^3),  3(x+1) → 3*(x+1),  2sin(x) → 2*sin(x)
    fn parse_primary(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Token::Num(n)) => {
                self.consume();
                let lhs = Expr::Num(n);
                // implicit multiplication: NUMBER immediately followed by ident or '('
                // e.g. 3x^2 → 3*(x^2),  2sin(x) → 2*sin(x),  3(x+1) → 3*(x+1)
                // We call parse_power (not parse_primary) so that 3x^2 = 3*(x^2).
                match self.peek() {
                    Some(Token::Ident(_)) | Some(Token::LParen) => {
                        let rhs = self.parse_power()?;
                        Ok(Expr::Mul(Box::new(lhs), Box::new(rhs)))
                    }
                    _ => Ok(lhs),
                }
            }
            Some(Token::LParen) => {
                self.consume();
                let e = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(e)
            }
            Some(Token::Ident(ref name)) => {
                let name = name.clone();
                self.consume();
                match name.as_str() {
                    "pi" | "PI" => {
                        let lhs = Expr::Num(std::f64::consts::PI);
                        match self.peek() {
                            Some(Token::LParen) => {
                                let rhs = self.parse_power()?;
                                Ok(Expr::Mul(Box::new(lhs), Box::new(rhs)))
                            }
                            _ => Ok(lhs),
                        }
                    }
                    "e" | "E" => {
                        let lhs = Expr::Num(std::f64::consts::E);
                        match self.peek() {
                            Some(Token::LParen) => {
                                let rhs = self.parse_power()?;
                                Ok(Expr::Mul(Box::new(lhs), Box::new(rhs)))
                            }
                            _ => Ok(lhs),
                        }
                    }
                    // Known function names: must be followed by '('
                    "sin" | "cos" | "tan" | "exp" | "ln" | "log" | "sqrt" | "abs" => {
                        self.expect(&Token::LParen)?;
                        let arg = self.parse_expr()?;
                        self.expect(&Token::RParen)?;
                        match name.as_str() {
                            "sin"  => Ok(Expr::Sin(Box::new(arg))),
                            "cos"  => Ok(Expr::Cos(Box::new(arg))),
                            "tan"  => Ok(Expr::Tan(Box::new(arg))),
                            "exp"  => Ok(Expr::Exp(Box::new(arg))),
                            "ln" | "log" => Ok(Expr::Ln(Box::new(arg))),
                            "sqrt" => Ok(Expr::Sqrt(Box::new(arg))),
                            "abs"  => Ok(Expr::Abs(Box::new(arg))),
                            _      => unreachable!(),
                        }
                    }
                    // Any other identifier is the variable
                    _ => {
                        let lhs = Expr::Var(name);
                        // implicit multiplication: var immediately followed by '('
                        // e.g. t(t+1) → t*(t+1)
                        match self.peek() {
                            Some(Token::LParen) => {
                                let rhs = self.parse_power()?;
                                Ok(Expr::Mul(Box::new(lhs), Box::new(rhs)))
                            }
                            _ => Ok(lhs),
                        }
                    }
                }
            }
            Some(t) => Err(format!("Unexpected token: {:?}", t)),
            None    => Err("Unexpected end of input".to_string()),
        }
    }
}

// ────────────────────────────────────────────────────────────
// Public API
// ────────────────────────────────────────────────────────────

/// Parse a mathematical expression string into an AST.
pub fn parse(input: &str) -> Result<Expr, String> {
    let tokens = tokenize(input)?;
    let mut parser = Parser::new(tokens);
    let expr = parser.parse_expr()?;
    if parser.pos < parser.tokens.len() {
        return Err(format!(
            "Unexpected token after expression: {:?}",
            parser.tokens[parser.pos]
        ));
    }
    Ok(expr)
}
