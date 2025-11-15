use std::error::Error;
use std::fmt;
use std::io;
use std::path::PathBuf;

#[derive(Debug)]
pub enum BinaryFastaError {
    UnexpectedEof,
    InvalidUtf8Descriptor,
    Io(io::Error), // Wraps general IO errors
    InvalidFileExtension { path: PathBuf },
}

impl fmt::Display for BinaryFastaError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            BinaryFastaError::UnexpectedEof => write!(f, "unexpected end of file stream."),
            BinaryFastaError::InvalidUtf8Descriptor => {
                write!(f, "descriptor contains invalid UTF-8.")
            }
            BinaryFastaError::Io(e) => write!(f, "I/O error: {}", e),
            BinaryFastaError::InvalidFileExtension { path } => write!(
                f,
                "expected a FASTA (.fa/.fasta) or BASTA (.ba/.basta) file, but found '{}'.",
                path.display()
            ),
        }
    }
}

impl Error for BinaryFastaError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            BinaryFastaError::Io(e) => Some(e), // chain underlying error
            _ => None,
        }
    }
}

// Also io::Error to be converted into a BinaryFastaError
impl From<io::Error> for BinaryFastaError {
    fn from(e: io::Error) -> Self {
        BinaryFastaError::Io(e)
    }
}
