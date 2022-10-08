use crate::errors::AsmValidationError;


/// Takes a line of assembly code, for example `ADD $g0, $zero, $g1`, and returns an `Err` if it is not valid Iridium assembly.
pub fn validate_asm_line(line:&str, data_mode:bool) -> Result<(), AsmValidationError> {
    validate_label(line)?;

    // validate if the line is not just a label
    if !line.ends_with(":") {
        if !data_mode {
            let opcode = validate_opcode(line)?;
            validate_operands(line, opcode)?;
        } else {
            let data_type = validate_data_type(line)?;
            validate_data_format(line, data_type)?;
        }
    }

    Ok(())
}


/// Takes a line of assembly and removes any label there may be
fn remove_label(line:&str) -> &str {
    match line.find(":") {
        Some(index) => {
            &line[index+1..].trim()
        },
        None => line,
    }
}


/// Takes a line of assembly and checks if it is a valid data instruction, such as .text or .float. Returns an `AsmValidationErr` if there is no valid data type, and 
/// returns the data type if there is.
fn validate_data_type(line:&str) -> Result<&str, AsmValidationError> {
    let valid_data_types:[&str;7] = [".int", ".long", ".half", ".float", ".section", ".char", ".text"];
    let data_type = remove_label(line).split(" ").collect::<Vec<&str>>()[0];
    if !valid_data_types.contains(&data_type) {
        return Err(AsmValidationError(format!("{} is not a valid data type on line {}", data_type, line)));
    }

    Ok(data_type)
}


/// Checks that a `Vec<&str>` has a certain number of items and returns an `AsmValidationError` if it does not
fn validate_token_vec(line:&str, vec:&Vec<&str>, req_len:usize) -> Result<(), AsmValidationError> {
    if vec.len() != req_len {
        return Err(AsmValidationError(format!("Incorrect format for tokenisation on line {}", line)));
    }

    Ok(())
}


/// Takes a line of assembly of a data instruction and its data type and checks that the data provided matches that data type
fn validate_data_format(line:&str, data_type:&str) -> Result<(), AsmValidationError> {
    println!("{}", remove_label(line));
    let tokens:Vec<&str> = remove_label(line).split(" ").collect();
    println!("Line: {},\tTokens: {:?}", line, tokens);
    match data_type {
        ".int" => { // label: .int <16-bit integer>
            validate_token_vec(line, &tokens, 2)?;
            validate_immediate(tokens[1], 16, true)?;
        },

        ".long" => { // label: .long <32-bit integer>
            validate_token_vec(line, &tokens, 2)?;
            validate_immediate(tokens[1], 32, true)?;
        },

        ".half" => { // label: .half <16-bit IEEE 754 float>

        },

        ".float" => { // label: .half <32-bit IEEE 754 float>

        },

        ".section" => { // label: .section [<bytes>]

        },

        ".char" => { // label: .char '<character>'

        },

        ".text" => { // label: .text "<string>"

        },

        _ => {
            return Err(AsmValidationError(format!("{} is not a valid data type on line {}", data_type, line)));
        }
    }

    Ok(())
}


/// Takes a line of assembly, extracts the opcode from it, and checks that it is a valid opcode. If an invalid opcode is found, an `AsmValidationError` will be thrown.
fn validate_opcode(line:&str) -> Result<&str, AsmValidationError> {
    let valid_opcodes:[&str;28] = [
        "ADD", "SUB", "ADDI", "SUBI", "SLL", "SRL", "SRA", "NAND", "OR", "ADDC", "SUBC",
        "LOAD", "STORE", "JUMP", "JAL", "CMP", "BEQ", "BNE", "BLT", "BGT", "NOP", "MOVUI",
        "IN", "OUT", "syscall", "HALT", "NOP", "MOVLI"
    ];

    // get the opcode and remove any label there may be 
    let opcode:&str = remove_label(line).split(" ").collect::<Vec<&str>>()[0];
    if !valid_opcodes.contains(&opcode) {
        return Err(AsmValidationError(format!("{} is not a valid opcode on line {}", opcode, line)));
    }

    Ok(opcode)
}


/// Gets operands from a string by removing the operand and any comments and labels, and then split it up using commas
fn get_operands_from_line<'a>(line:&'a str, opcode:&str) -> Vec<String> {    
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


/// Checks that a given immediate is a valid immediate and returns an `AsmValidationError` if not. Will ensure 
/// that immediate is within the range the given number of bits can handle, and is in a valid format given the 
/// prefix (0x for hexadecimal and 0b for binary, no prefix for decimal).
fn validate_immediate(operand:&str, bits:i16, signed:bool) -> Result<(), AsmValidationError> {
    let immediate:i64;
    if operand.starts_with("0b") {
        immediate = match i64::from_str_radix(&operand[2..], 2) {
            Ok(val) => val,
            Err(_) => {
                return Err(AsmValidationError(format!("Could not parse binary immediate {}", operand)));
            }
        }
    } else if operand.starts_with("0x") {
        immediate = match i64::from_str_radix(&operand[2..], 16) {
            Ok(val) => val,
            Err(_) => {
                return Err(AsmValidationError(format!("Could not parse hexadecimal immediate {}", operand)));
            }
        }
    } else {
        immediate = match operand.parse() {
            Ok(val) => val,
            Err(_) => {
                return Err(AsmValidationError(format!("Could not parse immediate {}", operand)));
            }
        }
    }

    let max_immediate = match signed {
        true => ((2_i64.pow(bits.try_into().unwrap())) / 2) - 1,
        false => (2_i64.pow(bits.try_into().unwrap())) - 1
    };

    let min_immediate = match signed {
        true => -((2_i64.pow(bits.try_into().unwrap())) / 2),
        false => 0
    };


    println!("{} < {} < {}", min_immediate, operand, max_immediate);


    if immediate < 0 && !signed {
        return Err(AsmValidationError(format!("Unsigned immediate operand {} cannot be negative", operand))); 
    } else if immediate > max_immediate || (immediate < min_immediate && signed) {
        return Err(AsmValidationError(format!("Immediate {} cannot fit into {} bits", operand, bits)));
    }

    Ok(())
}


/// Takes a line of assembly and the associated opcode (which should already be validated), and checks that the operands are valid
fn validate_operands(line:&str, opcode:&str) -> Result<(), AsmValidationError> {
    let operands = get_operands_from_line(line, opcode);
    match opcode {
        "ADD" | "SUB" | "NAND" | "OR" | "LOAD" | "STORE" => { // require 3 registers
            if operands.len() != 3 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_register(&operands[0])?;
            validate_register(&operands[1])?;
            validate_register(&operands[2])?;
        },

        "ADDI" | "SUBI" | "SLL" | "SRL" | "SRA" => { // require 2 registers and an immediate
            if operands.len() != 3 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_register(&operands[0])?;
            validate_register(&operands[1])?;
            validate_immediate(&operands[2], 4, false)?;
        },

        "ADDC" | "SUBC" | "JUMP" | "CMP" | "JAL" | "BEQ" | "BNE" | "BLT" | "BGT" | "IN" | "OUT" => { // require 2 registers
            if operands.len() != 2 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_register(&operands[0])?;
            validate_register(&operands[1])?;
        },

        "MOVUI" | "MOVLI" | "syscall" => { // requires a register and an 8-bit immediate
            if operands.len() != 2 {
                return Err(AsmValidationError(format!("Incorrect number of operands on line {}", line)));
            }

            validate_register(&operands[0])?;
            validate_immediate(&operands[1], 8, false)?;
        },

        "NOP" | "HALT" => { // no operands
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


/// Takes a line of assembly and checks if it contains a label and, if it does, checks that the label is valid - if not, the function will return an error.
fn validate_label(line:&str) -> Result<(), AsmValidationError> {
    let label = match line.find(":") {
        Some(index) => line[..index].to_owned(),
        None => return Ok(()),
    };

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


#[cfg(test)]
mod tests {
    use crate::validation::*;


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
    fn test_opcodes_with_label() {
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
        validate_label("adding_nums: ADD $r0, $r1, $r2").unwrap();
        validate_label("adding_nums:ADD $r0, $r1, $r2").unwrap();
        validate_label("addingNums: ADD $r0, $r1, $r2").unwrap();
        validate_label("x: ADD $r0, $r1, $r2").unwrap();
        validate_label("adding_nums123: ADD $r0, $r1, $r2").unwrap();
        validate_label("add1ng_num5: ADD $r0, $r1, $r2").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_invalid_label_chars() {
        validate_label("adding-nums: ADD $r0, $r1, $r2").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_invalid_label_start_with_num() {
        validate_label("123adding_nums: ADD $r0, $r1, $r2").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_blank_label() {
        validate_label(":ADD $r0, $r1, $r2").unwrap();
    }


    #[test]
    fn label_only_line() {
        validate_asm_line("label_line:", false).unwrap();
    }


    #[test]
    fn test_no_operand_instrs() {
        validate_asm_line("NOP", false).unwrap();
        validate_asm_line("HALT", false).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_invalid_no_operand_instr() {
        validate_asm_line("NOP $g0", false).unwrap();
    }

    #[test]
    #[should_panic]
    fn test_wrong_number_of_operands() {
        validate_asm_line("ADDC $g0, $g1, $g2", false).unwrap();
    }


    #[test]
    fn test_rrr_format_instrs() {
        validate_asm_line("my_label: ADD $g0, $zero, $g1", false).unwrap();
        validate_asm_line("SUB $g1,$g2,$g3", false).unwrap();
        validate_asm_line("NAND $g4, $g5, $g6", false).unwrap();
        validate_asm_line("OR $g4, $g5, $g6", false).unwrap();
        validate_asm_line("LOAD $g7, $g8, $g9", false).unwrap();
        validate_asm_line("STORE $ua, $sp, $ra", false).unwrap();
        validate_asm_line("ADD $fp, $pc, $g0", false).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_rrr_invalid_operand() {
        validate_asm_line("ADD $g0, $q5, $g1", false).unwrap();
    }


    #[test]
    fn test_rri_format_instrs() {
        validate_asm_line("ADDI $g0, $zero, 5", false).unwrap();
        validate_asm_line("SUBI $g0, $g1, 0x000A", false).unwrap();
        validate_asm_line("SLL $g0, $g1, 0b1101", false).unwrap();
        validate_asm_line("SRL $g2, $g3, 13", false).unwrap();
        validate_asm_line("SRA $g3, $g4, 0x0004", false).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_negative_immediate() {
        validate_asm_line("ADDI $g0, $g1, -5", false).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_too_large_immediate() {
        validate_asm_line("ADDI $g0, $g1, 0xFFFF", false).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_malformed_immediate() {
        validate_asm_line("ADDI $g0, $g1, 1q", false).unwrap();
    }


    #[test]
    fn test_rro_format_instrs() {
        validate_asm_line("ADDC $g0, $g1", false).unwrap();
        validate_asm_line("SUBC $g0, $g1", false).unwrap();
        validate_asm_line("JUMP $g0, $g1", false).unwrap();
        validate_asm_line("CMP $g0, $g1", false).unwrap();
        validate_asm_line("JAL $g0, $g1", false).unwrap();
        validate_asm_line("BEQ $g0, $g1", false).unwrap();
        validate_asm_line("BNE $g0, $g1", false).unwrap();
        validate_asm_line("BLT $g0, $g1", false).unwrap();
        validate_asm_line("BGT $g0, $g1", false).unwrap();
        validate_asm_line("IN $g0, $g1", false).unwrap();
        validate_asm_line("OUT $g0, $g1", false).unwrap();
    }


    #[test]
    fn test_ri_format_instrs() {
        validate_asm_line("MOVUI $g0, 200", false).unwrap();
        validate_asm_line("MOVLI $g0, 0b11001010", false).unwrap();
        validate_asm_line("syscall $g0, 254", false).unwrap();
    }


    #[test]
    fn test_int_data() {
        validate_asm_line("my_label: .int 40", true).unwrap();
        validate_asm_line("my_label: .int 0xFF", true).unwrap();
        validate_asm_line("my_label: .int -100", true).unwrap();
        validate_asm_line("my_label: .int 0b00111010", true).unwrap();
        validate_asm_line("my_label: .int 0", true).unwrap();
        validate_asm_line("my_label: .int 32767", true).unwrap();
        validate_asm_line("my_label: .int -32768", true).unwrap();
    }


    #[test]
    fn test_long_data() {
        validate_asm_line("my_label: .long 40", true).unwrap();
        validate_asm_line("my_label: .long 0xFF", true).unwrap();
        validate_asm_line("my_label: .long -100", true).unwrap();
        validate_asm_line("my_label: .long 0b00111010", true).unwrap();
        validate_asm_line("my_label: .long 0", true).unwrap();
        validate_asm_line("my_label: .long 2147483647", true).unwrap();
        validate_asm_line("my_label: .long -2147483648", true).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_int_data_too_small() {
        validate_asm_line("my_label: .int -32769", true).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_int_data_too_large() {
        validate_asm_line("my_label: .int 32768", true).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_long_data_too_small() {
        validate_asm_line("my_label: .int -2147483649", true).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_long_data_too_large() {
        validate_asm_line("my_label: .int 2147483648", true).unwrap();
    }
}
