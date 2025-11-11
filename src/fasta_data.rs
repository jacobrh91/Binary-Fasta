use std::{
    fs::File,
    io::{self, BufRead, BufWriter, Write},
};

use crate::{binary_fasta_data::BinaryFastaData, fasta_section::FastaSection};

#[derive(Debug)]
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

    pub fn is_dna(&self) -> bool {
        for section in &self.sections {
            for c in section.sequence.chars() {
                match c {
                    'T' | 't' => return true,
                    'U' | 'u' => return false,
                    _ => (),
                }
            }
        }
        // default to assuming it is DNA, which might as well be true if there
        // are no T's or U's in any of the sequences.
        true
    }

    pub fn read(filepath: &str) -> io::Result<FastaData> {
        if !filepath.ends_with(".fasta") && !filepath.ends_with(".fa") {
            panic!("Invalid fasta file '{}'.", filepath)
        }
        let file = File::open(filepath)?;
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

    pub fn write(&self, file_path: &str) -> io::Result<()> {
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
