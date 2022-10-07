use crate::errors::AsmValidationError;


// Takes a line of assembly code, for example `ADD $g0, $zero, $g1`, and returns an `Err` if it
// is not valid Iridium assembly.
pub fn validate_asm_line(line:&str) -> Result<(), AsmValidationError> {
    validate_label(line)?;

    // validate if the line is not just a label
    if !line.ends_with(":") {
        let opcode = validate_opcode(line)?;
        validate_operands(line, opcode)?;
    }

    Ok(())
}


// Takes a line of assembly, extracts the opcode from it, and checks that it is a valid opcode. If an invalid opcode is found, an `AsmValidationError` will be thrown.
fn validate_opcode(line:&str) -> Result<&str, AsmValidationError> {
    let valid_opcodes:[&str;27] = [
        "ADD", "SUB", "ADDI", "SUBI", "SLL", "SRL", "SRA", "NAND", "OR", "ADDC", "SUBC",
        "LOAD", "STORE", "JUMP", "JAL", "CMP", "BEQ", "BNE", "BLT", "BGT", "NOP", "MOVUI",
        "IN", "OUT", "syscall", "HALT", "NOP"
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


// gets operands from a string using the assumption that the first operand will always be a register, or there will be no operands at all
fn get_operands_from_line(line:&str) -> Vec<&str> {
    let operands:Vec<&str> = Vec::new();
    if !line.contains("$") {
        return operands;
    }

    operands
}


// Takes a line of assembly and the associated opcode (which should already be validated), and checks that the operands are valid
fn validate_operands(line:&str, opcode:&str) -> Result<(), AsmValidationError> {
    let operands = get_operands_from_line(line);
    match opcode {
        "ADD" | "SUB" | "NAND" | "OR" | "LOAD" | "STORE" => { // require 3 registers

        },

        "ADDI" | "SUBI" | "SLL" | "SRL" | "SRA" => { // require 3 registers and an immediate

        },

        "ADDC" | "SUBC" | "JUMP" | "CMP" | "JAL" | "BEQ" | "BNE" | "BLT" | "BGT" | "IN" | "OUT" => { // require 2 registers

        },

        "MOVUI" => { // requires a register and a 16-bit immediate

        },

        "syscall" => { // requires a register and an 8-bit immediate

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


// Takes a line of assembly and checks if it contains a label and, if it does, checks that the label is valid - if not, the function will return an error.
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
}
