use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::BufReader;
use std::io::prelude::*;

use crate::validation::validate_opcode;


/// Takes a filename and generates a `HashMap<String, i64>` of all labels in the instructions section
/// and returns it.
pub fn generate_instr_label_table(filename:String) -> HashMap<String, i64> {
    let mut label_table:HashMap<String, i64> = HashMap::new();
    let input_file = BufReader::new(OpenOptions::new().read(true).open(filename).unwrap());

    let mut line_num = 0;
    for line_buffer in input_file.lines() {
        let line = line_buffer.unwrap();
        if line.trim().is_empty() {
            continue;
        }

        match line.find(":") {
            Some(index) => {
                let label = line[..index].to_owned();
                if label == "data" {
                    break;
                }

                match validate_opcode(&line) {
                    Ok(_) => { // this line has an instruction
                        label_table.insert(label, line_num);
                    },

                    Err(_) => { // this line is just a label
                        label_table.insert(label, line_num);
                    }
                }
            },

            None => { // no label detected
                line_num += 1;
            }
        }
    }

    label_table
}


#[cfg(test)]
mod tests {
    use crate::label_table::*;


    #[test]
    #[ignore]
    fn test_detect_label() {
        let label_table = generate_instr_label_table("test_files/test_detect_label.asm".to_owned());

        assert!(label_table.contains_key("init"));
        assert!(label_table.contains_key("loop"));
        assert!(label_table.contains_key("end"));
        assert!(!label_table.contains_key("data"));

        assert_eq!(label_table["init"], 0);
        assert_eq!(label_table["loop"], 3);
        assert_eq!(label_table["end"], 10);
        assert_eq!(label_table.len(), 3);
    }
}
