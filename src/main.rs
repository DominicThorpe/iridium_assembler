use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;

mod errors;
mod validation;
mod token_generator;
mod label_table;
mod pseudo_substitution;
mod token_types;


/// Takes a filename and returns a `Vec<FileTokens>` representing the tokens of all the lines of assembly in the file
/// which can be either `DataTokens` or `InstrTokens`.
pub fn process_file_into_tokens(input_file:&str) -> Vec<token_types::FileTokens> {
    let mut data_mode = false;
    let input_file = BufReader::new(OpenOptions::new().read(true).open(input_file.to_owned()).unwrap());
    let mut tokens:Vec<token_types::FileTokens> = Vec::new();
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
                tokens.push(token_types::FileTokens::InstrTokens(token_generator::generate_instr_tokens(line, next_label.clone())));
                next_label = None;
            }
        } else {
            if line.ends_with(":") {
                next_label = Some(line[..line.len() - 1].to_owned());
            } else {
                tokens.push(token_types::FileTokens::DataTokens(token_generator::generate_data_tokens(line, next_label.clone())));
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
    let tokens = pseudo_substitution::substitute_pseudo_instrs(tokens);
    let label_table = label_table::generate_label_table(&tokens);

    let mut sorted_vec:Vec<_> = label_table.iter().collect();
    sorted_vec.sort_by(|a, b| a.1.cmp(b.1));
    for (label, line) in sorted_vec {
        println!("{:<16} {:06X}", label, line);
    }

    for token in tokens {
        println!("{:?}", token);
    }

    println!("Assembly successful!");

    Ok(())
}
