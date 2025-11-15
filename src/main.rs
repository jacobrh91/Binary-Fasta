mod binary_fasta_data;
mod binary_fasta_section;
mod errors;
mod fasta_data;
mod fasta_section;
mod nucleotide_file;
mod parser;

use std::{io, path::Path};

use clap::Parser;
use parser::Args;

use crate::{errors::BinaryFastaError, nucleotide_file::NucleotideFile};

fn main() -> Result<(), BinaryFastaError> {
    let cli = Args::parse();

    let input_path = Path::new(&cli.input);
    if !input_path.exists() {
        return Err(BinaryFastaError::Io(io::Error::new(
            io::ErrorKind::NotFound,
            format!("File '{}' not found.", input_path.display()),
        )));
    }

    let input_file: NucleotideFile = NucleotideFile::new(input_path)?;

    let output_file_option: Option<NucleotideFile> = cli
        .output
        .map(|output_str| NucleotideFile::new(Path::new(&output_str)))
        .transpose()?;
    let output_file: NucleotideFile =
        output_file_option.unwrap_or_else(|| input_file.switch_extension());

    match input_file.format {
        nucleotide_file::FileFormat::Fasta => {
            let read_fasta_iter = fasta_data::read(&input_file.file_path)?;
            let binary_iter = binary_fasta_data::from_fasta(read_fasta_iter);
            binary_fasta_data::write(binary_iter, &output_file.file_path)?;
        }
        nucleotide_file::FileFormat::Basta => {
            let read_basta_iter = binary_fasta_data::read(&input_file.file_path)?;
            let fasta_iter = fasta_data::from_basta(read_basta_iter);
            fasta_data::write(fasta_iter, &output_file.file_path)?;
        }
    }
    Ok(())
}
