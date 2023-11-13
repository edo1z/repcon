# repcon

![Crates.io](https://img.shields.io/crates/v/repcon)
![MIT](https://img.shields.io/github/license/edo1z/repcon)
![Downloads](https://img.shields.io/crates/d/repcon)
![CodeSize](https://img.shields.io/github/languages/code-size/edo1z/repcon)


`repcon` automatically combines a group of files in the repository into one file. 20 files can be uploaded to the OpenAI assistant, but I wanted to upload the entire repository. You can install cargo.

Note that `repcon` is an independent project, not officially affiliated with OpenAI.

# Features

- Automatically ignores files set to `.gitignore`.
- Additional ignore file settings can be added with `.repconignore` or `-i` options.
- You can set the maximum size of one file and the maximum number of files.
- Non-text files are automatically ignored.

## Installation

```bash
cargo install repcon
```

## Usage

Execute a command in the form `repcon <repository path> <options>`. Where `repository path` is the path to the root directory of the repository you want to aggregate. Absolute paths can also be used.

```bash
repcon . -i "*.log"
```

## Output Example

The generated text documents will have sections for each file, formatted like this:

```
# repcon_file_name: xxxxx
# repcon_page_number: 1
// START OF CODE BLOCK: xxxxx
fn main() {
    // Example code here
}
// END OF CODE BLOCK: xxxxx
```

Files will be named according to the format specified by the user, or default to `{output_file_name}_{file_no}.txt`, where `file_no` is a sequence number.

## Custom Ignore Rules

You can define `repcon` specific ignore patterns by creating a `.repconignore` file. The syntax is similar to `.gitignore`, and these patterns are only used to filter files and directories when `repcon` generates text documents.

Example of a `.repconignore` file:

```
# This is a comment
# Ignore all .log files
*.log

# Ignore specific directory
node_modules/
```

## Contributing

PR is always welcome. Thank you.

## License

[MIT](LICENSE)
