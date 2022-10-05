use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

mod errors;
mod validation;


fn main() -> Result<(), errors::CmdArgsError> {
    // Check that the command line arguments supplies are correct
    let cmd_args: Vec<String> = env::args().collect();
    if cmd_args.len() != 3 || !cmd_args[1].ends_with(".asm") {
        return Err(errors::CmdArgsError);
    }

    println!("Compiling {} into {}", cmd_args[1], cmd_args[2]);

    // Iterate through each line and validate it
    let input_file = BufReader::new(OpenOptions::new().read(true).open(cmd_args[1].to_owned()).unwrap());
    for line_buffer in input_file.lines() {
        let line = line_buffer.unwrap();
        if line.is_empty() { // skip if line is blank
            continue;
        }

        validation::validate_asm_line(&line).unwrap();
    }

    Ok(())
}
