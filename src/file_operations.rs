use ignore::{overrides::OverrideBuilder, WalkBuilder};
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

fn to_relative_path(root: &Path, file_path: &Path) -> PathBuf {
    file_path
        .strip_prefix(root)
        .unwrap_or(file_path)
        .to_path_buf()
}

// Function to collect the list of target files
pub fn collect_target_files(
    dir: &Path,
    ignore_patterns: &[String],
    reconignore_path: Option<&String>,
) -> io::Result<Vec<PathBuf>> {
    let mut files = Vec::new();
    let mut override_builder = OverrideBuilder::new(dir);

    // Add ignore patterns from args
    for rule in ignore_patterns {
        override_builder
            .add(format!("!{}", rule).as_str())
            .map_err(convert_ignore_error)?;
    }

    // If a .repconignore file is provided, add its ignore patterns
    if let Some(ignore_file) = reconignore_path {
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
                let relative_path = to_relative_path(dir, entry.path());
                files.push(relative_path);
            }
        }
    }

    Ok(files)
}

fn convert_ignore_error(e: ignore::Error) -> io::Error {
    io::Error::new(io::ErrorKind::Other, e)
}

// Function to calculate the total size of the files from a list
pub fn get_dir_size(root_path: &Path, files: &[PathBuf]) -> io::Result<u64> {
    let total_size: u64 = files
        .iter()
        .filter_map(|f| fs::metadata(root_path.join(f)).ok())
        .map(|metadata| metadata.len())
        .sum();

    Ok(total_size)
}

pub fn check_size_limits(total_size: u64, total_allowed_size: u64) -> io::Result<()> {
    if total_size > total_allowed_size {
        eprintln!(
            "Error: The total size of the files ({}) exceeds the allowed limit of {} bytes.",
            total_size, total_allowed_size
        );
        std::process::exit(1);
    }
    Ok(())
}

pub fn format_file_size(size: u64) -> String {
    const KILOBYTE: u64 = 1024;
    const MEGABYTE: u64 = KILOBYTE * 1024;
    const GIGABYTE: u64 = MEGABYTE * 1024;

    if size >= GIGABYTE {
        format!("{:.2} GB", size as f64 / GIGABYTE as f64)
    } else if size >= MEGABYTE {
        format!("{:.2} MB", size as f64 / MEGABYTE as f64)
    } else if size >= KILOBYTE {
        format!("{:.2} KB", size as f64 / KILOBYTE as f64)
    } else {
        format!("{} B", size)
    }
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
    ) -> io::Result<(
        tempfile::TempDir,
        PathBuf,
        Vec<String>,
        Option<String>,
        FileInfo,
    )> {
        let dir = tempdir()?;
        let path_to_repo = dir.path().to_path_buf();
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

        Ok((
            dir,
            path_to_repo,
            ignore_patterns,
            repconignore_content.map(|_| repconignore_path.to_str().unwrap().to_string()),
            file_info,
        ))
    }

    #[test]
    fn test_no_ignore() -> io::Result<()> {
        let (dir, path_to_repo, ignore_patterns, ignore_path, file_info) =
            setup_test_environment(vec![], None)?;
        let files = collect_target_files(&path_to_repo, &ignore_patterns, ignore_path.as_ref())?;
        let size = get_dir_size(&path_to_repo, &files)?;
        assert_eq!(size, file_info.total_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_ignore_single_file() -> io::Result<()> {
        let (dir, path_to_repo, ignore_patterns, ignore_path, file_info) =
            setup_test_environment(vec!["test_file2*".to_string()], None)?;
        let files = collect_target_files(&path_to_repo, &ignore_patterns, ignore_path.as_ref())?;
        let size = get_dir_size(&path_to_repo, &files)?;
        assert_eq!(size, file_info.file1_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_ignore_nonexistent_pattern() -> io::Result<()> {
        let (dir, path_to_repo, ignore_patterns, ignore_path, file_info) =
            setup_test_environment(vec!["hoge".to_string()], None)?;
        let files = collect_target_files(&path_to_repo, &ignore_patterns, ignore_path.as_ref())?;
        let size = get_dir_size(&path_to_repo, &files)?;
        assert_eq!(size, file_info.total_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_ignore_multiple_patterns() -> io::Result<()> {
        let ignore_patterns = vec!["*_file.*".to_string(), "*_file2.*".to_string()];
        let (dir, path_to_repo, ignore_patterns, ignore_path, _file_info) =
            setup_test_environment(ignore_patterns, None)?;
        let files = collect_target_files(&path_to_repo, &ignore_patterns, ignore_path.as_ref())?;
        let size = get_dir_size(&path_to_repo, &files)?;
        assert_eq!(size, 0);
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_repconignore_single_pattern() -> io::Result<()> {
        let repconignore_content = "test_file2*";
        let (dir, path_to_repo, ignore_patterns, ignore_path, file_info) =
            setup_test_environment(vec![], Some(repconignore_content))?;
        let files = collect_target_files(&path_to_repo, &ignore_patterns, ignore_path.as_ref())?;
        let size = get_dir_size(&path_to_repo, &files)?;
        assert_eq!(size, file_info.file1_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_repconignore_all_pattern() -> io::Result<()> {
        let repconignore_content = "*";
        let (dir, path_to_repo, ignore_patterns, ignore_path, _file_info) =
            setup_test_environment(vec![], Some(repconignore_content))?;
        let files = collect_target_files(&path_to_repo, &ignore_patterns, ignore_path.as_ref())?;
        let size = get_dir_size(&path_to_repo, &files)?;
        assert_eq!(size, 0);
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_repconignore_irrelevant_pattern() -> io::Result<()> {
        let repconignore_content = "hoge";
        let (dir, path_to_repo, ignore_patterns, ignore_path, file_info) =
            setup_test_environment(vec![], Some(repconignore_content))?;
        let files = collect_target_files(&path_to_repo, &ignore_patterns, ignore_path.as_ref())?;
        let size = get_dir_size(&path_to_repo, &files)?;
        assert_eq!(size, file_info.file1_size() + file_info.file2_size());
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_repconignore_multiple_patterns() -> io::Result<()> {
        let repconignore_content = "*_file.*\n*_file2.*";
        let (dir, path_to_repo, ignore_patterns, ignore_path, _file_info) =
            setup_test_environment(vec![], Some(repconignore_content))?;
        let files = collect_target_files(&path_to_repo, &ignore_patterns, ignore_path.as_ref())?;
        let size = get_dir_size(&path_to_repo, &files)?;
        assert_eq!(size, 0);
        dir.close()?;
        Ok(())
    }

    #[test]
    fn test_ignore_with_repconignore() -> io::Result<()> {
        let ignore_patterns = vec!["test_file.*".to_string()];
        let repconignore_content = "test_file2.*";
        let (dir, path_to_repo, ignore_patterns, ignore_path, _file_info) =
            setup_test_environment(ignore_patterns, Some(repconignore_content))?;
        let files = collect_target_files(&path_to_repo, &ignore_patterns, ignore_path.as_ref())?;
        let size = get_dir_size(&path_to_repo, &files)?;
        assert_eq!(size, 0);
        dir.close()?;
        Ok(())
    }
}
