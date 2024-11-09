use crate::assembly::generator::AssemblyNode;
use std::fs::File;
use std::io::{Error, ErrorKind, Write};

fn convert_aast(aast: &Box<AssemblyNode>) -> std::io::Result<String> {
    let mut output = String::new();

    match &**aast {
        AssemblyNode::Int(x) => output += format!("${x}").as_str(),
        AssemblyNode::Str(x) => output += x,
        AssemblyNode::Register(reg) => match *reg {
            super::generator::AssemblyRegister::AX => output += "%eax",
            super::generator::AssemblyRegister::R10 => output += "%r10d",
        },
        AssemblyNode::AllocateStack(depth) => {
            output += format!("    subq ${depth}, %rsp\n").as_str()
        }
        AssemblyNode::Stack(depth) => output += format!("{depth}(%rbp)").as_str(),
        AssemblyNode::Terminal { op } => match op {
            super::generator::AssemblyOperations::Return => {
                output += "    movq %rbp, %rsp\n";
                output += "    popq %rbp\n";
                output += "    ret\n";
            }
            _ => {
                let error = format!("Unexpected terminal assembly AST node {:?}", op);
                return Err(Error::new(ErrorKind::InvalidInput, error));
            }
        },
        AssemblyNode::Unary { op, node } => match op {
            super::generator::AssemblyOperations::Program => {
                output += convert_aast(node)?.as_str();
                if std::env::consts::OS == "linux" {
                    output += "    .section .note.GNU-stack,\"\",@progbits\n";
                }
            }
            super::generator::AssemblyOperations::Imm => match &**node {
                AssemblyNode::Int(_) => output += convert_aast(node)?.as_str(),
                _ => return Err(Error::new(ErrorKind::InvalidInput, "Imm got bad type")),
            },
            super::generator::AssemblyOperations::Neg => {
                output += format!("    negl {}\n", convert_aast(node)?).as_str()
            }
            super::generator::AssemblyOperations::Not => {
                output += format!("    notl {}\n", convert_aast(node)?).as_str()
            }
            _ => {
                let error = format!("Unexpected unary assembly AST node {:?}", op);
                return Err(Error::new(ErrorKind::InvalidInput, error));
            }
        },
        AssemblyNode::Binary { op, lhs, rhs } => match op {
            super::generator::AssemblyOperations::Function => {
                let name = convert_aast(lhs)?;
                output += format!("    .globl {name}\n").as_str();
                output += format!("{name}:\n").as_str();
                output += format!("    pushq %rbp\n").as_str();
                output += format!("    movq %rsp, %rbp\n").as_str();
                output += convert_aast(rhs)?.as_str();
            }
            super::generator::AssemblyOperations::Mov => {
                output +=
                    format!("    movl {}, {}\n", convert_aast(lhs)?, convert_aast(rhs)?).as_str();
            }
            _ => {
                let error = format!("Unexpected binary assembly AST node {:?}", op);
                return Err(Error::new(ErrorKind::InvalidInput, error));
            }
        },
        AssemblyNode::Sequence(vec_deque) => {
            for x in vec_deque {
                output += convert_aast(x)?.as_str();
            }
        }
    }

    Ok(output)
}

pub fn emit(aast: &Box<AssemblyNode>, assembly: &str, debug_mode: bool) -> std::io::Result<()> {
    let res = convert_aast(aast)?;
    if debug_mode {
        println!("{:?}", res)
    };

    let mut file = File::create(assembly)?;
    file.write_all(res.as_bytes())?;

    Ok(())
}
