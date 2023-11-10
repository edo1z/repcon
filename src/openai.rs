use reqwest::{self, multipart};
use std::env;
use std::io;
use std::path::Path;
use tokio::fs::File;
use tokio::io::AsyncReadExt;

pub fn upload_to_openai(upload: &Option<Option<String>>) -> io::Result<()> {
    match upload {
        Some(Some(api_key)) => {
            println!("Uploading with provided API key.");
            println!("{}", api_key)
        }
        Some(None) => match env::var("OPENAI_API_KEY") {
            Ok(env_api_key) => {
                println!("Uploading with API key from environment variable.");
                println!("{}", env_api_key)
            }
            Err(_) => {
                println!("API key not specified and not found in environment. Skipping upload.");
            }
        },
        None => {
            println!("No upload option provided. Skipping upload.");
        }
    }
    Ok(())
}

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
