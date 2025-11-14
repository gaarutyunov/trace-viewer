use serde_json;
use trace_viewer::models::*;

#[test]
fn test_trace_model_default() {
    let model = TraceModel::default();
    assert!(model.contexts.is_empty());
}

#[test]
fn test_trace_model_new() {
    let model = TraceModel::new();
    assert!(model.contexts.is_empty());
}

#[test]
fn test_context_options_event_deserialization() {
    let json = r#"{
        "type": "context-options",
        "version": 8,
        "browserName": "chromium",
        "platform": "linux",
        "playwrightVersion": "1.40.0",
        "wallTime": 1700000000000,
        "monotonicTime": 1000,
        "title": "Test Title"
    }"#;

    let event: TraceEvent = serde_json::from_str(json).unwrap();

    match event {
        TraceEvent::ContextOptions(ctx) => {
            assert_eq!(ctx.version, 8);
            assert_eq!(ctx.browser_name, "chromium");
            assert_eq!(ctx.platform.as_deref(), Some("linux"));
            assert_eq!(ctx.playwright_version.as_deref(), Some("1.40.0"));
            assert_eq!(ctx.title.as_deref(), Some("Test Title"));
        }
        _ => panic!("Expected ContextOptions event"),
    }
}

#[test]
fn test_before_action_event_deserialization() {
    let json = r#"{
        "type": "before",
        "callId": "call@1",
        "startTime": 1000.5,
        "title": "page.goto",
        "class": "Page",
        "method": "goto",
        "params": {"url": "https://example.com"},
        "pageId": "page@1"
    }"#;

    let event: TraceEvent = serde_json::from_str(json).unwrap();

    match event {
        TraceEvent::Before(before) => {
            assert_eq!(before.call_id, "call@1");
            assert_eq!(before.start_time, 1000.5);
            assert_eq!(before.title.as_deref(), Some("page.goto"));
            assert_eq!(before.class, "Page");
            assert_eq!(before.method, "goto");
            assert!(before.params.contains_key("url"));
        }
        _ => panic!("Expected Before event"),
    }
}

#[test]
fn test_after_action_event_deserialization() {
    let json = r#"{
        "type": "after",
        "callId": "call@1",
        "endTime": 2000.5
    }"#;

    let event: TraceEvent = serde_json::from_str(json).unwrap();

    match event {
        TraceEvent::After(after) => {
            assert_eq!(after.call_id, "call@1");
            assert_eq!(after.end_time, 2000.5);
            assert!(after.error.is_none());
        }
        _ => panic!("Expected After event"),
    }
}

#[test]
fn test_after_action_with_error() {
    let json = r#"{
        "type": "after",
        "callId": "call@2",
        "endTime": 3000.0,
        "error": {
            "message": "Element not found",
            "stack": "Error: Element not found\n  at page.click"
        }
    }"#;

    let event: TraceEvent = serde_json::from_str(json).unwrap();

    match event {
        TraceEvent::After(after) => {
            assert!(after.error.is_some());
            let error = after.error.unwrap();
            assert_eq!(error.message.as_deref(), Some("Element not found"));
            assert!(error.stack.is_some());
        }
        _ => panic!("Expected After event"),
    }
}

#[test]
fn test_screencast_frame_event_deserialization() {
    let json = r#"{
        "type": "screencast-frame",
        "pageId": "page@1",
        "sha1": "abc123",
        "width": 1280,
        "height": 720,
        "timestamp": 1500.0
    }"#;

    let event: TraceEvent = serde_json::from_str(json).unwrap();

    match event {
        TraceEvent::ScreencastFrame(frame) => {
            assert_eq!(frame.page_id, "page@1");
            assert_eq!(frame.sha1, "abc123");
            assert_eq!(frame.width, 1280);
            assert_eq!(frame.height, 720);
            assert_eq!(frame.timestamp, 1500.0);
        }
        _ => panic!("Expected ScreencastFrame event"),
    }
}

#[test]
fn test_action_entry_serialization() {
    let action = ActionEntry {
        action_type: "before".to_string(),
        call_id: "call@1".to_string(),
        start_time: 1000.0,
        end_time: 2000.0,
        title: Some("Test Action".to_string()),
        class: Some("Page".to_string()),
        method: Some("click".to_string()),
        params: std::collections::HashMap::new(),
        page_id: Some("page@1".to_string()),
        parent_id: None,
        error: None,
        log: vec![],
    };

    let json = serde_json::to_string(&action).unwrap();
    let deserialized: ActionEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.call_id, action.call_id);
    assert_eq!(deserialized.start_time, action.start_time);
    assert_eq!(deserialized.end_time, action.end_time);
}

#[test]
fn test_context_entry_with_pages() {
    let mut context = ContextEntry {
        start_time: 0.0,
        end_time: 5000.0,
        browser_name: "chromium".to_string(),
        platform: Some("linux".to_string()),
        playwright_version: Some("1.40.0".to_string()),
        wall_time: 1700000000000.0,
        title: Some("Test".to_string()),
        pages: vec![],
        actions: vec![],
        resources: vec![],
        events: vec![],
        errors: vec![],
    };

    let page = PageEntry {
        page_id: "page@1".to_string(),
        screencast_frames: vec![ScreencastFrame {
            sha1: "abc123".to_string(),
            timestamp: 1000.0,
            width: 1280,
            height: 720,
            frame_swap_wall_time: None,
        }],
    };

    context.pages.push(page);

    assert_eq!(context.pages.len(), 1);
    assert_eq!(context.pages[0].screencast_frames.len(), 1);
}

#[test]
fn test_serialized_error() {
    let error = SerializedError {
        message: Some("Test error".to_string()),
        stack: Some("Error: Test error\n  at test.js:10".to_string()),
    };

    let json = serde_json::to_string(&error).unwrap();
    let deserialized: SerializedError = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.message, error.message);
    assert_eq!(deserialized.stack, error.stack);
}

#[test]
fn test_log_entry() {
    let log = LogEntry {
        time: 1234.56,
        message: "Test log message".to_string(),
    };

    let json = serde_json::to_string(&log).unwrap();
    let deserialized: LogEntry = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.time, log.time);
    assert_eq!(deserialized.message, log.message);
}

#[test]
fn test_resource_snapshot() {
    let resource = ResourceSnapshot {
        url: "https://example.com/script.js".to_string(),
        content_type: Some("application/javascript".to_string()),
        sha1: Some("abc123".to_string()),
    };

    let json = serde_json::to_string(&resource).unwrap();
    let deserialized: ResourceSnapshot = serde_json::from_str(&json).unwrap();

    assert_eq!(deserialized.url, resource.url);
    assert_eq!(deserialized.content_type, resource.content_type);
    assert_eq!(deserialized.sha1, resource.sha1);
}

#[test]
fn test_unknown_event_type() {
    let json = r#"{
        "type": "unknown-event-type",
        "data": "some data"
    }"#;

    // Should deserialize to TraceEvent::Other without error
    let event: TraceEvent = serde_json::from_str(json).unwrap();
    assert!(matches!(event, TraceEvent::Other));
}

#[test]
fn test_action_with_params() {
    use std::collections::HashMap;

    let mut params = HashMap::new();
    params.insert("url".to_string(), serde_json::json!("https://example.com"));
    params.insert("timeout".to_string(), serde_json::json!(30000));
    params.insert("waitUntil".to_string(), serde_json::json!("load"));

    let action = ActionEntry {
        action_type: "before".to_string(),
        call_id: "call@1".to_string(),
        start_time: 1000.0,
        end_time: 0.0,
        title: Some("page.goto".to_string()),
        class: Some("Frame".to_string()),
        method: Some("goto".to_string()),
        params,
        page_id: Some("page@1".to_string()),
        parent_id: None,
        error: None,
        log: vec![],
    };

    assert_eq!(action.params.len(), 3);
    assert!(action.params.contains_key("url"));
    assert!(action.params.contains_key("timeout"));
}
