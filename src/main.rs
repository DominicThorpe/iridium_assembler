use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

mod errors;
mod validation;
mod token_generator;


pub fn run_assembler(input_file:&str, output_file:&str) {
    println!("Assembling {} into {}", input_file, output_file);

    // Iterate through each line and validate it
    let mut data_mode = false;
    let input_file = BufReader::new(OpenOptions::new().read(true).open(input_file.to_owned()).unwrap());
    let mut _tokens:Vec<token_generator::InstrTokens> = Vec::new();
    let mut next_label:Option<String> = None;
    for line_buffer in input_file.lines() {
        let line = line_buffer.unwrap();
        let line = line.trim();
        if line.is_empty() { // skip if line is blank
            continue;
        } else if line == "data:" {
            data_mode = true;
            continue;
        }

        validation::validate_asm_line(&line, data_mode).unwrap();

        if !data_mode {
            if line.ends_with(":") {
                next_label = Some(line[..line.len() - 1].to_owned());
            } else {
                _tokens.push(token_generator::generate_instr_tokens(line, next_label.clone()));
                next_label = None;
            }
        }
    }

    println!("Label\tOpcode\tOp A\tOp B\tOp C\tImm\tOp Label");
    for token in _tokens {
        println!("{:?}", token);
    }

    println!("Assembly successful!");
}


fn main() -> Result<(), errors::CmdArgsError> {
    // Check that the command line arguments supplies are correct
    let cmd_args: Vec<String> = env::args().collect();
    if cmd_args.len() != 3 || !cmd_args[1].ends_with(".asm") {
        return Err(errors::CmdArgsError);
    }

    run_assembler(&cmd_args[1], &cmd_args[2]);

    Ok(())
}
