use clap::Parser;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::io;

/// Recursively calculates the total size of files in a directory, respecting custom ignore rules.
fn get_dir_size(dir: &str, ignore_rules: &[String]) -> io::Result<u64> {
    let mut total_size: u64 = 0;

    // Build custom ignore rules
    let mut override_builder = OverrideBuilder::new(dir);
    for rule in ignore_rules {
        override_builder
            .add(format!("!{}", rule).as_str())
            .expect("Invalid override pattern");
    }
    let overrides = override_builder.build().expect("Could not build overrides");

    // Build the walker with the custom overrides
    let walker = WalkBuilder::new(dir).overrides(overrides).build(); // `.gitignore` is automatically respected

    for result in walker {
        if let Ok(entry) = result {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                }
            }
        }
    }

    Ok(total_size)
}

fn main() -> io::Result<()> {
    let args = Args::parse();
    let total_size = get_dir_size(&args.path_to_repo, &args.ignore_patterns)?;
    println!("Total size: {}", total_size);

    Ok(())
}

/// Repcon - A CLI tool to efficiently condense repository files, with custom ignore rules
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the repository's root directory
    #[clap(value_parser)]
    path_to_repo: String,

    /// Custom ignore patterns
    #[clap(short = 'i', long = "ignore", value_parser)]
    ignore_patterns: Vec<String>,
}
