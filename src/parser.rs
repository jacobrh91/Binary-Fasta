use clap::{arg, command, Parser};

fn parse_input(s: &str) -> Result<String, String> {
    if s.ends_with(".fa") || s.ends_with(".fasta") || s.ends_with(".ba") || s.ends_with(".basta") {
        Ok(s.to_string())
    } else {
        Err(format!("Invalid input file '{}'. Input file must end with .fa/.fasta for FASTA files, or .ba/.basta for binary FASTA files.", s))
    }
}

#[derive(Parser, Debug)]
#[command(version, about = "Binary FASTA encoder/decoder")]
pub struct Args {
    #[arg(short = 'i', long="input", required=true, value_parser = parse_input, value_name = "file to convert")]
    pub input: String,
    #[arg(value_name = "output")]
    pub output: Option<String>,
}
