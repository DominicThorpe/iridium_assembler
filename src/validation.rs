use std::str;
use crate::errors::AsmValidationError;


/// Takes a line of assembly code, for example `ADD $g0, $zero, $g1`, and returns an `Err` if it is not 
/// valid Iridium assembly.
pub fn validate_asm_line(line:&str, mode:char) -> Result<(), AsmValidationError> {
    validate_line_label(line)?;
    if line.ends_with(":") {
        return Ok(());
    }

    // validate if the line is not just a label
    if mode == 'c' { // if in the code section
        let opcode = match validate_opcode(line) {
            Ok(val) => val,
            Err(e) => {
                match validate_data_type(line, mode) {
                    Ok(_) => {
                        return Err(AsmValidationError(format!("{} is for data, but is in the instructions section, which is invalid", line)));
                    },

                    Err(_) => {
                        return Err(e);
                    }
                }
            }
        };

        validate_operands(line, opcode)?;
        return Ok(());
    } 
    
    let data_type = match validate_data_type(line, mode) {
        Ok(val) => val,
        Err(e) => {
            match validate_opcode(line) {
                Ok(_) => {
                    return Err(AsmValidationError(format!("{} is an instruction, but is in the data section, which is invalid", line)));
                },

                Err(_) => {
                    return Err(e);
                }
            }
        }
    };

    validate_data_format(line, data_type)?;
    Ok(())
}


/// Takes a line of assembly and removes any label there may be
pub fn remove_label(line:&str) -> &str {
    match line.find(":") {
        Some(index) => {
            &line[index+1..].trim()
        },
        None => line,
    }
}


/// Takes a line of assembly and checks if it is a valid data instruction, such as .text or .float. Returns 
/// an `AsmValidationErr` if there is no valid data type, and returns the data type if there is.
pub fn validate_data_type(line:&str, mode:char) -> Result<&str, AsmValidationError> {
    let valid_data_types:[&str;7] = [".int", ".long", ".half", ".float", ".section", ".char", ".text"];
    let data_type = remove_label(line).split(" ").collect::<Vec<&str>>()[0];
    if !valid_data_types.contains(&data_type) {
        return Err(AsmValidationError(format!("{} is not a valid data type on line {}", data_type, line)));
    }

    if mode == 't' && data_type != ".text" {
        return Err(AsmValidationError(format!("{} is not text, yet is in the text section", line)));
    } else if mode != 't' && data_type == ".text" {
        return Err(AsmValidationError(format!("{} is text, yet is not in the text section", line)));
    }

    Ok(data_type)
}


/// Checks that a `Vec<&str>` has a certain number of items and returns an `AsmValidationError` if it 
/// does not.
fn validate_token_vec(line:&str, vec:&Vec<&str>, req_len:usize) -> Result<(), AsmValidationError> {
    if vec.len() != req_len {
        return Err(AsmValidationError(format!("Incorrect format for tokenisation on line {}", line)));
    }

    Ok(())
}


/// Takes an immediate in floating point format and checks if it can fit into an IEEE 754 floating point 
/// format with the given parameters, either half or regular format. Will return an `AsmValidationError` 
/// if the immediate is invalid.
fn validate_float_immediate(line:&str, immediate:&str, short:bool) -> Result<(), AsmValidationError> {
    match immediate.parse::<f32>() {
        Ok(val) => {
            if short {
                let min_max_value = 4_293_918_720.0;
                if val > min_max_value || val < -min_max_value {
                    return Err(AsmValidationError(format!(
                        "{} cannot fit into a 16-bit IEEE 754 format number on line {}", immediate, line
                    ))); 
                }
            } else {
                let min_max_value:f32 = f32::MAX;
                if val > min_max_value || val < -min_max_value {
                    return Err(AsmValidationError(format!(
                        "{} cannot fit into a 32-bit IEEE 754 format number on line {}", immediate, line
                    ))); 
                }
            }
        },

        Err(_) => {
            return Err(AsmValidationError(format!("{} is not a valid immediate on line {}", immediate, line)));
        }
    };

    Ok(())
}


/// Takes a character immediate in the format `'<char>'` and checks that it is a valid UTF-8 character in 
/// that format. If not, an `AsmValidationError` is returned.
fn validate_char_immediate(line:&str, immediate:&str) -> Result<(), AsmValidationError> {
    if !immediate.starts_with("'") || !immediate.ends_with("'") {
        return Err(AsmValidationError(format!(
            "Immediate {} on line \"{}\" is not in a valid format - should be label: .char '<char>'", 
            immediate, line
        )));
    }

    let imm_char:&str = &immediate[1..immediate.len() - 1];
    if imm_char.chars().collect::<Vec<char>>().len() != 1 {
        return Err(AsmValidationError(format!(
            "Immediate {} on line \"{}\" is not in a valid format - more than 1 character found", 
            immediate, line
        )));
    }

    Ok(())
}


/// Takes a line of assembly containing a character data instruction in the form <label>: .char '<char>' 
/// and returns `Ok(())` if it is valid, and `AsmValidationError` if it is not.
fn validate_char_instr(line:&str) -> Result<(), AsmValidationError> {
    let mut instr = remove_label(line).trim();
    if !instr.starts_with(".char") {
        return Err(AsmValidationError(format!("{} is not a valid character data instruction", line)));
    }

    // checks that the character immediate format is '<character>'
    instr = &instr[5..].trim();
    if !(instr.starts_with("'") && instr.ends_with("'")) {
        return Err(AsmValidationError(format!("{} is not a valid character data instruction", line)));
    }

    match validate_char_immediate(line, instr) {
        Ok(_) => Ok(()),
        Err(e) => {
            let character = &instr[1..instr.len() - 1];
            if character == "\t" || character == "\n" || character == "\0" || character == "\r" {
               return Ok(());
            }

            Err(e)
        },
    }
}


/// Takes a line of assembly for a data instruction that should have a specified length, such as `.text`
/// or `.section`, anc checks that it does. Returns the array size if valid, and an `AsmValidationError`
/// if not.
///
/// ASSUMES LABEL HAS ALREADY BEEN REMOVED!
fn get_valid_array_size(line:&str) -> Result<i64, AsmValidationError> {
    let tokens:Vec<&str> = line.split(" ").collect();
    match i64::from_str_radix(tokens[1].trim(), 10) {
        Ok(val) => Ok(val),
        Err(_) => {
            Err(AsmValidationError(format!(
                "{} is not a valid size for the array on line {}", tokens[1].trim(), line
            )))
        }
    }
}


/// Takes a line of assembly containing a .text data instruction and determines if it is valid or not,
/// will return an `AsmValidationError` if not.
fn validate_text_instr(line:&str) -> Result<(), AsmValidationError> {
    let instr = remove_label(line);
    let array_size = get_valid_array_size(instr)?;

    let text_start_index = match instr.find("\"") {
        Some(index) => index,
        None => {
            return Err(AsmValidationError(format!(
                "{} is not a correctly formatted .text data instruction - have you used double quotes?", 
                line
            )));
        }
    };
    
    if !instr.ends_with("\"") {
        return Err(AsmValidationError(format!(
            "{} is not a correctly formatted .text data instruction - have you used double quotes?", line
        )));
    }

    let text = &instr[text_start_index..];
    match str::from_utf8(instr.as_bytes()) {
        Ok(_) => {},
        Err(_) => {
            return Err(AsmValidationError(format!(
                "Text {} on line \"{}\" is not valid UTF-8", text, line
            )));
        }
    };

    let str_len = text.chars().collect::<Vec<char>>().len() - 1;
    if str_len > array_size.try_into().unwrap() {
        return Err(AsmValidationError(format!(
            "Text is too long for {} bytes on line {}. Have you taken the null terminator into account?",
            array_size, line
        )));
    }

    Ok(())
}


/// Takes a line of assembly for a bytes section and checks that it is formatted properly. Will return
/// an `AsmValidationError` if not.
fn validate_bytes_section_instr(line:&str) -> Result<(), AsmValidationError> {
    let instr = remove_label(line);
    let array_size = get_valid_array_size(instr)?;

    let array_start_index = match instr.find("[") {
        Some(index) => index,
        None => {
            return Err(AsmValidationError(format!(
                "{} is not a properly formatted array, which requires square brackets []", instr
            )));
        }
    };

    if !instr.ends_with("]") {
        return Err(AsmValidationError(format!(
            "{} is not a properly formatted array, which requires square brackets []", instr
        ))); 
    }

    let array_contents_str = &instr[array_start_index + 1..instr.len() - 1];
    let array_contents:Vec<&str> = array_contents_str.split(",")
                                        .map(|item| item.trim())
                                        .filter(|item| item != &"")
                                        .collect();
    for item in &array_contents {
        validate_int_immediate(item, 16, true)?;
    }

    if array_contents.len() > array_size.try_into().unwrap() {
        return Err(AsmValidationError(format!(
            "Bytes array is too long for section of length {} on line {}.", array_size, line
        )));
    }

    Ok(())    
}


/// Takes a line of assembly of a data instruction and its data type and checks that the data provided 
/// matches that data type
fn validate_data_format(line:&str, data_type:&str) -> Result<(), AsmValidationError> {
    let tokens:Vec<&str> = remove_label(line).split(" ").collect();
    match data_type {
        ".int" => { // label: .int <16-bit integer>
            validate_token_vec(line, &tokens, 2)?;
            validate_int_immediate(tokens[1], 16, true)?;
        },

        ".long" => { // label: .long <32-bit integer>
            validate_token_vec(line, &tokens, 2)?;
            validate_int_immediate(tokens[1], 32, true)?;
        },

        ".half" => { // label: .half <16-bit IEEE 754 float>
            validate_token_vec(line, &tokens, 2)?;
            validate_float_immediate(line, tokens[1], true)?;
        },

        ".float" => { // label: .half <32-bit IEEE 754 float>
            validate_token_vec(line, &tokens, 2)?;
            validate_float_immediate(line, tokens[1], false)?;
        },

        ".section" => { // label: .section [<bytes>]
            validate_bytes_section_instr(line)?;
        },

        ".char" => { // label: .char '<character>'
            validate_char_instr(line)?;
        },

        ".text" => { // label: .text "<string>"
            validate_text_instr(line)?;
        },

        _ => {
            return Err(AsmValidationError(format!("{} is not a valid data type on line {}", data_type, line)));
        }
    }

    Ok(())
}


/// Takes a line of assembly, extracts the opcode from it, and checks that it is a valid opcode. If an 
/// invalid opcode is found, an `AsmValidationError` will be thrown.
pub fn validate_opcode(line:&str) -> Result<&str, AsmValidationError> {
    let valid_opcodes:[&str;28] = [
        "ADD", "SUB", "ADDI", "SUBI", "SLL", "SRL", "SRA", "NAND", "OR", "ADDC", "SUBC",
        "LOAD", "STORE", "JUMP", "JAL", "CMP", "BEQ", "BNE", "BLT", "BGT", "NOP", "MOVUI",
        "IN", "OUT", "syscall", "HALT", "MOVLI", "ATOM"
    ];

    // get the opcode and remove any label there may be
    let opcode:&str = remove_label(line).split(" ").filter(|item| *item != "").collect::<Vec<&str>>()[0];
    if !valid_opcodes.contains(&opcode) {
        return Err(AsmValidationError(format!("{} is not a valid opcode on line {}", opcode, line)));
    }

    Ok(opcode)
}


/// Gets operands from a string by removing the operand and any comments and labels, and then split it up 
/// using commas
pub fn get_operands_from_line<'a>(line:&'a str, opcode:&str) -> Vec<String> {    
    let opcode_start_index = line.find(opcode).expect(&format!("Could not find opcode {} in line {}", opcode, line));
    let opcode_end_index = opcode_start_index + opcode.len();
    let comment_start_index = line.find(";").unwrap_or(line.len());

    let operands_section = line[opcode_end_index..comment_start_index].to_owned();
    let operands:Vec<String> = operands_section.split(",")
                                    .map(|operand| operand.trim().to_owned())
                                    .filter(|operand| operand != "")
                                    .collect();

    operands
}


/// Checks that a given register string is a valid register and returns an `AsmValidationError` if not
fn validate_register(register:&str) -> Result<(), AsmValidationError> {
    let valid_registers:[&str;16] = [
        "$zero", "$g0", "$g1", "$g2", "$g3", "$g4", "$g5", "$g6", "$g7", "$g8", "$g9",
        "$ua", "$sp", "$ra", "$fp", "$pc"
    ];

    if !valid_registers.contains(&register) {
        return Err(AsmValidationError(format!("{} is not a valid register", register)));
    }

    Ok(())
}


/// Checks that a given immediate is a valid immediate and returns it or an `AsmValidationError` if not. 
/// Will ensure that immediate is within the range the given number of bits can handle, and is in a valid 
/// format given the prefix (0x for hexadecimal and 0b for binary, no prefix for decimal).
fn validate_int_immediate(operand:&str, bits:i16, signed:bool) -> Result<i64, AsmValidationError> {
    let immediate:i64;
    let decimal:bool;
    if operand.starts_with("0b") {
        immediate = match i64::from_str_radix(&operand[2..], 2) {
            Ok(val) => val,
            Err(_) => {
                return Err(AsmValidationError(format!("Could not parse binary immediate {}", operand)));
            }
        };

        decimal = false;
    } else if operand.starts_with("0x") {
        immediate = match i64::from_str_radix(&operand[2..], 16) {
            Ok(val) => val,
            Err(_) => {
                return Err(AsmValidationError(format!("Could not parse hexadecimal immediate {}", operand)));
            }
        };

        decimal = false;
    } else {
        immediate = match operand.parse() {
            Ok(val) => val,
            Err(_) => {
                return Err(AsmValidationError(format!("Could not parse immediate {}", operand)));
            }
        };

        decimal = true;
    }

    let max_immediate:i64;
    let min_immediate:i64;
    if signed && decimal {
        max_immediate = ((2_i64.pow(bits.try_into().unwrap())) / 2) - 1;
        min_immediate = -((2_i64.pow(bits.try_into().unwrap())) / 2);
    } else {
        max_immediate = (2_i64.pow(bits.try_into().unwrap())) - 1;
        min_immediate = 0;
    }

    if immediate < 0 && !(signed && decimal) {
        return Err(AsmValidationError(format!("Unsigned immediate operand {} cannot be negative", operand))); 
    } else if immediate > max_immediate || (immediate < min_immediate && signed) {
        return Err(AsmValidationError(format!("Immediate {} cannot fit into {} bits", operand, bits)));
    }

    Ok(immediate)
}


/// Takes an operand from an instruction and verifies that it is a valid label operand in the form
/// @<operand> where operand contains only alphanumeric characters and underscores, and does not
/// start with a number. 
///
/// Returns an `AsmValidationError` if the label operand is invalid.
fn validate_label_operand(line:&str, operand:&str) -> Result<(), AsmValidationError> {
    if !operand.starts_with("@") {
        return Err(AsmValidationError(format!(
            "{} on line {} is not a valid operand as it does not start with an '@' symbol", line, operand
        )));
    }

    validate_operand_label(line, operand)?;

    Ok(())
}


/// Takes a line of assembly and the associated opcode (which should already be validated), and checks 
/// that the operands are valid
fn validate_operands(line:&str, opcode:&str) -> Result<(), AsmValidationError> {
    let operands = get_operands_from_line(line, opcode);
    match opcode {
        "ADD" | "SUB" | "NAND" | "OR" => { // require 3 registers
            if operands.len() != 3 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_register(&operands[0])?;
            validate_register(&operands[1])?;
            validate_register(&operands[2])?;
        },

        "LOAD" | "STORE" => { // requires 3 registers, optional label operand
            if operands.len() != 3 && operands.len() != 4 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_register(&operands[0])?;
            validate_register(&operands[1])?;
            validate_register(&operands[2])?;

            if operands.len() == 4 {
                validate_label_operand(line, &operands[3])?;
            }
        },

        "ADDI" | "SUBI" | "SLL" | "SRL" | "SRA" => { // require 2 registers and an immediate
            if operands.len() != 3 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_register(&operands[0])?;
            validate_register(&operands[1])?;
            validate_int_immediate(&operands[2], 4, false)?;
        },

        "ADDC" | "SUBC" | "CMP" | "IN" | "OUT" => { // require 2 registers
            if operands.len() != 2 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_register(&operands[0])?;
            validate_register(&operands[1])?;
        },

        "JUMP" | "JAL" | "BEQ" | "BNE" | "BLT" | "BGT" => {
            match operands.len() {
                1 => {
                    validate_register(&operands[0])?;
                    if operands[0] != "$sp" && operands[0] != "$fp" && operands[0] != "$ra" && operands[0] != "$pc" {
                        return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
                    }
                },

                2 => {
                    validate_register(&operands[0])?;
                    validate_register(&operands[1])?;
                },

                3 => {
                    validate_register(&operands[0])?;
                    validate_register(&operands[1])?;
                    validate_label_operand(line, &operands[2])?;
                },

                _ => {
                    return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
                }
            }
        }

        "MOVUI" | "MOVLI" => {
            if operands.len() != 2 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_register(&operands[0])?;
            if operands[1].starts_with("@") {
                validate_label_operand(line, &operands[1])?;
            } else {
                validate_int_immediate(&operands[1], 8, false)?;
            }
        }
        
        "syscall" => { // requires only an 8-bit immediate
            if operands.len() != 1 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_int_immediate(&operands[0], 8, false)?;
        },

        "NOP" | "ATOM" | "HALT" => { // no operands
            if operands.is_empty() {
                return Ok(());
            } else {
                return Err(AsmValidationError(format!("Instruction {} takes no arguments", line)));
            }
        },

        _ => {
            return Err(AsmValidationError(format!("Invalid opcode: {} on line {}", opcode, line)));
        }
    }

    Ok(())
}


/// Takes a label and checks that it meets all the requirements, giving an `AsmValidationError` if not.
/// The requirements for a valid label are:
///  - Alphanumeric characters and '_' only
///  - No digits 0-9 as the first character 
fn validate_label(line:&str, label:&str) -> Result<(), AsmValidationError> {
    if label.chars().collect::<Vec<char>>()[0].is_numeric() {
        return Err(AsmValidationError(format!(
            "The label {} on the line {} is not valid - labels may not start with numeric characters.", label, line)
        ));
    }

    if !label.chars().all(|c| c.is_alphanumeric() || c == '_') {
        return Err(AsmValidationError(format!(
            "The label {} on the line {} is not valid - labels may only contain alphanumeric characters or _.", label, line)
        ));
    }

    Ok(())
}


/// Takes a line of assembly and checks if it contains a label and, if it does, checks that the label is 
/// valid - if not, the function will return an error.
fn validate_line_label(line:&str) -> Result<(), AsmValidationError> {
    match line.find(":") {
        Some(index) => validate_label(line, &line[..index])?,
        None => return Ok(()),
    };

    Ok(())
}


/// Takes a label operand and checks that it is valid; if not, it will output an `AsmValidationError`.
fn validate_operand_label(line:&str, label:&str) -> Result<(), AsmValidationError> {
    if !label.starts_with("@") {
        return Err(AsmValidationError(format!("Label operand {} on line {} must start with an '@' symbol", label, line)));
    }

    validate_label(line, &label[1..])?;

    Ok(())
} 


#[cfg(test)]
mod tests {
    use crate::validation::*;


    #[test]
    fn test_label_only_line() {
        validate_asm_line("my_label1:", 'c').unwrap();
        validate_asm_line("my_label1:", 'd').unwrap();
    }


    #[test]
    fn test_valid_opcodes() {
        validate_opcode("ADD $r0, $r1, $r2").unwrap();
        validate_opcode("SUB $r0, $r1, $r2").unwrap();
        validate_opcode("BEQ $r0").unwrap();
        validate_opcode("MOVUI $r0, 600").unwrap();
        validate_opcode("syscall").unwrap();
        validate_opcode("STORE $r0, $r1, $r2").unwrap();
    }


    #[test]
    fn test_opcodes_with_line_label() {
        validate_opcode("adding_nums: ADD $r0, $r1, $r2").unwrap();
        validate_opcode("noSpace:ADD $r0, $r1, $r2").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_invalid_opcode() {
        validate_opcode("ADDUII $r0, 600").unwrap();
    }


    #[test]
    fn test_valid_label() {
        validate_line_label("adding_nums: ADD $r0, $r1, $r2").unwrap();
        validate_line_label("adding_nums:ADD $r0, $r1, $r2").unwrap();
        validate_line_label("addingNums: ADD $r0, $r1, $r2").unwrap();
        validate_line_label("x: ADD $r0, $r1, $r2").unwrap();
        validate_line_label("adding_nums123: ADD $r0, $r1, $r2").unwrap();
        validate_line_label("add1ng_num5: ADD $r0, $r1, $r2").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_invalid_label_chars() {
        validate_line_label("adding-nums: ADD $r0, $r1, $r2").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_invalid_label_start_with_num() {
        validate_line_label("123adding_nums: ADD $r0, $r1, $r2").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_blank_label() {
        validate_line_label(":ADD $r0, $r1, $r2").unwrap();
    }


    #[test]
    fn label_only_line() {
        validate_asm_line("label_line:", 'c').unwrap();
    }


    #[test]
    fn test_no_operand_instrs() {
        validate_asm_line("NOP", 'c').unwrap();
        validate_asm_line("HALT", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_invalid_no_operand_instr() {
        validate_asm_line("NOP $g0", 'c').unwrap();
    }

    #[test]
    #[should_panic]
    fn test_wrong_number_of_operands() {
        validate_asm_line("ADDC $g0, $g1, $g2", 'c').unwrap();
    }


    #[test]
    fn test_rrr_format_instrs() {
        validate_asm_line("my_label: ADD $g0, $zero, $g1", 'c').unwrap();
        validate_asm_line("SUB $g1,$g2,$g3", 'c').unwrap();
        validate_asm_line("NAND $g4, $g5, $g6", 'c').unwrap();
        validate_asm_line("OR $g4, $g5, $g6", 'c').unwrap();
        validate_asm_line("LOAD $g7, $g8, $g9", 'c').unwrap();
        validate_asm_line("STORE $ua, $sp, $ra", 'c').unwrap();
        validate_asm_line("ADD $fp, $pc, $g0", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_rrr_invalid_operand() {
        validate_asm_line("ADD $g0, $q5, $g1", 'c').unwrap();
    }


    #[test]
    fn test_rri_format_instrs() {
        validate_asm_line("ADDI $g0, $zero, 5", 'c').unwrap();
        validate_asm_line("SUBI $g0, $g1, 0x000A", 'c').unwrap();
        validate_asm_line("SLL $g0, $g1, 0b1101", 'c').unwrap();
        validate_asm_line("SRL $g2, $g3, 13", 'c').unwrap();
        validate_asm_line("SRA $g3, $g4, 0x0004", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_negative_immediate() {
        validate_asm_line("ADDI $g0, $g1, -5", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_too_large_immediate() {
        validate_asm_line("ADDI $g0, $g1, 0xFFFF", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_malformed_immediate() {
        validate_asm_line("ADDI $g0, $g1, 1q", 'c').unwrap();
    }


    #[test]
    fn test_rro_format_instrs() {
        validate_asm_line("ADDC $g0, $g1", 'c').unwrap();
        validate_asm_line("SUBC $g0, $g1", 'c').unwrap();
        validate_asm_line("JUMP $g0, $g1", 'c').unwrap();
        validate_asm_line("CMP $g0, $g1", 'c').unwrap();
        validate_asm_line("JAL $g0, $g1", 'c').unwrap();
        validate_asm_line("BEQ $g0, $g1", 'c').unwrap();
        validate_asm_line("BNE $g0, $g1", 'c').unwrap();
        validate_asm_line("BLT $g0, $g1", 'c').unwrap();
        validate_asm_line("BGT $g0, $g1", 'c').unwrap();
        validate_asm_line("IN $g0, $g1", 'c').unwrap();
        validate_asm_line("OUT $g0, $g1", 'c').unwrap();
    }


    #[test]
    fn test_orr_format_instrs_one_register() {
        validate_asm_line("JUMP $sp", 'c').unwrap();
        validate_asm_line("JAL  $sp", 'c').unwrap();
        validate_asm_line("BEQ  $ra", 'c').unwrap();
        validate_asm_line("BNE  $pc", 'c').unwrap();
        validate_asm_line("BLT  $ra", 'c').unwrap();
        validate_asm_line("BGT  $ra", 'c').unwrap();
    }

    #[test]
    #[should_panic]
    fn test_orr_format_instrs_one_register_16_bits() {
        validate_asm_line("JUMP $g0", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_orr_format_instrs_one_register_zero() {
        validate_asm_line("JUMP $zero", 'c').unwrap();
    }


    #[test]
    fn test_ri_format_instrs() {
        validate_asm_line("MOVUI $g0, 200", 'c').unwrap();
        validate_asm_line("MOVLI $g0, 0b11001010", 'c').unwrap();
        validate_asm_line("syscall 254", 'c').unwrap();
    }

    #[test]
    #[should_panic]
    fn test_syscall_with_register_operand() {
        validate_asm_line("syscall $g0, 254", 'c').unwrap();
    }


    #[test]
    fn test_int_data() {
        validate_asm_line("my_label: .int 40", 'd').unwrap();
        validate_asm_line("my_label: .int 0xFF", 'd').unwrap();
        validate_asm_line("my_label: .int -100", 'd').unwrap();
        validate_asm_line("my_label: .int 0b00111010", 'd').unwrap();
        validate_asm_line("my_label: .int 0", 'd').unwrap();
        validate_asm_line("my_label: .int 32767", 'd').unwrap();
        validate_asm_line("my_label: .int -32768", 'd').unwrap();
    }


    #[test]
    fn test_long_data() {
        validate_asm_line("my_label: .long 40", 'd').unwrap();
        validate_asm_line("my_label: .long 0xFF", 'd').unwrap();
        validate_asm_line("my_label: .long -100", 'd').unwrap();
        validate_asm_line("my_label: .long 0b00111010", 'd').unwrap();
        validate_asm_line("my_label: .long 0", 'd').unwrap();
        validate_asm_line("my_label: .long 2147483647", 'd').unwrap();
        validate_asm_line("my_label: .long -2147483648", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_int_data_too_small() {
        validate_asm_line("my_label: .int -32769", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_int_data_too_large() {
        validate_asm_line("my_label: .int 32768", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_long_data_too_small() {
        validate_asm_line("my_label: .int -2147483649", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_long_data_too_large() {
        validate_asm_line("my_label: .int 2147483648", 'd').unwrap();
    }


    #[test]
    fn test_floating_point_half_data() {
        validate_asm_line("my_label:.half 0", 'd').unwrap();
        validate_asm_line("my_label: .half 0.001", 'd').unwrap();
        validate_asm_line("my_label: .half 5.25", 'd').unwrap();
        validate_asm_line("my_label: .half -5.25", 'd').unwrap();
        validate_asm_line("my_label: .half -4293918721", 'd').unwrap();
        validate_asm_line("my_label: .half 4293918721", 'd').unwrap();
    }


    #[test]
    fn test_floating_point_full_data() {
        validate_asm_line("my_label:.float 0", 'd').unwrap();
        validate_asm_line("my_label: .float 0.001", 'd').unwrap();
        validate_asm_line("my_label: .float 5.25", 'd').unwrap();
        validate_asm_line("my_label: .float -5.25", 'd').unwrap();
        validate_asm_line(&format!("my_label: .float {}", -f32::MAX), 'd').unwrap();
        validate_asm_line(&format!("my_label: .float {}", f32::MAX), 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_half_float_data_too_small() {
        validate_asm_line("my_label: .int -4293918722", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_half_float_data_too_large() {
        validate_asm_line("my_label: .int 4293918722", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_full_float_data_too_small() {
        let min:f64 = f32::MIN.into();
        validate_asm_line(&format!("my_label: .float {}", min * 2.0), 'd').unwrap(); // multiply to take into account underflow
    }


    #[test]
    #[should_panic]
    fn test_full_float_data_too_large() {
        let max:f64 = f32::MAX.into();
        validate_asm_line(&format!("my_label: .float {}", max * 2.0), 'd').unwrap(); // multiply to take into account underflow
    }


    #[test]
    fn test_character_data() {
        validate_asm_line("my_label: .char 'a'", 'd').unwrap();
        validate_asm_line("my_label: .char 'b'", 'd').unwrap();
        validate_asm_line("my_label: .char '.'", 'd').unwrap();
        validate_asm_line("my_label: .char ' '", 'd').unwrap();
        validate_asm_line("my_label: .char '你'", 'd').unwrap();
        validate_asm_line("my_label: .char '\t'", 'd').unwrap();
        validate_asm_line("my_label: .char '\n'", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_string_in_char_data() {
        validate_asm_line("my_label: .char 'hi'", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_wrong_quotes_char_data() {
        validate_asm_line("my_label: .char \"h\"", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_empty_quotes_char_data() {
        validate_asm_line("my_label: .char ''", 'd').unwrap();
    }


    #[test]
    fn test_valid_text() {
        validate_asm_line("my_text: .text 13 \"Hello world!\"", 't').unwrap();
        validate_asm_line("my_text: .text 8 \"你好我很高兴!\"", 't').unwrap();
        validate_asm_line("empty_text: .text 1 \"\"", 't').unwrap();
        validate_asm_line("multiline:.text 50 \"My longer\nparagraph of some\rgood text\"", 't').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_too_short_text() {
        validate_asm_line("my_text: .text 5 \"This is too  long for the array\"", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_no_length_text() {
        validate_asm_line("my_text: .text \"Hello world!\"", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_invalid_quotes_text() {
        validate_asm_line("my_text: .text 10 'hello'", 'd').unwrap();
    }


    #[test]
    fn test_valid_bytes_section() {
        validate_asm_line("my_label: .section 4 [0xFFFF, 0x1234, 0xAAAA, 0x1212]", 'd').unwrap();
        validate_asm_line("empty: .section 0 []", 'd').unwrap();
        validate_asm_line("my_label: .section 10 [0xFFFF, 0x1234, 0xAAAA, 0x1212]", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_too_small_bytes_section() {
        validate_asm_line("my_label: .section 3 [0xFFFF, 0x1234, 0xAAAA, 0x1212]", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_wrong_brackets_bytes_section() {
        validate_asm_line("my_label: .section 4 (0xFFFF, 0x1234, 0xAAAA, 0x1212)", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_no_size_bytes_section() {
        validate_asm_line("my_label: .section [0xFFFF, 0x1234, 0xAAAA, 0x1212]", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_bytes_section_item_too_large() {
        validate_asm_line("my_label: .section 4 [0xFFFFF, 0x1234, 0xAAAA, 0x1212]", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_bytes_section_invalid_item() {
        validate_asm_line("my_label: .section 4 [0xFFFF, 0x1234, 'a', 0x1212]", 'd').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_instr_in_data_section() {
        validate_asm_line("my_label: .long 0xFFFFFF", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_data_in_instrs_section() {
        validate_asm_line("my_label: ADD $g0, $g1, $g2", 'd').unwrap();
    }


    #[test]
    fn test_opcodes_with_jump_label() {
        validate_asm_line("JUMP $g0, $g1, @jump_label", 'c').unwrap();
        validate_asm_line("JAL $g0, $g1, @jal_label", 'c').unwrap();
        validate_asm_line("BEQ $g0, $g1, @beq_label", 'c').unwrap();
        validate_asm_line("BNE $g0, $g1, @bne_label", 'c').unwrap();
        validate_asm_line("BLT $g0, $g1, @blt_label", 'c').unwrap();
        validate_asm_line("BGT $g0, $g1, @bgt_label", 'c').unwrap();
        validate_asm_line("LOAD $g0, $g1, $g2, @load_label", 'c').unwrap();
        validate_asm_line("STORE $g0, $g1, $g2, @store_label", 'c').unwrap();
        validate_asm_line("MOVUI $g0, @movui_label", 'c').unwrap();
        validate_asm_line("MOVLI $g0, @movli_label", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_movli_with_invalid_label() {
        validate_asm_line("ADD $g0, $g1, $g2, jump_label", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_non_jump_with_jump_label() {
        validate_asm_line("ADD $g0, $g1, $g2, @jump_label", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_jump_with_invalid_jump_label() {
        validate_asm_line("JUMP $g0, $g1, jump_label", 'c').unwrap();
    }


    #[test]
    #[should_panic]
    fn test_jump_with_invalid_jump_label_char() {
        validate_asm_line("JUMP $g0, $g1, @jump~label", 'c').unwrap();
    }


    #[test]
    fn test_atom_opcode() {
        validate_asm_line("my_label: ATOM", 'c').unwrap();
    }
}
