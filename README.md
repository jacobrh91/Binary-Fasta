# Binary Fasta

Binary Fasta is a utility for compressing nucleotide FASTA files by encoding sequence data using a 2-bit representation. This reduces file size while preserving exact sequence information.

## Background


### RNA, DNA, and nucleotides

**DNA** (Deoxyribonucleic Acid) and **RNA** (Ribonucleic Acid) are long biological molecules that store genetic information.
They are built from repeating units called nucleotides, each containing a sugar, a phosphate group, and one of four bases:
  * DNA bases: A (Adenine), C (Cytosine), G (Guanine), T (Thymine)
  * RNA bases: A (Adenine), C (Cytosine), G (Guanine), U (Uracil)

These bases form the “letters” of genomic sequences.

## What is the FASTA format?

FASTA is a text-based format for biological sequences. Each entry contains:

* A descriptor line, starting with `>`
  * Example: 

    ```>human chr 1```

* A sequence consisting of `A`, `C`, `G`, `T/U`
  * Example: 

    ```ACGTACGTACGT...```

FASTA is human-readable but inefficient for large genomes because every nucleotide is stored as a full 1-byte ASCII character.

### Why encode nucleotides in a binary format?

Because each nucleotide is one of only four values (A, C, G, T/U), it can be represented by 2 bits:


| Nucleotide | Type     | ASCII Representation | 2-bit Encoding |
| ---------- | -------- | -------------------- | -------------- |
| **A**      | DNA/RNA  | `'A'`                | `00`           |
| **C**      | DNA/RNA  | `'C'`                | `01`           |
| **G**      | DNA/RNA  | `'G'`                | `10`           |
| **T**      | DNA only | `'T'`                | `11`           |
| **U**      | RNA only | `'U'`                | `11`           |

This compression dramatically reduces the storage required for large genomic sequences while preserving the original information exactly.

## How to install

Follow the instructions <a href=https://www.rust-lang.org/tools/install>here</a> to install Rust and Cargo (Rust's package manager).

Clone this repository with `git clone https://github.com/jacobrh91/Binary-Fasta.git`

Compile and run the program with the following:

```
# Move inside the project's directory
cd Binary-Fasta/

# Compile a release build
cargo build --release

# Copy the executable into the current directory
cp target/release/binary .

# Run the program
./binary_fasta
```

## Examples

### To print the help menu

```
./binary_fasta --help
```

```
Arguments:
  [output (optional)]  

Options:
  -i, --input <file to convert>  
  -h, --help                     Print help
  -V, --version                  Print version
```

### Convert FASTA to BASTA

#### Have the program infer the output path from the input: 
```./binary_fasta --input /path/to/my_file.fasta```

Which writes a binary fasta file to 
```/path/to/my_file.basta```.

#### (Optionally) pass in the output path explicitly

```./binary_fasta --input /path/to/my_file.fasta --output /path/to/other.basta```

### Convert BASTA to FASTA

#### Have the program infer the output path from the input: 
```./binary_fasta --input /path/to/my_file.basta```

Which writes a regular fasta file to 
```/path/to/my_file.fasta```.

#### (Optionally) pass in the output path explicitly

```./binary_fasta --input /path/to/my_file.bfasta --output /path/to/other.fasta```

## Appendix

### Implementation details

#### Binary Fasta (.basta) file layout

| Field             | Size     | Meaning                                                             |
| ----------------- | -------- | ------------------------------------------------------------------- |
| Descriptor length | 1 byte   | Number of bytes that make up the sequence’s UTF-8 descriptor        |
| Sequence length   | 4 bytes  | Number of nucleotides in the sequence (sign bit encodes DNA vs RNA) |
| Descriptor        | Variable | UTF-8-encoded sequence description (e.g., FASTA header)             |
| Encoded sequence  | Variable | Sequence encoded at 2 bits per nucleotide                           |

This pattern repeats for all sequences in the FASTA file.

#### File size reduction

Obviously, storing 4 nucleotides per byte vs a single nucleotide will reduce the
sequence size by 75% (note the description line for each FASTA section also has to be stored, so the compression on the entire FASTA file may be less than a 75% reduction).

However, there are more general compression tools, such as gzip, that are widely used
when transferring large files over a network. As a test, I downloaded the human genome in FASTA format, and experimented with layering different compression and seeing how small the file would get. 

(The 'hs1' human reference genome was downloaded from
https://hgdownload.gi.ucsc.edu/goldenPath/hs1/bigZips/)


| Format                | Size     | Ratio vs FASTA | Space Saved | Bits / nt |
| --------------------- | -------- | -------------- | ----------- | --------- |
| **FASTA**             | 3.180 GB | 1.00×          | —           | 8.00      |
| **FASTA.gz**          | 974.8 MB | 3.26×          | 69.4%       | ~2.45     |
| **BA (Binary FASTA)** | 779.3 MB | 4.08×          | 75.5%       | 2.00      |
| **BA.gz**             | 698.2 MB | 4.55×          | 78.0%       | ~1.79     |

#### Why does .basta sometimes exceed a 4× ratio?

You might expect exactly a 4× improvement (8 bits → 2 bits). But .basta performed slightly better.
Reasons:

The dataset contains only a tiny number of FASTA headers compared to billions of bases: Headers are negligible.

FASTA stores sequences with newlines every 50 characters.
Removing these newlines eliminates millions of characters (~1% of total size).

Therefore, the .basta file can exceed the naïve 4× limit.
