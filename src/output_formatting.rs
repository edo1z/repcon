use std::path::{Path, PathBuf};

/// Represents the format of a page in the output file.
///
/// This struct holds the header and footer strings for a specific file,
/// along with their respective sizes, the path of the file, and the current page number.
pub struct PageFormat {
    pub header: String,
    pub footer: String,
    pub header_size: u64,
    pub footer_size: u64,
    pub file_path: String,
    pub page_nubmer: u64,
}
impl PageFormat {
    /// Constructs a new `PageFormat` instance for a given file path.
    ///
    /// Initializes the header and footer for the first page of the file,
    /// calculates their sizes, and sets the initial page number to 1.
    ///
    /// # Arguments
    /// * `file_path` - The path of the file for which the page format is being created.
    pub fn new(file_path: String, root: Option<&Path>) -> Self {
        let file_path = if let Some(root) = root {
            Self::to_relative_path(root, Path::new(&file_path))
                .to_str()
                .unwrap()
                .to_string()
        } else {
            file_path
        };
        let header = PageFormat::create_page_header(&file_path, 1);
        let footer = PageFormat::create_page_footer(&file_path);
        let header_size = header.as_bytes().len() as u64;
        let footer_size = footer.as_bytes().len() as u64;
        Self {
            header,
            footer,
            header_size,
            footer_size,
            file_path: file_path.to_string(),
            page_nubmer: 1,
        }
    }

    /// Creates a header string for a given file path and page number.
    ///
    /// The header includes the file name and page number.
    ///
    /// # Arguments
    /// * `file_path` - The file path for which the header is being created.
    /// * `page_number` - The current page number.
    pub fn create_page_header(file_path: &str, page_number: u64) -> String {
        format!(
            "# repcon_file_name: {}\n# repcon_page_number: {}\n// START OF CODE BLOCK: {}\n",
            file_path, page_number, file_path
        )
    }

    /// Creates a footer string for a given file path.
    ///
    /// The footer marks the end of a code block for the file.
    ///
    /// # Arguments
    /// * `file_path` - The file path for which the footer is being created.
    pub fn create_page_footer(file_path: &str) -> String {
        format!("// END OF CODE BLOCK: {}\n\n", file_path)
    }

    /// Converts an absolute file path to a relative path based on the root directory.
    /// If the file path is not relative to the root, returns the original path.
    pub fn to_relative_path(root: &Path, file_path: &Path) -> PathBuf {
        file_path
            .strip_prefix(root)
            .unwrap_or(file_path)
            .to_path_buf()
    }

    /// Returns the size of the current page header.
    pub fn get_page_header_size(&self) -> u64 {
        self.header.as_bytes().len() as u64
    }

    /// Returns the size of the current page footer.
    pub fn get_page_footer_size(&self) -> u64 {
        self.footer.as_bytes().len() as u64
    }

    /// Increments the page number and updates the header and footer.
    ///
    /// This method should be called when moving to the next page of the output file.
    /// It updates the page number, and recalculates the header and footer along with their sizes.
    pub fn increment_page_number(&mut self) {
        self.page_nubmer += 1;
        self.header = PageFormat::create_page_header(&self.file_path, self.page_nubmer);
        self.footer = PageFormat::create_page_footer(&self.file_path);
        self.header_size = self.get_page_header_size();
        self.footer_size = self.get_page_footer_size();
    }
}

#[cfg(test)]
mod output_formattings_tests {
    use super::*;

    #[test]
    fn test_page_format_new() {
        let file_path = "test_file.rs";
        let page_format = PageFormat::new(file_path.to_string(), None);

        assert_eq!(page_format.file_path, file_path);
        assert!(!page_format.header.is_empty());
        assert!(!page_format.footer.is_empty());
    }

    #[test]
    fn test_page_format_increment_page_number() {
        let mut page_format = PageFormat::new("test_file.rs".to_string(), None);
        let initial_page_number = page_format.page_nubmer;

        page_format.increment_page_number();

        assert_eq!(page_format.page_nubmer, initial_page_number + 1);
    }
}
