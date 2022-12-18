use std::collections::HashMap;
use crate::token_types::FileTokens;


/// Takes a filename and generates a `HashMap<String, i64>` of all labels in the instructions and data
/// section and returns it. Will include paging (pages are 4Kb) to ensure data is on different page to
/// instructions. 
pub fn generate_label_table(tokens_stream:&Vec<FileTokens>) -> HashMap<String, i64> {
    let mut instr_addr = 0;
    let page_size = 0x1000;
    let mut data_addr:i64 = page_size;
    let mut label_table:HashMap<String, i64> = HashMap::new();
    for tokens in tokens_stream {
        match tokens {
            FileTokens::DataTokens(t) => {
                match &t.label {
                    Some(label) => {
                        let num_bytes:i64 = t.bytes.len().try_into().unwrap();
                        label_table.insert(label.to_owned(), data_addr);
                        data_addr += num_bytes;
                    },

                    None => data_addr += 1
                }
            },

            FileTokens::InstrTokens(t) => {
                match &t.label {
                    Some(label) => {
                        label_table.insert(label.to_owned(), instr_addr);
                        instr_addr += 1;
                        if instr_addr % page_size == 0 && instr_addr != 0 {
                            data_addr += page_size;
                        } 
                    },

                    None => {
                        instr_addr += 1;
                        if instr_addr % page_size == 0 && instr_addr != 0 {
                            data_addr += page_size;
                        } 
                    }
                }
            }
        };
    }

    label_table
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
        let label_table = label_table::generate_label_table(&tokens);

        assert_eq!(label_table.len(), 10);
        assert_eq!(label_table["init"], 0x0000);
        assert_eq!(label_table["loop"], 0x0005);
        assert_eq!(label_table["end"], 0x0014);
        assert_eq!(label_table["target"], 0x1000);
        assert_eq!(label_table["int_long"], 0x1001);
        assert_eq!(label_table["half_float"], 0x1003);
        assert_eq!(label_table["float"], 0x1004);
        assert_eq!(label_table["eszet"], 0x1006);
        assert_eq!(label_table["text_data"], 0x1007);
        assert_eq!(label_table["list"], 0x101B);
    }


    #[test]
    fn test_label_paging() {
        let tokens = process_file_into_tokens("test_files/test_large_prog.asm");
        let tokens = pseudo_substitution::substitute_pseudo_instrs(tokens);
        let label_table = label_table::generate_label_table(&tokens);

        assert_eq!(label_table.len(), 5);
        assert_eq!(label_table["start"], 0);
        assert_eq!(label_table["pg_end"], 0x0FFF);
        assert_eq!(label_table["pg_start"], 0x1000);
        assert_eq!(label_table["some_data"], 0x2000);
        assert_eq!(label_table["some_other_data"], 0x2001);
    }
}
