use clap::Parser;
use std::fs;
use std::io::{self, Read, Write};
use std::path::PathBuf;

use cel_fmt::{format_cel, FormatOptions};

#[derive(Parser, Debug)]
#[command(
    name = "cel-fmt",
    version,
    about = "A code formatter for Common Expression Language (CEL)",
    long_about = None
)]
struct Args {
    /// Input file(s) to format. If not specified, reads from stdin
    #[arg(value_name = "FILE")]
    files: Vec<PathBuf>,

    /// Write result to stdout instead of updating files
    #[arg(short = 'c', long = "check")]
    check: bool,

    /// Maximum line width
    #[arg(short = 'w', long = "max-width", default_value = "80")]
    max_width: usize,

    /// Number of spaces per indentation level
    #[arg(short = 'i', long = "indent", default_value = "2")]
    indent_width: usize,

    /// Use tabs instead of spaces for indentation
    #[arg(long = "use-tabs")]
    use_tabs: bool,

    /// Don't add trailing commas
    #[arg(long = "no-trailing-comma")]
    no_trailing_comma: bool,

    /// Print the formatted output (don't modify files)
    #[arg(short = 'p', long = "print")]
    print: bool,
}

fn main() -> anyhow::Result<()> {
    let args = Args::parse();

    let options = FormatOptions::new()
        .with_max_width(args.max_width)
        .with_indent_width(args.indent_width)
        .with_trailing_comma(!args.no_trailing_comma);

    let options = if args.use_tabs {
        options.with_tabs()
    } else {
        options
    };

    if args.files.is_empty() {
        // Read from stdin
        let mut input = String::new();
        io::stdin().read_to_string(&mut input)?;

        match format_cel(&input, &options) {
            Ok(formatted) => {
                print!("{}", formatted);
                Ok(())
            }
            Err(e) => {
                eprintln!("Error: {}", e);
                std::process::exit(1);
            }
        }
    } else {
        // Process files
        let mut has_error = false;

        for file_path in &args.files {
            match process_file(file_path, &options, args.check || args.print) {
                Ok(changed) => {
                    if args.check && changed {
                        println!("Would reformat: {}", file_path.display());
                        has_error = true;
                    } else if args.print {
                        // Output was already printed
                    } else if changed {
                        println!("Formatted: {}", file_path.display());
                    }
                }
                Err(e) => {
                    eprintln!("Error processing {}: {}", file_path.display(), e);
                    has_error = true;
                }
            }
        }

        if has_error {
            std::process::exit(1);
        }

        Ok(())
    }
}

fn process_file(path: &PathBuf, options: &FormatOptions, dry_run: bool) -> anyhow::Result<bool> {
    let content = fs::read_to_string(path)?;
    let formatted = format_cel(&content, options)?;

    if dry_run {
        io::stdout().write_all(formatted.as_bytes())?;
        Ok(content != formatted)
    } else {
        let changed = content != formatted;
        if changed {
            fs::write(path, formatted)?;
        }
        Ok(changed)
    }
}
