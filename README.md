# repcon

`repcon` is a Rust-based CLI tool that efficiently condenses the entire set of files in a repository into a maximum of 20 markdown-formatted documents. This tool is especially useful for developers who want to bundle their repository's files for simplified upload and review, particularly when working with platforms that limit the number of uploadable files.

## Features

- **Selective Inclusion**: Automatically excludes files listed in `.gitignore`, ensuring only the necessary files are processed.
- **Custom Ignore Rules**: Users can specify additional patterns to ignore files or directories, granting more control over the output.
- **Markdown Formatting**: Each file's content is enclosed within markdown code blocks, accompanied by a header indicating the file's path and repository name for easy navigation and readability.

## Usage

Upon installation via `cargo install repcon`, navigate to your repository's root directory and run the following command:

```bash
repcon --path ./ --ignore node_modules --ignore "*.log"
```

This command will process all files in the repository, excluding anything matched by `.gitignore`, the `node_modules` directory, and all `.log` files.

## Output Example

The generated markdown document will have sections for each file, like so:

````
# src/main.rs (Repository Name)

```
fn main() {
    let message = "Hello, repcon users!";
    println!("{}", message);

    let sum = add(5, 3);
    println!("5 + 3 = {}", sum);
}

fn add(a: i32, b: i32) -> i32 {
    a + b
}

```
````

Along with the code, `repcon` will generate a directory structure to give an overview of the repository layout.

## Installation

To install `repcon`, use the following command:

```bash
cargo install repcon
```

Ensure you have Rust and Cargo installed on your system. For more information on installing Rust, visit [the official Rust installation guide](https://www.rust-lang.org/tools/install).

## Error Handling

If `repcon` encounters an issue, it will provide an error message with details. For common issues:

- `Permission Denied`: Ensure you have read and write permissions for the directory where repcon is run.
- `File Not Found`: Verify the path and ignore patterns to ensure they are correct and accessible.

## Contributing

Feel free to dive in! Open an issue or submit PRs. For major changes, please open an issue first to discuss what you would like to change.

## License

[MIT](LICENSE)
