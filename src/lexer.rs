use crate::error::RispyError;

fn is_valid_number(s: &str) -> bool {
    if s.is_empty() {
        return false;
    }

    // Handle optional leading +/-
    let s = if s.starts_with('+') || s.starts_with('-') {
        &s[1..]
    } else {
        s
    };

    if s.is_empty() {
        return false;
    }

    let dot_count = s.chars().filter(|&c| c == '.').count();

    // More than one dot is invalid
    if dot_count > 1 {
        return false;
    }

    // Check for patterns like "123." or ".456" which should be symbols
    if s.ends_with('.') || s.starts_with('.') {
        return false;
    }

    // All other characters must be digits
    s.chars().all(|c| c.is_ascii_digit() || c == '.')
}

#[derive(Debug, Clone, PartialEq)]
pub enum Token {
    LeftParen,
    RightParen,
    Symbol(String),
    Number(f64),
    String(String),
    Quote,
}

pub fn tokenize(input: &str) -> Result<Vec<Token>, RispyError> {
    let mut tokens = Vec::new();
    let mut chars = input.chars().peekable();

    while let Some(&ch) = chars.peek() {
        match ch {
            ' ' | '\t' | '\n' | '\r' => {
                chars.next();
            }
            ';' => {
                chars.next();
                while let Some(&c) = chars.peek() {
                    if c == '\n' {
                        break;
                    }
                    chars.next();
                }
            }
            '(' => {
                chars.next();
                tokens.push(Token::LeftParen);
            }
            ')' => {
                chars.next();
                tokens.push(Token::RightParen);
            }
            '\'' => {
                chars.next();
                tokens.push(Token::Quote);
            }
            '"' => {
                chars.next();
                let mut string_val = String::new();
                let mut escaped = false;
                let mut terminated = false;

                while let Some(c) = chars.next() {
                    if escaped {
                        match c {
                            'n' => string_val.push('\n'),
                            't' => string_val.push('\t'),
                            'r' => string_val.push('\r'),
                            '\\' => string_val.push('\\'),
                            '"' => string_val.push('"'),
                            _ => {
                                string_val.push('\\');
                                string_val.push(c);
                            }
                        }
                        escaped = false;
                    } else if c == '\\' {
                        escaped = true;
                    } else if c == '"' {
                        terminated = true;
                        break;
                    } else {
                        string_val.push(c);
                    }
                }

                if !terminated {
                    return Err(RispyError::LexError(
                        "Unterminated string literal".to_string(),
                    ));
                }

                tokens.push(Token::String(string_val));
            }
            c if c.is_ascii_digit() || c == '-' || c == '+' || c == '.' => {
                let mut number_str = String::new();

                if c == '-' || c == '+' {
                    number_str.push(chars.next().unwrap());
                    if let Some(&next_ch) = chars.peek() {
                        if !next_ch.is_ascii_digit() && next_ch != '.' {
                            // Check if there are non-whitespace characters following
                            if !next_ch.is_whitespace() && !"()\"';".contains(next_ch) {
                                // This is part of a larger symbol like "+x" or "-y"
                                while let Some(&c) = chars.peek() {
                                    if c.is_whitespace() || "()\"';".contains(c) {
                                        break;
                                    }
                                    number_str.push(chars.next().unwrap());
                                }
                            }
                            tokens.push(Token::Symbol(number_str));
                            continue;
                        }
                    } else {
                        tokens.push(Token::Symbol(number_str));
                        continue;
                    }
                } else if c == '.' {
                    number_str.push(chars.next().unwrap());
                    // If it starts with '.', check if next character is a digit
                    if let Some(&next_ch) = chars.peek() {
                        if !next_ch.is_ascii_digit() {
                            // Treat as symbol if not followed by digit
                            tokens.push(Token::Symbol(number_str));
                            continue;
                        }
                    } else {
                        // Lone dot, treat as symbol
                        tokens.push(Token::Symbol(number_str));
                        continue;
                    }
                }

                while let Some(&c) = chars.peek() {
                    if c.is_ascii_digit() || c == '.' {
                        number_str.push(chars.next().unwrap());
                    } else {
                        break;
                    }
                }

                // Check if the next character would make this part of a larger symbol
                if let Some(&next_ch) = chars.peek() {
                    if !next_ch.is_whitespace() && !"()\"';".contains(next_ch) {
                        // This is part of a larger symbol, parse as symbol instead
                        while let Some(&c) = chars.peek() {
                            if c.is_whitespace() || "()\"';".contains(c) {
                                break;
                            }
                            number_str.push(chars.next().unwrap());
                        }
                        tokens.push(Token::Symbol(number_str));
                        continue;
                    }
                }

                // Check if it's a valid number format before parsing
                if is_valid_number(&number_str) {
                    match number_str.parse::<f64>() {
                        Ok(num) => tokens.push(Token::Number(num)),
                        Err(_) => {
                            return Err(RispyError::LexError(format!(
                                "Invalid number: {}",
                                number_str
                            )));
                        }
                    }
                } else {
                    // Treat as symbol if not a valid number format
                    tokens.push(Token::Symbol(number_str));
                }
            }
            _ => {
                let mut symbol = String::new();

                while let Some(&c) = chars.peek() {
                    if c.is_whitespace() || "()\"';".contains(c) {
                        break;
                    }
                    symbol.push(chars.next().unwrap());
                }

                if symbol.is_empty() {
                    return Err(RispyError::LexError(format!(
                        "Unexpected character: {}",
                        ch
                    )));
                }

                tokens.push(Token::Symbol(symbol));
            }
        }
    }

    Ok(tokens)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_tokens() {
        let tokens = tokenize("()").unwrap();
        assert_eq!(tokens, vec![Token::LeftParen, Token::RightParen]);
    }

    #[test]
    fn test_simple_expression() {
        let tokens = tokenize("(+ 1 2)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Symbol("+".to_string()),
                Token::Number(1.0),
                Token::Number(2.0),
                Token::RightParen
            ]
        );
    }

    #[test]
    fn test_string_literal() {
        let tokens = tokenize("\"hello world\"").unwrap();
        assert_eq!(tokens, vec![Token::String("hello world".to_string())]);
    }

    #[test]
    fn test_quote() {
        let tokens = tokenize("'(1 2 3)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Quote,
                Token::LeftParen,
                Token::Number(1.0),
                Token::Number(2.0),
                Token::Number(3.0),
                Token::RightParen
            ]
        );
    }

    #[test]
    fn test_comments() {
        let tokens = tokenize("; this is a comment\n(+ 1 2)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Symbol("+".to_string()),
                Token::Number(1.0),
                Token::Number(2.0),
                Token::RightParen
            ]
        );
    }

    #[test]
    fn test_multiple_comments() {
        let tokens = tokenize("; first comment\n; second comment\n42").unwrap();
        assert_eq!(tokens, vec![Token::Number(42.0)]);
    }

    #[test]
    fn test_comment_at_end_of_line() {
        let tokens = tokenize("(+ 1 2) ; inline comment\n(* 3 4)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Symbol("+".to_string()),
                Token::Number(1.0),
                Token::Number(2.0),
                Token::RightParen,
                Token::LeftParen,
                Token::Symbol("*".to_string()),
                Token::Number(3.0),
                Token::Number(4.0),
                Token::RightParen
            ]
        );
    }

    #[test]
    fn test_empty_input() {
        let tokens = tokenize("").unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_whitespace_only() {
        let tokens = tokenize("   \t\n\r   ").unwrap();
        assert_eq!(tokens, vec![]);
    }

    #[test]
    fn test_negative_numbers() {
        let tokens = tokenize("-42 -3.14").unwrap();
        assert_eq!(tokens, vec![Token::Number(-42.0), Token::Number(-3.14)]);
    }

    #[test]
    fn test_positive_numbers() {
        let tokens = tokenize("+42 +3.14").unwrap();
        assert_eq!(tokens, vec![Token::Number(42.0), Token::Number(3.14)]);
    }

    #[test]
    fn test_zero_numbers() {
        let tokens = tokenize("0 0.0 -0 +0").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(0.0),
                Token::Number(0.0),
                Token::Number(0.0),
                Token::Number(0.0)
            ]
        );
    }

    #[test]
    fn test_decimal_numbers() {
        let tokens = tokenize("3.14159 0.5 123.456").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Number(3.14159),
                Token::Number(0.5),
                Token::Number(123.456)
            ]
        );
    }

    #[test]
    fn test_symbols_with_special_chars() {
        let tokens = tokenize("+ - * / = < > <= >= atom? null?").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("+".to_string()),
                Token::Symbol("-".to_string()),
                Token::Symbol("*".to_string()),
                Token::Symbol("/".to_string()),
                Token::Symbol("=".to_string()),
                Token::Symbol("<".to_string()),
                Token::Symbol(">".to_string()),
                Token::Symbol("<=".to_string()),
                Token::Symbol(">=".to_string()),
                Token::Symbol("atom?".to_string()),
                Token::Symbol("null?".to_string())
            ]
        );
    }

    #[test]
    fn test_symbols_with_hyphens() {
        let tokens = tokenize("my-var kebab-case test-123").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("my-var".to_string()),
                Token::Symbol("kebab-case".to_string()),
                Token::Symbol("test-123".to_string())
            ]
        );
    }

    #[test]
    fn test_empty_string() {
        let tokens = tokenize("\"\"").unwrap();
        assert_eq!(tokens, vec![Token::String("".to_string())]);
    }

    #[test]
    fn test_string_with_escapes() {
        let tokens = tokenize("\"hello\\nworld\\t\\\"quoted\\\"\"").unwrap();
        assert_eq!(
            tokens,
            vec![Token::String("hello\nworld\t\"quoted\"".to_string())]
        );
    }

    #[test]
    fn test_string_with_backslashes() {
        let tokens = tokenize("\"path\\\\to\\\\file\"").unwrap();
        assert_eq!(tokens, vec![Token::String("path\\to\\file".to_string())]);
    }

    #[test]
    fn test_multiple_strings() {
        let tokens = tokenize("\"first\" \"second\" \"third\"").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::String("first".to_string()),
                Token::String("second".to_string()),
                Token::String("third".to_string())
            ]
        );
    }

    #[test]
    fn test_nested_parentheses() {
        let tokens = tokenize("((()))").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::LeftParen,
                Token::LeftParen,
                Token::RightParen,
                Token::RightParen,
                Token::RightParen
            ]
        );
    }

    #[test]
    fn test_multiple_quotes() {
        let tokens = tokenize("'x ''y '''z").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Quote,
                Token::Symbol("x".to_string()),
                Token::Quote,
                Token::Quote,
                Token::Symbol("y".to_string()),
                Token::Quote,
                Token::Quote,
                Token::Quote,
                Token::Symbol("z".to_string())
            ]
        );
    }

    #[test]
    fn test_mixed_numbers_and_symbols() {
        let tokens = tokenize("x123 123x x-123 123-x").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("x123".to_string()),
                Token::Symbol("123x".to_string()),
                Token::Symbol("x-123".to_string()),
                Token::Symbol("123-x".to_string())
            ]
        );
    }

    #[test]
    fn test_plus_minus_as_symbols() {
        let tokens = tokenize("+ - +x -y + -").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("+".to_string()),
                Token::Symbol("-".to_string()),
                Token::Symbol("+x".to_string()),
                Token::Symbol("-y".to_string()),
                Token::Symbol("+".to_string()),
                Token::Symbol("-".to_string())
            ]
        );
    }

    #[test]
    fn test_numbers_adjacent_to_parens() {
        let tokens = tokenize("(42)(3.14)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Number(42.0),
                Token::RightParen,
                Token::LeftParen,
                Token::Number(3.14),
                Token::RightParen
            ]
        );
    }

    #[test]
    fn test_symbols_adjacent_to_parens() {
        let tokens = tokenize("(foo)(bar)").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::LeftParen,
                Token::Symbol("foo".to_string()),
                Token::RightParen,
                Token::LeftParen,
                Token::Symbol("bar".to_string()),
                Token::RightParen
            ]
        );
    }

    #[test]
    fn test_long_symbol() {
        let long_name = "a".repeat(1000);
        let input = format!("{}", long_name);
        let tokens = tokenize(&input).unwrap();
        assert_eq!(tokens, vec![Token::Symbol(long_name)]);
    }

    #[test]
    fn test_many_tokens() {
        let input = "(+ 1 2 3 4 5 6 7 8 9 10)";
        let tokens = tokenize(input).unwrap();
        assert_eq!(tokens.len(), 13); // ( + 1 2 3 4 5 6 7 8 9 10 )
    }

    #[test]
    fn test_complex_expression() {
        let input = "(define factorial (lambda (n) (if (= n 0) 1 (* n (factorial (- n 1))))))";
        let tokens = tokenize(input).unwrap();
        assert!(tokens.len() > 20);
        assert_eq!(tokens[0], Token::LeftParen);
        assert_eq!(tokens[1], Token::Symbol("define".to_string()));
        assert_eq!(tokens[2], Token::Symbol("factorial".to_string()));
    }

    #[test]
    fn test_unterminated_string_error() {
        let result = tokenize("\"unterminated string");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_number_as_symbol() {
        let tokens = tokenize("123.456.789").unwrap();
        assert_eq!(tokens, vec![Token::Symbol("123.456.789".to_string())]);
    }

    #[test]
    fn test_decimal_without_digits() {
        let tokens = tokenize("123. .456").unwrap();
        assert_eq!(
            tokens,
            vec![
                Token::Symbol("123.".to_string()),
                Token::Symbol(".456".to_string())
            ]
        );
    }
}
