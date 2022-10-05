use std::fmt;


// Used if the command line arguments supplied are incorrect
#[derive(Debug, Clone)]
pub struct CmdArgsError;

// Ensures that the error is displayed appropriately in the console when raised.
impl fmt::Display for CmdArgsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incorrect number or format of command line instructions. Proper usage is 'iridium_assembler [source filename] [target_filename]'")
    }
}
