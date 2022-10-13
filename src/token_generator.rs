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
/// Assumes that the file and its contents have already been validated and line is an instruction and not
/// blank.
pub fn generate_instr_tokens(line:&str) -> InstrTokens {
    // TODO: handle labels
    if line.ends_with(":") {
        return InstrTokens::new(None, String::from(""), None, None, None, None, None);
    }

    let label:Option<String> = match line.find(":") {
        Some(index) => Some(line[..index].to_owned()),
        None => None
    };

    let opcode = validate_opcode(&line).unwrap();
    let operands:Vec<String> = get_operands_from_line(&line, opcode).iter().map(|operand| operand.to_owned()).collect();

    match operands.len() {
        0 => InstrTokens::new(label, opcode.to_owned(), None, None, None, None, None),
        1 => InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), None, None, None, None),
        2 => {
            if !operands[1].starts_with("$") {
                InstrTokens::new(label.clone(), opcode.to_owned(), Some(operands[0].clone()), None, None, Some(get_immediate_from_string(&operands[1])), None);
            }

            InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), None, None, None)
        },

        4 => InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), Some(operands[2].clone()), None, Some(operands[3].clone())),
        3 => { // may or may not contain a label as an operand
            let mut tokens = InstrTokens::new(label.clone(), opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), Some(operands[2].clone()), 
                                                        None, None);
            if operands[2].starts_with("@") {
                tokens = InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), None, None, Some(operands[2].clone()))
            } else if !operands[2].starts_with("$") {
                tokens = InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), Some(operands[1].clone()), None, Some(get_immediate_from_string(&operands[2])), None)
            }
            
            tokens
        },
        _ => panic!("Invalid number of operands (validation module has failed!)"),
    }
}


#[cfg(test)]
mod tests {
    use crate::token_generator::generate_instr_tokens;


    #[test]
    fn test_token_generation_addi() {
        let tokens = generate_instr_tokens("init: ADDI $g0, $zero, 1");
        assert_eq!(tokens.label.as_ref().unwrap(), "init");
        assert_eq!(tokens.opcode, "ADDI");
        assert_eq!(tokens.operand_a.as_ref().unwrap(), "$g0");
        assert_eq!(tokens.operand_b.as_ref().unwrap(), "$zero");
        assert_eq!(tokens.operand_c.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens.op_label.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
    }


    #[test]
    fn test_token_addi_all_bases() {
        let tokens_decimal = generate_instr_tokens("init: ADDI $g0, $zero, 1");
        assert_eq!(*tokens_decimal.immediate.as_ref().unwrap(), 1);

        let tokens_hex = generate_instr_tokens("init: ADDI $g0, $zero, 0b0010");
        assert_eq!(*tokens_hex.immediate.as_ref().unwrap(), 2);

        let tokens_binary = generate_instr_tokens("init: ADDI $g0, $zero, 0x0004");
        assert_eq!(*tokens_binary.immediate.as_ref().unwrap(), 4);
    }


    #[test]
    fn test_load_token_generation_with_label_opcode() {
        let tokens = generate_instr_tokens("LOAD $g5, $g8, $g9, @target");
        assert_eq!(tokens.label.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens.opcode, "LOAD");
        assert_eq!(tokens.operand_a.as_ref().unwrap(), "$g5");
        assert_eq!(tokens.operand_b.as_ref().unwrap(), "$g8");
        assert_eq!(tokens.operand_c.as_ref().unwrap(), "$g9");
        assert_eq!(tokens.op_label.as_ref().unwrap(), "@target");
    }


    #[test]
    fn test_jump_token_generation_with_label_opcode() {
        let tokens = generate_instr_tokens("JUMP $g8, $g9, @loop");
        assert_eq!(tokens.label.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens.opcode, "JUMP");
        assert_eq!(tokens.operand_a.as_ref().unwrap(), "$g8");
        assert_eq!(tokens.operand_b.as_ref().unwrap(), "$g9");
        assert_eq!(tokens.operand_c.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens.op_label.as_ref().unwrap(), "@loop");
    }
}
