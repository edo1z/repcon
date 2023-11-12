use std::process::Command;

#[test]
fn test_output_directory_specified() {
    let output = Command::new("cargo")
        .args(["run", ".", "-o", "output/test_output"])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
}

#[test]
fn test_output_filename_specified() {
    let output = Command::new("cargo")
        .args(["run", ".", "-n", "custom_output"])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
}

#[test]
fn test_ignore_pattern_specified() {
    let output = Command::new("cargo")
        .args(["run", ".", "-i", "temp*"])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
}

#[test]
fn test_max_files_specified() {
    let output = Command::new("cargo")
        .args(["run", ".", "-f", "5"])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
}

#[test]
fn test_max_file_size_specified() {
    let output = Command::new("cargo")
        .args(["run", ".", "-s", "100"])
        .output()
        .expect("Failed to execute command");
    assert!(output.status.success());
}
