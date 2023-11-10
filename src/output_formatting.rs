pub fn create_page_header(repository_name: &str, file_path: &str, page_number: u64) -> String {
    format!(
        "# repcon_repository_name: {}\n# repcon_file_name: {}\n# repcon_page_number: {}\n// START OF CODE BLOCK: {}\n",
        repository_name,
        file_path,
        page_number,
        file_path
    )
}

pub fn create_page_footer(file_path: &str) -> String {
    format!("// END OF CODE BLOCK: {}\n", file_path)
}
