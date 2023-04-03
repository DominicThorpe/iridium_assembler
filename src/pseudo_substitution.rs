use crate::token_types::{FileTokens, InstrTokens};
use crate::errors::LabelNotFoundError;
use std::collections::HashMap;



/// Locates any instructions with label operands and makes the neccessary substitutions as per the 
/// `substitute_labels` function. If any single-operand branch instructions are found, then the 
/// 1st operand is swapped to be the 2nd, and the 1st is turned into `None`.
pub fn substitute_pseudo_instrs(tokens: Vec<FileTokens>) -> Vec<FileTokens> {
    let mut new_tokens:Vec<FileTokens> = Vec::new();
    for token in &tokens {
        match token {
            FileTokens::InstrTokens(t) => {
                match &t.op_label {
                    Some(operand) => {
                        if t.opcode == "LOAD" || t.opcode == "STORE" {
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, "MOVLI".to_owned(), t.operand_b.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, "MOVUI".to_owned(), t.operand_b.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, t.opcode.clone(), t.operand_a.clone(), t.operand_b.clone(), t.operand_c.clone(), None, None)));
                        } else if t.opcode != "MOVLI" && t.opcode != "MOVUI" { // Branch opcodes
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(t.label.clone(), "MOVLI".to_owned(), t.operand_a.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, "MOVUI".to_owned(), t.operand_a.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, t.opcode.clone(), t.operand_a.clone(), t.operand_b.clone(), t.operand_c.clone(), None, None)));
                        } else {
                            new_tokens.push(token.clone());
                        }
                    },

                    None => {
                        if t.opcode == "JUMP" || t.opcode == "BEQ" || t.opcode == "BNE" || t.opcode == "BLT" || t.opcode == "BGT" || t.opcode == "JAL" {
                            match &t.operand_b {
                                Some(_) => new_tokens.push(token.clone()),
                                None => new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(t.label.clone(), t.opcode.clone(), None, t.operand_a.clone(), None, None, None))),
                            }
                        } else {
                            new_tokens.push(FileTokens::InstrTokens(t.clone()));
                        }
                    }
                }
            },

            FileTokens::DataTokens(_) => {
                new_tokens.push(token.clone());
            },

            FileTokens::TextTokens(_) => {
                new_tokens.push(token.clone());
            }
        }
    }

    new_tokens
}


/// Takes a label table and a vector of `FileTokens` as arguments and returns a new vector which has,
/// where appropriate, converted the label operands into immediates.
pub fn substitute_labels(tokens:Vec<FileTokens>, label_table:&HashMap<String, i64>) -> Result<Vec<FileTokens>, LabelNotFoundError> {
    let mut new_tokens:Vec<FileTokens> = Vec::new();
    for token in tokens {
        match token {
            FileTokens::DataTokens(t) => new_tokens.push(FileTokens::DataTokens(t.clone())),
            FileTokens::TextTokens(t) => new_tokens.push(FileTokens::TextTokens(t.clone())),
            FileTokens::InstrTokens(mut t) => {
                match t.op_label {
                    Some(label) => {
                        let label = label.replace("@", "");
                        let new_imm:i64 = match t.opcode.as_str() {
                            "MOVLI" => label_table.get(&label).expect(&format!("The label {} was not found!", label)) & 0x00FF,
                            "MOVUI" => (label_table.get(&label).expect(&format!("The label {} was not found!", label)) & 0xFF00) >> 8,
                            opcode => panic!("The instruction {} cannot take label operands!", opcode)
                        };

                        t.immediate = Option::from(new_imm as u64);
                        t.op_label = None;

                        new_tokens.push(FileTokens::InstrTokens(t.clone()))
                    },

                    None => new_tokens.push(FileTokens::InstrTokens(t.clone()))
                }
            }
        }
    }

    Ok(new_tokens)
}


#[cfg(test)]
mod tests {
    use crate::process_file_into_tokens;
    use crate::pseudo_substitution::{substitute_pseudo_instrs, substitute_labels};
    use crate::token_types::InstrTokens;
    use crate::label_table::generate_label_table;


    fn assert_instr_token(token:InstrTokens, operand:String, operand_a:Option<String>, 
        operand_b:Option<String>, operand_c:Option<String>, immediate:Option<u64>, op_label:Option<String>) {
            assert_eq!(token.opcode, operand);
            assert_eq!(token.operand_a, operand_a);
            assert_eq!(token.operand_b, operand_b);
            assert_eq!(token.operand_c, operand_c);
            assert_eq!(token.immediate, immediate);
            assert_eq!(token.op_label, op_label);
    }


    #[test]
    fn test_load_substitution() {
        let tokens = process_file_into_tokens("test_files/test_expand_pseudoinstrs.asm");
        let subbed_tokens = substitute_pseudo_instrs(tokens);

        let mut token = subbed_tokens[0].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "ADDI".to_string(), Option::from("$g0".to_string()), Option::from("$zero".to_string()), None, Option::from(10), None);

        token = subbed_tokens[1].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g6".to_string()), None, None, None, Option::from("@test_1".to_string()));

        token = subbed_tokens[2].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g6".to_string()), None, None, None, Option::from("@test_1".to_string()));

        token = subbed_tokens[3].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "LOAD".to_string(), Option::from("$g5".to_string()), Option::from("$g6".to_string()), Option::from("$g7".to_string()), None, None);

        assert_eq!(subbed_tokens.len(), 15);
    }


    #[test]
    fn test_store_substitution() {
        let tokens = process_file_into_tokens("test_files/test_expand_pseudoinstrs.asm");
        let subbed_tokens = substitute_pseudo_instrs(tokens);

        let mut token = subbed_tokens[5].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g1".to_string()), None, None, None, Option::from("@test_2".to_string()));

        token = subbed_tokens[6].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g1".to_string()), None, None, None, Option::from("@test_2".to_string()));

        token = subbed_tokens[7].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "STORE".to_string(), Option::from("$g0".to_string()), Option::from("$g1".to_string()), Option::from("$g2".to_string()), None, None);
    }


    #[test]
    fn test_beq_substitution() {
        let tokens = process_file_into_tokens("test_files/test_expand_pseudoinstrs.asm");
        let subbed_tokens = substitute_pseudo_instrs(tokens);

        let mut token = subbed_tokens[9].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g3".to_string()), None, None, None, Option::from("@test_3".to_string()));

        token = subbed_tokens[10].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g3".to_string()), None, None, None, Option::from("@test_3".to_string()));

        token = subbed_tokens[11].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "BEQ".to_string(), Option::from("$g3".to_string()), None, None, None, None);
    }


    #[test]
    fn test_bgt_substitution() {
        let tokens = process_file_into_tokens("test_files/test_expand_pseudoinstrs.asm");
        let subbed_tokens = substitute_pseudo_instrs(tokens);

        let mut token = subbed_tokens[12].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g6".to_string()), None, None, None, Option::from("@test_4".to_string()));

        token = subbed_tokens[13].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g6".to_string()), None, None, None, Option::from("@test_4".to_string()));

        token = subbed_tokens[14].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "BGT".to_string(), Option::from("$g6".to_string()), None, None, None, None);
    }


    #[test]
    #[should_panic]
    fn test_non_existant_label() {
        let tokens = process_file_into_tokens("test_files/test_detect_bad_label.asm");
        let tokens = substitute_pseudo_instrs(tokens);
        let label_table = generate_label_table(&tokens).unwrap();
        let _tokens = substitute_labels(tokens, &label_table).unwrap();
    }


    #[test]
    fn test_label_substitution() {
        let tokens = process_file_into_tokens("test_files/test_sub_label_addrs.asm");
        let tokens = substitute_pseudo_instrs(tokens);
        let label_table = generate_label_table(&tokens).unwrap();
        let tokens = substitute_labels(tokens, &label_table).unwrap();

        println!("Table:\n{:#?}", label_table);

        assert_instr_token(
            tokens[2].try_get_instr_tokens().unwrap(), "MOVLI".to_string(), 
            Option::from("$g8".to_owned()), None, None, Option::from(0), None
        );

        assert_instr_token(
            tokens[3].try_get_instr_tokens().unwrap(), "MOVUI".to_string(), 
            Option::from("$g8".to_owned()), None, None, Option::from(136), None
        );

        assert_instr_token(
            tokens[4].try_get_instr_tokens().unwrap(), "LOAD".to_string(), 
            Option::from("$g5".to_owned()), Option::from("$g8".to_owned()), 
            Option::from("$g9".to_owned()), None, None
        );

        assert_instr_token(
            tokens[10].try_get_instr_tokens().unwrap(), "MOVLI".to_string(), 
            Option::from("$g8".to_owned()), None, None, Option::from(16), None
        );

        assert_instr_token(
            tokens[11].try_get_instr_tokens().unwrap(), "MOVUI".to_string(), 
            Option::from("$g8".to_owned()), None, None, Option::from(36), None
        );

        assert_instr_token(
            tokens[12].try_get_instr_tokens().unwrap(), "BGT".to_string(), 
            Option::from("$g8".to_owned()), None, None, None, None
        );
    }


    #[test]
    fn test_single_operand_branch_substitution() {
        let tokens = process_file_into_tokens("test_files/test_single_operand_branch_sub.asm");
        let tokens = substitute_pseudo_instrs(tokens);

        let label_table = generate_label_table(&tokens).unwrap();
        let tokens = substitute_labels(tokens, &label_table).unwrap();

        assert_instr_token(
            tokens[0].try_get_instr_tokens().unwrap(), "JUMP".to_string(), 
            None, Option::from("$ra".to_owned()), None, None, None
        );

        assert_instr_token(
            tokens[1].try_get_instr_tokens().unwrap(), "BNE".to_string(), 
            None, Option::from("$sp".to_owned()), None, None, None
        );

        assert_instr_token(
            tokens[2].try_get_instr_tokens().unwrap(), "BEQ".to_string(), 
            None, Option::from("$fp".to_owned()), None, None, None
        );

        assert_instr_token(
            tokens[3].try_get_instr_tokens().unwrap(), "BGT".to_string(), 
            None, Option::from("$pc".to_owned()), None, None, None
        );

        assert_instr_token(
            tokens[4].try_get_instr_tokens().unwrap(), "BLT".to_string(), 
            None, Option::from("$ra".to_owned()), None, None, None
        );

        assert_instr_token(
            tokens[5].try_get_instr_tokens().unwrap(), "JAL".to_string(), 
            None, Option::from("$ra".to_owned()), None, None, None
        );
    }
}
