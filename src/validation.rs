use crate::errors::AsmValidationError;


/// Takes a line of assembly code, for example `ADD $g0, $zero, $g1`, and returns an `Err` if it is not valid Iridium assembly.
pub fn validate_asm_line(line:&str) -> Result<(), AsmValidationError> {
    validate_label(line)?;

    // validate if the line is not just a label
    if !line.ends_with(":") {
        let opcode = validate_opcode(line)?;
        validate_operands(line, opcode)?;
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
    let opcode:&str = match line.find(":") {
        Some(index) => {
            let label_removed = &line[index+1..].trim();
            label_removed.split(" ").collect::<Vec<&str>>()[0]
        },
        None => line.split(" ").collect::<Vec<&str>>()[0],
    };

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
fn validate_immediate(operand:&str, bits:i16) -> Result<(), AsmValidationError> {
    let immediate:i32;
    if operand.starts_with("0b") {
        immediate = match i32::from_str_radix(&operand[2..], 2) {
            Ok(val) => val,
            Err(_) => {
                return Err(AsmValidationError(format!("Could not parse binary immediate {}", operand)));
            }
        }
    } else if operand.starts_with("0x") {
        immediate = match i32::from_str_radix(&operand[2..], 16) {
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

    let max_immediate = (2_i32.pow(bits.try_into().unwrap())) - 1;
    if immediate < 0 {
        return Err(AsmValidationError(format!("Immediate operand {} cannot be negative", operand))); 
    } else if immediate > max_immediate {
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
            validate_immediate(&operands[2], 4)?;
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
            validate_immediate(&operands[1], 8)?;
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
        validate_asm_line("label_line:").unwrap();
    }


    #[test]
    fn test_no_operand_instrs() {
        validate_asm_line("NOP").unwrap();
        validate_asm_line("HALT").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_invalid_no_operand_instr() {
        validate_asm_line("NOP $g0").unwrap();
    }

    #[test]
    #[should_panic]
    fn test_wrong_number_of_operands() {
        validate_asm_line("ADDC $g0, $g1, $g2").unwrap();
    }


    #[test]
    fn test_rrr_format_instrs() {
        validate_asm_line("my_label: ADD $g0, $zero, $g1").unwrap();
        validate_asm_line("SUB $g1,$g2,$g3").unwrap();
        validate_asm_line("NAND $g4, $g5, $g6").unwrap();
        validate_asm_line("OR $g4, $g5, $g6").unwrap();
        validate_asm_line("LOAD $g7, $g8, $g9").unwrap();
        validate_asm_line("STORE $ua, $sp, $ra").unwrap();
        validate_asm_line("ADD $fp, $pc, $g0").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_rrr_invalid_operand() {
        validate_asm_line("ADD $g0, $q5, $g1").unwrap();
    }


    #[test]
    fn test_rri_format_instrs() {
        validate_asm_line("ADDI $g0, $zero, 5").unwrap();
        validate_asm_line("SUBI $g0, $g1, 0x000A").unwrap();
        validate_asm_line("SLL $g0, $g1, 0b1101").unwrap();
        validate_asm_line("SRL $g2, $g3, 13").unwrap();
        validate_asm_line("SRA $g3, $g4, 0x0004").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_negative_immediate() {
        validate_asm_line("ADDI $g0, $g1, -5").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_too_large_immediate() {
        validate_asm_line("ADDI $g0, $g1, 0xFFFF").unwrap();
    }


    #[test]
    #[should_panic]
    fn test_malformed_immediate() {
        validate_asm_line("ADDI $g0, $g1, 1q").unwrap();
    }


    #[test]
    fn test_rro_format_instrs() {
        validate_asm_line("ADDC $g0, $g1").unwrap();
        validate_asm_line("SUBC $g0, $g1").unwrap();
        validate_asm_line("JUMP $g0, $g1").unwrap();
        validate_asm_line("CMP $g0, $g1").unwrap();
        validate_asm_line("JAL $g0, $g1").unwrap();
        validate_asm_line("BEQ $g0, $g1").unwrap();
        validate_asm_line("BNE $g0, $g1").unwrap();
        validate_asm_line("BLT $g0, $g1").unwrap();
        validate_asm_line("BGT $g0, $g1").unwrap();
        validate_asm_line("IN $g0, $g1").unwrap();
        validate_asm_line("OUT $g0, $g1").unwrap();
    }


    #[test]
    fn test_ri_format_instrs() {
        validate_asm_line("MOVUI $g0, 200").unwrap();
        validate_asm_line("MOVLI $g0, 0b11001010").unwrap();
        validate_asm_line("syscall $g0, 254").unwrap();
    }
}
