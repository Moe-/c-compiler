use crate::assembly::emission::emit;
use crate::parser::intermediate::{IntermediateNode, IntermediateOperations};
use std::collections::{HashMap, VecDeque};
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
    Stack(i64),
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

pub fn process_stack(aast: &mut Box<AssemblyNode>, stack_map:  &mut HashMap<String, i64>) -> Result<(), Error> {
    match &mut **aast {
        AssemblyNode::Int(_) => Ok(()),
        AssemblyNode::Str(_) => Ok(()),
        AssemblyNode::Register(_) => Ok(()),
        AssemblyNode::Stack(_) => Err(Error::new(ErrorKind::InvalidInput, "There should be no stack in AAST yet")),
        AssemblyNode::Terminal{ op: _} => Ok(()),
        AssemblyNode::Unary { op, node } => {
            match op {
                AssemblyOperations::Pseudo => {
                    let pseudo_name = match &**node {
                        AssemblyNode::Str(x) => x.clone(),
                        _ => return Err(Error::new(ErrorKind::InvalidInput, "Pseudo must reference Str in AAST")),
                    };
                    if !stack_map.contains_key(&pseudo_name) {
                        stack_map.insert(pseudo_name.clone(), -4i64 * (stack_map.len() as i64 + 1i64));
                    }
                    *aast = Box::new(AssemblyNode::Stack(stack_map[&pseudo_name]));
                },
                _ => {
                    process_stack( node, stack_map)?;
                },
            };
            Ok(())
        },
        AssemblyNode::Binary { op: _, lhs, rhs } => {
            process_stack(lhs, stack_map)?;
            process_stack(rhs, stack_map)?;
            Ok(())
        },
        AssemblyNode::Sequence(vec_deque) => {
            vec_deque.iter_mut().for_each(|x|{process_stack(x, stack_map).unwrap();});
            Ok(())
        },
    }
}

pub fn generate(ast: &Box<IntermediateNode>, assembly: &str, debug_mode: bool) -> std::io::Result<()> {
    let mut res = convert_ast(ast)?;

    println!("    - Convert");
    if debug_mode {
        println!("{:?}", res);
    }

    let mut stack_map: HashMap<String, i64> = HashMap::new();
    let _ = process_stack(&mut res, &mut stack_map)?;
    println!("    - Stack update");
    if debug_mode {
        println!("{:?}", res);
    }


    emit(&res, assembly, debug_mode)?;
    Ok(())
}
