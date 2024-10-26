use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::process::Command;
use std::process::Output;

enum Stage {
    Lex,
    Parse,
    Codegen,
    All,
}

fn compile(preprocessed: &str, assembly: &str) {

}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut stage = if args.iter().any(|x| x == "--lex") {
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
    compile(&preprocessed, &assembly);
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
