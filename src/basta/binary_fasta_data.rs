use std::fs::File;
use std::io::{self, prelude::*, BufWriter};
use std::path::Path;

use crate::basta::binary_fasta_section::BinaryFastaSection;
use crate::errors::BinaryFastaError;
use crate::fasta::fasta_section::FastaSection;

pub fn from_fasta<I>(
    fasta_data: I,
) -> impl Iterator<Item = Result<BinaryFastaSection, BinaryFastaError>>
where
    I: Iterator<Item = Result<FastaSection, BinaryFastaError>>,
{
    fasta_data.map(|res| match res {
        Ok(section) => Ok(BinaryFastaSection::from_fasta(section)),
        Err(_) => Err(BinaryFastaError::UnexpectedEof),
    })
}

pub fn write<I>(iter: I, file_path: &Path) -> Result<(), BinaryFastaError>
where
    I: Iterator<Item = Result<BinaryFastaSection, BinaryFastaError>>,
{
    let mut writer = BufWriter::new(File::create(file_path)?);

    for section in iter {
        writer.write_all(&section?.convert_to_bytes())?;
    }

    writer.flush()?;
    Ok(())
}

pub fn read(
    file_path: &Path,
) -> Result<impl Iterator<Item = Result<BinaryFastaSection, BinaryFastaError>>, BinaryFastaError> {
    let file = File::open(file_path)?;
    let reader = io::BufReader::new(file);

    let mut bytes_iter = reader
        .bytes()
        .map(|res| res.expect("I/O error while reading bytes"))
        .peekable();

    Ok(std::iter::from_fn(move || {
        if bytes_iter.peek().is_some() {
            // Consume exactly one section from the byte stream
            let section = BinaryFastaSection::from_bytes(&mut bytes_iter);
            Some(section)
        } else {
            None
        }
    }))
}

#[cfg(test)]
mod tests {
    use crate::basta::binary_fasta_data;

    use super::*;

    #[test]
    fn test_from_fasta_dna() {
        let descr1 = "test 1";
        let descr2 = "test 2";

        let fasta_sections = vec![
            Ok(FastaSection {
                descriptor: String::from(descr1),
                sequence: String::from("AAAACCCCGGGGTTTT"),
            }),
            Ok(FastaSection {
                descriptor: String::from(descr2),
                sequence: String::from("ACGTCG"),
            }),
        ]
        .into_iter();

        let basta_vec: Vec<_> = binary_fasta_data::from_fasta(fasta_sections)
            .map(Result::unwrap)
            .collect();

        let expected = vec![
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
        ];
        assert_eq!(expected, basta_vec);
    }

    #[test]
    fn test_from_fasta_rna() {
        let descr1 = "test 1";
        let descr2 = "test 2";

        let fasta_sections = vec![
            Ok(FastaSection {
                descriptor: String::from(descr1),
                sequence: String::from("AAAACCCCGGGGUUUU"),
            }),
            Ok(FastaSection {
                descriptor: String::from(descr2),
                sequence: String::from("ACGUCG"),
            }),
        ]
        .into_iter();

        let basta_vec: Vec<_> = binary_fasta_data::from_fasta(fasta_sections)
            .map(Result::unwrap)
            .collect();

        let expected = vec![
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
        ];
        assert_eq!(expected, basta_vec);
    }
}
