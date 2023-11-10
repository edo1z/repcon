use clap::Parser;
use dotenv::dotenv;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::env;
use std::fs::{self, File};
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

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

// Function to collect the list of target files
fn collect_target_files(dir: &Path, args: &Args) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut override_builder = OverrideBuilder::new(dir);

    // Add ignore patterns from args
    for rule in &args.ignore_patterns {
        override_builder
            .add(format!("!{}", rule).as_str())
            .map_err(convert_ignore_error)?;
    }

    // If a .repconignore file is provided, add its ignore patterns
    if let Some(ref ignore_file) = args.repconignore_path {
        if Path::new(ignore_file).exists() {
            let ignore_content = fs::read_to_string(ignore_file)?;
            for line in ignore_content.lines() {
                if !line.trim().is_empty() && !line.starts_with('#') {
                    override_builder
                        .add(format!("!{}", line).as_str())
                        .map_err(convert_ignore_error)?;
                }
            }
        }
    }

    let overrides = override_builder.build().map_err(convert_ignore_error)?;
    let walker = WalkBuilder::new(dir).overrides(overrides).build();

    // Collect files that are not ignored
    for result in walker {
        if let Ok(entry) = result {
            if entry.file_type().map_or(false, |ft| ft.is_file()) {
                files.push(entry.into_path());
            }
        }
    }

    Ok(files)
}

fn convert_ignore_error(e: ignore::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

// Function to calculate the total size of the files from a list
fn get_dir_size(files: &[PathBuf]) -> io::Result<u64> {
    let total_size: u64 = files
        .iter()
        .filter_map(|f| fs::metadata(f).ok())
        .map(|metadata| metadata.len())
        .sum();

    Ok(total_size)
}

fn create_page_header(repository_name: &str, file_path: &str, page_number: u64) -> String {
    format!(
        "# repcon_repository_name: {}\n# repcon_file_name: {}\n# repcon_page_number: {}\n// START OF CODE BLOCK: {}\n",
        repository_name,
        file_path,
        page_number,
        file_path
    )
}

fn create_page_footer(file_path: &str) -> String {
    format!("// END OF CODE BLOCK: {}\n", file_path)
}

fn check_size_limits(total_size: u64, total_allowed_size: u64) -> io::Result<()> {
    if total_size > total_allowed_size {
        eprintln!(
            "Error: The total size of the files ({}) exceeds the allowed limit of {} bytes.",
            total_size, total_allowed_size
        );
        std::process::exit(1);
    }
    Ok(())
}

// Function to create a new file and write the initial page header
fn create_new_file(
    output_directory: &Path,
    file_counter: u64,
    repository_name: &str,
) -> io::Result<File> {
    let file_path = output_directory.join(format!("{}_{}.txt", repository_name, file_counter));
    let file = File::create(&file_path)?;
    Ok(file)
}

fn split_files_into_chunks(
    files: &[PathBuf],
    output_directory: &Path,
    max_file_size: u64,
    repository_name: &str,
) -> io::Result<()> {
    let mut file_counter: u64 = 1;
    let mut current_file_size: u64 = 0;
    let mut current_file_name = "";
    let mut page_number: u64;
    let mut output_file = create_new_file(output_directory, file_counter, repository_name)?;
    let mut page_header: String;
    let mut page_header_size: u64;
    let page_footer = create_page_footer(current_file_name);
    let page_footer_size = page_footer.as_bytes().len() as u64;

    for file_path in files {
        current_file_name = file_path.to_str().unwrap(); // Safely convert PathBuf to &str
        page_number = 1;
        page_header = create_page_header(repository_name, current_file_name, page_number);
        page_header_size = page_header.as_bytes().len() as u64 + 1; // +1 for the newline character

        if page_header_size + page_footer_size > max_file_size {
            eprintln!(
                "Error: The maximum file size ({}) is too small to contain the page header and footer.",
                max_file_size
            );
            std::process::exit(1);
        }

        if current_file_size + page_header_size > max_file_size {
            // Finish the current file and create a new one
            file_counter += 1;
            current_file_size = 0;
            output_file = create_new_file(output_directory, file_counter, repository_name)?;
            continue;
        } else {
            writeln!(output_file, "{}", page_header)?;
            current_file_size += page_header_size;
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let line_size = line.as_bytes().len() as u64 + 1; // +1 for the newline character

            // Check if the current line can be added to the current file
            if current_file_size + line_size + page_footer_size > max_file_size {
                // Write the footer before closing the current file
                write!(output_file, "{}", create_page_footer(current_file_name))?;

                // Finish the current file and create a new one
                file_counter += 1;
                current_file_size = 0;
                output_file = create_new_file(output_directory, file_counter, repository_name)?;

                // Write the header to the new file
                page_number += 1;
                page_header = create_page_header(repository_name, current_file_name, page_number);
                page_header_size = page_header.as_bytes().len() as u64 + 1; // +1 for the newline character
                if page_header_size + page_footer_size > max_file_size {
                    eprintln!(
                        "Error: The maximum file size ({}) is too small to contain the page header and footer.",
                        max_file_size
                    );
                    std::process::exit(1);
                }
                writeln!(output_file, "{}", page_header)?;
                current_file_size += page_header_size;
            }

            writeln!(output_file, "{}", line)?;
            current_file_size += line_size;
        }
        // Don't forget to add a footer when you finish writing a file
        write!(output_file, "{}", create_page_footer(current_file_name))?;
    }
    Ok(())
}

fn upload_to_openai(upload: &Option<Option<String>>) -> io::Result<()> {
    match upload {
        Some(Some(api_key)) => {
            println!("Uploading with provided API key.");
            println!("{}", api_key)
        }
        Some(None) => match env::var("OPENAI_API_KEY") {
            Ok(env_api_key) => {
                println!("Uploading with API key from environment variable.");
                println!("{}", env_api_key)
            }
            Err(_) => {
                println!("API key not specified and not found in environment. Skipping upload.");
            }
        },
        None => {
            println!("No upload option provided. Skipping upload.");
        }
    }
    Ok(())
}

fn main() -> io::Result<()> {
    dotenv().ok();
    let args = Args::parse();
    let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    struct FileInfo {
        file1_str: String,
        file2_str: String,
    }
    impl FileInfo {
        fn new() -> Self {
            Self {
                file1_str: "Hello World!".to_string(),
                file2_str: "Another file content!".to_string(),
            }
        }
        fn file1_size(&self) -> u64 {
            let str = format!("{}\n", self.file1_str);
            str.as_bytes().len() as u64
        }
        fn file2_size(&self) -> u64 {
            let str = format!("{}\n", self.file2_str);
            str.as_bytes().len() as u64
        }
        fn total_size(&self) -> u64 {
            self.file1_size() + self.file2_size()
        }
    }

    fn setup_test_environment(
        ignore_patterns: Vec<String>,
        repconignore_content: Option<&str>,
    ) -> io::Result<(tempfile::TempDir, Args, FileInfo)> {
        let dir = tempdir()?;
        let test_file_path = dir.path().join("test_file.txt");
        let test_file2_path = dir.path().join("test_file2.txt");
        let repconignore_path = dir.path().join(".repconignore");
        let file_info = FileInfo::new();

        // Create test files
        let mut file = File::create(&test_file_path)?;
        writeln!(file, "{}", file_info.file1_str)?;

        let mut file2 = File::create(&test_file2_path)?;
        writeln!(file2, "{}", file_info.file2_str)?;

        // Create .repconignore file if content is provided
        if let Some(content) = repconignore_content {
            fs::write(&repconignore_path, content)?;
        }

        // Construct Args instance
        let args = Args {
            path_to_repo: dir.path().to_str().unwrap().to_string(),
            ignore_patterns,
            repconignore_path: repconignore_content
                .map(|_| repconignore_path.to_str().unwrap().to_string()),
            max_files: 20,
            max_file_size: 540,
            output_directory: "output".to_string(),
            upload: None,
        };

        Ok((dir, args, file_info))
    }

    #[test]
    fn test_no_ignore() -> io::Result<()> {
        let (dir, args, file_info) = setup_test_environment(vec![], None)?;
        let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
        let size = get_dir_size(&files)?;
        assert_eq!(size, file_info.total_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_ignore_single_file() -> io::Result<()> {
        let (dir, args, file_info) = setup_test_environment(vec!["test_file2*".to_string()], None)?;
        let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
        let size = get_dir_size(&files)?;
        assert_eq!(size, file_info.file1_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_ignore_nonexistent_pattern() -> io::Result<()> {
        let (dir, args, file_info) = setup_test_environment(vec!["hoge".to_string()], None)?;
        let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
        let size = get_dir_size(&files)?;
        assert_eq!(size, file_info.total_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_ignore_multiple_patterns() -> io::Result<()> {
        let ignore_patterns = vec!["*_file.*".to_string(), "*_file2.*".to_string()];
        let (dir, args, _file_info) = setup_test_environment(ignore_patterns, None)?;
        let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
        let size = get_dir_size(&files)?;
        assert_eq!(size, 0);
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_repconignore_single_pattern() -> io::Result<()> {
        let repconignore_content = "test_file2*";
        let (dir, args, file_info) = setup_test_environment(vec![], Some(repconignore_content))?;
        let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
        let size = get_dir_size(&files)?;
        assert_eq!(size, file_info.file1_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_repconignore_all_pattern() -> io::Result<()> {
        let repconignore_content = "*";
        let (dir, args, _file_info) = setup_test_environment(vec![], Some(repconignore_content))?;
        let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
        let size = get_dir_size(&files)?;
        assert_eq!(size, 0);
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_repconignore_irrelevant_pattern() -> io::Result<()> {
        let repconignore_content = "hoge";
        let (dir, args, file_info) = setup_test_environment(vec![], Some(repconignore_content))?;
        let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
        let size = get_dir_size(&files)?;
        assert_eq!(size, file_info.file1_size() + file_info.file2_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_repconignore_multiple_patterns() -> io::Result<()> {
        let repconignore_content = "*_file.*\n*_file2.*";
        let (dir, args, _file_info) = setup_test_environment(vec![], Some(repconignore_content))?;
        let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
        let size = get_dir_size(&files)?;
        assert_eq!(size, 0);
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_ignore_with_repconignore() -> io::Result<()> {
        let ignore_patterns = vec!["test_file.*".to_string()];
        let repconignore_content = "test_file2.*";
        let (dir, args, _file_info) =
            setup_test_environment(ignore_patterns, Some(repconignore_content))?;
        let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
        let size = get_dir_size(&files)?;
        assert_eq!(size, 0);
        dir.close()?;
        Ok(())
    }
}
