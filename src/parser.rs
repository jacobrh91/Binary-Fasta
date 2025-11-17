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
    #[arg(short = 'o', long = "output", value_name = "output file")]
    pub output: Option<String>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use clap::Parser;

    #[test]
    fn parses_with_input_and_output() {
        let args = Args::parse_from(["test-bin", "-i", "input.fa", "-o", "output.ba"]);

        assert_eq!(args.input, "input.fa");
        assert_eq!(args.output.as_deref(), Some("output.ba"));
    }

    #[test]
    fn parses_with_input_only_and_no_output() {
        let args = Args::parse_from(["test-bin", "--input", "sequence.fasta"]);

        assert_eq!(args.input, "sequence.fasta");
        assert!(args.output.is_none());
    }
}
