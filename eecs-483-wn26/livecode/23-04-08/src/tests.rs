use super::*;

// Helper: lex a string into tokens
fn lex(input: &str) -> Result<Vec<Token>, ()> {
    Token::lexer(input).collect()
}

// Helper: lex then parse
fn lex_and_parse(input: &str) -> Result<Ast, String> {
    let tokens = lex(input).map_err(|()| "lex error".to_string())?;
    parse(&mut tokens.iter())
}

fn num(n: i64) -> Ast {
    Ast::Num(n)
}

fn plus(l: Ast, r: Ast) -> Ast {
    Ast::Plus(Box::new(l), Box::new(r))
}

// ---- Lexer tests ----

#[test]
fn lex_single_number() {
    assert_eq!(lex("42"), Ok(vec![Token::Num(42)]));
}

#[test]
fn lex_expression_with_spaces() {
    assert_eq!(
        lex("1 + 2"),
        Ok(vec![Token::Num(1), Token::Plus, Token::Num(2)])
    );
}

#[test]
fn lex_no_spaces() {
    assert_eq!(
        lex("1+2"),
        Ok(vec![Token::Num(1), Token::Plus, Token::Num(2)])
    );
}

#[test]
fn lex_parens() {
    assert_eq!(
        lex("(1)"),
        Ok(vec![Token::LParen, Token::Num(1), Token::RParen])
    );
}

#[test]
fn lex_invalid_char() {
    assert!(lex("1 * 2").is_err());
}

#[test]
fn lex_empty() {
    assert_eq!(lex(""), Ok(vec![]));
}

// ---- Parser tests: valid inputs ----

#[test]
fn parse_single_number() {
    assert_eq!(lex_and_parse("42"), Ok(num(42)));
}

#[test]
fn parse_simple_addition() {
    assert_eq!(lex_and_parse("1 + 2"), Ok(plus(num(1), num(2))));
}

#[test]
fn parse_chained_addition_is_right_assoc() {
    // S -> E S', S' -> + S -> + E S' -> + E + S -> ...
    // So 1 + 2 + 3 = Plus(1, Plus(2, 3))
    assert_eq!(
        lex_and_parse("1 + 2 + 3"),
        Ok(plus(num(1), plus(num(2), num(3))))
    );
}

#[test]
fn parse_parenthesized() {
    assert_eq!(lex_and_parse("(1 + 2)"), Ok(plus(num(1), num(2))));
}

#[test]
fn parse_nested_parens() {
    assert_eq!(lex_and_parse("((1))"), Ok(num(1)));
}

#[test]
fn parse_complex_expression() {
    // (1 + 2 + (3 + 4)) + 5
    let expected = plus(plus(num(1), plus(num(2), plus(num(3), num(4)))), num(5));
    assert_eq!(lex_and_parse("(1 + 2 + (3 + 4)) + 5"), Ok(expected));
}

#[test]
fn parse_left_grouping_with_parens() {
    // (1 + 2) + 3 should be Plus(Plus(1. 2), 3)
    assert_eq!(
        lex_and_parse("(1 + 2) + 3"),
        Ok(plus(plus(num(1), num(2)), num(3)))
    );
}

// ---- Parser tests: invalid inputs ----

#[test]
fn parse_empty_input_is_error() {
    assert!(lex_and_parse("").is_err());
}

#[test]
fn parse_unmatched_lparen() {
    assert!(lex_and_parse("(1 + 2").is_err());
}

#[test]
fn parse_unmatched_rparen() {
    assert!(lex_and_parse("1 + 2)").is_err());
}

#[test]
fn parse_empty_parens() {
    assert!(lex_and_parse("()").is_err());
}

#[test]
fn parse_double_plus() {
    assert!(lex_and_parse("1 + + 2").is_err());
}

#[test]
fn parse_leading_plus() {
    assert!(lex_and_parse("+ 1").is_err());
}

#[test]
fn parse_trailing_plus() {
    assert!(lex_and_parse("1 +").is_err());
}

#[test]
fn parse_adjacent_numbers() {
    assert!(lex_and_parse("1 2").is_err());
}

#[test]
fn parse_extra_tokens_after_expr() {
    // "1 ) " — the ) after a complete expression should be an error
    assert!(lex_and_parse("1 )").is_err());
}

// ---- Large input tests ----

#[test]
fn parse_long_chain() {
    // 1 + 2 + 3 + ... + 100, right-associative
    let input: String = (1..=100)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(" + ");
    let ast = lex_and_parse(&input).unwrap();
    // Build expected: Plus(1, Plus(2, ... Plus(99, 100)))
    let expected = (1..=100)
        .rev()
        .fold(None, |acc: Option<Ast>, n| {
            Some(match acc {
                None => num(n),
                Some(right) => plus(num(n), right),
            })
        })
        .unwrap();
    assert_eq!(ast, expected);
}

#[test]
fn parse_deeply_nested_parens() {
    // (((((...(1)...)))))  — 100 layers of parens
    let depth = 100;
    let input = format!("{}1{}", "(".repeat(depth), ")".repeat(depth),);
    assert_eq!(lex_and_parse(&input), Ok(num(1)));
}

#[test]
fn parse_nested_parens_with_addition() {
    // (((1 + 2))) + (((3 + 4)))
    let input = "(((1 + 2))) + (((3 + 4)))";
    assert_eq!(
        lex_and_parse(input),
        Ok(plus(plus(num(1), num(2)), plus(num(3), num(4))))
    );
}

#[test]
fn parse_long_chain_in_parens() {
    // (1 + 2 + 3 + ... + 50) + (51 + 52 + ... + 100)
    let left: String = (1..=50)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(" + ");
    let right: String = (51..=100)
        .map(|n| n.to_string())
        .collect::<Vec<_>>()
        .join(" + ");
    let input = format!("({}) + ({})", left, right);
    let ast = lex_and_parse(&input).unwrap();

    let left_expected = (1..=50)
        .rev()
        .fold(None, |acc: Option<Ast>, n| {
            Some(match acc {
                None => num(n),
                Some(r) => plus(num(n), r),
            })
        })
        .unwrap();
    let right_expected = (51..=100)
        .rev()
        .fold(None, |acc: Option<Ast>, n| {
            Some(match acc {
                None => num(n),
                Some(r) => plus(num(n), r),
            })
        })
        .unwrap();
    assert_eq!(ast, plus(left_expected, right_expected));
}

#[test]
fn parse_large_numbers() {
    let input = "999999999 + 1000000000";
    assert_eq!(
        lex_and_parse(input),
        Ok(plus(num(999999999), num(1000000000)))
    );
}

#[test]
fn parse_right_leaning_nested() {
    // 1 + (2 + (3 + (4 + 5))) — explicit right nesting matching natural associativity
    let input = "1 + (2 + (3 + (4 + 5)))";
    assert_eq!(
        lex_and_parse(input),
        Ok(plus(
            num(1),
            plus(num(2), plus(num(3), plus(num(4), num(5))))
        ))
    );
}

#[test]
fn parse_left_leaning_nested() {
    // (((1 + 2) + 3) + 4) + 5 — left nesting via parens
    let input = "(((1 + 2) + 3) + 4) + 5";
    assert_eq!(
        lex_and_parse(input),
        Ok(plus(
            plus(plus(plus(num(1), num(2)), num(3)), num(4)),
            num(5)
        ))
    );
}
