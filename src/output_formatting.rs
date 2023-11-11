pub struct PageFormat {
    pub header: String,
    pub footer: String,
    pub header_size: u64,
    pub footer_size: u64,
    pub file_path: String,
    pub page_nubmer: u64,
}
impl PageFormat {
    pub fn new(file_path: &str) -> Self {
        let header = PageFormat::create_page_header(file_path, 1);
        let footer = PageFormat::create_page_footer(file_path);
        let header_size = header.as_bytes().len() as u64 + 1; // +1 for the newline character
        let footer_size = footer.as_bytes().len() as u64 + 1; // +1 for the newline character
        Self {
            header,
            footer,
            header_size,
            footer_size,
            file_path: file_path.to_string(),
            page_nubmer: 1,
        }
    }

    pub fn create_page_header(file_path: &str, page_number: u64) -> String {
        format!(
            "# repcon_file_name: {}\n# repcon_page_number: {}\n// START OF CODE BLOCK: {}\n",
            file_path, page_number, file_path
        )
    }

    pub fn create_page_footer(file_path: &str) -> String {
        format!("// END OF CODE BLOCK: {}\n", file_path)
    }

    pub fn get_page_header_size(&self) -> u64 {
        self.header.as_bytes().len() as u64 + 1 // +1 for the newline character
    }

    pub fn get_page_footer_size(&self) -> u64 {
        self.footer.as_bytes().len() as u64 + 1 // +1 for the newline character
    }

    pub fn increment_page_number(&mut self) {
        self.page_nubmer += 1;
        self.header = PageFormat::create_page_header(&self.file_path, self.page_nubmer);
        self.footer = PageFormat::create_page_footer(&self.file_path);
        self.header_size = self.get_page_header_size();
        self.footer_size = self.get_page_footer_size();
    }
}
