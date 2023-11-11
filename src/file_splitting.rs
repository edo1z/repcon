use crate::PageFormat;
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

// Function to create a new file and write the initial page header
fn create_new_file(
    output_directory: &Path,
    file_counter: u64,
    output_name: &str,
) -> io::Result<(File, PathBuf)> {
    let file_path = output_directory.join(format!("{}_{}.txt", output_name, file_counter));
    let file = File::create(&file_path)?;
    Ok((file, file_path))
}

fn check_max_file_size(page_format: &PageFormat, max_file_size: u64) -> io::Result<()> {
    if page_format.get_page_header_size() + page_format.get_page_footer_size() > max_file_size {
        let error_message = format!(
            "Error: The maximum file size ({}) is too small to contain the page header and footer.",
            max_file_size
        );
        Err(io::Error::new(io::ErrorKind::InvalidData, error_message))
    } else {
        Ok(())
    }
}

pub fn split_files_into_chunks(
    files: &[PathBuf],
    output_directory: &Path,
    max_file_size: u64,
    output_name: &str,
) -> io::Result<Vec<PathBuf>> {
    let mut generated_files = Vec::new();
    let mut file_counter: u64 = 1;
    let mut current_file_size: u64 = 0;
    let mut current_file_name: &str;
    let mut page_format: PageFormat;

    // Create the first file
    let (mut output_file, mut output_file_path) =
        create_new_file(output_directory, file_counter, output_name)?;
    generated_files.push(output_file_path);

    for file_path in files {
        current_file_name = match file_path.to_str() {
            Some(name) => name,
            None => {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "File path contains invalid UTF-8 characters",
                ));
            }
        };
        page_format = PageFormat::new(current_file_name);
        check_max_file_size(&page_format, max_file_size)?;

        if current_file_size + page_format.header_size > max_file_size {
            file_counter += 1;
            current_file_size = 0;
            (output_file, output_file_path) =
                create_new_file(output_directory, file_counter, output_name)?;
            generated_files.push(output_file_path);
            continue;
        } else {
            writeln!(output_file, "{}", page_format.header)?;
            current_file_size += page_format.header_size;
        }

        let file = File::open(file_path)?;
        let reader = BufReader::new(file);
        for line in reader.lines() {
            let line = line?;
            let line_size = line.as_bytes().len() as u64 + 1; // +1 for the newline character

            if current_file_size + line_size + page_format.footer_size > max_file_size {
                write!(output_file, "{}", page_format.footer)?;

                file_counter += 1;
                current_file_size = 0;
                (output_file, output_file_path) =
                    create_new_file(output_directory, file_counter, output_name)?;
                generated_files.push(output_file_path);

                page_format.increment_page_number();
                check_max_file_size(&page_format, max_file_size)?;
                writeln!(output_file, "{}", page_format.header)?;
                current_file_size += page_format.header_size;
            }

            writeln!(output_file, "{}", line)?;
            current_file_size += line_size;
        }
        write!(output_file, "{}", page_format.footer)?;
    }
    Ok(generated_files)
}
