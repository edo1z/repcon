use clap::Parser;
use dotenv::dotenv;
use repcon::{
    check_size_limits, collect_target_files, get_dir_size, split_files_into_chunks,
    upload_to_openai,
};
use std::fs;
use std::io;
use std::path::Path;

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

    /// Path to the repconignore file
    #[clap(
        short = 'r',
        long = "repconignore",
        value_parser,
        default_value = ".repconignore"
    )]
    repconignore_path: Option<String>,

    /// Maximum number of files to output
    #[clap(short = 'n', long = "max-files", default_value_t = 20, value_parser = clap::value_parser!(u64).range(1..1001))]
    max_files: u64,

    /// Maximum size of each output file in megabytes
    #[clap(short = 's', long = "max-size", default_value_t = 540, value_parser = clap::value_parser!(u64).range(1..100001))]
    max_file_size: u64,

    /// Path to the output directory for the condensed files
    #[clap(short = 'o', long = "output", value_parser, default_value = "output")]
    output_directory: String,

    /// OpenAI API key for uploading the condensed files
    #[clap(short = 'u', long = "upload", value_parser)]
    upload: Option<Option<String>>,
}

fn main() -> io::Result<()> {
    dotenv().ok();
    let args = Args::parse();
    let files = collect_target_files(
        Path::new(&args.path_to_repo),
        &args.ignore_patterns,
        args.repconignore_path.as_ref(),
    )?;
    let total_size = get_dir_size(&files)?;

    // Convert max file size from megabytes to bytes
    let max_file_size_bytes = (args.max_file_size as u64) * 1024 * 1024;

    // Calculate the total allowed size based on max files and max file size
    let total_allowed_size = max_file_size_bytes * (args.max_files as u64);

    println!("Total size: {}", total_size);
    println!("Maximum file size: {} bytes", max_file_size_bytes);
    println!("Total allowed size: {} bytes", total_allowed_size);

    // If total size exceeds the allowed size, throw an error
    check_size_limits(total_size, total_allowed_size)?;

    // Create the output directory if it doesn't exist
    fs::create_dir_all(&args.output_directory)?;

    // Split the files into chunks
    split_files_into_chunks(
        &files,
        Path::new(&args.output_directory),
        max_file_size_bytes,
        "repo_hoge",
    )?;

    // Upload to OpenAI
    upload_to_openai(&args.upload)?;

    Ok(())
}
