use itertools::Itertools;

use crate::{errors::BinaryFastaError, fasta::fasta_section::FastaSection};

#[derive(Debug, PartialEq)]
pub struct BinaryFastaSection {
    pub descriptor: String,
    // Sequence of bytes holding the 2-bit nucleotide encoding
    // Where A = 00; C = 01; G = 10; T/U = 11
    pub sequence: Vec<u8>,
    // Need the exact number, because the last byte written may not be full.
    // The sign represents whether the original sequence was DNA (1) or RNA (0).
    pub sequence_length: i32,
}

impl BinaryFastaSection {
    fn get_descriptor_byte_length(&self) -> u8 {
        u8::try_from(self.descriptor.len()).expect("Descriptor is too long.")
    }

    pub fn from_fasta(fasta_section: FastaSection) -> Self {
        // The sign bit signals whether the source data was DNA (+) or RNA (-)
        let sign = if fasta_section.is_dna() { 1 } else { -1 };
        let sequence_length = sign
            * i32::try_from(fasta_section.sequence.chars().count()).expect("Sequence is too long.");

        BinaryFastaSection {
            descriptor: fasta_section.descriptor,
            sequence: BinaryFastaSection::translate_to_binary(&fasta_section.sequence),
            sequence_length,
        }
    }

    pub fn from_bytes(
        byte_stream: &mut impl Iterator<Item = u8>,
    ) -> Result<BinaryFastaSection, BinaryFastaError> {
        // Read descriptor length from 1st byte.
        let descriptor_length = byte_stream.next().ok_or(BinaryFastaError::UnexpectedEof)?;

        // Next 4 bytes contain a signed 32-bit integer.
        // Sign represents whether the sequence is DNA, and magnitude is the sequence length.
        let seq_len_vec: Vec<u8> = byte_stream.take(4).collect();
        if seq_len_vec.len() != 4 {
            return Err(BinaryFastaError::UnexpectedEof);
        }

        let sequence_length = {
            let mut arr = [0u8; 4];
            arr.copy_from_slice(&seq_len_vec); // safe because length == 4
            i32::from_be_bytes(arr)
        };

        // Read descriptor bytes
        let description_vector: Vec<u8> = byte_stream.take(descriptor_length.into()).collect();
        if description_vector.len() != descriptor_length as usize {
            return Err(BinaryFastaError::UnexpectedEof);
        }

        let descriptor = String::from_utf8(description_vector)
            .map_err(|_| BinaryFastaError::InvalidUtf8Descriptor)?;

        // The sequence has 4 nucleotides per byte, so divide by 4, but get 1
        // more byte if the length is not divisible by 4 (there is a final
        // byte that is partially filled with nucleotide data.)
        let sequence_bytes: usize =
            ((sequence_length / 4) + if sequence_length % 4 != 0 { 1 } else { 0 }) as usize;

        let sequence: Vec<u8> = byte_stream.take(sequence_bytes).collect();
        if sequence.len() != sequence_bytes {
            return Err(BinaryFastaError::UnexpectedEof);
        }

        Ok(BinaryFastaSection {
            descriptor,
            sequence,
            sequence_length,
        })
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        let mut bytes = Vec::<u8>::new();

        // 1st byte: descriptor length
        bytes.push(self.get_descriptor_byte_length());

        // Next 4 bytes: 2-bit sequence length as i32
        // the sign represents whether the data was DNA or RNA
        // Bytes are big-endian.
        bytes.extend_from_slice(&self.sequence_length.to_be_bytes());

        // Next bytes are the descriptor text.
        bytes.extend_from_slice(self.descriptor.as_bytes());

        // Final bytes are the sequence bits.
        bytes.extend_from_slice(&self.sequence);
        bytes
    }

    pub fn translate_to_binary(sequence: &str) -> Vec<u8> {
        // Convert 4 utf-8 characters to 1 byte of binary data with 2-bits per nucleotide
        let mut results: Vec<u8> = Vec::new();

        for chunk in &sequence.chars().chunks(4) {
            let chars: Vec<char> = chunk.collect();
            let mut current_position = 0; // Manipulate bits from left to right
            let mut binary_data: u8 = 0b0000_0000;

            for c in chars {
                match c.to_ascii_uppercase() {
                    'A' => {
                        // 00 (no bits to flip)
                    }
                    'C' => {
                        // 01 (flip the second bit)
                        binary_data += 2u8.pow(7 - current_position - 1)
                    }
                    'G' => {
                        // 10 (flip the first bit)
                        binary_data += 2u8.pow(7 - current_position)
                    }
                    'T' | 'U' => {
                        // 11 (flip both bits)
                        binary_data = binary_data
                            + 2u8.pow(7 - current_position)
                            + 2u8.pow(7 - current_position - 1)
                    }
                    _ => panic!("Invalid sequence character '{}'", c),
                }
                // Move to the next 2 bits
                current_position += 2;
            }
            results.push(binary_data)
        }
        results
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_ascii_to_binary_translations() {
        assert_eq!(BinaryFastaSection::translate_to_binary(""), vec!());

        assert_eq!(
            BinaryFastaSection::translate_to_binary("a"),
            vec!(0b0000_0000)
        );
        assert_eq!(
            BinaryFastaSection::translate_to_binary("c"),
            vec!(0b0100_0000)
        );
        assert_eq!(
            BinaryFastaSection::translate_to_binary("g"),
            vec!(0b1000_0000)
        );
        assert_eq!(
            BinaryFastaSection::translate_to_binary("t"),
            vec!(0b1100_0000)
        );
        assert_eq!(
            BinaryFastaSection::translate_to_binary("u"),
            vec!(0b1100_0000)
        );

        assert_eq!(
            BinaryFastaSection::translate_to_binary("acgt"),
            vec!(0b0001_1011)
        );
        assert_eq!(
            BinaryFastaSection::translate_to_binary("TGCA"),
            vec!(0b1110_0100)
        );

        assert_eq!(
            BinaryFastaSection::translate_to_binary("aaAAccCCggGGttTTuuUU"),
            vec!(
                0b0000_0000,
                0b0101_0101,
                0b1010_1010,
                0b1111_1111,
                0b1111_1111
            )
        );
    }

    #[test]
    fn test_from_fasta_dna() {
        let descr1 = "test 1";

        let fasta_section = FastaSection {
            descriptor: String::from(descr1),
            sequence: String::from("TtTtGgGgCcAaAaCc"),
        };

        let expected = BinaryFastaSection {
            descriptor: String::from(descr1),
            sequence: vec![0b1111_1111, 0b1010_1010, 0b0101_0000, 0b0000_0101],
            sequence_length: 16i32,
        };
        assert_eq!(BinaryFastaSection::from_fasta(fasta_section), expected);
    }
}
