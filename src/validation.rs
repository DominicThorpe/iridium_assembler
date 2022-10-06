use crate::errors::AsmValidationError;


// Takes a line of assembly code, for example `ADD $g0, $zero, $g1`, and returns an `Err` if it
// is not valid Iridium assembly.
pub fn validate_asm_line(line:&str) -> Result<(), AsmValidationError> {
    validate_label(line)?;
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
}
