use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::error::Error;
use phf::phf_map;
use crate::errors::TokenTypeError;
use crate::token_types::FileTokens;



static OPCODE_BINARIES:phf::Map<&'static str, u16> = phf_map!{
    "NOP"   => 0x0000,   "ADD"   => 0x1000, "SUB"   => 0x2000, "ADDI"  => 0x3000, "SUBI"  => 0x4000, 
    "SLL"   => 0x5000,   "SRL"   => 0x6000, "SRA"   => 0x7000, "NAND"  => 0x8000, "OR"    => 0x9000, 
    "LOAD"  => 0xA000,   "STORE" => 0xB000, "MOVUI" => 0xC000, "MOVLI" => 0xD000, "ADDC"  => 0xF000, 
    "SUBC"  => 0xF100,   "JUMP"  => 0xF200, "JAL"   => 0xF300, "CMP"   => 0xF400, "BEQ"   => 0xF500, 
    "BNE"   => 0xF600,   "BLT"   => 0xF700, "BGT"   => 0xF800, "IN"    => 0xF900, "OUT"   => 0xFA00, 
    "syscall" => 0xFC00, "HALT"  => 0xFFFF
};

static REGISTER_BINARIES:phf::Map<&'static str, u16> = phf_map!{
    "$zero" => 0x0, "$g0" => 0x1, "$g1" => 0x2, "$g2" => 0x3, "$g3" => 0x4, "$g4" => 0x5, 
    "$g5"   => 0x6, "$g6" => 0x7, "$g7" => 0x8, "$g8" => 0x9, "$g9" => 0xA, "$ua" => 0xB, 
    "$sp"   => 0xC, "$fp" => 0xD, "$ra" => 0xE, "$pc" => 0xF
};


/// Takes a token in the form of a `FileTokens` struct and converts it into a vector f bytes which can be written to a file or printed.
pub fn get_binary_from_tokens(tokens:FileTokens) -> Result<Vec<u16>, TokenTypeError> {
    match tokens {
        FileTokens::InstrTokens(t) => {
            let mut binary:u16 = 0x0000;
            let opcode = *OPCODE_BINARIES.get(&t.opcode as &str).unwrap();
            binary |= opcode;

            // Insert the opcode and first register into the binary instruction based on if the opcode is 4 or 8 bits unless it is a 
            // syscall, in which case skip as there is no register, only immediate
            if opcode != 0xFC00 {
                let register_a:u16 = *REGISTER_BINARIES.get(&t.clone().operand_a.unwrap_or("$zero".to_owned()) as &str).unwrap() as u16;
                if binary & 0xF000 == 0xF000 {
                    binary |= register_a << 4;
                } else {
                    binary |= register_a << 8;
                }
            }

            match opcode {
                0x0000 | 0xFFFF => { // NOP, and HALT 
                    return Ok(vec![opcode]); 
                },

                0x1000 | 0x2000 | 0x5000 | 0x6000 | 0x7000 | 0x8000 | 0x9000 | 0xA000 | 0xB000 => { // rrr format
                    binary |= (*REGISTER_BINARIES.get(&t.operand_b.unwrap_or("$zero".to_owned()) as &str).unwrap() << 4) as u16;
                    binary |= *REGISTER_BINARIES.get(&t.operand_c.unwrap_or("$zero".to_owned()) as &str).unwrap() as u16;
                },

                0x3000 | 0x4000 => { // rri format
                    binary |= (*REGISTER_BINARIES.get(&t.operand_b.unwrap_or("$zero".to_owned()) as &str).unwrap() << 4) as u16;
                    binary |= (t.immediate.unwrap() & 0x000F) as u16; // TODO: this could be unsafe? 
                },

                0xC000 | 0xD000 => { // rii format
                    binary |= (*REGISTER_BINARIES.get(&t.operand_b.unwrap_or("$zero".to_owned()) as &str).unwrap() << 4) as u16;
                    binary |= (t.immediate.unwrap() & 0x00FF) as u16;
                },

                0xF000 | 0xF100 | 0xF200 | 0xF300 | 0xF400 | 0xF500 | 0xF600 | 0xF700 | 0xF800 => { // orr format
                    binary |= *REGISTER_BINARIES.get(&t.operand_b.unwrap_or("$zero".to_owned()) as &str).unwrap() as u16;
                },

                0xF900 | 0xFA00 => { // ori format
                    binary |= (t.immediate.unwrap() & 0x000F) as u16;
                },

                0xFC00 => {
                    binary |= (t.immediate.unwrap() & 0x00FF) as u16;
                },

                _ => { // TODO: replace with an error
                    return Err(TokenTypeError(format!("{} is not a valid opcode", opcode)));
                }
            }
            return Ok(vec![binary]);
        },

        FileTokens::DataTokens(t) => {
            return Ok(t.bytes);
        },

        FileTokens::TextTokens(t) => {
            return Ok(t.bytes);
        }
    }
}


/// Takes a `Vec<FileTokens>` as input and converts it to binary[0], then writes it to the given file
pub fn generate_binary(filename:&str, tokens:&Vec<FileTokens>) -> Result<(), Box<dyn Error>> {
    let mut section_mode = 'c';
    let mut output_file = BufWriter::new(
        OpenOptions::new().create(true).write(true).open(filename.to_owned()).unwrap());
    let mut text_instrs:Vec<FileTokens> = Vec::new(); // These are for the text section, processed last
    
    for token in tokens {
        let binary_vec = match token {
            FileTokens::InstrTokens(_) => get_binary_from_tokens(token.clone()).unwrap(),
            FileTokens::TextTokens(_) => {
                text_instrs.push(token.clone());
                continue;
            },

            FileTokens::DataTokens(_) => {
                // switch to data mode if a non-text data instr is found
                if section_mode == 'c' {
                    section_mode = 'd';
                    output_file.write("data:\0".as_bytes())?;
                }
                
                get_binary_from_tokens(token.clone()).unwrap()
            }
        };

        // write instr to file
        for binary in binary_vec {
            output_file.write(&[(binary & 0x00FF) as u8])?;
            output_file.write(&[((binary & 0xFF00) >> 8) as u8])?;
        }
    }

    if !text_instrs.is_empty() {
        output_file.write("text:\0".as_bytes())?;
        
        for token in text_instrs {
            for binary in get_binary_from_tokens(token.clone()).unwrap() {
                output_file.write(&[(binary & 0x00FF) as u8])?;
                output_file.write(&[((binary & 0xFF00) >> 8) as u8])?;
            }
        }
    }

    output_file.flush().unwrap();
    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::generate_code::*;
    use crate::token_types::*;


    #[test]
    fn test_nop_token() {
        let token = FileTokens::InstrTokens(InstrTokens::new(None, "NOP".to_string(), None, None, None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x0000);
    }


    #[test]
    fn test_halt_token() {
        let token = FileTokens::InstrTokens(InstrTokens::new(None, "HALT".to_string(), None, None, None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xFFFF);
    }


    #[test]
    fn test_rrr_tokens() {
        let token = FileTokens::InstrTokens(InstrTokens::new(None, "ADD".to_string(), Some("$g0".to_string()), Some("$zero".to_string()), Some("$g1".to_string()), None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x1102);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "SUB".to_string(), Some("$g2".to_string()), Some("$g3".to_string()), Some("$g4".to_string()), None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x2345);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "SLL".to_string(), Some("$g5".to_string()), Some("$g6".to_string()), Some("$g7".to_string()), None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x5678);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "SRL".to_string(), Some("$g8".to_string()), Some("$g9".to_string()), Some("$ua".to_string()), None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x69AB);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "SRA".to_string(), Some("$sp".to_string()), Some("$fp".to_string()), Some("$ra".to_string()), None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x7CDE);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "NAND".to_string(), Some("$pc".to_string()), Some("$g0".to_string()), Some("$g1".to_string()), None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x8F12);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "OR".to_string(), Some("$g0".to_string()), Some("$g1".to_string()), Some("$g2".to_string()), None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x9123);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "LOAD".to_string(), Some("$g0".to_string()), Some("$g1".to_string()), Some("$g2".to_string()), None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xA123);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "STORE".to_string(), Some("$g0".to_string()), Some("$g1".to_string()), Some("$g2".to_string()), None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xB123);
    }


    #[test]
    fn test_rri_tokens() {
        let token = FileTokens::InstrTokens(InstrTokens::new(None, "ADDI".to_string(), Some("$g8".to_string()), Some("$g9".to_string()), None, Some(10), None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x39AA);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "SUBI".to_string(), Some("$g8".to_string()), Some("$g9".to_string()), None, Some(5), None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0x49A5);
    }


    #[test]
    fn test_rii_format() {
        let token = FileTokens::InstrTokens(InstrTokens::new(None, "MOVUI".to_string(), Some("$g5".to_string()), None, None, Some(0x75), None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xC675);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "MOVLI".to_string(), Some("$g5".to_string()), None, None, Some(0xFF), None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xD6FF);
    }


    #[test]
    fn test_orr_format() {
        let token = FileTokens::InstrTokens(InstrTokens::new(None, "ADDC".to_string(), Some("$g4".to_string()), None, None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF050);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "SUBC".to_string(), Some("$g4".to_string()), None, None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF150);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "JUMP".to_string(), Some("$g1".to_string()), Some("$g2".to_string()), None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF223);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "JAL".to_string(), Some("$g2".to_string()), Some("$g3".to_string()), None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF334);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "CMP".to_string(), Some("$g3".to_string()), Some("$g4".to_string()), None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF445);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "BEQ".to_string(), Some("$g3".to_string()), Some("$g4".to_string()), None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF545);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "BNE".to_string(), Some("$g3".to_string()), Some("$g4".to_string()), None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF645);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "BLT".to_string(), Some("$g3".to_string()), Some("$g4".to_string()), None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF745);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "BGT".to_string(), Some("$g3".to_string()), Some("$g4".to_string()), None, None, None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF845);
    }


    #[test]
    fn test_ori_format() {
        let token = FileTokens::InstrTokens(InstrTokens::new(None, "IN".to_string(), Some("$g3".to_string()), None, None, Some(0), None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xF940);

        let token = FileTokens::InstrTokens(InstrTokens::new(None, "OUT".to_string(), Some("$g3".to_string()), None, None, Some(1), None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xFA41);
    }


    #[test]
    fn test_syscall_format() {
        let token = FileTokens::InstrTokens(InstrTokens::new(None, "syscall".to_string(), None, None, None, Some(19), None));
        let binary = get_binary_from_tokens(token).unwrap();
        assert_eq!(binary[0], 0xFC13);
    }


    #[test]
    fn test_section_data_instrs() {
        let bytes:Vec<u16> = vec![0x0100, 0x01A0, 0x0200, 0x1000, 0x0000];
        let token = FileTokens::DataTokens(DataTokens::new(None, "section".to_string(), bytes));
        let binary = get_binary_from_tokens(token).unwrap();

        assert_eq!(binary[0], 0x0100);
        assert_eq!(binary[1], 0x01A0);
        assert_eq!(binary[2], 0x0200);
        assert_eq!(binary[3], 0x1000);
        assert_eq!(binary[4], 0x0000);
    }
}
