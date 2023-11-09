use clap::Parser;
use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::fs;
use std::io;
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

fn main() -> io::Result<()> {
    let args = Args::parse();
    let files = collect_target_files(Path::new(&args.path_to_repo), &args)?;
    let total_size = get_dir_size(&files)?;
    println!("Total size: {}", total_size);

    // Convert max file size from megabytes to bytes
    let max_file_size_bytes = (args.max_file_size as u64) * 1024 * 1024;

    // Calculate the total allowed size based on max files and max file size
    let total_allowed_size = max_file_size_bytes * (args.max_files as u64);

    println!("Maximum file size: {} bytes", max_file_size_bytes);
    println!("Total allowed size: {} bytes", total_allowed_size);

    // If total size exceeds the allowed size, throw an error
    if total_size > total_allowed_size {
        eprintln!(
            "Error: The total size of the files ({}) exceeds the allowed limit of {} bytes.",
            total_size, total_allowed_size
        );
        std::process::exit(1); // Exit with error code
    }

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
