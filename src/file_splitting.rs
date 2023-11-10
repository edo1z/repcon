use crate::output_formatting::{create_page_footer, create_page_header};
use std::fs::File;
use std::io::{self, BufRead, BufReader, Write};
use std::path::{Path, PathBuf};

// Function to create a new file and write the initial page header
fn create_new_file(
    output_directory: &Path,
    file_counter: u64,
    repository_name: &str,
) -> io::Result<(File, PathBuf)> {
    let file_path = output_directory.join(format!("{}_{}.txt", repository_name, file_counter));
    let file = File::create(&file_path)?;
    Ok((file, file_path))
}

pub fn split_files_into_chunks(
    files: &[PathBuf],
    output_directory: &Path,
    max_file_size: u64,
    repository_name: &str,
) -> io::Result<Vec<PathBuf>> {
    let mut generated_files = Vec::new();
    let mut file_counter: u64 = 1;
    let mut current_file_size: u64 = 0;
    let mut current_file_name = "";
    let mut page_number: u64;
    let (mut output_file, mut output_file_path) =
        create_new_file(output_directory, file_counter, repository_name)?;
    generated_files.push(output_file_path);
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
            (output_file, output_file_path) =
                create_new_file(output_directory, file_counter, repository_name)?;
            generated_files.push(output_file_path);
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
                (output_file, output_file_path) =
                    create_new_file(output_directory, file_counter, repository_name)?;
                generated_files.push(output_file_path);

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
    Ok(generated_files)
}
