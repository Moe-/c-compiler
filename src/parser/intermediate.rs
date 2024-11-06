use std::io::{Error, ErrorKind};
use crate::parser::parser::Node;
use std::collections::VecDeque;

#[derive(Debug,Clone)]
pub enum IntermediateOperations {
    Program,
    Function,
    Return,
    Unary,
    Constant,
    Var,
    Complement,
    Negate,
}

#[derive(Debug,Clone)]
pub enum IntermediateNode {
    Int(i32),
    Str(String),
    Terminal {
        op: IntermediateOperations,
    },
    Unary {
        op: IntermediateOperations,
        node: Box<IntermediateNode>,
    },
    Binary {
        op: IntermediateOperations,
        lhs: Box<IntermediateNode>,
        rhs: Box<IntermediateNode>,
    },
    Sequence(VecDeque<Box<IntermediateNode>>),
}

static mut TEMPORARY_COUNT: u32 = 0u32;

fn get_current_temporary() -> String {
    unsafe {
        format!("temp.{TEMPORARY_COUNT}")
    }
}

fn get_next_temporary() -> String {
    unsafe {
        TEMPORARY_COUNT += 1;
    }
    get_current_temporary()
}

fn get_constant_or_var(src: Box<IntermediateNode>) -> Box<IntermediateNode> {
    match &*src {
        IntermediateNode::Unary{op: IntermediateOperations::Constant, node: _} => src,
        _ => Box::new(IntermediateNode::Unary{ op: IntermediateOperations::Var, node: Box::new(IntermediateNode::Str(get_current_temporary()))} ),
    }
}

fn get_next_node() -> Box<IntermediateNode> {
    Box::new(IntermediateNode::Unary{ op: IntermediateOperations::Var, node: Box::new(IntermediateNode::Str(get_next_temporary()))} )
}

fn create_functions(ast: &Box<Node>, sequence: &mut VecDeque<Box<IntermediateNode>>) -> std::io::Result<Box<IntermediateNode>> {
    match &**ast {
        Node::Int(val) => Ok(Box::new(IntermediateNode::Int(*val))),
        Node::Unary { op, node } => { 
                match op {
                    crate::parser::parser::Operations::Return => {
                        let src = create_functions(node, sequence)?;
                        let src_node = get_constant_or_var(src);
                        sequence.push_back(Box::new(IntermediateNode::Unary { op: IntermediateOperations::Return, node: src_node.clone() }));
                        Ok(src_node)
                    },
                    crate::parser::parser::Operations::Constant => Ok(Box::new(IntermediateNode::Unary {
                        op: IntermediateOperations::Constant,
                        node: create_intermediate(node)?,
                    })),
                    crate::parser::parser::Operations::Negate => {
                        let src = create_functions(node, sequence)?;
                        let src_node = get_constant_or_var(src);
                        let dst = get_next_node();
                        sequence.push_back(Box::new(IntermediateNode::Binary { op: IntermediateOperations::Negate, lhs: src_node, rhs: dst.clone() }));
                        Ok(dst)
                    },
                    crate::parser::parser::Operations::Complement => {
                        let src = create_functions(node, sequence)?;
                        let src_node = get_constant_or_var(src);
                        let dst = get_next_node();
                        sequence.push_back(Box::new(IntermediateNode::Binary { op: IntermediateOperations::Complement, lhs: src_node, rhs: dst.clone() }));
                        Ok(dst)
        
                    },
                    _ => {
                        let error = format!("Unexpected unary AST node {:?}", op);
                        Err(Error::new(ErrorKind::InvalidInput, error))
                    }
            }
    },
        _ => {
            let error = format!("Unexpected AST node");
            Err(Error::new(ErrorKind::InvalidInput, error))
        }
    }
}

pub fn create_intermediate(ast: &Box<Node>) -> std::io::Result<Box<IntermediateNode>> {
    match &**ast {
        Node::Int(val) => Ok(Box::new(IntermediateNode::Int(*val))),
        Node::Str(val) => Ok(Box::new(IntermediateNode::Str(val.clone()))),
        Node::Unary { op, node } => match op {
            crate::parser::parser::Operations::Program => Ok(Box::new(IntermediateNode::Unary {
                op: IntermediateOperations::Program,
                node: create_intermediate(node)?,
            })),
            crate::parser::parser::Operations::Constant => Ok(Box::new(IntermediateNode::Unary {
                op: IntermediateOperations::Constant,
                node: create_intermediate(node)?,
            })),
            _ => {
                let error = format!("Unexpected unary AST node {:?}", op);
                Err(Error::new(ErrorKind::InvalidInput, error))
            }
        },
        Node::Binary { op, lhs, rhs } => match op {
            crate::parser::parser::Operations::Function => 
            {
                let mut sequence: VecDeque<Box<IntermediateNode>> = VecDeque::new();
                create_functions(rhs, &mut sequence)?;
                Ok(Box::new(IntermediateNode::Binary {
                    op: IntermediateOperations::Function,
                    lhs: create_intermediate(lhs)?,
                    rhs: Box::new(IntermediateNode::Sequence(sequence)),
                }))}
            ,
            _ => {
                let error = format!("Unexpected binary AST node {:?}", op);
                Err(Error::new(ErrorKind::InvalidInput, error))
            }
        },
    }
}



pub fn intermediate(ast: &Box<Node>, debug_mode: bool) -> std::io::Result<Box<IntermediateNode>> {
    let res = create_intermediate(ast)?;
    if debug_mode {
        println!("{:?}", res)
    };
    Ok(res)
}
