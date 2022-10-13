use std::fmt;
use crate::validation::*;



/// Represents the core components of an instruction, including the opcode, and the optional label and 
/// operands, and possible operand label
pub struct InstrTokens {
    label: Option<String>,
    opcode: String,
    operand_a: Option<String>,
    operand_b: Option<String>,
    operand_c: Option<String>,
    immediate: Option<u64>, // used as a set of bytes
    op_label: Option<String>
}

impl InstrTokens {
    fn new(label:Option<String>, opcode:String, operand_a:Option<String>, 
        operand_b:Option<String>, operand_c:Option<String>, immediate:Option<u64>, 
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

impl fmt::Debug for InstrTokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}\t{}\t{}\t{}\t0x{:04x}\t{}", 
                self.label.as_ref().unwrap_or(&"none".to_owned()), 
                self.opcode, 
                self.operand_a.as_ref().unwrap_or(&"none".to_owned()), 
                self.operand_b.as_ref().unwrap_or(&"none".to_owned()),
                self.operand_c.as_ref().unwrap_or(&"none".to_owned()), 
                self.immediate.unwrap_or(0), 
                self.op_label.as_ref().unwrap_or(&"none".to_owned())
            )
    }
}


/// Represents the components of a data instruction, including the label, category, and value
pub struct DataTokens {
    label: String,
    category: String,
    bytes: Vec<u16>
}


impl DataTokens {
    fn new(label:String, category:String, bytes:Vec<u16>) -> DataTokens {
        DataTokens {
            label: label,
            category: category,
            bytes: bytes
        }
    }
}


impl fmt::Debug for DataTokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}\t{:04x?}", self.label, self.category, self.bytes)
    }
}


/// Takes some data in the form of a string which can be any data type (e.g. long, text, integer, section...) and converts it
/// to an array of bytes
fn get_bytes_array_from_line(category:&str, data:&str) -> Vec<u16> {
    let data = remove_label(data);
    let mut bytes:Vec<u16> = Vec::new();
    match category {
        "int" => {
            let integer = data.split(" ").filter(|token| !token.is_empty()).collect::<Vec<&str>>()[1];
            bytes.push(get_int_immediate_from_string(integer).try_into().unwrap());
        },

        _ => panic!("Invalid or unsupported data type: {}", category)
    }

    bytes
}


/// Takes a line of assembly representing a data instruction and returns its token equivalent.
///
/// Assumes that the line has already been validated and line is an instruction and not blank.
pub fn generate_data_tokens(line:&str, prev_label:Option<String>) -> DataTokens {
    let label:Option<String> = match line.find(":") {
        Some(index) => Some(line[..index].to_owned()),
        None => {
            match prev_label.clone() {
                Some(l) => Some(l.to_string()),
                None => None
            }
        }
    };

    let category = &validate_data_type(line).unwrap()[1..];
    DataTokens::new(label.unwrap(), category.to_owned(), get_bytes_array_from_line(category, line))
}


/// Takes a string of an integer in binary, decimal, or hexadecimal and returns it. Assumes that the
/// input has already been validated.
fn get_int_immediate_from_string(immediate:&str) -> i64 {
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


/// Takes a line of assembly representing an instruction and generates a `InstrTokens` from it.
///
/// Assumes that the line has already been validated and line is an instruction and not blank.
pub fn generate_instr_tokens(line:&str, prev_label:Option<String>) -> InstrTokens {
    let label:Option<String> = match line.find(":") {
        Some(index) => Some(line[..index].to_owned()),
        None => {
            match prev_label.clone() {
                Some(l) => Some(l.to_string()),
                None => None
            }
        }
    };

    let opcode = validate_opcode(&line).unwrap();
    let operands:Vec<String> = get_operands_from_line(&line, opcode).iter()
                                                .map(|operand| operand.to_owned())
                                                .collect();

    match operands.len() {
        0 => InstrTokens::new(label, opcode.to_owned(), None, None, None, None, None),
        1 => InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), None, None, None, None),
        2 => {
            if !operands[1].starts_with("$") {
                InstrTokens::new(label.clone(), opcode.to_owned(), Some(operands[0].clone()), None, None, 
                                                Some(get_int_immediate_from_string(&operands[1])
                                                        .try_into().unwrap()), None);
            }

            InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), 
                                                Some(operands[1].clone()), None, None, None)
        },

        4 => InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), 
                                                Some(operands[1].clone()), 
                                                Some(operands[2].clone()), None, 
                                                Some(operands[3].clone())),
        3 => { // may or may not contain a label as an operand
            let mut tokens = InstrTokens::new(label.clone(), opcode.to_owned(), Some(operands[0].clone()), 
                                                Some(operands[1].clone()), Some(operands[2].clone()), 
                                                None, None);
            if operands[2].starts_with("@") {
                tokens = InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), 
                                                Some(operands[1].clone()), None, None, 
                                                Some(operands[2].clone()))
            } else if !operands[2].starts_with("$") {
                tokens = InstrTokens::new(label, opcode.to_owned(), Some(operands[0].clone()), 
                                                Some(operands[1].clone()), None, 
                                                Some(get_int_immediate_from_string(&operands[2])
                                                        .try_into().unwrap()), None)
            }
            
            tokens
        },
        _ => panic!("Invalid number of operands (validation module has failed!)"),
    }
}


#[cfg(test)]
mod tests {
    use crate::token_generator::*;


    #[test]
    fn test_token_generation_addi() {
        let tokens = generate_instr_tokens("init: ADDI $g0, $zero, 1", None);
        assert_eq!(tokens.label.as_ref().unwrap(), "init");
        assert_eq!(tokens.opcode, "ADDI");
        assert_eq!(tokens.operand_a.as_ref().unwrap(), "$g0");
        assert_eq!(tokens.operand_b.as_ref().unwrap(), "$zero");
        assert_eq!(tokens.operand_c.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens.op_label.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
    }


    #[test]
    fn test_instr_token_addi_all_bases() {
        let tokens_decimal = generate_instr_tokens("init: ADDI $g0, $zero, 1", None);
        assert_eq!(*tokens_decimal.immediate.as_ref().unwrap(), 1);

        let tokens_hex = generate_instr_tokens("init: ADDI $g0, $zero, 0b0010", None);
        assert_eq!(*tokens_hex.immediate.as_ref().unwrap(), 2);

        let tokens_binary = generate_instr_tokens("init: ADDI $g0, $zero, 0x0004", None);
        assert_eq!(*tokens_binary.immediate.as_ref().unwrap(), 4);
    }


    #[test]
    fn test_load_token_generation_with_label_opcode() {
        let tokens = generate_instr_tokens("LOAD $g5, $g8, $g9, @target", None);
        assert_eq!(tokens.label.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens.opcode, "LOAD");
        assert_eq!(tokens.operand_a.as_ref().unwrap(), "$g5");
        assert_eq!(tokens.operand_b.as_ref().unwrap(), "$g8");
        assert_eq!(tokens.operand_c.as_ref().unwrap(), "$g9");
        assert_eq!(tokens.op_label.as_ref().unwrap(), "@target");
    }


    #[test]
    fn test_jump_token_generation_with_label_opcode() {
        let tokens = generate_instr_tokens("JUMP $g8, $g9, @loop", None);
        assert_eq!(tokens.label.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens.opcode, "JUMP");
        assert_eq!(tokens.operand_a.as_ref().unwrap(), "$g8");
        assert_eq!(tokens.operand_b.as_ref().unwrap(), "$g9");
        assert_eq!(tokens.operand_c.as_ref().unwrap_or(&"none".to_owned()), &"none".to_owned());
        assert_eq!(tokens.op_label.as_ref().unwrap(), "@loop");
    }


    #[test]
    fn test_label_on_prev_line() {
        let tokens = generate_instr_tokens("JUMP $g8, $g9, @loop", Some("prev_label".to_owned())); 
        assert_eq!(tokens.label.as_ref().unwrap(), "prev_label"); 
    }


    #[test]
    fn test_data_token_int() {
        let tokens_decimal = generate_data_tokens("my_data: .int 50", None);
        assert_eq!(tokens_decimal.label, "my_data");
        assert_eq!(tokens_decimal.category, "int");
        assert_eq!(tokens_decimal.bytes[0], 50);
        assert_eq!(tokens_decimal.bytes.len(), 1);

        let tokens_hex = generate_data_tokens("my_data: .int 0b0101", None);
        assert_eq!(tokens_hex.label, "my_data");
        assert_eq!(tokens_hex.category, "int");
        assert_eq!(tokens_hex.bytes[0], 0b0101);
        assert_eq!(tokens_hex.bytes.len(), 1);

        let tokens_binary = generate_data_tokens("init: .int 0x001A", None);
        assert_eq!(tokens_binary.label, "init");
        assert_eq!(tokens_binary.category, "int");
        assert_eq!(tokens_binary.bytes[0], 0x001A);
        assert_eq!(tokens_binary.bytes.len(), 1);
    }
}
