use std::fs::OpenOptions;
use std::io::{BufWriter, Write};
use std::collections::HashMap;
use std::error::Error;
use crate::errors::TokenTypeError;
use crate::token_types::FileTokens;



/// Takes a token in the form of a `FileTokens` struct and converts it into a vector f bytes which can be written to a file or printed.
pub fn get_binary_from_tokens(tokens:FileTokens) -> Result<Vec<u16>, TokenTypeError> {
    let opcode_binaries:HashMap<String, u16> = HashMap::from([
        ("NOP".to_owned(), 0x0000), ("ADD".to_owned(), 0x1000), ("SUB".to_owned(), 0x2000), ("ADDI".to_owned(), 0x3000), ("SUBI".to_owned(), 0x4000), 
        ("SLL".to_owned(), 0x5000), ("SRL".to_owned(), 0x6000), ("SRA".to_owned(), 0x7000), ("NAND".to_owned(), 0x8000), ("OR".to_owned(), 0x9000), 
        ("LOAD".to_owned(), 0xA000), ("STORE".to_owned(), 0xB000), ("MOVUI".to_owned(), 0xC000), ("MOVLI".to_owned(), 0xD000), ("ADDC".to_owned(), 0xF000), 
        ("SUBC".to_owned(), 0xF100), ("JUMP".to_owned(), 0xF200), ("JAL".to_owned(), 0xF300), ("CMP".to_owned(), 0xF400), ("BEQ".to_owned(), 0xF500), 
        ("BNE".to_owned(), 0xF600), ("BLT".to_owned(), 0xF700), ("BGT".to_owned(), 0xF800), ("IN".to_owned(), 0xF900), ("OUT".to_owned(), 0xFA00), 
        ("syscall".to_owned(), 0xFC00), ("HALT".to_owned(), 0xFFFF)
    ]);

    let register_binaries:HashMap<String, u8> = HashMap::from([
        ("$zero".to_owned(), 0x0), ("$g0".to_owned(), 0x1), ("$g1".to_owned(), 0x2), ("$g2".to_owned(), 0x3), ("$g3".to_owned(), 0x4), ("$g4".to_owned(), 0x5), 
        ("$g5".to_owned(), 0x6),   ("$g6".to_owned(), 0x7), ("$g7".to_owned(), 0x8), ("$g8".to_owned(), 0x9), ("$g9".to_owned(), 0xA), ("$ua".to_owned(), 0xB), 
        ("$sp".to_owned(), 0xC),   ("$fp".to_owned(), 0xD), ("$ra".to_owned(), 0xE), ("$pc".to_owned(), 0xF)
    ]);

    match tokens {
        FileTokens::InstrTokens(t) => {
            let mut binary:u16 = 0x0000;
            let opcode = *opcode_binaries.get(&t.opcode).unwrap();
            binary |= opcode;

            // Insert the opcode and first register into the binary instruction based on if the opcode is 4 or 8 bits unless it is a 
            // syscall, in which case skip as there is no register, only immediate
            if opcode != 0xFC00 {
                let register_a:u16 = *register_binaries.get(&t.clone().operand_a.unwrap_or("$zero".to_owned())).unwrap() as u16;
                if binary & 0xF000 == 0xF000 {
                    binary |= register_a << 4;
                } else {
                    binary |= register_a << 8;
                }
            }

            match opcode {
                0x0000 => { return Ok(vec![0x0000]); }, // NOP is all 0s
                0xFFFF => { return Ok(vec![0xFFFF]); }, // HALT is all 1s

                0x1000 | 0x2000 | 0x5000 | 0x6000 | 0x7000 | 0x8000 | 0x9000 | 0xA000 | 0xB000 => { // rrr format
                    binary |= (*register_binaries.get(&t.operand_b.unwrap_or("$zero".to_owned())).unwrap() << 4) as u16;
                    binary |= *register_binaries.get(&t.operand_c.unwrap_or("$zero".to_owned())).unwrap() as u16;
                },

                0x3000 | 0x4000 => { // rri format
                    binary |= (*register_binaries.get(&t.operand_b.unwrap_or("$zero".to_owned())).unwrap() << 4) as u16;
                    binary |= (t.immediate.unwrap() & 0x000F) as u16; // TODO: this could be unsafe? 
                },

                0xC000 | 0xD000 => { // rii format
                    binary |= (*register_binaries.get(&t.operand_b.unwrap_or("$zero".to_owned())).unwrap() << 4) as u16;
                    binary |= (t.immediate.unwrap() & 0x00FF) as u16;
                },

                0xF000 | 0xF100 | 0xF200 | 0xF300 | 0xF400 | 0xF500 | 0xF600 | 0xF700 | 0xF800 => { // orr format
                    binary |= *register_binaries.get(&t.operand_b.unwrap_or("$zero".to_owned())).unwrap() as u16;
                },

                0xF900 | 0xFA00 => { // ori format
                    binary |= (t.immediate.unwrap() & 0x000F) as u16;
                },

                0xFC00 => {
                    println!("{:?}", t);
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
        }
    }
}


/// Takes a `Vec<FileTokens>` as input and converts it to binary[0], then writes it to the given file
pub fn generate_binary(filename:&str, tokens:&Vec<FileTokens>) -> Result<(), Box<dyn Error>> {
    let mut data_mode = false;
    let mut output_file = BufWriter::new(OpenOptions::new().create(true).write(true).open(filename.to_owned()).unwrap());
    for token in tokens {
        let binary_vec = match token {
            FileTokens::InstrTokens(_) => get_binary_from_tokens(token.clone()).unwrap(),
            FileTokens::DataTokens(_) => {
                if !data_mode {
                    data_mode = true;
                    output_file.write("data:".as_bytes())?;
                }
                
                get_binary_from_tokens(token.clone()).unwrap()
            }
        };

        for binary in binary_vec {
            output_file.write(&[((binary & 0xFF00) >> 8) as u8])?;
            output_file.write(&[(binary & 0x00FF) as u8])?;
        }
    }

    Ok(())
}


#[cfg(test)]
mod tests {
    use crate::generate_code::get_binary_from_tokens;
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
    fn test_data_instr() {
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
