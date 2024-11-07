use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::process::Command;

pub mod assembly;
pub mod lexer;
pub mod parser;

#[derive(PartialEq)]
enum Stage {
    Lex,
    Parse,
    Intermediate,
    Codegen,
    All,
}

fn compile(
    preprocessed: &str,
    assembly: &str,
    stage: &Stage,
    debug_mode: bool,
) -> std::io::Result<()> {
    println! {"Compiling..."};
    println! {"   Lexer"};
    let result = lexer::lexer::lex(preprocessed, debug_mode);
    if result.is_err() || *stage == Stage::Lex {
        return result.map(|_x| ());
    }
    println! {"   Parse"};
    let result = parser::parser::parse(&mut result.unwrap(), debug_mode);
    if result.is_err() || *stage == Stage::Parse {
        return result.map(|_x| ());
    }
    println! {"   Intermediate"};
    let result = parser::intermediate::intermediate(&mut result.as_ref().unwrap(), debug_mode);
    if result.is_err() || *stage == Stage::Intermediate {
        return result.map(|_x| ());
    }
    println! {"   Codegen"};
    assembly::generator::generate(&result.unwrap(), assembly, debug_mode)?;
    println! {"Done."};
    Ok(())
}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let stage = if args.iter().any(|x| x == "--lex") {
        Stage::Lex
    } else if args.iter().any(|x| x == "--parse") {
        Stage::Parse
    } else if args.iter().any(|x| x == "--tacky") {
        Stage::Intermediate
    } else if args.iter().any(|x| x == "--codegen") {
        Stage::Codegen
    } else {
        Stage::All
    };
    let dump_assembly = args.iter().any(|x| x == "-S");
    let debug_mode = args.iter().any(|x| x == "-D");
    let input = args.iter().find(|x| x.ends_with(".c"));
    //let input = &args[1];
    //if !input.ends_with(".c") {
    if input.is_none() {
        return Err(Error::new(ErrorKind::InvalidInput, "Not a c file"));
    }
    let input = input.unwrap();
    let mut preprocessed = input.clone();
    preprocessed.pop();
    preprocessed += "i";
    println!("{}", input);
    println!("{}", preprocessed);
    let _cmd_output = Command::new("gcc")
        .args(["-E", "-P", input, "-o", &preprocessed])
        .output()?;

    let mut assembly = input.clone();
    assembly.pop();
    assembly += "s";
    let mut output = input.clone();
    output.pop();
    output.pop();
    let result = compile(&preprocessed, &assembly, &stage, debug_mode);
    if result.is_err() || stage != Stage::All {
        return result;
    }
    fs::remove_file(preprocessed)?;

    println!("{}", assembly);
    println!("{}", output);
    let _cmd_output = Command::new("gcc")
        .args([&assembly, "-o", &output])
        .output()?;
    if !dump_assembly {
        fs::remove_file(assembly)?;
    }
    Ok(())
}
