use std::{
    fs::File,
    io::{self, BufRead, BufWriter, Write},
    path::Path,
};

use crate::{binary_fasta_data::BinaryFastaData, fasta_section::FastaSection};

#[derive(Debug, PartialEq)]
pub struct FastaData {
    pub sections: Vec<FastaSection>,
}

impl FastaData {
    pub fn from_basta(binary_fasta_data: BinaryFastaData) -> FastaData {
        let fasta_section = binary_fasta_data
            .sections
            .iter()
            .map(FastaSection::from_basta)
            .collect();
        FastaData {
            sections: fasta_section,
        }
    }

    pub fn read(file_path: &Path) -> io::Result<FastaData> {
        let file = File::open(file_path)?;
        let reader = io::BufReader::new(file);

        let mut fasta_sections: Vec<FastaSection> = Vec::new();

        // Gather data from fasta sections
        let mut fasta_descriptor: Option<String> = None;
        let mut fasta_data = String::new();

        for line_result in reader.lines() {
            let line = line_result?;
            // If the current line is a descriptor line
            if line.starts_with(">") {
                if let Some(descriptor) = fasta_descriptor {
                    fasta_sections.push(FastaSection::new(&descriptor, &fasta_data));
                }
                fasta_descriptor = Some(line);
                fasta_data = String::new();
            } else {
                fasta_data.push_str(line.trim_end_matches("\n"));
            }
        }
        // After all lines have been iterated through, store the final FASTA section.
        if let Some(descriptor) = fasta_descriptor {
            fasta_sections.push(FastaSection::new(&descriptor, &fasta_data));
        }
        Ok(FastaData {
            sections: fasta_sections,
        })
    }

    pub fn write(&self, file_path: &Path) -> io::Result<()> {
        let section_bytes: Vec<Vec<u8>> =
            self.sections.iter().map(|x| x.convert_to_bytes()).collect();

        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        for bytes in section_bytes {
            writer.write_all(&bytes)?;
            writer.write_all(b"\n")?;
        }
        writer.flush()?;
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use crate::binary_fasta_data::BinaryFastaData;
    use crate::binary_fasta_section::BinaryFastaSection;

    use super::*;

    #[test]
    fn test_is_dna() {
        let dna_section = FastaSection {
            descriptor: String::from("DNA"),
            sequence: String::from("AAAACCCCGGGGTTTT"),
        };
        let rna_section = FastaSection {
            descriptor: String::from("RNA"),
            sequence: String::from("AAAACCCCGGGGUUUU"),
        };

        assert!(dna_section.is_dna());
        assert!(!rna_section.is_dna());
    }

    #[test]
    fn test_from_fasta_rna() {
        let descr1 = "test 1";
        let descr2 = "test 2";

        let basta_data = BinaryFastaData {
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

        let fasta_data = FastaData::from_basta(basta_data);

        let expected = FastaData {
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
        assert_eq!(fasta_data, expected);
    }
}
