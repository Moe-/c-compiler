use crate::lexer::lexer::{Token, TokenValue};
use std::collections::VecDeque;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub enum Operations {
    Program,
    Function,
    //Statement,
    //Expression,
    Return,
    Constant,
    Unary,
    Complement,
    Negate,
}

#[derive(Debug)]
pub enum Node {
    Int(i32),
    Str(String),
    Unary {
        op: Operations,
        node: Box<Node>,
    },
    Binary {
        op: Operations,
        lhs: Box<Node>,
        rhs: Box<Node>,
    },
}

fn check_token(token: &Option<TokenValue>, token_type: Token) -> std::io::Result<()> {
    if token.is_none() {
        let error = format!("Missing token, expected {:?}", token_type);
        return Err(Error::new(ErrorKind::InvalidInput, error));
    } else if token.as_ref().unwrap().token != token_type {
        let error = format!(
            "Bad token, found {:?}, but expected {:?}",
            token.as_ref().unwrap().token,
            token_type
        );
        return Err(Error::new(ErrorKind::InvalidInput, error));
    }
    Ok(())
}

fn identifier(tokens: &mut VecDeque<TokenValue>) -> std::io::Result<Box<Node>> {
    let token = tokens.pop_front();
    check_token(&token, Token::Identifier)?;
    Ok(Box::new(Node::Str(token.unwrap().data.unwrap())))
}

fn int(tokens: &mut VecDeque<TokenValue>) -> std::io::Result<Box<Node>> {
    let token = tokens.pop_front();
    check_token(&token, Token::Constant)?;
    let string = token.unwrap().data.unwrap();
    let int = string.parse::<i32>();
    if int.is_err() {
        return Err(Error::new(ErrorKind::InvalidInput, "Bad integer"));
    }
    Ok(Box::new(Node::Int(int.unwrap())))
}

fn unop(tokens: &mut VecDeque<TokenValue>) -> std::io::Result<Operations> {
    let token = tokens.pop_front().ok_or(Error::new(ErrorKind::InvalidInput, "Missing unop operator"))?;
    match token.token {
        Token::Hyphen => Ok(Operations::Negate),
        Token::Tilde => Ok(Operations::Complement),
        _ => Err(Error::new(ErrorKind::InvalidInput, "Bad unop")),
    }
}

fn exp(tokens: &mut VecDeque<TokenValue>) -> std::io::Result<Box<Node>> {
    let next = tokens.front().ok_or(Error::new(ErrorKind::InvalidInput, "Missing exp token"))?;
    match next.token {
        Token::Constant => Ok(Box::new(Node::Unary {
            op: Operations::Constant,
            node: int(tokens)?,
        })),
        Token::Hyphen | Token::Tilde => {
            let op = unop(tokens)?;
            let exp = exp(tokens)?;
            Ok(Box::new(Node::Unary { op: op, node: exp }))
        },
        Token::OpenParenthesis => {
            check_token(&tokens.pop_front(), Token::OpenParenthesis)?;
            let res = exp(tokens);
            check_token(&tokens.pop_front(), Token::CloseParenthesis)?;
            res
        }
        _ => Err(Error::new(ErrorKind::InvalidInput, "Unknown expression")),
    }
}

fn statement(tokens: &mut VecDeque<TokenValue>) -> std::io::Result<Box<Node>> {
    check_token(&tokens.pop_front(), Token::ReturnKeyword)?;
    let e = exp(tokens)?;
    check_token(&tokens.pop_front(), Token::Semicolon)?;
    Ok(Box::new(Node::Unary {
        op: Operations::Return,
        node: e,
    }))
}

fn function(tokens: &mut VecDeque<TokenValue>) -> std::io::Result<Box<Node>> {
    check_token(&tokens.pop_front(), Token::IntKeyword)?;
    let i = identifier(tokens)?;
    check_token(&tokens.pop_front(), Token::OpenParenthesis)?;
    check_token(&tokens.pop_front(), Token::VoidKeyword)?;
    check_token(&tokens.pop_front(), Token::CloseParenthesis)?;
    check_token(&tokens.pop_front(), Token::OpenBrace)?;
    let s = statement(tokens)?;
    check_token(&tokens.pop_front(), Token::CloseBrace)?;
    Ok(Box::new(Node::Binary {
        op: Operations::Function,
        lhs: i,
        rhs: s,
    }))
}

fn program(tokens: &mut VecDeque<TokenValue>) -> std::io::Result<Box<Node>> {
    let f = function(tokens)?;
    if !tokens.is_empty() {
        return Err(Error::new(
            ErrorKind::InvalidInput,
            "invalid top level identifer",
        ));
    }
    Ok(Box::new(Node::Unary {
        op: Operations::Program,
        node: f,
    }))
}

pub fn parse(tokens: &mut VecDeque<TokenValue>, debug_mode: bool) -> std::io::Result<Box<Node>> {
    let p = program(tokens)?;
    if debug_mode {
        println!("{:?}", p)
    };
    Ok(p)
}
