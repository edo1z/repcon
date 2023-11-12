use reqwest::{self, multipart};
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

/// Asynchronously uploads a file to OpenAI using the provided API key.
///
/// This function reads the file asynchronously, creates a multipart form with the file and purpose,
/// and sends it to the OpenAI API for uploading.
///
/// # Arguments
/// * `api_key` - The API key for authentication with OpenAI.
/// * `file_path` - The path of the file to be uploaded.
/// * `purpose` - The purpose of the file upload, typically related to its intended use.
///
/// # Returns
/// Result indicating success or failure.
pub async fn upload_file_to_openai(
    api_key: &str,
    file_path: &str,
    purpose: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let client = reqwest::Client::new();

    // Asynchronously read the file contents
    let path = Path::new(file_path);
    let mut file = File::open(path).await?;
    let mut contents = Vec::new();
    file.read_to_end(&mut contents).await?;

    // Create a file Part from the contents
    let file_part = multipart::Part::bytes(contents)
        .file_name(file_path.to_string())
        .mime_str("text/plain")?;

    // Create the multipart Form
    let form = multipart::Form::new()
        .part("file", file_part)
        .text("purpose", purpose.to_string());

    // Send the request
    let response = client
        .post("https://api.openai.com/v1/files")
        .bearer_auth(api_key)
        .multipart(form)
        .send()
        .await?;

    if response.status().is_success() {
        println!("File uploaded successfully!");
    } else {
        println!("Failed to upload file: {:?}", response.text().await?);
    }

    Ok(())
}
