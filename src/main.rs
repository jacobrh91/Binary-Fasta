mod binary_fasta_data;
mod binary_fasta_section;
mod fasta_data;
mod fasta_section;

use std::{env, io};

use crate::{binary_fasta_data::BinaryFastaData, fasta_data::FastaData};

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    if args.len() < 2 {
        println!("Provide the path to a FASTA file");
    } else {
        let fasta_file = FastaData::read(&args[1])?;
        println!("Sections: {:?}", fasta_file);

        let basta_data = BinaryFastaData::from_fasta(fasta_file);
        println!("Binary: {:?}", basta_data);

        let basta_file_path = "data/test_small_dna.basta";
        basta_data.write(basta_file_path)?;

        let basta_data = BinaryFastaData::read(basta_file_path)?;
        println!("Out binary: {:?}", basta_data);

        let fasta_data = FastaData::from_basta(basta_data);
        println!("Sections: {:?}", fasta_data);

        fasta_data.write("data/test_small_dna.fasta")?;
    }

    Ok(())
}
