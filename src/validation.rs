use crate::errors::AsmValidationError;


// Takes a line of assembly code, for example `ADD $g0, $zero, $g1`, and returns an `Err` if it
// is not valid Iridium assembly.
pub fn validate_asm_line(line:&str) -> Result<(), AsmValidationError> {
    validate_opcode(line)?;
    Ok(())
}


// Takes a line of assembly, extracts the opcode from it, and checks that it is a valid opcode. If an invalid opcode is found, an `AsmValidationError` will be thrown.
fn validate_opcode(line:&str) -> Result<(), AsmValidationError> {
    let valid_opcodes:[&str;26] = [
        "ADD", "SUB", "ADDI", "SUBI", "SLL", "SRL", "SRA", "NAND", "OR", "ADDC", "SUBC",
        "LOAD", "STORE", "JUMP", "JAL", "CMP", "BEQ", "BNE", "BLT", "BGT", "NOP", "MOVUI",
        "IN", "OUT", "syscall", "HALT"
    ];

    let tokens:Vec<&str> = line.split(" ").collect();
    if !valid_opcodes.contains(&tokens[0]) {
        return Err(AsmValidationError(format!("{} is not a valid opcode on line {}", tokens[0], line)));
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::validation::validate_opcode;


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
    #[should_panic]
    fn test_invalid_opcode() {
        validate_opcode("ADDUII $r0, 600").unwrap();
    }
}
