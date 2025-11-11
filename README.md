Downloaded hs1.fa.gz from
https://genome.ucsc.edu/cgi-bin/hgGateway

Fasta format

>Description of file
ACTGATCG

Plan
File.basta will contain

A encoded as 00

C encoded as 01

G encoded as 10

T (or U) encoded as 11.

RNA has Uracil (U), DNA has Thymine (T). I am making the assumption that you know if you have RNA or DNA data in your fasta file.

Finally, compare this technique with Genozip.

Goal
-----
How small does a fasta file get if you encode the data in a binary format?
If an ASCII character is 1 byte, but I encode the same info in 2 bits
(1/4 of a byte), the data should get 75% smaller.

# File layout

| Category             | Size          | Purpose                                           |
|----------------------|---------------|---------------------------------------------------|
| Descriptor length    | 1 byte        | Where to start reading the sequence               |
| Sequence length      | 4 bytes       | When to stop reading the sequence                 |
|                      |               | The sign bit stores if the sequence is DNA or RNA |
| Sequence descriptor  | Variable      | Stores the sequence descriptor as UTF-8           |
| Nucleotide sequence  | Variable      | The 2-bit-encoded nucleotide sequence             |

This pattern repeats for all sequences in the FASTA file.

## Descriptor length

Assumptions:
  1. Less than or equal to 80 characters (including the '>' character)
  2. No '\n' or \r\n' characters (not sure if I even care about this, but I'm making the assumption anyway.)

## Sequence length

The first 4 bytes is the length of the nucleotide sequence, stored as a signed 32-bit integer. The sign bit shows whether the original sequence was DNA (positive) or RNA (negative).

This allows for a maximum sequence length of 
2^31 - 1 = 2,147,483,647 nucleotides.


## Sequence descriptor

The descriptor text stored as ASCII.


## Nucleotide sequence

The nucleotide sequence encoded in a 2-bit encoding.




Line with '>' character is stored as is.
Next line stores the number of nucleotides that make up the subsequent sequence
stores as an unsigned 64-bit integer.





# Ignoring
  * Error correction codes
  * More complicated base labeling scenarios, such as N, R, Y, K, M, etc.
  * Nucleotide casing
  * What if my fasta file is invalid (well... fix it and try again)


Storage format