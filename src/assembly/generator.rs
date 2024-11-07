use crate::assembly::emission::emit;
use crate::parser::intermediate::{IntermediateNode, IntermediateOperations};
use std::collections::VecDeque;
use std::io::{Error, ErrorKind};

#[derive(Debug,Clone)]
pub enum AssemblyOperations {
    Program,
    Function,
    //Statement,
    //Expression,
    Return,
    Imm,
    Mov,
    AllocateStack,
    Neg,
    Not,
    Pseudo,
    Stack,
}

#[derive(Debug,Clone)]
pub enum AssemblyRegister {
    AX,
    R10,
}

#[derive(Debug,Clone)]
pub enum AssemblyNode {
    Int(i32),
    Str(String),
    Register(AssemblyRegister),
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

pub fn convert_ast(ast: &Box<IntermediateNode>) -> std::io::Result<Box<AssemblyNode>> {
    match &**ast {
        IntermediateNode::Int(val) => Ok(Box::new(AssemblyNode::Int(*val))),
        IntermediateNode::Str(val) => Ok(Box::new(AssemblyNode::Str(val.clone()))),
        IntermediateNode::Unary { op, node } => match op {
            IntermediateOperations::Program => Ok(Box::new(AssemblyNode::Unary {
                op: AssemblyOperations::Program,
                node: convert_ast(node)?,
            })),
            IntermediateOperations::Return => {
                Ok(Box::new(AssemblyNode::Sequence(VecDeque::from([
                    Box::new(AssemblyNode::Binary {
                        op: AssemblyOperations::Mov,
                        lhs: convert_ast(node)?,
                        rhs: Box::new(AssemblyNode::Register(AssemblyRegister::AX)),
                    }),
                    Box::new(AssemblyNode::Terminal {
                        op: AssemblyOperations::Return,
                    }),
                ]))))
            }
            IntermediateOperations::Constant => Ok(Box::new(AssemblyNode::Unary {
                op: AssemblyOperations::Imm,
                node: convert_ast(node)?,
            })),
            IntermediateOperations::Var => Ok(Box::new(AssemblyNode::Unary { op: AssemblyOperations::Pseudo, node: convert_ast(node)? })),
            _ => {
                let error = format!("Unexpected unary AST node {:?}", op);
                Err(Error::new(ErrorKind::InvalidInput, error))
            }
        },
        IntermediateNode::Binary { op, lhs, rhs } => match op {
            IntermediateOperations::Function => Ok(Box::new(AssemblyNode::Binary {
                op: AssemblyOperations::Function,
                lhs: convert_ast(lhs)?,
                rhs: convert_ast(rhs)?,
            })),
            IntermediateOperations::Complement => {
                let src = convert_ast(lhs)?;
                let dst = convert_ast(rhs)?;
                Ok(Box::new(AssemblyNode::Sequence(VecDeque::from([
                    Box::new(AssemblyNode::Binary {
                        op: AssemblyOperations::Mov,
                        lhs: src,
                        rhs: dst.clone(),
                    }),
                    Box::new(AssemblyNode::Unary { op: AssemblyOperations::Not, node: dst }),
                ]))))
            },
            IntermediateOperations::Negate => {
                let src = convert_ast(lhs)?;
                let dst = convert_ast(rhs)?;
                Ok(Box::new(AssemblyNode::Sequence(VecDeque::from([
                    Box::new(AssemblyNode::Binary {
                        op: AssemblyOperations::Mov,
                        lhs: src,
                        rhs: dst.clone(),
                    }),
                    Box::new(AssemblyNode::Unary { op: AssemblyOperations::Neg, node: dst }),
                ]))))
            },
            _ => {
                let error = format!("Unexpected binary AST node {:?}", op);
                Err(Error::new(ErrorKind::InvalidInput, error))
            }
        },
        IntermediateNode::Sequence(vec_deque) => {
            let seq: VecDeque<Box<AssemblyNode>> = vec_deque.iter().map(|f|convert_ast(f).unwrap()).collect();
            let mut queue : VecDeque<Box<AssemblyNode>> = VecDeque::new();
            for s in seq {
                match *s {
                    AssemblyNode::Sequence(s) => s.iter().for_each(|x|queue.push_back(x.clone())),
                    _ => queue.push_back(s),
                }
            }
            Ok(Box::new(AssemblyNode::Sequence(queue)))
        },
    }
}

pub fn generate(ast: &Box<IntermediateNode>, assembly: &str, debug_mode: bool) -> std::io::Result<()> {
    let res = convert_ast(ast)?;
    if debug_mode {
        println!("{:?}", res)
    };
    emit(&res, assembly, debug_mode)?;
    Ok(())
}
