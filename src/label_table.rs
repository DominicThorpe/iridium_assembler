use std::collections::HashMap;
use crate::FileTokens;


/// Takes a filename and generates a `HashMap<String, i64>` of all labels in the instructions section
/// and returns it.
pub fn generate_instr_label_table(tokens_stream:Vec<FileTokens>) -> HashMap<String, i64> {
    let mut instr_addr = 0;
    let mut data_addr = 0x0010_0000;
    let mut label_table:HashMap<String, i64> = HashMap::new();
    for tokens in tokens_stream {
        match tokens {
            FileTokens::DataTokens(t) => {
                match t.label {
                    Some(label) => {
                        label_table.insert(label, data_addr);
                        data_addr += 1;
                    },

                    None => data_addr += 1
                }
            },

            FileTokens::InstrTokens(t) => {
                match t.label {
                    Some(label) => {
                        label_table.insert(label, instr_addr);
                        instr_addr += 1;
                    },

                    None => instr_addr += 1
                }
            }
        };
    }

    label_table
}
