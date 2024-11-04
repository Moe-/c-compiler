use crate::assembly::emission::emit;
use crate::parser::parser::Node;
use std::collections::VecDeque;
use std::io::{Error, ErrorKind};

#[derive(Debug)]
pub enum AssemblyOperations {
    Program,
    Function,
    //Statement,
    //Expression,
    Return,
    Imm,
    Mov,
}

#[derive(Debug)]
pub enum AssemblyNode {
    Int(i32),
    Str(String),
    Register,
    Terminal {
        op: AssemblyOperations,
    },
    Unary {
        op: AssemblyOperations,
        node: Box<AssemblyNode>,
    },
    Binary {
        op: AssemblyOperations,
        lhs: Box<AssemblyNode>,
        rhs: Box<AssemblyNode>,
    },
    Sequence(VecDeque<Box<AssemblyNode>>),
}

pub fn convert_ast(ast: &Box<Node>) -> std::io::Result<Box<AssemblyNode>> {
    match &**ast {
        Node::Int(val) => Ok(Box::new(AssemblyNode::Int(*val))),
        Node::Str(val) => Ok(Box::new(AssemblyNode::Str(val.clone()))),
        Node::Unary { op, node } => match op {
            crate::parser::parser::Operations::Program => Ok(Box::new(AssemblyNode::Unary {
                op: AssemblyOperations::Program,
                node: convert_ast(node)?,
            })),
            crate::parser::parser::Operations::Return => {
                Ok(Box::new(AssemblyNode::Sequence(VecDeque::from([
                    Box::new(AssemblyNode::Binary {
                        op: AssemblyOperations::Mov,
                        lhs: convert_ast(node)?,
                        rhs: Box::new(AssemblyNode::Register),
                    }),
                    Box::new(AssemblyNode::Terminal {
                        op: AssemblyOperations::Return,
                    }),
                ]))))
            }
            crate::parser::parser::Operations::Constant => Ok(Box::new(AssemblyNode::Unary {
                op: AssemblyOperations::Imm,
                node: convert_ast(node)?,
            })),
            _ => {
                let error = format!("Unexpected unary AST node {:?}", op);
                Err(Error::new(ErrorKind::InvalidInput, error))
            }
        },
        Node::Binary { op, lhs, rhs } => match op {
            crate::parser::parser::Operations::Function => Ok(Box::new(AssemblyNode::Binary {
                op: AssemblyOperations::Function,
                lhs: convert_ast(lhs)?,
                rhs: convert_ast(rhs)?,
            })),
            _ => {
                let error = format!("Unexpected binary AST node {:?}", op);
                Err(Error::new(ErrorKind::InvalidInput, error))
            }
        },
    }
}

pub fn generate(ast: &Box<Node>, assembly: &str, debug_mode: bool) -> std::io::Result<()> {
    let res = convert_ast(ast)?;
    if debug_mode {
        println!("{:?}", res)
    };
    emit(&res, assembly, debug_mode)?;
    Ok(())
}
