use std::env;
use std::fs;
use std::io::{Error, ErrorKind};
use std::process::Command;
use std::process::Output;

fn compile(preprocessed: &str, assembly: &str) {

}

fn main() -> std::io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let input = &args[1];
    if !input.ends_with(".c") {
        return  Err(Error::new(ErrorKind::InvalidInput, "Not a c file"));
    }
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
    fs::remove_file(assembly)?;
    Ok(())
}
