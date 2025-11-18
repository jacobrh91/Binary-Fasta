use assert_cmd::Command;
use std::error::Error;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn normalize_fasta_text(s: &str) -> &str {
    s.trim_end_matches(&['\n', '\r'][..])
}

fn roundtrip_fasta(input: &Path) -> Result<(), Box<dyn Error>> {
    let tmp_dir = tempdir()?;
    let basta_path = tmp_dir.path().join("out.basta");
    let roundtrip_fasta_path = tmp_dir.path().join("roundtrip.fasta");

    // FASTA to BASTA
    Command::new(assert_cmd::cargo::cargo_bin!("binary_fasta"))
        .arg("--input")
        .arg(input)
        .arg("--output")
        .arg(&basta_path)
        .assert()
        .success();

    // BASTA (back to) FASTA
    Command::new(assert_cmd::cargo::cargo_bin!("binary_fasta"))
        .arg("--input")
        .arg(&basta_path)
        .arg("--output")
        .arg(&roundtrip_fasta_path)
        .assert()
        .success();

    let original = fs::read_to_string(input)?;
    let roundtrip = fs::read_to_string(&roundtrip_fasta_path)?;

    let original_normalized = normalize_fasta_text(&original);
    let roundtrip_normalized = normalize_fasta_text(&roundtrip);

    assert_eq!(
        original_normalized, roundtrip_normalized,
        "round-tripped FASTA does not match original for {:?}",
        input
    );

    Ok(())
}

#[test]
fn roundtrip_small_dna() -> Result<(), Box<dyn Error>> {
    roundtrip_fasta(Path::new("tests/data/small_dna.fasta"))
}

#[test]
fn roundtrip_small_rna() -> Result<(), Box<dyn Error>> {
    roundtrip_fasta(Path::new("tests/data/small_rna.fasta"))
}

#[test]
fn roundtrip_multiline_section() -> Result<(), Box<dyn Error>> {
    roundtrip_fasta(Path::new("tests/data/multiline_section.fasta"))
}

#[test]
fn malformed_fasta_should_fail() -> Result<(), Box<dyn Error>> {
    Command::new(assert_cmd::cargo::cargo_bin!("binary_fasta"))
        .arg("--input")
        .arg("tests/data/malformed.fasta")
        .assert()
        .failure()
        .code(1)
        .stderr(predicates::str::contains(
            r#"MalformedFastaHeader { path: "tests/data/malformed.fasta" }"#,
        ));
    Ok(())
}
