use std::env;
use std::fs::OpenOptions;
use std::io::prelude::*;
use std::io::BufReader;
use std::time::Instant;

mod errors;
mod validation;
mod token_generator;
mod label_table;
mod pseudo_substitution;
mod token_types;
mod generate_code;


/// Takes a filename and returns a `Vec<FileTokens>` representing the tokens of all the lines of assembly in the file
/// which can be either `DataTokens` or `InstrTokens`.
pub fn process_file_into_tokens(input_file:&str) -> Vec<token_types::FileTokens> {
    let mut mode = 'c';
    let input_file = BufReader::new(OpenOptions::new().read(true).open(input_file.to_owned()).unwrap())
        .lines()
        .map(|l| l.unwrap().trim().to_string())
        .filter(|l| !l.is_empty())
        .collect::<Vec<String>>();

    let mut tokens:Vec<token_types::FileTokens> = Vec::new();
    let mut next_label:Option<String> = None;
    for line in input_file {
        if line == "data:" {
            mode = 'd';
            continue;
        } else if line == "text:" {
            mode = 't';
            continue;
        }

        validation::validate_asm_line(&line, mode).unwrap();
        
        if line.ends_with(":") {
            next_label = Some(line[..line.len() - 1].to_owned());
            continue;
        }

        match mode {
            'c' => tokens.push(token_types::FileTokens::InstrTokens(token_generator::generate_instr_tokens(&line, next_label))),
            'd' => tokens.push(token_types::FileTokens::DataTokens(token_generator::generate_data_tokens(&line, next_label, mode))),
            't' => tokens.push(token_types::FileTokens::TextTokens(token_generator::generate_text_tokens(&line, next_label, mode))),
            _ => panic!("Invalid section mode '{}'", mode)
        }

        next_label = None;
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

    let now = Instant::now();

    let since = Instant::now();
    let tokens = process_file_into_tokens(&cmd_args[1]);
    println!("Tokenizer: {:?}", since.elapsed());

    let since = Instant::now();
    let tokens = pseudo_substitution::substitute_pseudo_instrs(tokens);
    println!("Pseudo Substitution: {:?}", since.elapsed());

    let since = Instant::now();
    let label_table = label_table::generate_label_table(&tokens).unwrap();
    println!("Label table: {:?}", since.elapsed());

    let since = Instant::now();
    let tokens = pseudo_substitution::substitute_labels(tokens, &label_table).unwrap();
    println!("Label substitution: {:?}", since.elapsed());

    let since = Instant::now();
    generate_code::generate_binary(&cmd_args[2], &tokens).unwrap();
    println!("Binary Generation: {:?}", since.elapsed());

    let mut sorted_vec:Vec<_> = label_table.iter().collect();
    sorted_vec.sort_by(|a, b| a.1.cmp(b.1));
    for (label, line) in sorted_vec {
        println!("{:<16} {:06X}", label, line);
    }
    
    for token in &tokens {
        println!("{:?}", token);
    }

    println!("Assembly successful! Took {:?} to process {} lines", now.elapsed(), tokens.len());

    Ok(())
}
