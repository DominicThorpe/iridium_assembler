use std::{env, fmt};


// Used if the command line arguments supplied are incorrect
#[derive(Debug, Clone)]
struct CmdArgsError;

// Ensures that the error is displayed appropriately in the console when raised.
impl fmt::Display for CmdArgsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incorrect number or format of command line instructions. Proper usage is 'iridium_assembler [source filename] [target_filename]'")
    }
}


fn main() -> Result<(), CmdArgsError> {
    // Check that the command line arguments supplies are correct
    let cmd_args: Vec<String> = env::args().collect();
    if cmd_args.len() != 3 || !cmd_args[1].ends_with(".ird") {
        return Err(CmdArgsError);
    }

    println!("Compiling {} into {}", cmd_args[1], cmd_args[2]);
    Ok(())
}
