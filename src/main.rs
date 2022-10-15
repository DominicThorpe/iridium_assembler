use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

mod errors;
mod validation;
mod token_generator;
mod label_table;


/// Can contain both types of tokens a line of asm can take
#[derive(Debug)]
pub enum FileTokens {
    InstrTokens(token_generator::InstrTokens),
    DataTokens(token_generator::DataTokens)
}


/// Takes a filename and returns a `FileTokens`
pub fn process_file_into_tokens(input_file:&str) -> Vec<FileTokens> {
    let mut data_mode = false;
    let input_file = BufReader::new(OpenOptions::new().read(true).open(input_file.to_owned()).unwrap());
    // let mut instr_tokens:Vec<token_generator::InstrTokens> = Vec::new();
    // let mut data_tokens:Vec<token_generator::DataTokens> = Vec::new();

    let mut tokens:Vec<FileTokens> = Vec::new();
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
                tokens.push(FileTokens::InstrTokens(token_generator::generate_instr_tokens(line, next_label.clone())));
                next_label = None;
            }
        } else {
            if line.ends_with(":") {
                next_label = Some(line[..line.len() - 1].to_owned());
            } else {
                tokens.push(FileTokens::DataTokens(token_generator::generate_data_tokens(line, next_label.clone())));
                next_label = None;
            }
        }
    }

    tokens
}


/// Runs the assebler through the process of assembling the input file into the output file.
///
/// Iterates through each line of the input file and validates and tokensizes the lines then:
///  - Converts any lines with label operands into several instructions which load the
///    necessary values into registers
///  - Builds a table of labels and what address they point to
///  - Substitutes labels for immediates
///  - Converts each set of tokens rperesenting an instruction into bytes
///  - Writes the bytes to the output file
fn main() -> Result<(), errors::CmdArgsError> {
    // Check that the command line arguments supplies are correct
    let cmd_args: Vec<String> = env::args().collect();
    if cmd_args.len() != 3 || !cmd_args[1].ends_with(".asm") {
        return Err(errors::CmdArgsError);
    }

    println!("Assembling {} into {}", cmd_args[1], cmd_args[2]);

    let tokens = process_file_into_tokens(&cmd_args[1]);
    let label_table = label_table::generate_instr_label_table(tokens);
    for (label, line) in label_table {
        println!("{:<16} {:06x}", label, line);
    }

    println!("Assembly successful!");

    Ok(())
}
