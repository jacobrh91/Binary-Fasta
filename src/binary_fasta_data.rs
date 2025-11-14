use std::fs::{self, File};
use std::io::{self, prelude::*, BufWriter};
use std::path::Path;

use crate::binary_fasta_section::BinaryFastaSection;
use crate::fasta_data::FastaData;

#[derive(Debug, PartialEq)]
pub struct BinaryFastaData {
    pub sections: Vec<BinaryFastaSection>,
}

impl BinaryFastaData {
    pub fn from_fasta(fasta_data: FastaData) -> BinaryFastaData {
        let binary_fasta_section = fasta_data
            .sections
            .iter()
            .map(BinaryFastaSection::from_fasta)
            .collect();
        BinaryFastaData {
            sections: binary_fasta_section,
        }
    }

    pub fn write(&self, file_path: &Path) -> io::Result<()> {
        let bytes = self.sections.iter().flat_map(|x| x.convert_to_bytes());

        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        for byte in bytes {
            writer.write_all(&[byte])?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn read(file_path: &Path) -> io::Result<BinaryFastaData> {
        let file = fs::read(file_path)?;

        let mut iter = file.into_iter().peekable();
        let mut sections: Vec<BinaryFastaSection> = Vec::new();

        while iter.peek().is_some() {
            let new_section = BinaryFastaSection::from_bytes(&mut iter);
            sections.push(new_section);
        }
        Ok(BinaryFastaData { sections })
    }
}

#[cfg(test)]
mod tests {
    use crate::fasta_section::FastaSection;

    use super::*;

    #[test]
    fn test_from_fasta_dna() {
        let descr1 = "test 1";
        let descr2 = "test 2";

        let fasta_data = FastaData {
            sections: vec![
                FastaSection {
                    descriptor: String::from(descr1),
                    sequence: String::from("AAAACCCCGGGGTTTT"),
                },
                FastaSection {
                    descriptor: String::from(descr2),
                    sequence: String::from("ACGTCG"),
                },
            ],
        };

        let basta_data = BinaryFastaData::from_fasta(fasta_data);

        let expected = BinaryFastaData {
            sections: vec![
                BinaryFastaSection {
                    descriptor: String::from(descr1),
                    sequence: vec![0b0000_0000, 0b0101_0101, 0b1010_1010, 0b1111_1111],
                    sequence_length: 16i32,
                },
                BinaryFastaSection {
                    descriptor: String::from(descr2),
                    sequence: vec![0b0001_1011, 0b0110_0000],
                    sequence_length: 6i32,
                },
            ],
        };
        assert_eq!(basta_data, expected);
    }

    #[test]
    fn test_from_fasta_rna() {
        let descr1 = "test 1";
        let descr2 = "test 2";

        let fasta_data = FastaData {
            sections: vec![
                FastaSection {
                    descriptor: String::from(descr1),
                    sequence: String::from("AAAACCCCGGGGUUUU"),
                },
                FastaSection {
                    descriptor: String::from(descr2),
                    sequence: String::from("ACGUCG"),
                },
            ],
        };

        let basta_data = BinaryFastaData::from_fasta(fasta_data);
        let expected = BinaryFastaData {
            sections: vec![
                BinaryFastaSection {
                    descriptor: String::from(descr1),
                    sequence: vec![0b0000_0000, 0b0101_0101, 0b1010_1010, 0b1111_1111],
                    sequence_length: -16i32, // Length is negative because sign bit signals DNA (+) or RNA (-)
                },
                BinaryFastaSection {
                    descriptor: String::from(descr2),
                    sequence: vec![0b0001_1011, 0b0110_0000],
                    sequence_length: -6i32, // Length is negative because sign bit signals DNA (+) or RNA (-)
                },
            ],
        };
        assert_eq!(basta_data, expected);
    }
}
