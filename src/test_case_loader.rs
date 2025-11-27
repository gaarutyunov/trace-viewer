use crate::models::*;
use base64::{engine::general_purpose, Engine as _};
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[derive(Debug)]
pub enum TestCaseLoadError {
    ZipError(String),
    IoError(String),
    ParseError(String),
}

impl std::fmt::Display for TestCaseLoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            TestCaseLoadError::ZipError(e) => write!(f, "ZIP error: {}", e),
            TestCaseLoadError::IoError(e) => write!(f, "IO error: {}", e),
            TestCaseLoadError::ParseError(e) => write!(f, "Parse error: {}", e),
        }
    }
}

impl std::error::Error for TestCaseLoadError {}

/// Load test cases from a ZIP archive containing test case folders
/// Expected structure:
/// - test-case-1/
///   - error-context.md
///   - test-failed-1.png
///   - trace.zip
///   - video.webm
pub fn load_test_cases_from_zip(bytes: &[u8]) -> Result<TestCaseCollection, TestCaseLoadError> {
    log::info!("Parsing test cases ZIP archive...");

    let cursor = Cursor::new(bytes);
    let mut archive =
        ZipArchive::new(cursor).map_err(|e| TestCaseLoadError::ZipError(e.to_string()))?;

    log::info!("ZIP archive opened, {} entries found", archive.len());

    // Group files by test case folder
    let mut test_case_folders: std::collections::HashMap<String, Vec<String>> =
        std::collections::HashMap::new();

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| TestCaseLoadError::ZipError(e.to_string()))?;
        let name = file.name().to_string();

        // Skip directories and __MACOSX
        if file.is_dir() || name.starts_with("__MACOSX") || name.starts_with("._") {
            continue;
        }

        // Extract folder name
        if let Some(folder) = extract_folder_name(&name) {
            test_case_folders
                .entry(folder.to_string())
                .or_default()
                .push(name.clone());
        }
    }

    log::info!("Found {} test case folders", test_case_folders.len());

    let mut test_cases = Vec::new();

    for (folder_name, files) in test_case_folders {
        log::info!("Processing test case folder: {}", folder_name);

        match load_test_case_from_folder(&mut archive, &folder_name, &files) {
            Ok(test_case) => test_cases.push(test_case),
            Err(e) => {
                log::warn!("Failed to load test case {}: {}", folder_name, e);
                // Continue processing other test cases
            }
        }
    }

    log::info!("Loaded {} test cases", test_cases.len());

    Ok(TestCaseCollection { test_cases })
}

fn extract_folder_name(path: &str) -> Option<&str> {
    // Remove leading and trailing slashes
    let path = path.trim_start_matches('/').trim_end_matches('/');

    // Get the first path component
    path.split('/').next()
}

fn load_test_case_from_folder(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    folder_name: &str,
    files: &[String],
) -> Result<TestCase, TestCaseLoadError> {
    let mut markdown_content = None;
    let mut screenshots = Vec::new();
    let mut video = None;
    let mut trace_file = None;

    for file_path in files {
        let file_name = file_path
            .split('/')
            .next_back()
            .unwrap_or(file_path)
            .to_lowercase();

        if file_name.ends_with(".md") {
            // Load markdown file
            markdown_content = Some(read_text_file_from_archive(archive, file_path)?);
        } else if file_name.ends_with(".png")
            || file_name.ends_with(".jpg")
            || file_name.ends_with(".jpeg")
        {
            // Load screenshot
            let attachment = load_binary_file_as_attachment(archive, file_path)?;
            screenshots.push(attachment);
        } else if file_name.ends_with(".webm") || file_name.ends_with(".mp4") {
            // Load video
            video = Some(load_binary_file_as_attachment(archive, file_path)?);
        } else if file_name.ends_with(".zip") && file_name.contains("trace") {
            // Load trace file
            trace_file = Some(load_binary_file_as_attachment(archive, file_path)?);
        }
    }

    // Determine test status based on folder name and presence of error-context.md
    let status = if folder_name.to_lowercase().contains("fail")
        || folder_name.to_lowercase().contains("error")
        || markdown_content.is_some()
    {
        TestStatus::Failed
    } else {
        TestStatus::Passed
    };

    // Extract error message from markdown if available
    let error_message = if status == TestStatus::Failed {
        markdown_content
            .as_ref()
            .and_then(|md| extract_first_line(md))
    } else {
        None
    };

    Ok(TestCase {
        id: folder_name.to_string(),
        name: format_test_name(folder_name),
        status,
        markdown_content,
        screenshots,
        video,
        trace_file,
        duration_ms: None,
        error_message,
    })
}

fn read_text_file_from_archive(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<String, TestCaseLoadError> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| TestCaseLoadError::ZipError(format!("Failed to read {}: {}", name, e)))?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| TestCaseLoadError::IoError(e.to_string()))?;

    Ok(content)
}

fn load_binary_file_as_attachment(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<TestAttachment, TestCaseLoadError> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| TestCaseLoadError::ZipError(format!("Failed to read {}: {}", name, e)))?;

    let mut bytes = Vec::new();
    file.read_to_end(&mut bytes)
        .map_err(|e| TestCaseLoadError::IoError(e.to_string()))?;

    let size_bytes = bytes.len();

    // Determine MIME type from extension
    let mime_type = determine_mime_type(name);

    // Encode as base64 data URL
    let base64_data = general_purpose::STANDARD.encode(&bytes);
    let data_url = format!("data:{};base64,{}", mime_type, base64_data);

    let file_name = name.split('/').next_back().unwrap_or(name).to_string();

    Ok(TestAttachment {
        name: file_name,
        mime_type: mime_type.to_string(),
        data_url,
        size_bytes: Some(size_bytes),
    })
}

fn determine_mime_type(filename: &str) -> &str {
    let filename = filename.to_lowercase();
    if filename.ends_with(".png") {
        "image/png"
    } else if filename.ends_with(".jpg") || filename.ends_with(".jpeg") {
        "image/jpeg"
    } else if filename.ends_with(".webm") {
        "video/webm"
    } else if filename.ends_with(".mp4") {
        "video/mp4"
    } else if filename.ends_with(".zip") {
        "application/zip"
    } else {
        "application/octet-stream"
    }
}

fn format_test_name(folder_name: &str) -> String {
    // Convert folder name to readable test name
    // e.g., "test-case-1" -> "Test Case 1"
    folder_name
        .replace(['-', '_'], " ")
        .split_whitespace()
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().collect::<String>() + chars.as_str(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn extract_first_line(text: &str) -> Option<String> {
    text.lines()
        .find(|line| !line.trim().is_empty())
        .map(|line| line.trim().to_string())
}
