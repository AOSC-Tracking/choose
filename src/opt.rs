use std::path::PathBuf;
use std::process;

use crate::choice::Choice;
use crate::parse::{self, choice, PARSE_CHOICE_RE};

pub struct Opt {
    /// Choose fields by character number
    pub character_wise: bool,

    /// Activate debug mode
    pub debug: bool,

    /// Use exclusive ranges, similar to array indexing in many programming languages
    pub exclusive: bool,

    /// Specify field separator other than whitespace, using Rust `regex` syntax
    pub field_separator: Option<String>,

    /// Input file
    pub input: Option<PathBuf>,

    /// Use non-greedy field separators
    pub non_greedy: bool,

    /// Index from 1 instead of 0
    pub one_indexed: bool,

    /// Specify output field separator
    pub output_field_separator: Option<String>,

    /// Fields to print. Either a, a:b, a..b, or a..=b, where a and b are integers. The beginning
    /// or end of a range can be omitted, resulting in including the beginning or end of the line,
    /// respectively. a:b is inclusive of b (unless overridden by -x). a..b is
    /// exclusive of b and a..=b is inclusive of b.
    pub choices: Vec<Choice>,
}

macro_rules! HELP {
    () => {
        r"{} {}
`choose` sections from each line of files

USAGE:
    choose [FLAGS] [OPTIONS] <choices>...

FLAGS:
    -c, --character-wise    Choose fields by character number
    -d, --debug             Activate debug mode
    -x, --exclusive         Use exclusive ranges, similar to array indexing in many programming languages
    -h, --help              Prints help information
    -n, --non-greedy        Use non-greedy field separators
        --one-indexed       Index from 1 instead of 0
    -V, --version           Prints version information

OPTIONS:
    -f, --field-separator <field-separator>
            Specify field separator other than whitespace, using Rust `regex` syntax

    -i, --input <input>                                      Input file
    -o, --output-field-separator <output-field-separator>    Specify output field separator

ARGS:
    <choices>...    Fields to print. Either a, a:b, a..b, or a..=b, where a and b are integers. The beginning or end
                    of a range can be omitted, resulting in including the beginning or end of the line,
                    respectively. a:b is inclusive of b (unless overridden by -x). a..b is exclusive of b and a..=b
                    is inclusive of b
"
    };
}

macro_rules! USAGE {
    () => {
        r"USAGE:
    choose [FLAGS] [OPTIONS] <choices>...

For more information try --help"
    };
}

/// Print a clap-style error (message + short usage) to stderr and exit(2),
/// instead of panicking with a Rust backtrace.
fn usage_error(msg: &str) -> ! {
    eprintln!("error: {}", msg);
    eprintln!();
    eprintln!("{}", USAGE!());
    process::exit(2);
}

fn is_choice(arg: &str) -> bool {
    PARSE_CHOICE_RE.captures_iter(arg).next().is_some() || arg.parse::<isize>().is_ok()
}

fn parse_choice_or_die(arg: &str) -> Choice {
    choice(arg).unwrap_or_else(|e| usage_error(&format!("failed to parse choice '{}': {}", arg, e)))
}

/// Resolve the value for a `--long-opt` that takes an argument, whether it was
/// written as `--long-opt=value` or `--long-opt value`.
/// Returns the value and how many argv slots (1 or 2) were consumed.
fn take_long_value<'a>(
    name: &str,
    inline: Option<&'a str>,
    args: &[&'a str],
    i: usize,
) -> (String, usize) {
    match inline {
        Some(v) => (v.to_string(), 1),
        None => {
            let v = args
                .get(i + 1)
                .copied()
                .unwrap_or_else(|| usage_error(&format!("'--{}' requires a value", name)));
            (v.to_string(), 2)
        }
    }
}

impl Opt {
    pub fn new(args: Vec<&str>) -> Self {
        let mut character_wise = false;
        let mut debug = false;
        let mut exclusive = false;
        let mut field_separator = None;
        let mut input = None;
        let mut non_greedy = false;
        let mut one_indexed = false;
        let mut output_field_separator = None;
        let mut choices = vec![];

        // Once a bare `--` is seen, everything after it is a positional
        // choice, even if it looks like a flag (POSIX end-of-options marker).
        let mut only_positional = false;
        // Skip args[0] (the program name), matching the previous behavior.
        let mut i = 1;

        while i < args.len() {
            let arg = args[i];

            if only_positional {
                choices.push(parse_choice_or_die(arg));
                i += 1;
                continue;
            }

            if arg == "--" {
                only_positional = true;
                i += 1;
                continue;
            }

            if is_choice(arg) {
                choices.push(parse_choice_or_die(arg));
                i += 1;
                continue;
            }

            if let Some(rest) = arg.strip_prefix("--") {
                let (name, inline_value) = match rest.split_once('=') {
                    Some((name, value)) => (name, Some(value)),
                    None => (rest, None),
                };

                match name {
                    "character-wise" | "debug" | "exclusive" | "non-greedy" | "one-indexed" => {
                        if inline_value.is_some() {
                            usage_error(&format!("'--{}' does not take a value", name));
                        }
                        match name {
                            "character-wise" => character_wise = true,
                            "debug" => debug = true,
                            "exclusive" => exclusive = true,
                            "non-greedy" => non_greedy = true,
                            "one-indexed" => one_indexed = true,
                            _ => unreachable!(),
                        }
                        i += 1;
                    }
                    "field-separator" => {
                        let (value, consumed) = take_long_value(name, inline_value, &args, i);
                        field_separator = Some(value);
                        i += consumed;
                    }
                    "input" => {
                        let (value, consumed) = take_long_value(name, inline_value, &args, i);
                        input = Some(PathBuf::from(value));
                        i += consumed;
                    }
                    "output-field-separator" => {
                        let (value, consumed) = take_long_value(name, inline_value, &args, i);
                        output_field_separator = Some(
                            parse::output_field_separator(&value).unwrap_or_else(|e| {
                                usage_error(&format!(
                                    "invalid value for '--output-field-separator': {}",
                                    e
                                ))
                            }),
                        );
                        i += consumed;
                    }
                    _ => usage_error(&format!("unrecognized flag '--{}'", name)),
                }
                continue;
            }

            if let Some(rest) = arg.strip_prefix('-') {
                if rest.is_empty() {
                    // A bare "-" with no flag letters after it.
                    usage_error("found '-' with no flag letters");
                }

                let mut chars = rest.chars();
                let mut extra_args = 0;

                while let Some(ch) = chars.next() {
                    match ch {
                        'c' => character_wise = true,
                        'd' => debug = true,
                        'x' => exclusive = true,
                        'n' => non_greedy = true,
                        'f' | 'i' | 'o' => {
                            // Whatever is left in this token (e.g. the "3" in
                            // "-f3" or "-f=3") is the value; otherwise take
                            // the next argv slot, even if it starts with '-'
                            // (so separators like "-" or "-1" work).
                            let attached: String = chars.by_ref().collect();
                            let attached = match attached.strip_prefix('=') {
                                Some(v) => v.to_string(),
                                None => attached,
                            };

                            let value = if !attached.is_empty() {
                                attached
                            } else {
                                let v = args.get(i + 1).copied().unwrap_or_else(|| {
                                    usage_error(&format!("'-{}' requires a value", ch))
                                });
                                extra_args = 1;
                                v.to_string()
                            };

                            match ch {
                                'f' => field_separator = Some(value),
                                'i' => input = Some(PathBuf::from(value)),
                                'o' => {
                                    output_field_separator = Some(
                                        parse::output_field_separator(&value).unwrap_or_else(
                                            |e| {
                                                usage_error(&format!(
                                                    "invalid value for '-o': {}",
                                                    e
                                                ))
                                            },
                                        ),
                                    )
                                }
                                _ => unreachable!(),
                            }
                            break;
                        }
                        other => usage_error(&format!("unrecognized flag '-{}'", other)),
                    }
                }

                i += 1 + extra_args;
                continue;
            }

            usage_error(&format!("unrecognized argument '{}'", arg));
        }

        if choices.is_empty() {
            eprintln!(HELP!(), env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            process::exit(2);
        }

        Self {
            character_wise,
            debug,
            exclusive,
            field_separator,
            input,
            non_greedy,
            one_indexed,
            output_field_separator,
            choices,
        }
    }

    pub fn parse() -> Self {
        let args = std::env::args().collect::<Vec<_>>();

        if args.iter().any(|a| a == "--help" || a == "-h") {
            println!(HELP!(), env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }

        if args.iter().any(|a| a == "--version" || a == "-V") {
            println!("{} {}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"));
            process::exit(0);
        }

        Self::new(args.iter().map(|x| x.as_str()).collect::<Vec<_>>())
    }
}

