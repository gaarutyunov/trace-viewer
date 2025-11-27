use base64::Engine;
use std::fs;
use trace_viewer::models::{TestCase, TestStatus};
use trace_viewer::test_case_loader::{load_test_cases_from_zip, TestCaseLoadError};

#[test]
fn test_load_test_cases_from_valid_zip() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let result = load_test_cases_from_zip(&bytes);

    assert!(result.is_ok(), "Failed to load test cases: {:?}", result);

    let test_cases = result.unwrap();
    assert!(
        !test_cases.test_cases.is_empty(),
        "Expected at least one test case"
    );

    // Verify we have multiple test cases
    assert!(
        test_cases.test_cases.len() >= 3,
        "Expected at least 3 test cases, got {}",
        test_cases.test_cases.len()
    );
}

#[test]
fn test_test_case_has_required_fields() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    let first_test = &test_cases.test_cases[0];

    // Verify required fields
    assert!(
        !first_test.id.is_empty(),
        "Test case ID should not be empty"
    );
    assert!(
        !first_test.name.is_empty(),
        "Test case name should not be empty"
    );

    // Verify test status is set
    assert!(
        matches!(
            first_test.status,
            TestStatus::Failed | TestStatus::Passed | TestStatus::Skipped | TestStatus::Pending
        ),
        "Test status should be valid"
    );
}

#[test]
fn test_test_case_has_markdown_content() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    // Find a test case with markdown content
    let test_with_markdown = test_cases
        .test_cases
        .iter()
        .find(|tc| tc.markdown_content.is_some());

    assert!(
        test_with_markdown.is_some(),
        "Expected at least one test case with markdown content"
    );

    let markdown = test_with_markdown
        .unwrap()
        .markdown_content
        .as_ref()
        .unwrap();

    assert!(!markdown.is_empty(), "Markdown content should not be empty");
    assert!(
        markdown.contains("Page snapshot") || markdown.contains("#"),
        "Markdown should contain expected content"
    );
}

#[test]
fn test_test_case_has_screenshots() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    // Find a test case with screenshots
    let test_with_screenshots = test_cases
        .test_cases
        .iter()
        .find(|tc| !tc.screenshots.is_empty());

    assert!(
        test_with_screenshots.is_some(),
        "Expected at least one test case with screenshots"
    );

    let screenshot = &test_with_screenshots.unwrap().screenshots[0];

    // Verify screenshot properties
    assert!(
        !screenshot.name.is_empty(),
        "Screenshot name should not be empty"
    );
    assert!(
        screenshot.mime_type.starts_with("image/"),
        "Screenshot MIME type should be image/*"
    );
    assert!(
        screenshot.data_url.starts_with("data:image/"),
        "Screenshot should have data URL"
    );
    assert!(
        screenshot.data_url.contains("base64"),
        "Screenshot data URL should contain base64 encoding"
    );
    assert!(
        screenshot.size_bytes.is_some() && screenshot.size_bytes.unwrap() > 0,
        "Screenshot should have size information"
    );
}

#[test]
fn test_test_case_has_video() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    // Find a test case with video
    let test_with_video = test_cases.test_cases.iter().find(|tc| tc.video.is_some());

    assert!(
        test_with_video.is_some(),
        "Expected at least one test case with video"
    );

    let video = test_with_video.unwrap().video.as_ref().unwrap();

    // Verify video properties
    assert!(!video.name.is_empty(), "Video name should not be empty");
    assert!(
        video.mime_type.starts_with("video/"),
        "Video MIME type should be video/*"
    );
    assert!(
        video.data_url.starts_with("data:video/"),
        "Video should have data URL"
    );
    assert!(
        video.data_url.contains("base64"),
        "Video data URL should contain base64 encoding"
    );
    assert!(
        video.size_bytes.is_some() && video.size_bytes.unwrap() > 0,
        "Video should have size information"
    );
}

#[test]
fn test_test_case_has_trace_file() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    // Find a test case with trace file
    let test_with_trace = test_cases
        .test_cases
        .iter()
        .find(|tc| tc.trace_file.is_some());

    assert!(
        test_with_trace.is_some(),
        "Expected at least one test case with trace file"
    );

    let trace = test_with_trace.unwrap().trace_file.as_ref().unwrap();

    // Verify trace file properties
    assert!(!trace.name.is_empty(), "Trace name should not be empty");
    assert!(
        trace.mime_type.contains("zip") || trace.mime_type.contains("application"),
        "Trace MIME type should be application/zip or similar"
    );
    assert!(
        trace.data_url.starts_with("data:"),
        "Trace should have data URL"
    );
    assert!(
        trace.size_bytes.is_some() && trace.size_bytes.unwrap() > 0,
        "Trace should have size information"
    );
}

#[test]
fn test_test_case_name_formatting() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    for test_case in &test_cases.test_cases {
        // Test case names should be formatted (capitalized, spaces instead of dashes)
        assert!(
            !test_case.name.contains("--"),
            "Test case name should not contain double dashes: {}",
            test_case.name
        );

        // Names should start with capital letter
        let first_char = test_case.name.chars().next().unwrap();
        assert!(
            first_char.is_uppercase() || first_char.is_numeric(),
            "Test case name should start with capital letter or number: {}",
            test_case.name
        );
    }
}

#[test]
fn test_test_case_status_detection() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    // All test cases in this fixture should be marked as failed (they have error-context.md)
    for test_case in &test_cases.test_cases {
        assert_eq!(
            test_case.status,
            TestStatus::Failed,
            "Test case '{}' should be marked as failed",
            test_case.name
        );
    }
}

#[test]
fn test_test_case_error_message_extraction() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    // Find test cases with error messages
    let tests_with_errors: Vec<&TestCase> = test_cases
        .test_cases
        .iter()
        .filter(|tc| tc.error_message.is_some())
        .collect();

    assert!(
        !tests_with_errors.is_empty(),
        "Expected at least one test case with error message"
    );

    for test_case in tests_with_errors {
        let error_msg = test_case.error_message.as_ref().unwrap();
        assert!(
            !error_msg.is_empty(),
            "Error message should not be empty for test '{}'",
            test_case.name
        );
    }
}

#[test]
fn test_load_invalid_zip() {
    let invalid_bytes = b"This is not a valid ZIP file";
    let result = load_test_cases_from_zip(invalid_bytes);

    assert!(result.is_err(), "Should fail to load invalid ZIP");
    assert!(matches!(
        result.unwrap_err(),
        TestCaseLoadError::ZipError(_)
    ));
}

#[test]
fn test_load_empty_zip() {
    // Create a minimal valid ZIP file with no entries
    use std::io::Cursor;
    use zip::write::ZipWriter;

    let mut cursor = Cursor::new(Vec::new());
    {
        let _zip = ZipWriter::new(&mut cursor);
        // Don't add any files
    }

    let bytes = cursor.into_inner();
    let result = load_test_cases_from_zip(&bytes);

    // Should succeed but return no test cases
    assert!(result.is_ok(), "Should handle empty ZIP gracefully");
    let test_cases = result.unwrap();
    assert_eq!(
        test_cases.test_cases.len(),
        0,
        "Empty ZIP should result in no test cases"
    );
}

#[test]
fn test_multiple_screenshots_per_test() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    // Check that we can handle test cases with multiple screenshots
    let total_screenshots: usize = test_cases
        .test_cases
        .iter()
        .map(|tc| tc.screenshots.len())
        .sum();

    assert!(
        total_screenshots > 0,
        "Should have at least one screenshot across all tests"
    );
}

#[test]
fn test_test_case_id_uniqueness() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    let mut ids = std::collections::HashSet::new();
    for test_case in &test_cases.test_cases {
        assert!(
            ids.insert(&test_case.id),
            "Duplicate test case ID found: {}",
            test_case.id
        );
    }
}

#[test]
fn test_screenshot_data_url_is_valid_base64() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    for test_case in &test_cases.test_cases {
        for screenshot in &test_case.screenshots {
            // Extract base64 part from data URL
            if let Some(base64_part) = screenshot.data_url.split("base64,").nth(1) {
                // Try to decode the first 100 characters to verify it's valid base64
                let sample = &base64_part[..std::cmp::min(100, base64_part.len())];
                let decoded = base64::engine::general_purpose::STANDARD.decode(sample);
                assert!(
                    decoded.is_ok(),
                    "Screenshot data URL should contain valid base64 for test '{}'",
                    test_case.name
                );
            }
        }
    }
}

#[test]
fn test_video_data_url_is_valid_base64() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    for test_case in &test_cases.test_cases {
        if let Some(video) = &test_case.video {
            // Extract base64 part from data URL
            if let Some(base64_part) = video.data_url.split("base64,").nth(1) {
                // Try to decode the first 100 characters to verify it's valid base64
                let sample = &base64_part[..std::cmp::min(100, base64_part.len())];
                let decoded = base64::engine::general_purpose::STANDARD.decode(sample);
                assert!(
                    decoded.is_ok(),
                    "Video data URL should contain valid base64 for test '{}'",
                    test_case.name
                );
            }
        }
    }
}

#[test]
fn test_mime_type_detection() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let test_cases = load_test_cases_from_zip(&bytes).expect("Failed to load test cases");

    for test_case in &test_cases.test_cases {
        // Check screenshot MIME types
        for screenshot in &test_case.screenshots {
            assert!(
                screenshot.mime_type == "image/png" || screenshot.mime_type == "image/jpeg",
                "Unexpected screenshot MIME type: {}",
                screenshot.mime_type
            );
        }

        // Check video MIME types
        if let Some(video) = &test_case.video {
            assert!(
                video.mime_type == "video/webm" || video.mime_type == "video/mp4",
                "Unexpected video MIME type: {}",
                video.mime_type
            );
        }

        // Check trace file MIME types
        if let Some(trace) = &test_case.trace_file {
            assert!(
                trace.mime_type == "application/zip",
                "Unexpected trace MIME type: {}",
                trace.mime_type
            );
        }
    }
}

#[test]
fn test_handles_macosx_hidden_files() {
    let bytes = fs::read("tests/fixtures/test-cases.zip").expect("Failed to read test file");
    let result = load_test_cases_from_zip(&bytes);

    // Should successfully load even if there are __MACOSX or ._ files
    assert!(
        result.is_ok(),
        "Should handle __MACOSX and ._ files gracefully"
    );
}

#[test]
fn test_test_case_collection_default() {
    use trace_viewer::models::TestCaseCollection;

    let collection = TestCaseCollection::default();
    assert_eq!(collection.test_cases.len(), 0);

    let collection2 = TestCaseCollection::new();
    assert_eq!(collection2.test_cases.len(), 0);
}

#[test]
fn test_test_status_to_string() {
    assert_eq!(TestStatus::Passed.to_string(), "passed");
    assert_eq!(TestStatus::Failed.to_string(), "failed");
    assert_eq!(TestStatus::Skipped.to_string(), "skipped");
    assert_eq!(TestStatus::Pending.to_string(), "pending");
}
