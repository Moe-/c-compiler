use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::process::Command;
use std::process::Output;

pub mod lexer;

#[derive(PartialEq)]
enum Stage {
    Lex,
    Parse,
    Codegen,
    All,
}

fn compile(preprocessed: &str, assembly: &str, stage: &Stage) -> std::io::Result<()> {
    let result = lexer::lexer::lex(preprocessed);
    if result.is_err() || *stage == Stage::Lex {
        return result;
    }
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let stage = if args.iter().any(|x| x == "--lex") {
        Stage::Lex
    } else if args.iter().any(|x| x == "--parse") {
        Stage::Parse
    } else if args.iter().any(|x| x == "--codegen") {
        Stage::Codegen
    } else {
        Stage::All
    };
    let dump_assembly = args.iter().any(|x| x == "-S");
    let input = args.iter().find(|x| x.ends_with(".c"));
    //let input = &args[1];
    //if !input.ends_with(".c") {
    if input.is_none() {
        return  Err(Error::new(ErrorKind::InvalidInput, "Not a c file"));
    }
    let input = input.unwrap();
    let mut preprocessed = input.clone();
    preprocessed.pop();
    preprocessed += "i";
    println!("{}", input);
    println!("{}", preprocessed);
    let cmd_output = Command::new("gcc")
        .args(["-E", "-P", input, "-o", &preprocessed])
        .output();


    let mut assembly = input.clone();
    assembly.pop();
    assembly += "s";
    let mut output = input.clone();
    output.pop();
    output.pop();
    let result = compile(&preprocessed, &assembly, &stage);
    if result.is_err() || stage != Stage::All {
        return result;
    }
    fs::remove_file(preprocessed)?;

    
    println!("{}", assembly);
    println!("{}", output);
    let cmd_output = Command::new("gcc")
        .args([&assembly, "-o", &output])
        .output();
    if !dump_assembly {
        fs::remove_file(assembly)?;
    }
    Ok(())
}
