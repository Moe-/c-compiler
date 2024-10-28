
use std::fs;
use std::{clone, io::{Error, ErrorKind}};
use regex::Regex;

#[derive(Clone,PartialEq,Debug,Copy)]
enum Token {
    Identifier,
    Constant,
    IntKeyword,
    VoidKeyword,
    ReturnKeyword,
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Semicolon
}

impl Token {
    const VALUES: [Self; 10] = [Self::Identifier, Self::Constant, Self::IntKeyword, Self::VoidKeyword, Self::ReturnKeyword, Self::OpenParenthesis, Self::CloseParenthesis, Self::OpenBrace, Self::CloseBrace, Self::Semicolon];

    fn expr(&self) -> &str {
        match *self {
            Token::Identifier => r"[a-zA-Z_]\w*\b",
            Token::Constant => r"[0-9]+\b",
            Token::IntKeyword => r"int\b",
            Token::VoidKeyword => r"void\b",
            Token::ReturnKeyword => r"return\b",
            Token::OpenParenthesis => r"\(",
            Token::CloseParenthesis => r"\)",
            Token::OpenBrace => r"\{",
            Token::CloseBrace => r"\}",
            Token::Semicolon => r";",
        }
    }

    fn sorted_by_size() -> [Self; 10] {
        let mut values = Token::VALUES.clone();
        values.sort_by(|a, b| b.expr().len().cmp(&a.expr().len()));
        return values;
    }

    fn sorted_as_regex() -> [(Token, Regex); 10] {
        Token::sorted_by_size().map(|x|(x, Regex::new(x.expr()).unwrap()))
    }

    fn regex() -> [(Token, Regex); 10] {
        Token::VALUES.clone().map(|x|(x, Regex::new(x.expr()).unwrap()))
    }
}

#[derive(Debug)]
struct TokenValue {
    token: Token,
    data: Option<String>,
}

pub fn lex (preprocessed: &str) -> std::io::Result<()> {
    let mut to_parse = fs::read_to_string(preprocessed)?;
    let whitespace = Regex::new(r"\A\s+").unwrap();
    let regex = Token::regex();
    let mut token: Vec<TokenValue> = Vec::new();
    while !to_parse.is_empty() {
        println!("{}", to_parse);
        if whitespace.is_match(&to_parse) {
            to_parse = String::from(to_parse.trim_start());
        } else {
            let mut candidates: Vec<_> = regex.iter()
                .map(|x|(x.0, x.1.find(&to_parse)))
                .filter(|x|x.1.is_some() && x.1.unwrap().start() == 0)
                .map(|x|(x.0, x.1.unwrap()))
                .collect();
            if candidates.is_empty() {
                return Err(Error::new(ErrorKind::InvalidInput, "Bad token"));
            }
            candidates.sort_by(|a, b| {
                let dynamic = [Token::Constant, Token::Identifier];
                let b_len = b.1.len() + if dynamic.contains(&b.0) && !dynamic.contains(&a.0) {
                    0
                } else {
                    1
                };
                let a_len = a.1.len() + if dynamic.contains(&a.0) && !dynamic.contains(&b.0) {
                    0
                } else {
                    1
                };
                b_len.cmp(&a_len)
            });

            let res = candidates.first().unwrap();
            if res.0 == Token::Identifier || res.0 == Token::Constant {
                token.push(TokenValue { token: res.0, data: Some(String::from(res.1.as_str())) });
            } else {
                token.push(TokenValue { token: res.0, data: None });
            }
            
            println!("{:?}", candidates);
            to_parse = to_parse[res.1.end()..].to_string();
        }
    }
    println!("{:?}", token);
    println!("{}", preprocessed);
    Ok(())
}