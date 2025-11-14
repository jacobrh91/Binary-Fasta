mod binary_fasta_data;
mod binary_fasta_section;
mod fasta_data;
mod fasta_section;
mod nucleotide_file;
mod parser;

use std::{error::Error, path::Path};

use clap::Parser;
use parser::Args;

use crate::{
    binary_fasta_data::BinaryFastaData, fasta_data::FastaData, nucleotide_file::NucleotideFile,
};

fn main() -> Result<(), Box<dyn Error>> {
    let cli = Args::parse();

    let input_path = Path::new(&cli.input);
    if !input_path.exists() {
        return Err(format!("File '{}' not found.", input_path.to_str().unwrap_or("")).into());
    }
    let input_file: NucleotideFile = NucleotideFile::new(input_path)?;

    let output_file_option: Option<NucleotideFile> = cli
        .output
        .map(|output_str| NucleotideFile::new(Path::new(&output_str)))
        .transpose()?;
    let output_file: NucleotideFile = output_file_option.unwrap_or(input_file.switch_extension());

    match input_file.format {
        nucleotide_file::FileFormat::Fasta => {
            let fasta = FastaData::read(&input_file.file_path)?;
            let binary = BinaryFastaData::from_fasta(fasta);
            binary.write(&output_file.file_path)?;
        }
        nucleotide_file::FileFormat::Basta => {
            let basta = BinaryFastaData::read(&input_file.file_path)?;
            let fasta = FastaData::from_basta(basta);
            fasta.write(&output_file.file_path)?;
        }
    }
    Ok(())
}
