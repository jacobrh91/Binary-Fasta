use std::fs::{self, File};
use std::io::{self, prelude::*, BufWriter};

use crate::binary_fasta_section::BinaryFastaSection;
use crate::fasta_data::FastaData;

#[derive(Debug)]
pub struct BinaryFastaData {
    pub sections: Vec<BinaryFastaSection>,
}

impl BinaryFastaData {
    pub fn from_fasta(fasta_data: FastaData) -> BinaryFastaData {
        let binary_fasta_section = fasta_data
            .sections
            .iter()
            .map(BinaryFastaSection::new)
            .collect();
        BinaryFastaData {
            sections: binary_fasta_section,
        }
    }

    pub fn write(&self, file_path: &str) -> io::Result<()> {
        let bytes = self.sections.iter().flat_map(|x| x.convert_to_bytes());

        let file = File::create(file_path)?;
        let mut writer = BufWriter::new(file);
        for byte in bytes {
            writer.write_all(&[byte])?;
        }
        writer.flush()?;
        Ok(())
    }

    pub fn read(file_path: &str) -> io::Result<BinaryFastaData> {
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
