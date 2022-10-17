use std::fmt;
use std::error::Error;


/// Used if the command line arguments supplied are incorrect
#[derive(Debug, Clone)]
pub struct CmdArgsError;
impl Error for CmdArgsError {}

/// Ensures that the `CmdArgsError` error type is displayed appropriately in the console when raised.
impl fmt::Display for CmdArgsError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Incorrect number or format of command line instructions. Proper usage is 'iridium_assembler [source filename] [target_filename]'")
    }
}


/// Used if the assembly validator finds an instruction that is not valid, such as `ADDQ $z0, 80`
#[derive(Debug, Clone)]
pub struct AsmValidationError(pub String);
impl Error for AsmValidationError {}

/// Ensures that the `AsmValidationError` error type is displayed appropriately in the console when raised, 
/// including a custom string to add to the error.
impl fmt::Display for AsmValidationError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Found invalid instruction: {}", self.0)
    }
}


/// Used if the wrong type of token is detected after processing the file into tokens
#[derive(Debug, Clone)]
pub struct TokenTypeError(pub String);
impl Error for TokenTypeError {}

/// Ensures that the `TokenTypeError` error type is displayed appropriately in the console when raised, 
/// including a custom string to add to the error.
impl fmt::Display for TokenTypeError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Found invalid token type: {}", self.0)
    }
}



/// Used if a non-existant label is used
#[derive(Debug, Clone)]
pub struct LabelNotFoundError(pub String);
impl Error for LabelNotFoundError {}

/// Ensures that the `LabelNotFoundError` error type is displayed appropriately in the console when raised, 
/// including a custom string to add to the error.
impl fmt::Display for LabelNotFoundError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Found invalid label: {}", self.0)
    }
}

