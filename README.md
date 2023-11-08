# Repo Condenser

`Repo Condenser` is a Rust-based CLI tool that efficiently condenses the entire set of files in a repository into a maximum of 20 markdown-formatted documents. This tool is especially useful for developers who want to bundle their repository's files for simplified upload and review, particularly when working with platforms that limit the number of uploadable files.

## Features

- **Selective Inclusion**: Automatically excludes files listed in `.gitignore`, ensuring only the necessary files are processed.
- **Custom Ignore Rules**: Users can specify additional patterns to ignore files or directories, granting more control over the output.
- **Markdown Formatting**: Each file's content is enclosed within markdown code blocks, accompanied by a header indicating the file's path and repository name for easy navigation and readability.

## Usage

Upon installation via `cargo install`, you can run `repo-condenser` in your repository's root directory. The tool will scan the entire directory, apply ignore rules, and generate markdown documents that encapsulate your repository's codebase.

Here's a sneak peek of how the output markdown file will look:

> \# Path/to/file.rs (Repository Name)
>
> \`\`\`
>
> code
>
> \`\`\`

In addition to the code, `Repo Condenser` will also attach a tree-like directory structure information to provide a comprehensive view of the repository's layout.

## Installation

Coming soon...

## Contributing

Feel free to dive in! Open an issue or submit PRs.

## License

[MIT](LICENSE)