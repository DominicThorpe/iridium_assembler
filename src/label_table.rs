use std::collections::HashMap;
use crate::token_types::FileTokens;
use crate::errors::AsmValidationError;


/// Takes a filename and generates a `HashMap<String, i64>` of all labels in the instructions and data
/// section and returns it. Will include paging (pages are 4Kb) to ensure data is on different page to
/// instructions. 
pub fn generate_label_table(tokens_stream:&Vec<FileTokens>) -> Result<HashMap<String, i64>, AsmValidationError> {
    let mut instr_addr = 0;
    let page_size = 0x1000;
    let mut data_addr:i64 = 0;
    let mut text_addr:i64 = 0;
    let mut mode:char = 'c';
    let mut label_table:HashMap<String, i64> = HashMap::new();
    for tokens in tokens_stream {
        match tokens {
            FileTokens::DataTokens(t) => {
                if mode == 'c' {
                    data_addr += page_size;
                    text_addr += page_size;
                    mode = 'd';
                }

                match &t.label {
                    Some(label) => {
                        if label_table.contains_key(label) {
                            return Err(AsmValidationError(format!("Duplicate label \"{}\" detected!", label)));
                        }

                        let num_bytes:i64 = t.bytes.len().try_into().unwrap();
                        label_table.insert(label.to_owned(), data_addr);

                        data_addr += num_bytes;
                        if data_addr % page_size == 0 && data_addr != 0 {
                            text_addr += page_size;
                        }
                    },

                    None => {
                        data_addr += 1;
                        if data_addr % page_size == 0 && data_addr != 0 {
                            text_addr += page_size;
                        }
                    }
                }
            },

            FileTokens::TextTokens(t) => {
                if mode != 't' {
                    text_addr += page_size;
                    mode = 't';
                }

                match &t.label {
                    Some(label) => {
                        if label_table.contains_key(label) {
                            return Err(AsmValidationError(format!("Duplicate label \"{}\" detected!", label)));
                        }

                        let num_bytes:i64 = t.bytes.len().try_into().unwrap();
                        label_table.insert(label.to_owned(), text_addr);
                        text_addr += num_bytes;
                    },

                    None => text_addr += 1
                }
            },

            FileTokens::InstrTokens(t) => {
                match &t.label {
                    Some(label) => {
                        if label_table.contains_key(label) {
                            return Err(AsmValidationError(format!("Duplicate label \"{}\" detected!", label)));
                        }

                        label_table.insert(label.to_owned(), instr_addr);
                        instr_addr += 1;
                        if instr_addr % page_size == 0 && instr_addr != 0 {
                            data_addr += page_size;
                            text_addr += page_size;
                        } 
                    },

                    None => {
                        instr_addr += 1;
                        if instr_addr % page_size == 0 && instr_addr != 0 {
                            data_addr += page_size;
                            text_addr += page_size;
                        } 
                    }
                }
            }
        };
    }

    Ok(label_table)
}


#[cfg(test)]
mod tests {
    use crate::process_file_into_tokens;
    use crate::pseudo_substitution;
    use crate::label_table;


    #[test]
    fn test_label_table_generation() {
        let tokens = process_file_into_tokens("test_files/test_label_table_gen.asm");
        let tokens = pseudo_substitution::substitute_pseudo_instrs(tokens);
        let label_table = label_table::generate_label_table(&tokens).unwrap();

        assert_eq!(label_table.len(), 10);
        assert_eq!(label_table["init"], 0x0000);
        assert_eq!(label_table["loop"], 0x0005);
        assert_eq!(label_table["end"], 0x0010);
        assert_eq!(label_table["target"], 0x1000);
        assert_eq!(label_table["int_long"], 0x1001);
        assert_eq!(label_table["half_float"], 0x1003);
        assert_eq!(label_table["float"], 0x1004);
        assert_eq!(label_table["eszet"], 0x1006);
        assert_eq!(label_table["list"], 0x1007);
        assert_eq!(label_table["text_data"], 0x2000);
    }


    #[test]
    fn test_label_paging() {
        let tokens = process_file_into_tokens("test_files/test_large_prog.asm");
        let tokens = pseudo_substitution::substitute_pseudo_instrs(tokens);
        let label_table = label_table::generate_label_table(&tokens).unwrap();

        assert_eq!(label_table.len(), 5);
        assert_eq!(label_table["start"], 0);
        assert_eq!(label_table["pg_end"], 0x0FFF);
        assert_eq!(label_table["pg_start"], 0x1000);
        assert_eq!(label_table["some_data"], 0x2000);
        assert_eq!(label_table["some_other_data"], 0x2001);
    }


    #[test]
    #[should_panic]
    fn test_duplicate_label() {
        let tokens = process_file_into_tokens("test_files/test_duplicate_label.asm");
        let tokens = pseudo_substitution::substitute_pseudo_instrs(tokens);
        let _ = label_table::generate_label_table(&tokens).unwrap();
    }


    #[test]
    #[should_panic]
    fn test_text_outside_text_section() {
        let _ = process_file_into_tokens("test_files/test_text_outside_section.asm");
    }


    #[test]
    fn test_text_without_data_section() {
        let tokens = process_file_into_tokens("test_files/test_text_without_data.asm");
        let tokens = pseudo_substitution::substitute_pseudo_instrs(tokens);
        let label_table = label_table::generate_label_table(&tokens).unwrap();

        assert_eq!(label_table.get("directory").unwrap(), &0x1000);
    }
}
