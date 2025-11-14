use std::{
    ffi::OsStr,
    path::{Path, PathBuf},
};

#[derive(PartialEq, Debug)]
pub enum FileFormat {
    Fasta,
    Basta,
}

#[derive(PartialEq, Debug)]
pub struct NucleotideFile {
    pub format: FileFormat,
    pub file_path: PathBuf,
    pub long_extension: bool, // true if the extension is fasta/basta (false if it is fa/ba).
}

impl NucleotideFile {
    pub fn new(file_path: &Path) -> Result<NucleotideFile, String> {
        let path_str = file_path.to_str().expect("Empty path found.");
        let error_message = format!("Expected a FASTA or BASTA file, but found '{}'.", path_str);

        let ext = match file_path.extension().and_then(OsStr::to_str) {
            Some(ext) => ext,
            None => return Err(error_message),
        };

        let (format, long_extension) = match ext {
            "fasta" | "fa" => (FileFormat::Fasta, ext.len() > 2),
            "basta" | "ba" => (FileFormat::Basta, ext.len() > 2),
            _ => return Err(error_message),
        };

        Ok(NucleotideFile {
            format,
            file_path: file_path.to_path_buf(),
            long_extension,
        })
    }

    fn get_opposite_type(&self) -> FileFormat {
        match self.format {
            FileFormat::Fasta => FileFormat::Basta,
            FileFormat::Basta => FileFormat::Fasta,
        }
    }

    pub fn switch_extension(&self) -> NucleotideFile {
        let new_extension = {
            if self.format == FileFormat::Fasta {
                if self.long_extension {
                    "basta"
                } else {
                    "ba"
                }
            } else if self.long_extension {
                "fasta"
            } else {
                "fa"
            }
        };

        NucleotideFile {
            format: self.get_opposite_type(),
            file_path: self.file_path.with_extension(new_extension),
            long_extension: self.long_extension,
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use std::path::Path;

    use crate::nucleotide_file::NucleotideFile;

    #[test]
    fn test_get_opposite_type_fasta() {
        let expected = NucleotideFile {
            format: FileFormat::Fasta,
            file_path: Path::new("test.fasta").to_path_buf(),
            long_extension: true,
        };
        assert_eq!(expected.get_opposite_type(), FileFormat::Basta);
    }

    #[test]
    fn test_get_opposite_type_basta() {
        let expected = NucleotideFile {
            format: FileFormat::Basta,
            file_path: Path::new("test.basta").to_path_buf(),
            long_extension: true,
        };
        assert_eq!(expected.get_opposite_type(), FileFormat::Fasta);
    }

    #[test]
    fn test_new_fasta_long_extension() {
        let expected = NucleotideFile {
            format: FileFormat::Fasta,
            file_path: Path::new("test.fasta").to_path_buf(),
            long_extension: true,
        };

        assert_eq!(
            NucleotideFile::new(Path::new("test.fasta")).unwrap(),
            expected
        );
    }

    #[test]
    fn test_new_fasta_short_extension() {
        let expected = NucleotideFile {
            format: FileFormat::Fasta,
            file_path: Path::new("test.fa").to_path_buf(),
            long_extension: false,
        };

        assert_eq!(NucleotideFile::new(Path::new("test.fa")).unwrap(), expected);
    }

    #[test]
    fn test_new_basta_long_extension() {
        let expected = NucleotideFile {
            format: FileFormat::Basta,
            file_path: Path::new("test.basta").to_path_buf(),
            long_extension: true,
        };

        assert_eq!(
            NucleotideFile::new(Path::new("test.basta")).unwrap(),
            expected
        );
    }

    #[test]
    fn test_new_basta_short_extension() {
        let expected = NucleotideFile {
            format: FileFormat::Basta,
            file_path: Path::new("test.ba").to_path_buf(),
            long_extension: false,
        };

        assert_eq!(NucleotideFile::new(Path::new("test.ba")).unwrap(), expected);
    }

    #[test]
    fn test_new_fasta_switch_extension() {
        let expected = NucleotideFile {
            format: FileFormat::Fasta,
            file_path: Path::new("test.fasta").to_path_buf(),
            long_extension: true,
        };

        assert_eq!(
            expected.switch_extension().file_path,
            Path::new("test.basta").to_path_buf()
        );
    }

    #[test]
    fn test_new_fasta_switch_extension_nested_path() {
        let expected = NucleotideFile {
            format: FileFormat::Fasta,
            file_path: Path::new("/path/to/test.fasta").to_path_buf(),
            long_extension: true,
        };

        assert_eq!(
            expected.switch_extension().file_path,
            Path::new("/path/to/test.basta").to_path_buf()
        );
    }
}
