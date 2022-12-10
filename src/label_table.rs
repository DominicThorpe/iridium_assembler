use std::collections::HashMap;
use crate::token_types::FileTokens;


/// Takes a filename and generates a `HashMap<String, i64>` of all labels in the instructions and data
/// section and returns it.
pub fn generate_label_table(tokens_stream:&Vec<FileTokens>) -> HashMap<String, i64> {
    let mut instr_addr = 0;
    let mut data_addr:i64 = 0x0000_0800;
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
                    },

                    None => instr_addr += 1
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
        assert_eq!(label_table["init"], 0x0000_0000);
        assert_eq!(label_table["loop"], 0x0000_0005);
        assert_eq!(label_table["end"], 0x0000_0014);
        assert_eq!(label_table["target"], 0x00C0_0000);
        assert_eq!(label_table["int_long"], 0x00C0_0001);
        assert_eq!(label_table["half_float"], 0x00C0_0003);
        assert_eq!(label_table["float"], 0x00C0_0004);
        assert_eq!(label_table["eszet"], 0x00C0_0006);
        assert_eq!(label_table["text_data"], 0x00C0_0007);
        assert_eq!(label_table["list"], 0x00C0_001B);
    }
}
