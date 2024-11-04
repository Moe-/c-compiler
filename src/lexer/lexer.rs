use regex::Regex;
use std::collections::VecDeque;
use std::fs;
use std::io::{Error, ErrorKind};

#[derive(Clone, PartialEq, Debug, Copy)]
pub enum Token {
    Identifier,
    Constant,
    IntKeyword,
    VoidKeyword,
    ReturnKeyword,
    OpenParenthesis,
    CloseParenthesis,
    OpenBrace,
    CloseBrace,
    Semicolon,
    Tilde,
    Hyphen,
    TwoHyphens,
}

impl Token {
    const VALUES: [Self; 13] = [
        Self::Identifier,
        Self::Constant,
        Self::IntKeyword,
        Self::VoidKeyword,
        Self::ReturnKeyword,
        Self::OpenParenthesis,
        Self::CloseParenthesis,
        Self::OpenBrace,
        Self::CloseBrace,
        Self::Semicolon,
        Self::Tilde,
        Self::Hyphen,
        Self::TwoHyphens,
    ];

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
            Token::Tilde => r"~",
            Token::Hyphen => r"-",
            Token::TwoHyphens => r"--",
        }
    }

    fn regex() -> [(Token, Regex); 13] {
        Token::VALUES
            .clone()
            .map(|x| (x, Regex::new(x.expr()).unwrap()))
    }
}

#[derive(Debug)]
pub struct TokenValue {
    pub token: Token,
    pub data: Option<String>,
}

pub fn lex(preprocessed: &str, debug_mode: bool) -> std::io::Result<VecDeque<TokenValue>> {
    let mut to_parse = fs::read_to_string(preprocessed)?;
    let whitespace = Regex::new(r"\A\s+").unwrap();
    let regex = Token::regex();
    let mut tokens: VecDeque<TokenValue> = VecDeque::new();
    while !to_parse.is_empty() {
        if whitespace.is_match(&to_parse) {
            to_parse = String::from(to_parse.trim_start());
        } else {
            let mut candidates: Vec<_> = regex
                .iter()
                .map(|x| (x.0, x.1.find(&to_parse)))
                .filter(|x| x.1.is_some() && x.1.unwrap().start() == 0)
                .map(|x| (x.0, x.1.unwrap()))
                .collect();
            if candidates.is_empty() {
                return Err(Error::new(ErrorKind::InvalidInput, "Bad token"));
            }
            candidates.sort_by(|a, b| {
                let dynamic = [Token::Constant, Token::Identifier];
                let b_len = b.1.len()
                    + if dynamic.contains(&b.0) && !dynamic.contains(&a.0) {
                        0
                    } else {
                        1
                    };
                let a_len = a.1.len()
                    + if dynamic.contains(&a.0) && !dynamic.contains(&b.0) {
                        0
                    } else {
                        1
                    };
                b_len.cmp(&a_len)
            });

            let res = candidates.first().unwrap();
            if res.0 == Token::Identifier || res.0 == Token::Constant {
                tokens.push_back(TokenValue {
                    token: res.0,
                    data: Some(String::from(res.1.as_str())),
                });
            } else {
                tokens.push_back(TokenValue {
                    token: res.0,
                    data: None,
                });
            }
            to_parse = to_parse[res.1.end()..].to_string();
        }
    }
    if debug_mode {
        println!("{:?}", tokens)
    };
    Ok(tokens)
}
