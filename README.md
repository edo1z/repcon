# repcon

`repcon` is a Rust-based CLI tool that efficiently condenses the entire set of files in a repository into a maximum of 20 text documents. This tool is particularly useful for developers who want to bundle their repository's files for simplified sharing and review, especially when dealing with platforms that limit the number of uploadable files.

## Features

- **Selective Inclusion**: Automatically excludes files listed in `.gitignore`, ensuring only the necessary files are included.
- **Custom Ignore Rules**: Users can specify additional patterns to ignore files or directories, providing more control over the included content.
- **Text Document Formatting**: Each file's content is enclosed within a clear start and end comment, accompanied by a header indicating the file's path and repository name for easy identification.

## Installation

To install `repcon`, use the following cargo command:

```bash
cargo install repcon
```

Make sure you have Rust and Cargo installed on your system. For more information on installing Rust, visit [the official Rust installation guide](https://www.rust-lang.org/tools/install).

## Usage

After installing `repcon` with `cargo install repcon`, navigate to your repository's root directory and run the following command:

```bash
repcon --path ./ --ignore node_modules --ignore "*.log"
```

This will process all files within the repository, excluding any matches found in `.gitignore`, the `node_modules` directory, and all `.log` files.

## Output Example

The generated text documents will have sections for each file, formatted like this:

```
# repcon_repository_name: xxxxx
# repcon_file_name: xxxxx
# repcon_page_number: 1
// START OF CODE BLOCK: xxxxx
fn main() {
    // Example code here
}
// END OF CODE BLOCK: xxxxx
```

Along with the code blocks, `repcon` will generate a directory structure to give an overview of the repository layout, helping users understand the organization of the files within the repository.

Each document's filename will follow the format `repository_name_NO.txt`, where `NO` is a sequence number.

## Custom Ignore Rules

By creating a `.repconignore` file in your repository's root directory, you can define additional ignore patterns that are specific to `repcon`. The syntax is similar to `.gitignore`, with these patterns being exclusively used by `repcon` to filter out files or directories when generating the text documents.

Example of a `.repconignore` file:

```
# This is a comment
# Ignore all .log files
*.log

# Ignore specific directory
node_modules/
```

## Error Handling

If `repcon` encounters an issue, it will provide an error message detailing the problem. Common issues may include:

- `Permission Denied`: Check you have read and write permissions for the directory where `repcon` is run.
- `File Not Found`: Verify the path and ignore patterns to ensure they are correctly specified and accessible.

## Contributing

Feel free to dive in! Open an issue or submit PRs. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](LICENSE)
