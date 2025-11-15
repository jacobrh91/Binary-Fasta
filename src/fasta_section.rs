use itertools::Itertools;

use crate::binary_fasta_section::BinaryFastaSection;

#[derive(Debug, PartialEq)]
pub struct FastaSection {
    pub descriptor: String,
    pub sequence: String,
}

impl FastaSection {
    pub fn new(descriptor: &str, fasta_data: &str) -> FastaSection {
        // Clean the descriptor
        let mut cleaned_descriptor = descriptor.trim_start_matches(">");
        cleaned_descriptor = cleaned_descriptor.trim_end_matches("\n");

        FastaSection {
            descriptor: cleaned_descriptor.to_string(),
            sequence: fasta_data.to_string(),
        }
    }

    pub fn from_basta(basta_section: BinaryFastaSection) -> Self {
        let char_sequence =
            Self::translate_from_binary(&basta_section.sequence, basta_section.sequence_length);

        FastaSection {
            descriptor: basta_section.descriptor.clone(),
            sequence: char_sequence,
        }
    }

    pub fn is_dna(&self) -> bool {
        for c in self.sequence.chars() {
            match c {
                'T' | 't' => return true,
                'U' | 'u' => return false,
                _ => (),
            }
        }
        // default to assuming it is DNA, which might as well be true if there
        // are no T's or U's in the sequence.
        true
    }

    pub fn translate_from_binary(bytes: &Vec<u8>, length: i32) -> String {
        let mut result = String::new();
        let mut chars_stored = 0;

        // Whether the sequence was DNA or RNA is encoded in the sign of the i32 length.
        let is_dna: bool = length >= 0;

        // Make it a positive number for arithmetic later in this function.
        let positive_length = if length < 0 { -length } else { length };

        for byte in bytes {
            let is_bit_set = |idx| Self::is_bit_set(byte, idx);

            let nucleotide_bits: [(bool, bool); 4] = [
                (is_bit_set(0), is_bit_set(1)),
                (is_bit_set(2), is_bit_set(3)),
                (is_bit_set(4), is_bit_set(5)),
                (is_bit_set(6), is_bit_set(7)),
            ];
            let nucleotide_chars = nucleotide_bits.map(|x| Self::decode_bits(x, is_dna));

            // If storing the final bit, and it is not full with sequence data.
            if positive_length - chars_stored < 4 {
                let nucleotides_in_last_byte: usize = (positive_length % 4).try_into().unwrap();
                result.extend(nucleotide_chars.iter().take(nucleotides_in_last_byte));
            } else {
                // Store all 4 nucleotides to the result list
                result.extend(nucleotide_chars);
                chars_stored += 4;
            }
        }
        result
    }

    fn is_bit_set(byte: &u8, bit_index: u8) -> bool {
        // Indexed from left to right, with the most-significant-bit as 0.
        if bit_index == 7 {
            // Bit is already in the least-significant-bit position.
            byte & 1 == 1
        } else {
            let bits_to_shift = 7 - bit_index;
            // Shift bit into the least-significant-bit, bitwise AND, and see
            // if bit is set.
            (byte >> bits_to_shift) & 1 == 1
        }
    }

    fn decode_bits(bits: (bool, bool), is_dna: bool) -> char {
        match bits {
            (false, false) => 'A',
            (false, true) => 'C',
            (true, false) => 'G',
            (true, true) => {
                if is_dna {
                    'T'
                } else {
                    'U'
                }
            }
        }
    }

    pub fn convert_to_bytes(&self) -> Vec<u8> {
        let mut result: Vec<u8> = Vec::new();
        // Add the header line
        result.push(b'>');
        result.extend(self.descriptor.as_bytes());
        result.push(b'\n');

        // Add the sequence section (broken up into 50 characters per line)
        // This works because the characters in the DNA/RNA sequence are ASCII characters,
        // meaning they all fit into a single byte.

        let sequence_bytes = self.sequence.clone();
        let sequence_bytes_iterator = &mut sequence_bytes.as_bytes().iter().peekable();

        let chars_per_line = 50;

        while sequence_bytes_iterator.peek().is_some() {
            let chunk = sequence_bytes_iterator.take(chars_per_line).collect_vec();
            let chunk_length = chunk.len();
            result.extend(chunk);
            // If the chunk is full of characters (and there are more characters to come)
            // add a newline character.
            if chunk_length == chars_per_line && sequence_bytes_iterator.peek().is_some() {
                result.push(b'\n')
            }
        }
        result
    }
}

#[cfg(test)]
mod test {

    #[test]
    fn translate_from_binary_dna() {
        let bytes = vec![0b0000_0000, 0b0101_0101, 0b1010_1010, 0b1111_1111];
        let length = 16i32;

        let expected = String::from("AAAACCCCGGGGTTTT");
        assert_eq!(
            super::FastaSection::translate_from_binary(&bytes, length),
            expected
        );
    }

    #[test]
    fn translate_from_binary_rna() {
        let bytes = vec![0b0000_0000, 0b0101_0101, 0b1010_1010, 0b1111_1111];
        let length = -16i32; // The value is negative because this sequence is RNA.

        let expected = String::from("AAAACCCCGGGGUUUU");
        assert_eq!(
            super::FastaSection::translate_from_binary(&bytes, length),
            expected
        );
    }
}
