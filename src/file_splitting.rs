use crate::PageFormat;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

// Function to create a new file and write the initial page header
fn create_new_output_file(
    output_directory: &Path,
    file_counter: u64,
    output_name: &str,
) -> io::Result<(File, PathBuf)> {
    let file_path = output_directory.join(format!("{}_{}.txt", output_name, file_counter));
    let file = File::create(&file_path)?;
    Ok((file, file_path))
}

fn check_max_output_file_size(
    page_format: &PageFormat,
    max_output_file_size: u64,
) -> io::Result<()> {
    if page_format.get_page_header_size() + page_format.get_page_footer_size()
        > max_output_file_size
    {
        let error_message = format!(
            "Error: The maximum file size ({}) is too small to contain the page header and footer.",
            max_output_file_size
        );
        Err(io::Error::new(io::ErrorKind::InvalidData, error_message))
    } else {
        Ok(())
    }
}

fn next_output_file(
    output_directory: &Path,
    output_file_counter: &mut u64,
    output_name: &str,
    current_output_file_size: &mut u64,
    generated_output_files: &mut Vec<PathBuf>,
    output_file: &mut File,
    output_file_path: &mut PathBuf,
) -> io::Result<()> {
    *output_file_counter += 1;
    *current_output_file_size = 0;
    (*output_file, *output_file_path) =
        create_new_output_file(output_directory, *output_file_counter, output_name)?;
    generated_output_files.push(output_file_path.to_path_buf());
    Ok(())
}

pub fn split_files_into_chunks(
    target_files: &[PathBuf],
    output_directory: &Path,
    max_output_file_size: u64,
    output_name: &str,
) -> io::Result<Vec<PathBuf>> {
    let mut generated_output_files = Vec::new();
    let mut output_file_counter: u64 = 1;
    let mut current_output_file_size: u64 = 0;
    let mut current_target_file_name: &str;
    let mut page_format: PageFormat;

    // Create the first file
    let (mut output_file, mut output_file_path) =
        create_new_output_file(output_directory, output_file_counter, output_name)?;
    generated_output_files.push(output_file_path.clone());

    for target_file_path in target_files {
        current_target_file_name = match target_file_path.to_str() {
            Some(name) => name,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "Target File path contains invalid UTF-8 characters",
                ));
            }
        };
        page_format = PageFormat::new(current_target_file_name);
        check_max_output_file_size(&page_format, max_output_file_size)?;

        if current_output_file_size + page_format.header_size + page_format.footer_size
            > max_output_file_size
        {
            next_output_file(
                output_directory,
                &mut output_file_counter,
                output_name,
                &mut current_output_file_size,
                &mut generated_output_files,
                &mut output_file,
                &mut output_file_path.clone(),
            )?;
        }

        write!(output_file, "{}", page_format.header)?;
        current_output_file_size += page_format.header_size;

        let file = File::open(target_file_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let line_size = line.as_bytes().len() as u64 + 1; // +1 for the newline character

            if current_output_file_size + line_size + page_format.footer_size > max_output_file_size
            {
                write!(output_file, "{}", page_format.footer)?;

                next_output_file(
                    output_directory,
                    &mut output_file_counter,
                    output_name,
                    &mut current_output_file_size,
                    &mut generated_output_files,
                    &mut output_file,
                    &mut output_file_path,
                )?;

                page_format.increment_page_number();
                check_max_output_file_size(&page_format, max_output_file_size)?;
                write!(output_file, "{}", page_format.header)?;
                current_output_file_size += page_format.header_size;
            }

            writeln!(output_file, "{}", line)?;
            current_output_file_size += line_size;
        }
        write!(output_file, "{}", page_format.footer)?;
    }
    Ok(generated_output_files)
}

#[cfg(test)]
mod split_tests {
    use super::*;
    use std::fs::{self, File};
    use std::io::Write;
    use tempfile::tempdir;

    #[test]
    fn test_split_files_into_small_chunks() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let max_output_file_size = 300;
        let output_name = "output";
        let num_test_files = 5;
        let mut files = Vec::new();

        for i in 0..num_test_files {
            let file_path = temp_dir.path().join(format!("test_file_{}.txt", i));
            let mut test_file = File::create(&file_path)?;
            writeln!(test_file, "Test data for file {}", i)?;
            files.push(file_path);
        }

        let output_directory = temp_dir.path();
        let generated_output_files =
            split_files_into_chunks(&files, output_directory, max_output_file_size, output_name)?;

        assert!(!generated_output_files.is_empty());
        println!("Generated files: {:?}", generated_output_files);
        assert_eq!(generated_output_files.len(), num_test_files);
        for generated_file_path in generated_output_files {
            let generated_file_content = fs::read_to_string(generated_file_path)?;
            assert!(generated_file_content.contains("// START OF CODE BLOCK"));
            assert!(generated_file_content.contains("// END OF CODE BLOCK"));
            println!("Generated file content: {}", generated_file_content);
            println!("Generated file size: {}", generated_file_content.len());
            assert!(generated_file_content.len() as u64 <= max_output_file_size);
        }

        Ok(())
    }

    #[test]
    fn test_split_files_into_single_large_chunk() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let max_output_file_size = 3000;
        let output_name = "output";
        let num_test_files = 5;
        let mut files = Vec::new();

        for i in 0..num_test_files {
            let file_path = temp_dir.path().join(format!("test_file_{}.txt", i));
            let mut test_file = File::create(&file_path)?;
            writeln!(test_file, "Test data for file {}", i)?;
            files.push(file_path);
        }

        let output_directory = temp_dir.path();
        let generated_output_files =
            split_files_into_chunks(&files, output_directory, max_output_file_size, output_name)?;

        assert!(!generated_output_files.is_empty());
        println!("Generated files: {:?}", generated_output_files);
        assert_eq!(generated_output_files.len(), 1);
        for generated_file_path in generated_output_files {
            let generated_file_content = fs::read_to_string(generated_file_path)?;
            assert!(generated_file_content.contains("// START OF CODE BLOCK"));
            assert!(generated_file_content.contains("// END OF CODE BLOCK"));
            println!("Generated file content: {}", generated_file_content);
            println!("Generated file size: {}", generated_file_content.len());
            assert!(generated_file_content.len() as u64 <= max_output_file_size);
        }

        Ok(())
    }

    #[test]
    fn test_split_files_with_insufficient_size() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let max_output_file_size = 10;
        let output_name = "output";
        let mut files = Vec::new();

        let file_path = temp_dir.path().join("test_file.txt");
        let mut test_file = File::create(&file_path)?;
        writeln!(test_file, "Test data for file")?;
        files.push(file_path);

        let output_directory = temp_dir.path();
        let result =
            split_files_into_chunks(&files, output_directory, max_output_file_size, output_name);

        assert!(result.is_err());
        Ok(())
    }

    #[test]
    fn test_split_binary_files_error() -> io::Result<()> {
        let temp_dir = tempdir()?;
        let max_output_file_size = 300;
        let output_name = "output";
        let mut files = Vec::new();

        let file_path = temp_dir.path().join("test_file.bin");
        let mut test_file = File::create(&file_path)?;
        test_file.write_all(&[0, 159, 146, 150])?;
        files.push(file_path);

        let output_directory = temp_dir.path();
        let result =
            split_files_into_chunks(&files, output_directory, max_output_file_size, output_name);

        assert!(result.is_err());
        Ok(())
    }
}
