use clap::Parser;
use ignore::WalkBuilder;
use std::io;

/// Recursively calculates the total size of all files in a directory.
fn get_dir_size(dir: &str) -> io::Result<u64> {
    let mut total_size: u64 = 0;
    let walker = WalkBuilder::new(dir).build(); // `.gitignore` is automatically respected

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

/// Repo Condenser - A CLI tool to efficiently condense repository files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the repository's root directory
    #[clap(value_parser)]
    path_to_repo: String,
}

fn main() -> io::Result<()> {
    let args = Args::parse();

    let total_size = get_dir_size(&args.path_to_repo)?;
    println!("Total size of non-ignored files: {} bytes", total_size);

    Ok(())
}
