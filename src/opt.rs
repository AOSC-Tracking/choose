use std::path::PathBuf;

use clap::Parser;

use crate::choice::Choice;
use crate::parse;

#[derive(Debug, Parser)]
#[command(about = "`choose` sections from each line of files")]
#[command(next_line_help = true)]
#[command(allow_hyphen_values = true)]
pub struct Opt {
    /// Choose fields by character number
    #[clap(short, long)]
    pub character_wise: bool,

    /// Activate debug mode
    #[clap(short, long)]
    pub debug: bool,

    /// Use exclusive ranges, similar to array indexing in many programming languages
    #[clap(short = 'x', long)]
    pub exclusive: bool,

    /// Specify field separator other than whitespace, using Rust `regex` syntax
    #[clap(short, long)]
    pub field_separator: Option<String>,

    /// Input file
    #[clap(short, long)]
    pub input: Option<PathBuf>,

    /// Use non-greedy field separators
    #[clap(short, long)]
    pub non_greedy: bool,

    /// Index from 1 instead of 0
    #[clap(long)]
    pub one_indexed: bool,

    /// Specify output field separator
    #[clap(short, long, value_parser = parse::output_field_separator)]
    pub output_field_separator: Option<String>,

    /// Fields to print. Either a, a:b, a..b, or a..=b, where a and b are integers. The beginning
    /// or end of a range can be omitted, resulting in including the beginning or end of the line,
    /// respectively. a:b is inclusive of b (unless overridden by -x). a..b is
    /// exclusive of b and a..=b is inclusive of b.
    #[arg(value_parser = parse::choice, required = true, num_args = 1..)]
    pub choices: Vec<Choice>,
}
