//! ELID CLI - Command-line interface for encoding and decoding embeddings to/from ELIDs.

use clap::{Parser, Subcommand};
use elid_cli::commands::{handle_decode, handle_encode};

#[derive(Parser)]
#[command(name = "elid")]
#[command(about = "ELID: Embedding Locality IDentifier encoder/decoder", long_about = None)]
#[command(version)]
#[command(
    after_help = "ELID (Embedding Locality IDentifier) is a tool for encoding high-dimensional \
embeddings into compact, locality-preserving string identifiers.\n\n\
For more information, visit: https://github.com/zachhandley/ELID"
)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

const ENCODE_EXAMPLES: &str = "\
Examples:
  # Encode CSV embeddings to text IDs
  elid encode --profile mini128 --input embeddings.csv --output ids.txt

  # Use Morton profile with progress bar
  elid encode -p morton10x10 -i data.csv --progress -o ids.csv

  # Encode from Parquet file
  elid encode -p hilbert10x10 -i embeddings.parquet --format parquet -o ids.txt

  # Use custom column name
  elid encode -p mini128 -i data.csv --column vectors -o ids.txt

  # Pipe from stdin to stdout
  cat embeddings.csv | elid encode -p mini128 -i - > ids.txt

  # Output as CSV with embeddings
  elid encode -p mini128 -i data.csv -o output.csv --output-format csv

  # Output as JSONL
  elid encode -p mini128 -i data.jsonl --format jsonl --output-format jsonl -o output.jsonl";

const DECODE_EXAMPLES: &str = "\
Examples:
  # Decode to hex format
  elid decode --input ids.txt --format hex

  # Decode to JSON with profile info
  elid decode -i ids.txt --format json -o decoded.json

  # Decode from stdin
  cat ids.txt | elid decode -i - --format json

  # Decode to binary
  elid decode -i ids.txt --format binary -o output.bin

  # Pipe to other tools
  elid decode -i ids.txt --format json | jq '.profile'";

#[derive(Subcommand)]
enum Commands {
    /// Encode embeddings to ELIDs
    #[command(after_help = ENCODE_EXAMPLES)]
    Encode {
        /// Profile to use for encoding
        ///
        /// Available profiles:
        ///   - mini128: MiniHash profile for 128+ dimensional embeddings
        ///   - morton10x10: 10-dimensional Morton curve (100 bits)
        ///   - hilbert10x10: 10-dimensional Hilbert curve (100 bits)
        #[arg(long, short = 'p', value_name = "PROFILE")]
        profile: String,

        /// Input file path (use "-" for stdin)
        ///
        /// When using stdin, the input must be in the specified format.
        /// Note: Parquet format does not support stdin input.
        #[arg(long, short = 'i', value_name = "FILE")]
        input: Option<String>,

        /// Input format
        ///
        /// Supported formats:
        ///   - csv: Comma-separated values with embeddings as arrays
        ///   - parquet: Apache Parquet format (file input only)
        ///   - jsonl: JSON Lines format with one embedding per line
        #[arg(long, default_value = "csv", value_name = "FORMAT")]
        format: String,

        /// Column name containing embeddings
        ///
        /// For CSV/Parquet: The column containing embedding arrays.
        /// For JSONL: The JSON field containing the embedding array.
        #[arg(long, default_value = "embedding", value_name = "COLUMN")]
        column: String,

        /// Output file path (use "-" for stdout)
        ///
        /// If not specified, outputs to stdout.
        #[arg(long, short = 'o', value_name = "FILE")]
        output: Option<String>,

        /// Output format
        ///
        /// Supported formats:
        ///   - text: One ELID per line (default)
        ///   - csv: CSV with both embeddings and ELIDs
        ///   - jsonl: JSON Lines with ELID field added
        #[arg(long, default_value = "text", value_name = "FORMAT")]
        output_format: String,

        /// Show progress bar during encoding
        ///
        /// Displays a progress bar when processing batches larger than 1000 embeddings.
        /// Progress information is written to stderr.
        #[arg(long)]
        progress: bool,
    },

    /// Decode ELIDs to raw bytes or profile info
    #[command(after_help = DECODE_EXAMPLES)]
    Decode {
        /// Input file containing ELIDs (use "-" for stdin)
        ///
        /// Each line should contain one ELID string.
        /// Empty lines are skipped.
        #[arg(long, short = 'i', value_name = "FILE")]
        input: Option<String>,

        /// Output format
        ///
        /// Supported formats:
        ///   - hex: Hexadecimal representation of bytes (default)
        ///   - json: JSON with ELID, bytes, and profile information
        ///   - binary: Raw binary output
        #[arg(long, default_value = "hex", value_name = "FORMAT")]
        format: String,

        /// Output file path (use "-" for stdout)
        ///
        /// If not specified, outputs to stdout.
        #[arg(long, short = 'o', value_name = "FILE")]
        output: Option<String>,
    },
}

fn main() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Encode {
            profile,
            input,
            format,
            column,
            output,
            output_format,
            progress,
        } => {
            handle_encode(
                &profile,
                input,
                &format,
                &column,
                output,
                &output_format,
                progress,
            )?;
        }
        Commands::Decode {
            input,
            format,
            output,
        } => {
            handle_decode(input, &format, output)?;
        }
    }

    Ok(())
}
