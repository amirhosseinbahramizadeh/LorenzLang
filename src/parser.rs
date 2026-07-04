use crate::engine::Expr;

/// Tokens produced by the lexer for the chaotic expression language.
#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    /// An identifier (variable name): e.g., `temp`, `humidity`
    Identifier(String),
    /// A numeric literal: e.g., `42.5`, `0.001`
    Number(f64),
    /// Addition operator: `+`
    Plus,
    /// Left parenthesis: `(`
    LParen,
    /// Right parenthesis: `)`
    RParen,
    /// Comma separator (for function arguments): `,`
    Comma,
    /// Semicolon separator (for statements): `;`
    Semicolon,
    /// Assignment operator: `=`
    Equals,
    /// Keyword: `let`
    Let,
    /// Keyword: `chaotic`
    Chaotic,
}

/// Lexer that tokenizes a string into `Token`s`.
///
/// Supports identifiers, numbers, `+`, `(`, `)`, `,`, `;`, `=`, and keywords.
/// Whitespace is ignored.
pub struct Lexer {
    input: Vec<char>,
    pos: usize,
}

impl Lexer {
    /// Creates a new Lexer for the given input string.
    pub fn new(input: &str) -> Self {
        Self {
            input: input.chars().collect(),
            pos: 0,
        }
    }

    /// Peeks at the current character without consuming it.
    fn peek(&self) -> Option<char> {
        self.input.get(self.pos).copied()
    }

    /// Consumes and returns the current character.
    fn advance(&mut self) -> Option<char> {
        let ch = self.input.get(self.pos).copied();
        if ch.is_some() {
            self.pos += 1;
        }
        ch
    }

    /// Skips whitespace characters (but not newlines).
    fn skip_whitespace(&mut self) {
        while let Some(ch) = self.peek() {
            if ch.is_whitespace() && ch != '\n' {
                self.advance();
            } else {
                break;
            }
        }
    }

    /// Reads a number (integer or decimal).
    fn read_number(&mut self) -> Result<f64, String> {
        let mut s = String::new();

        // Optional leading minus
        if let Some('-') = self.peek() {
            s.push(self.advance().unwrap());
        }

        // Digits before decimal point
        while let Some(ch) = self.peek() {
            if ch.is_ascii_digit() {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        // Decimal point and fractional digits
        if let Some('.') = self.peek() {
            s.push(self.advance().unwrap());
            while let Some(ch) = self.peek() {
                if ch.is_ascii_digit() {
                    s.push(self.advance().unwrap());
                } else {
                    break;
                }
            }
        }

        if s.is_empty() || s == "-" {
            return Err("Expected number".to_string());
        }

        s.parse::<f64>().map_err(|e| format!("Invalid number '{}': {}", s, e))
    }

    /// Reads an identifier (alphanumeric + underscore, starting with letter or underscore).
    /// Also checks for keywords and returns the appropriate token.
    fn read_identifier(&mut self) -> Token {
        let mut s = String::new();
        while let Some(ch) = self.peek() {
            if ch.is_alphanumeric() || ch == '_' {
                s.push(self.advance().unwrap());
            } else {
                break;
            }
        }

        // Check for keywords
        match s.as_str() {
            "let" => Token::Let,
            "chaotic" => Token::Chaotic,
            _ => Token::Identifier(s),
        }
    }

    /// Tokenizes the entire input and returns a list of tokens.
    pub fn tokenize(&mut self) -> Result<Vec<Token>, String> {
        let mut tokens = Vec::new();

        loop {
            self.skip_whitespace();

            match self.peek() {
                None => break, // End of input
                Some('\n') => {
                    // Newlines are significant for statement separation
                    self.advance();
                    // Don't add newline token - just let the parser handle line breaks
                }
                Some('+') => {
                    self.advance();
                    tokens.push(Token::Plus);
                }
                Some('(') => {
                    self.advance();
                    tokens.push(Token::LParen);
                }
                Some(')') => {
                    self.advance();
                    tokens.push(Token::RParen);
                }
                Some(',') => {
                    self.advance();
                    tokens.push(Token::Comma);
                }
                Some(';') => {
                    self.advance();
                    tokens.push(Token::Semicolon);
                }
                Some('=') => {
                    self.advance();
                    tokens.push(Token::Equals);
                }
                Some(ch) if ch.is_ascii_digit() || ch == '-' => {
                    let num = self.read_number()?;
                    tokens.push(Token::Number(num));
                }
                Some(ch) if ch.is_alphabetic() || ch == '_' => {
                    let token = self.read_identifier();
                    tokens.push(token);
                }
                Some(ch) => {
                    return Err(format!("Unexpected character: '{}'", ch));
                }
            }
        }

        Ok(tokens)
    }
}

/// Recursive descent parser for the chaotic expression language.
///
/// Grammar:
/// ```text
/// program      := statement*
/// statement    := let_stmt | expression
/// let_stmt     := 'let' IDENTIFIER '=' expression
/// expression   := term (('+' term)*)
/// term         := function_call | atom
/// atom         := identifier | number | chaotic_call | '(' expression ')'
/// function_call := 'propagate' '(' expression ',' number ')'
///                 | 'collapse' '(' expression ')'
/// chaotic_call := 'chaotic' '(' number ',' number ')'
/// ```
pub struct Parser {
    tokens: Vec<Token>,
    pos: usize,
}

impl Parser {
    /// Creates a new Parser from a string input. Tokenizes immediately.
    pub fn new(input: &str) -> Result<Self, String> {
        let mut lexer = Lexer::new(input);
        let tokens = lexer.tokenize()?;
        Ok(Self { tokens, pos: 0 })
    }

    /// Peeks at the current token without consuming it.
    fn peek(&self) -> Option<&Token> {
        self.tokens.get(self.pos)
    }

    /// Consumes and returns the current token.
    fn advance(&mut self) -> Option<Token> {
        let token = self.tokens.get(self.pos).cloned();
        if token.is_some() {
            self.pos += 1;
        }
        token
    }

    /// Expects the current token to be of a specific type, consumes and returns it.
    fn expect(&mut self, expected: &Token) -> Result<Token, String> {
        let token = self.advance().ok_or_else(|| {
            format!("Expected {:?}, found end of input", expected)
        })?;
        if std::mem::discriminant(&token) == std::mem::discriminant(expected) {
            Ok(token)
        } else {
            Err(format!("Expected {:?}, found {:?}", expected, token))
        }
    }

    /// Parses a full expression: `term (('+' term)*)`
    pub fn parse_expr(&mut self) -> Result<Expr, String> {
        let mut left = self.parse_term()?;

        while self.peek() == Some(&Token::Plus) {
            self.advance(); // consume '+'
            let right = self.parse_term()?;
            left = Expr::add(left, right);
        }

        Ok(left)
    }

    /// Parses a term: function call or atom.
    fn parse_term(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Identifier(name)) if name == "propagate" => self.parse_propagate(),
            Some(Token::Identifier(name)) if name == "collapse" => self.parse_collapse(),
            Some(Token::Chaotic) => self.parse_chaotic(),
            _ => self.parse_atom(),
        }
    }

    /// Parses a propagate call: `propagate '(' expression ',' number ')'`
    fn parse_propagate(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'propagate'
        self.expect(&Token::LParen)?;
        let inner = self.parse_expr()?;
        self.expect(&Token::Comma)?;
        let time_step = self.parse_number()?;
        self.expect(&Token::RParen)?;
        Ok(Expr::propagate(inner, time_step))
    }

    /// Parses a collapse call: `collapse '(' expression ')'`
    fn parse_collapse(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'collapse'
        self.expect(&Token::LParen)?;
        let inner = self.parse_expr()?;
        self.expect(&Token::RParen)?;
        Ok(Expr::collapse(inner))
    }

    /// Parses a chaotic constructor: `chaotic '(' number ',' number ')'`
    fn parse_chaotic(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'chaotic'
        self.expect(&Token::LParen)?;
        let mean = self.parse_number()?;
        self.expect(&Token::Comma)?;
        let variance = self.parse_number()?;
        self.expect(&Token::RParen)?;
        Ok(Expr::chaotic(mean, variance))
    }

    /// Parses an atom: identifier, number, or parenthesized expression.
    fn parse_atom(&mut self) -> Result<Expr, String> {
        match self.peek().cloned() {
            Some(Token::Identifier(name)) => {
                self.advance();
                Ok(Expr::var(&name))
            }
            Some(Token::Number(value)) => {
                self.advance();
                Ok(Expr::lit(value))
            }
            Some(Token::LParen) => {
                self.advance(); // consume '('
                let expr = self.parse_expr()?;
                self.expect(&Token::RParen)?;
                Ok(expr)
            }
            Some(Token::RParen) => {
                Err("Unexpected ')'".to_string())
            }
            Some(token) => {
                Err(format!("Unexpected token: {:?}", token))
            }
            None => {
                Err("Unexpected end of expression".to_string())
            }
        }
    }

    /// Parses a number token and returns its f64 value.
    fn parse_number(&mut self) -> Result<f64, String> {
        match self.advance() {
            Some(Token::Number(value)) => Ok(value),
            Some(token) => Err(format!("Expected number, found {:?}", token)),
            None => Err("Expected number, found end of input".to_string()),
        }
    }

    /// Parses a let statement: `let IDENTIFIER '=' expression`
    fn parse_let(&mut self) -> Result<Expr, String> {
        self.advance(); // consume 'let'

        // Expect an identifier
        let name = match self.advance() {
            Some(Token::Identifier(name)) => name,
            Some(token) => return Err(format!("Expected identifier after 'let', found {:?}", token)),
            None => return Err("Expected identifier after 'let', found end of input".to_string()),
        };

        // Expect '='
        self.expect(&Token::Equals)?;

        // Parse the expression
        let expr = self.parse_expr()?;

        Ok(Expr::let_binding(&name, expr))
    }

    /// Parses a statement (let or expression).
    fn parse_statement(&mut self) -> Result<Expr, String> {
        match self.peek() {
            Some(Token::Let) => self.parse_let(),
            _ => self.parse_expr(),
        }
    }

    /// Parses the entire input and returns the AST.
    pub fn parse(&mut self) -> Result<Expr, String> {
        let mut statements = Vec::new();

        // Skip leading newlines
        while let Some(Token::Semicolon) = self.peek() {
            self.advance();
        }

        // Parse statements separated by newlines or semicolons
        loop {
            // Skip trailing newlines and semicolons
            while let Some(Token::Semicolon) = self.peek() {
                self.advance();
            }

            match self.peek() {
                None => break,
                _ => {
                    let stmt = self.parse_statement()?;
                    statements.push(stmt);
                }
            }
        }

        if statements.is_empty() {
            return Err("Empty input".to_string());
        }

        // If there's only one statement, return it directly
        if statements.len() == 1 {
            return Ok(statements.remove(0));
        }

        // Otherwise, wrap in a block
        Ok(Expr::block(statements))
    }
}

/// Convenience function: parses a string into an `Expr`.
pub fn parse(input: &str) -> Result<Expr, String> {
    Parser::new(input)?.parse()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_lexer_simple_variable() {
        let mut lexer = Lexer::new("temp");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Identifier("temp".to_string())]);
    }

    #[test]
    fn test_lexer_number() {
        let mut lexer = Lexer::new("42.5");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(tokens, vec![Token::Number(42.5)]);
    }

    #[test]
    fn test_lexer_keywords() {
        let mut lexer = Lexer::new("let chaotic = chaotic(1.0, 2.0)");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Let,
                Token::Chaotic,
                Token::Equals,
                Token::Chaotic,
                Token::LParen,
                Token::Number(1.0),
                Token::Comma,
                Token::Number(2.0),
                Token::RParen,
            ]
        );
    }

    #[test]
    fn test_lexer_operators() {
        let mut lexer = Lexer::new("a + b; c = d");
        let tokens = lexer.tokenize().unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Identifier("a".to_string()),
                Token::Plus,
                Token::Identifier("b".to_string()),
                Token::Semicolon,
                Token::Identifier("c".to_string()),
                Token::Equals,
                Token::Identifier("d".to_string()),
            ]
        );
    }

    #[test]
    fn test_lexer_unexpected_char() {
        let mut lexer = Lexer::new("temp @ invalid");
        assert!(lexer.tokenize().is_err());
    }

    #[test]
    fn test_parse_literal() {
        let mut parser = Parser::new("42.5").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(expr, Expr::Literal(42.5));
    }

    #[test]
    fn test_parse_variable() {
        let mut parser = Parser::new("temp").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(expr, Expr::Var("temp".to_string()));
    }

    #[test]
    fn test_parse_addition() {
        let mut parser = Parser::new("temp + humidity").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::Add(
                Box::new(Expr::Var("temp".to_string())),
                Box::new(Expr::Var("humidity".to_string())),
            )
        );
    }

    #[test]
    fn test_parse_propagate() {
        let mut parser = Parser::new("propagate(temp, 2.0)").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::Propagate(
                Box::new(Expr::Var("temp".to_string())),
                2.0,
            )
        );
    }

    #[test]
    fn test_parse_collapse() {
        let mut parser = Parser::new("collapse(temp)").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::Collapse(Box::new(Expr::Var("temp".to_string())))
        );
    }

    #[test]
    fn test_parse_chaotic() {
        let mut parser = Parser::new("chaotic(101.0, 0.1)").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::ChaoticConstructor(101.0, 0.1)
        );
    }

    #[test]
    fn test_parse_let_statement() {
        let mut parser = Parser::new("let x = 42.0").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::Let(
                "x".to_string(),
                Box::new(Expr::Literal(42.0))
            )
        );
    }

    #[test]
    fn test_parse_let_with_chaotic() {
        let mut parser = Parser::new("let pressure = chaotic(101.0, 0.1)").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::Let(
                "pressure".to_string(),
                Box::new(Expr::ChaoticConstructor(101.0, 0.1))
            )
        );
    }

    #[test]
    fn test_parse_block() {
        let code = "let x = 1.0\nlet y = 2.0\nx + y";
        let mut parser = Parser::new(code).unwrap();
        let expr = parser.parse().unwrap();
        match expr {
            Expr::Block(stmts) => {
                assert_eq!(stmts.len(), 3);
                assert_eq!(stmts[0], Expr::Let("x".to_string(), Box::new(Expr::Literal(1.0))));
                assert_eq!(stmts[1], Expr::Let("y".to_string(), Box::new(Expr::Literal(2.0))));
                assert_eq!(stmts[2], Expr::Add(
                    Box::new(Expr::Var("x".to_string())),
                    Box::new(Expr::Var("y".to_string())),
                ));
            }
            _ => panic!("Expected Block"),
        }
    }

    #[test]
    fn test_parse_complex_expression() {
        let mut parser = Parser::new("propagate(temp + humidity, 5.0)").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::Propagate(
                Box::new(Expr::Add(
                    Box::new(Expr::Var("temp".to_string())),
                    Box::new(Expr::Var("humidity".to_string())),
                )),
                5.0,
            )
        );
    }

    #[test]
    fn test_parse_parentheses() {
        let mut parser = Parser::new("(temp + humidity)").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::Add(
                Box::new(Expr::Var("temp".to_string())),
                Box::new(Expr::Var("humidity".to_string())),
            )
        );
    }

    #[test]
    fn test_parse_missing_rparen() {
        let mut parser = Parser::new("(temp + humidity").unwrap();
        assert!(parser.parse().is_err());
    }

    #[test]
    fn test_parse_trailing_tokens() {
        let mut parser = Parser::new("temp +").unwrap();
        assert!(parser.parse().is_err());
    }

    #[test]
    fn test_parse_add_same_var_twice() {
        let mut parser = Parser::new("pressure + pressure").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(
            expr,
            Expr::Add(
                Box::new(Expr::Var("pressure".to_string())),
                Box::new(Expr::Var("pressure".to_string())),
            )
        );
    }

    #[test]
    fn test_parse_negative_number() {
        let mut parser = Parser::new("-3.14").unwrap();
        let expr = parser.parse().unwrap();
        assert_eq!(expr, Expr::Literal(-3.14));
    }

    #[test]
    fn test_parse_empty_input() {
        let mut parser = Parser::new("").unwrap();
        assert!(parser.parse().is_err());
    }

    #[test]
    fn test_parse_full_program() {
        let code = "let pressure = chaotic(101.0, 0.1)\nlet temp = chaotic(20.0, 0.5)\ncollapse(propagate(pressure + temp, 1.0))";
        let mut parser = Parser::new(code).unwrap();
        let expr = parser.parse().unwrap();
        match expr {
            Expr::Block(stmts) => {
                assert_eq!(stmts.len(), 3);
                // First statement: let pressure = chaotic(101.0, 0.1)
                assert!(matches!(&stmts[0], Expr::Let(name, _) if name == "pressure"));
                // Second statement: let temp = chaotic(20.0, 0.5)
                assert!(matches!(&stmts[1], Expr::Let(name, _) if name == "temp"));
                // Third statement: collapse(propagate(pressure + temp, 1.0))
                assert!(matches!(&stmts[2], Expr::Collapse(_)));
            }
            _ => panic!("Expected Block, got {:?}", expr),
        }
    }
}