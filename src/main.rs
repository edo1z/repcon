use clap::Parser;

/// Repo Condenser - A CLI tool to efficiently condense repository files
#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Path to the repository's root directory
    #[clap(value_parser)]
    path_to_repo: String,
}

fn main() {
    let args = Args::parse();

    // Display the input path provided by the user
    println!("Repository Path: {}", args.path_to_repo);

    // TODO: Implement the file size calculation logic here
}
