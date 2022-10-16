use crate::token_types::{FileTokens, InstrTokens};



pub fn substitute_pseudo_instrs(tokens: Vec<FileTokens>) -> Vec<FileTokens> {
    let mut new_tokens:Vec<FileTokens> = Vec::new();
    for token in &tokens {
        match token {
            FileTokens::InstrTokens(t) => {
                match &t.op_label {
                    Some(operand) => {
                        if t.opcode == "LOAD" || t.opcode == "STORE" {
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(t.label.clone(), "MOVLI".to_owned(), Some("$ua".to_owned()), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, "MOVUI".to_owned(), Some("$ua".to_owned()), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, "MOVLI".to_owned(), t.operand_b.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, "MOVUI".to_owned(), t.operand_b.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, t.opcode.clone(), t.operand_a.clone(), t.operand_b.clone(), t.operand_c.clone(), None, None)));
                        } else if t.opcode != "MOVLI" && t.opcode != "MOVUI" { // Branch opcodes
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(t.label.clone(), "MOVLI".to_owned(), t.operand_a.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, "MOVUI".to_owned(), t.operand_a.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, "MOVLI".to_owned(), t.operand_b.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, "MOVUI".to_owned(), t.operand_b.clone(), None, None, None, Some(operand.clone()))));
                            new_tokens.push(FileTokens::InstrTokens(InstrTokens::new(None, t.opcode.clone(), t.operand_a.clone(), t.operand_b.clone(), t.operand_c.clone(), None, None)));
                        } else {
                            new_tokens.push(token.clone());
                        }
                    },
                    None => new_tokens.push(FileTokens::InstrTokens(t.clone()))
                }
            },

            FileTokens::DataTokens(_) => {
                // TODO: handle the DataTokens case
            }
        }
    }

    new_tokens
}


#[cfg(test)]
mod tests {
    use crate::process_file_into_tokens;
    use crate::pseudo_substitution::substitute_pseudo_instrs;
    use crate::token_types::InstrTokens;


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
    fn check_load_substitution() {
        let tokens = process_file_into_tokens("test_files/test_expand_pseudoinstrs.asm");
        let subbed_tokens = substitute_pseudo_instrs(tokens);

        let mut token = subbed_tokens[0].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "ADDI".to_string(), Option::from("$g0".to_string()), Option::from("$zero".to_string()), None, Option::from(10), None);

        token = subbed_tokens[1].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$ua".to_string()), None, None, None, Option::from("@test_1".to_string()));

        token = subbed_tokens[2].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$ua".to_string()), None, None, None, Option::from("@test_1".to_string()));

        token = subbed_tokens[3].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g6".to_string()), None, None, None, Option::from("@test_1".to_string()));

        token = subbed_tokens[4].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g6".to_string()), None, None, None, Option::from("@test_1".to_string()));

        token = subbed_tokens[5].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "LOAD".to_string(), Option::from("$g5".to_string()), Option::from("$g6".to_string()), Option::from("$g7".to_string()), None, None);

        assert_eq!(subbed_tokens.len(), 23);
    }


    #[test]
    fn check_store_substitution() {
        let tokens = process_file_into_tokens("test_files/test_expand_pseudoinstrs.asm");
        let subbed_tokens = substitute_pseudo_instrs(tokens);

        let mut token = subbed_tokens[7].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$ua".to_string()), None, None, None, Option::from("@test_2".to_string()));

        token = subbed_tokens[8].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$ua".to_string()), None, None, None, Option::from("@test_2".to_string()));

        token = subbed_tokens[9].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g1".to_string()), None, None, None, Option::from("@test_2".to_string()));

        token = subbed_tokens[10].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g1".to_string()), None, None, None, Option::from("@test_2".to_string()));

        token = subbed_tokens[11].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "STORE".to_string(), Option::from("$g0".to_string()), Option::from("$g1".to_string()), Option::from("$g2".to_string()), None, None);
    }


    #[test]
    fn check_beq_substitution() {
        let tokens = process_file_into_tokens("test_files/test_expand_pseudoinstrs.asm");
        let subbed_tokens = substitute_pseudo_instrs(tokens);

        let mut token = subbed_tokens[13].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g3".to_string()), None, None, None, Option::from("@test_3".to_string()));

        token = subbed_tokens[14].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g3".to_string()), None, None, None, Option::from("@test_3".to_string()));

        token = subbed_tokens[15].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g4".to_string()), None, None, None, Option::from("@test_3".to_string()));

        token = subbed_tokens[16].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g4".to_string()), None, None, None, Option::from("@test_3".to_string()));

        token = subbed_tokens[17].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "BEQ".to_string(), Option::from("$g3".to_string()), Option::from("$g4".to_string()), None, None, None);
    }


    #[test]
    fn check_bgt_substitution() {
        let tokens = process_file_into_tokens("test_files/test_expand_pseudoinstrs.asm");
        let subbed_tokens = substitute_pseudo_instrs(tokens);

        let mut token = subbed_tokens[18].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g6".to_string()), None, None, None, Option::from("@test_4".to_string()));

        token = subbed_tokens[19].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g6".to_string()), None, None, None, Option::from("@test_4".to_string()));

        token = subbed_tokens[20].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVLI".to_string(), Option::from("$g7".to_string()), None, None, None, Option::from("@test_4".to_string()));

        token = subbed_tokens[21].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "MOVUI".to_string(), Option::from("$g7".to_string()), None, None, None, Option::from("@test_4".to_string()));

        token = subbed_tokens[22].try_get_instr_tokens().unwrap();
        assert_instr_token(token, "BGT".to_string(), Option::from("$g6".to_string()), Option::from("$g7".to_string()), None, None, None);
    }
}
