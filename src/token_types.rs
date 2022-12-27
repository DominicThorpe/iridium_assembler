use std::fmt;
use crate::errors::TokenTypeError;



/// Can contain both types of tokens a line of asm can take
#[derive(Debug, Clone)]
pub enum FileTokens {
    InstrTokens(InstrTokens),
    DataTokens(DataTokens),
    TextTokens(TextTokens)
}


impl FileTokens {
    /// Takes another `FileTokens` strust as the *other* argument and gets the labels from both, then 
    /// returns true if the labels are the same, and false if they are not
    ///
    /// TODO: make sure that a `FileTokens` with a label of "null" and one without a label do not return
    /// true when used with this function
    pub fn compare_label(&self, other: FileTokens) -> bool {
        let null_str = &"null".to_string();
        let self_label = match self {
            FileTokens::InstrTokens(t) => t.label.as_ref().unwrap_or(null_str),
            FileTokens::DataTokens(t) => t.label.as_ref().unwrap_or(null_str),
            FileTokens::TextTokens(t) => t.label.as_ref().unwrap_or(null_str)
        };

        let other_label = match other {
            FileTokens::InstrTokens(t) => t.label.unwrap_or("null".to_string()),
            FileTokens::DataTokens(t) => t.label.unwrap_or("null".to_string()),
            FileTokens::TextTokens(t) => t.label.unwrap_or("null".to_string())
        };

        self_label == &other_label
    }


    /// Attempts to get an `InstrTokens` from a `FileTokens` enum. Will return an `InstrTokens` if the enum
    /// is of the right type, or a `TokensTypeError` if not.
    pub fn try_get_instr_tokens(&self) -> Result<InstrTokens, TokenTypeError> {
        match self {
            FileTokens::InstrTokens(t) => Ok(t.clone()),
            FileTokens::DataTokens(_) => Err(TokenTypeError("Invalid token type detected!".to_string())),
            FileTokens::TextTokens(_) => Err(TokenTypeError("Invalid token type detected!".to_string()))
        }
    }


    /// Attempts to get a `DataTokens` from a `FileTokens` enum. Will return a `DataTokens` if the enum
    /// is of the right type, or a `TokensTypeError` if not.
    pub fn try_get_data_tokens(&self) -> Result<DataTokens, TokenTypeError> {
        match self {
            FileTokens::DataTokens(t) => Ok(t.clone()),
            FileTokens::InstrTokens(_) => Err(TokenTypeError("Invalid token type detected!".to_string())),
            FileTokens::TextTokens(_) => Err(TokenTypeError("Invalid token type detected!".to_string()))
        }
    }


    /// Attempts to get a `TextTokens` from a `FileTokens` enum. Will return a `DataTokens` if the enum
    /// is of the right type, or a `TokensTypeError` if not.
    pub fn try_get_text_tokens(&self) -> Result<TextTokens, TokenTypeError> {
        match self {
            FileTokens::DataTokens(_) => Err(TokenTypeError("Invalid token type detected!".to_string())),
            FileTokens::InstrTokens(_) => Err(TokenTypeError("Invalid token type detected!".to_string())),
            FileTokens::TextTokens(t) => Ok(t.clone())
        }
    }
}


/// Represents the core components of an instruction, including the opcode, and the optional label and 
/// operands, and possible operand label
#[derive(Clone)]
pub struct InstrTokens {
    pub label: Option<String>,
    pub opcode: String,
    pub operand_a: Option<String>,
    pub operand_b: Option<String>,
    pub operand_c: Option<String>,
    pub immediate: Option<u64>, // used as a set of bytes
    pub op_label: Option<String>
}

impl InstrTokens {
    /// Creates a new instance of `InstrTokens` according to the passed parameters
    pub fn new(label:Option<String>, opcode:String, operand_a:Option<String>, 
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
#[derive(Clone)]
pub struct DataTokens {
    pub label: Option<String>,
    pub category: String,
    pub bytes: Vec<u16>
}


impl DataTokens {
    pub fn new(label:Option<String>, category:String, bytes:Vec<u16>) -> DataTokens {
        DataTokens {
            label: label,
            category: category,
            bytes: bytes
        }
    }
}


impl fmt::Debug for DataTokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{}\t{:04X?}", self.label.clone().unwrap_or("null".to_string()), self.category, self.bytes)
    }
}


/// Represents the components of a data instruction, including the label, category, and value
#[derive(Clone)]
pub struct TextTokens {
    pub label: Option<String>,
    pub bytes: Vec<u16>
}


impl TextTokens {
    pub fn new(label:Option<String>, bytes:Vec<u16>) -> TextTokens {
        TextTokens {
            label: label,
            bytes: bytes
        }
    }
}


impl fmt::Debug for TextTokens {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}\t{:04X?}", self.label.clone().unwrap_or("null".to_string()), self.bytes)
    }
}
