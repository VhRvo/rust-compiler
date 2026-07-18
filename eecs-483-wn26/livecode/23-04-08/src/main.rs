use logos::Logos;
use std::iter::Peekable;

#[derive(Logos, Debug, Eq, PartialEq)]
#[logos(skip r"[ \t\n\r]+")]
enum Token {
    #[regex(r"[0-9]+", |lex| lex.slice().parse::<i64>().ok())]
    Num(i64),
    #[token("+")]
    Plus,
    #[token("(")]
    LParen,
    #[token(")")]
    RParen,
}

#[derive(Debug, PartialEq, Eq)]
enum Ast {
    Num(i64),
    Plus(Box<Ast>, Box<Ast>),
}

/* Our LL(1) grammar
 * T  ⟼ S$
 *
 * S  ⟼ ES'
 *
 * S’ ⟼ ε
 * S’ ⟼ + S
 *
 * E  ⟼ num | ( S )
 * */

struct ParserState<'t, I>
where
    I: Iterator<Item = &'t Token>,
{
    toks: Peekable<I>,
}

impl<'t, I> ParserState<'t, I>
where
    I: Iterator<Item = &'t Token>,
{
    fn parse_eof(&mut self) -> Result<(), String> {
        match self.toks.next() {
            None => {
                println!("consuming EOF");
                Ok(())
            }
            Some(c) => Err(format!("expected EOF, found {:?}", c)),
        }
    }

    fn parse_num(&mut self) -> Result<i64, String> {
        match self.toks.next() {
            Some(Token::Num(n)) => {
                println!("consuming num({:?})", n);
                Ok(*n)
            }
            _ => Err(String::from("expected num")),
        }
    }

    fn parse_tok(&mut self, c: Token) -> Result<(), String> {
        match self.toks.next() {
            None => Err(format!("expected {:?}, got EOF", c)),
            Some(d) => {
                if c == *d {
                    println!("consuming {:?}", c);
                    Ok(())
                } else {
                    Err(format!("expected {:?}, found {:?}", c, d))
                }
            }
        }
    }
    /* T
     * on number | (
     * transition to S $
     * */
    fn parse_t(&mut self) -> Result<Ast, String> {
        println!("State: T \t\tLookahead: {:?}", self.toks.peek());
        match self.toks.peek() {
            Some(Token::Num(_)) | Some(Token::LParen) => {
                println!("Transition T ⟼ S $");
                let ans = self.parse_s()?;
                let _ = self.parse_eof()?;
                Ok(ans)
            }
            _ => Err(String::from("expected '(' or number")),
        }
    }

    /* S
     * on number | (
     * transition to E S'
     */
    fn parse_s(&mut self) -> Result<Ast, String> {
        println!("State: S \t\tLookahead: {:?}", self.toks.peek());
        match self.toks.peek() {
            Some(Token::Num(_)) | Some(Token::LParen) => {
                println!("Transition S  ⟼ E S'");
                let prefix = self.parse_e()?;
                self.parse_s_prime(prefix)
            }
            _ => Err(String::from("expected '(' or number")),
        }
    }

    /* S'
     * on + transition to + S
     * on ) or EOF transition to ε
     * */
    fn parse_s_prime(&mut self, prefix: Ast) -> Result<Ast, String> {
        println!("State: S'\t\tLookahead: {:?}", self.toks.peek());
        match self.toks.peek() {
            Some(Token::Plus) => {
                println!("Transition S’ ⟼ + S");
                self.parse_tok(Token::Plus)?;
                let suffix = self.parse_s()?;
                Ok(Ast::Plus(Box::new(prefix), Box::new(suffix)))
            }
            Some(Token::RParen) | None => {
                println!("Transition S’ ⟼ ε");
                Ok(prefix)
            }
            _ => Err(String::from("expected '+', ')' or EOF")),
        }
    }

    /* E
     * on number transition to number
     * on ( transition to ( S )
     * */
    fn parse_e(&mut self) -> Result<Ast, String> {
        println!("State: E \t\tLookahead: {:?}", self.toks.peek());
        match self.toks.peek() {
            Some(Token::Num(_)) => {
                println!("Transition E ⟼ num");
                Ok(Ast::Num(self.parse_num()?))
            }
            Some(Token::LParen) => {
                println!("Transition E ⟼ ( S )");
                self.parse_tok(Token::LParen)?;
                let res = self.parse_s()?;
                self.parse_tok(Token::RParen)?;
                Ok(res)
            }
            _ => Err(String::from("expected '(' or number")),
        }
    }
}

fn parse<'t, T: Iterator<Item = &'t Token>>(toks: &mut T) -> Result<Ast, String> {
    let mut ps = ParserState {
        toks: toks.peekable(),
    };
    ps.parse_t()
}

fn main() {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("failed to read stdin");
    let tokens: Result<Vec<Token>, ()> = Token::lexer(&input).collect();
    match tokens {
        Err(()) => eprintln!("Lex error"),
        Ok(tokens) => {
            println!("Tokens: {:?}", tokens);
            match parse(&mut tokens.iter()) {
                Ok(ast) => println!("Parsed: {:?}", ast),
                Err(e) => eprintln!("Parse error: {}", e),
            }
        }
    }
}

#[cfg(test)]
mod tests;
