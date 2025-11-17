use std::{
    fs::File,
    io::{self, BufRead, BufWriter, Read, Write},
    path::Path,
};

use crate::{
    basta::binary_fasta_section::BinaryFastaSection, errors::BinaryFastaError,
    fasta::fasta_section::FastaSection,
};

pub fn from_basta<I>(
    binary_fasta_data: I,
) -> impl Iterator<Item = Result<FastaSection, BinaryFastaError>>
where
    I: Iterator<Item = Result<BinaryFastaSection, BinaryFastaError>>,
{
    binary_fasta_data.map(|res| res.map(FastaSection::from_basta))
}

pub fn read(
    file_path: &Path,
) -> Result<impl Iterator<Item = Result<FastaSection, BinaryFastaError>>, BinaryFastaError> {
    validate_fasta(file_path)?;

    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);
    let mut lines = reader.lines();

    let mut description: Option<String> = None;
    let mut data = String::new();

    Ok(std::iter::from_fn(move || {
        for line_result in lines.by_ref() {
            match line_result {
                Ok(mut line) => {
                    // lines() strips '\n' but not '\r' from files created on PC
                    if line.ends_with('\r') {
                        line.pop();
                    }

                    if line.starts_with('>') {
                        if let Some(prev) = description.replace(line) {
                            let section = FastaSection::new(&prev, &data);
                            data.clear();
                            return Some(Ok(section));
                        } else {
                            data.clear(); // first header encountered
                        }
                    } else {
                        data.push_str(&line);
                    }
                }
                Err(e) => {
                    // Map io::Error into BinaryFastaError
                    return Some(Err(e.into()));
                }
            }
        }

        // EOF: flush any pending section
        description.take().map(|d| Ok(FastaSection::new(&d, &data)))
    }))
}

fn validate_fasta(file_path: &Path) -> Result<(), BinaryFastaError> {
    {
        let mut f = File::open(file_path)?;
        let mut first = [0u8; 1];

        let n = f.read(&mut first)?;
        if n == 0 {
            return Err(BinaryFastaError::UnexpectedEof);
        }
        if first[0] != b'>' {
            return Err(BinaryFastaError::MalformedFastaHeader {
                path: file_path.to_path_buf(),
            });
        }
        Ok(())
    }
}

pub fn write<I>(iter: I, file_path: &Path) -> Result<(), BinaryFastaError>
where
    I: Iterator<Item = Result<FastaSection, BinaryFastaError>>,
{
    let file = File::create(file_path)?;
    let mut writer = BufWriter::new(file);

    for section_res in iter {
        let section = section_res?;
        let section_bytes: Vec<u8> = section.convert_to_bytes();
        writer.write_all(&section_bytes)?;
        writer.write_all(b"\n")?;
    }
    // Flush any bytes left in the buffer after the last section is written
    writer.flush()?;
    Ok(())
}

#[cfg(test)]
mod test {
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

        let basta_sections = vec![
            Ok(BinaryFastaSection {
                descriptor: String::from(descr1),
                sequence: vec![0b0000_0000, 0b0101_0101, 0b1010_1010, 0b1111_1111],
                sequence_length: -16i32, // Length is negative because sign bit signals DNA (+) or RNA (-)
            }),
            Ok(BinaryFastaSection {
                descriptor: String::from(descr2),
                sequence: vec![0b0001_1011, 0b0110_0000],
                sequence_length: -6i32, // Length is negative because sign bit signals DNA (+) or RNA (-)
            }),
        ]
        .into_iter();

        let fasta_vec: Vec<_> = from_basta(basta_sections).map(Result::unwrap).collect();

        let expected = vec![
            FastaSection {
                descriptor: String::from(descr1),
                sequence: String::from("AAAACCCCGGGGUUUU"),
            },
            FastaSection {
                descriptor: String::from(descr2),
                sequence: String::from("ACGUCG"),
            },
        ];
        assert_eq!(fasta_vec, expected);
    }
}
