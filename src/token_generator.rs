use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::prelude::*;

use crate::validation::{validate_opcode, get_operands_from_line};



/// Represents the core components of an instruction, including the opcode, and the optional label and 
/// operands, and possible operand label
#[allow(dead_code)]
pub struct InstrTokens {
    label: Option<String>,
    opcode: String,
    operand_a: Option<String>,
    operand_b: Option<String>,
    operand_c: Option<String>,
    immediate: Option<i64>,
    op_label: Option<String>
}


impl InstrTokens {
    fn new(label:Option<String>, opcode:String, operand_a:Option<String>, 
        operand_b:Option<String>, operand_c:Option<String>, immediate:Option<i64>, 
        op_label:Option<String>) -> InstrTokens {
            InstrTokens {
                label: label,
                opcode: opcode,
                operand_a: operand_a,
                operand_b: operand_b,
                operand_c: operand_c,
                immediate: immediate,
                op_label: op_label
            }
    }
}


/// Takes a string of an integer in binary, decimal, or hexadecimal and returns it. Assumes that the
/// input has already been validated.
fn get_immediate_from_string(immediate:&str) -> i64 {
    let parsed_immediate:i64;
    if immediate.starts_with("0x") {
        parsed_immediate = i64::from_str_radix(&immediate[2..], 16).unwrap();
    } else if immediate.starts_with("0b") {
        parsed_immediate = i64::from_str_radix(&immediate[2..], 2).unwrap();
    } else {
        parsed_immediate = immediate.parse().unwrap();
    }

    parsed_immediate
}


/// Takes the name of a file and iterates through its lines, generating a `Vec<InstrTokens>` from those 
/// lines representing their various components.
///
/// Assumes that the file and its contents have already been validated.
pub fn generate_instr_vec(filename:&str) -> Vec<InstrTokens> {
    let mut tokens_vec:Vec<InstrTokens> = Vec::new();

    let input_file = BufReader::new(OpenOptions::new().read(true).open(filename).unwrap());
    for line_buffer in input_file.lines() {
        let line = line_buffer.unwrap();
        if line.trim().is_empty() {
            continue;
        }

        if line.trim().starts_with("data:") {
            break;
        }

        let label:Option<String> = match line.find(":") {
            Some(index) => Some(line[..index].to_owned()),
            None => None
        };

        let opcode = validate_opcode(&line).unwrap();
        let operands:Vec<String> = get_operands_from_line(&line, opcode).iter().map(|operand| operand.to_owned()).collect();

        tokens_vec.append(
            &mut match operands.len() {
                0 => vec![InstrTokens::new(label, opcode.to_owned(), None, None, None, None, None)],
                1 => vec![InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), None, None, None, None)],
                2 => {
                    if !operands[1].starts_with("$") {
                        vec![InstrTokens::new(label.clone(), opcode.to_owned(), Some(operands[0].clone()), None, None, Some(get_immediate_from_string(&operands[1])), None)];
                    }

                    vec![InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), None, None, None)]
                },

                4 => vec![InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), Some(operands[2].clone()), None, Some(operands[3].clone()))],
                3 => { // may or may not contain a label as an operand
                    let mut tokens = vec![InstrTokens::new(label.clone(), opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), Some(operands[2].clone()), 
                                                                None, None)];
                    if operands[2].starts_with("@") {
                        tokens = vec![InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), None, None, Some(operands[2].clone()))]
                    } else if !operands[2].starts_with("$") {
                        tokens = vec![InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), None, Some(get_immediate_from_string(&operands[2])), None)]
                    }
                    
                    tokens
                },
                _ => panic!("Invalid number of operands (validation module has failed!)"),
            }
        );
    }

    tokens_vec
}


#[cfg(test)]
mod tests {
    use crate::token_generator::generate_instr_vec;


    #[test]
    fn test_token_generation() {
        let tokens = generate_instr_vec("test_files/test_token_generation.asm");

        assert_eq!(tokens.len(), 11);

        assert_eq!(tokens[0].label.as_ref().unwrap(), "init");
        assert_eq!(tokens[0].opcode, "ADDI");
        assert_eq!(tokens[0].operand_a.as_ref().unwrap(), "$g0");
        assert_eq!(tokens[0].operand_b.as_ref().unwrap(), "$zero");
        assert_eq!(tokens[0].operand_c.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(*tokens[0].immediate.as_ref().unwrap(), 1);
        assert_eq!(tokens[0].op_label.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());

        assert_eq!(*tokens[1].immediate.as_ref().unwrap(), 1);

        assert_eq!(tokens[2].label.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens[2].opcode, "LOAD");
        assert_eq!(tokens[2].operand_a.as_ref().unwrap(), "$g5");
        assert_eq!(tokens[2].operand_b.as_ref().unwrap(), "$g8");
        assert_eq!(tokens[2].operand_c.as_ref().unwrap(), "$g9");
        assert_eq!(tokens[2].op_label.as_ref().unwrap(), "@target");

        assert_eq!(tokens[9].label.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens[9].opcode, "JUMP");
        assert_eq!(tokens[9].operand_a.as_ref().unwrap(), "$g8");
        assert_eq!(tokens[9].operand_b.as_ref().unwrap(), "$g9");
        assert_eq!(tokens[9].operand_c.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens[9].op_label.as_ref().unwrap(), "@loop");
    }
}
