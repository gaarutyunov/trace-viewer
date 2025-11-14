use trace_viewer::models::*;
use trace_viewer::trace_loader::*;

#[test]
fn test_load_trace_from_zip_success() {
    // Load the sample trace file
    let trace_bytes = include_bytes!("fixtures/sample-trace.zip");

    let result = load_trace_from_zip(trace_bytes);
    assert!(result.is_ok(), "Failed to load trace: {:?}", result.err());

    let model = result.unwrap();
    assert!(!model.contexts.is_empty(), "No contexts loaded");
}

#[test]
fn test_trace_contexts_parsed() {
    let trace_bytes = include_bytes!("fixtures/sample-trace.zip");
    let model = load_trace_from_zip(trace_bytes).unwrap();

    // Find the context with chromium browser
    let context = model
        .contexts
        .iter()
        .find(|c| c.browser_name == "chromium")
        .expect("Chromium context not found");

    // Verify context metadata
    assert_eq!(context.browser_name, "chromium");
    assert!(context.platform.is_some());
    assert_eq!(context.platform.as_deref(), Some("linux"));

    // Verify title
    assert!(context.title.is_some());
    assert!(context.title.as_ref().unwrap().contains("Boid Pointer Tracking"));
}

#[test]
fn test_trace_actions_parsed() {
    let trace_bytes = include_bytes!("fixtures/sample-trace.zip");
    let model = load_trace_from_zip(trace_bytes).unwrap();

    // Check any context with actions
    let has_actions = model.contexts.iter().any(|c| !c.actions.is_empty());
    assert!(has_actions, "No actions parsed in any context");

    // Test sorting on contexts with actions
    for context in &model.contexts {
        if context.actions.is_empty() {
            continue;
        }

        // Actions should be sorted by start time
        let mut last_start = 0.0;
        for action in &context.actions {
            assert!(
                action.start_time >= last_start,
                "Actions not sorted: {} < {}",
                action.start_time,
                last_start
            );
            last_start = action.start_time;
        }
    }
}

#[test]
fn test_trace_action_details() {
    let trace_bytes = include_bytes!("fixtures/sample-trace.zip");
    let model = load_trace_from_zip(trace_bytes).unwrap();

    // Find newPage action across all contexts
    let new_page_action = model
        .contexts
        .iter()
        .flat_map(|c| &c.actions)
        .find(|a| a.method.as_deref() == Some("newPage"));

    assert!(new_page_action.is_some(), "newPage action not found");

    let action = new_page_action.unwrap();
    assert_eq!(action.class.as_deref(), Some("BrowserContext"));
    assert!(action.end_time > action.start_time, "Action has no duration");
}

#[test]
fn test_trace_goto_action() {
    let trace_bytes = include_bytes!("fixtures/sample-trace.zip");
    let model = load_trace_from_zip(trace_bytes).unwrap();

    // Find goto action across all contexts
    let goto_action = model
        .contexts
        .iter()
        .flat_map(|c| &c.actions)
        .find(|a| a.method.as_deref() == Some("goto"));

    assert!(goto_action.is_some(), "goto action not found");

    let action = goto_action.unwrap();
    assert_eq!(action.class.as_deref(), Some("Frame"));
    assert!(action.params.contains_key("url"), "goto action missing url param");
}

#[test]
fn test_trace_events_parsed() {
    let trace_bytes = include_bytes!("fixtures/sample-trace.zip");
    let model = load_trace_from_zip(trace_bytes).unwrap();

    let has_events = model.contexts.iter().any(|c| !c.events.is_empty());
    assert!(has_events, "No events parsed in any context");

    // Verify we have different event types across all contexts
    let all_events: Vec<_> = model.contexts.iter().flat_map(|c| &c.events).collect();
    
    let has_before = all_events.iter().any(|e| matches!(e, TraceEvent::Before(_)));
    let has_after = all_events.iter().any(|e| matches!(e, TraceEvent::After(_)));

    assert!(has_before, "No before events found");
    assert!(has_after, "No after events found");
}

#[test]
fn test_load_invalid_zip() {
    let invalid_data = b"not a zip file";
    let result = load_trace_from_zip(invalid_data);

    assert!(result.is_err(), "Should fail on invalid ZIP");
    assert!(matches!(result.unwrap_err(), LoadError::ZipError(_)));
}

#[test]
fn test_load_zip_without_trace_file() {
    // Create a minimal ZIP without trace files
    use std::io::Write;
    use zip::write::FileOptions;
    use zip::ZipWriter;

    let mut buf = Vec::new();
    {
        let mut zip = ZipWriter::new(std::io::Cursor::new(&mut buf));
        zip.start_file("dummy.txt", FileOptions::default()).unwrap();
        zip.write_all(b"dummy content").unwrap();
        zip.finish().unwrap();
    }

    let result = load_trace_from_zip(&buf);
    assert!(result.is_err(), "Should fail without trace files");
    assert!(matches!(result.unwrap_err(), LoadError::MissingTraceFile));
}

#[test]
fn test_action_timing() {
    let trace_bytes = include_bytes!("fixtures/sample-trace.zip");
    let model = load_trace_from_zip(trace_bytes).unwrap();

    // All completed actions should have end_time > start_time
    for context in &model.contexts {
        for action in &context.actions {
            if action.end_time > 0.0 {
                assert!(
                    action.end_time >= action.start_time,
                    "Action end time {} before start time {}",
                    action.end_time,
                    action.start_time
                );
            }
        }
    }
}

#[test]
fn test_context_time_bounds() {
    let trace_bytes = include_bytes!("fixtures/sample-trace.zip");
    let model = load_trace_from_zip(trace_bytes).unwrap();

    for context in &model.contexts {
        // Context start/end should encompass all actions
        for action in &context.actions {
            assert!(
                action.start_time >= context.start_time,
                "Action starts before context"
            );
            if action.end_time > 0.0 {
                assert!(
                    action.end_time <= context.end_time,
                    "Action ends after context"
                );
            }
        }
    }
}

#[test]
fn test_action_parent_child_relationships() {
    let trace_bytes = include_bytes!("fixtures/sample-trace.zip");
    let model = load_trace_from_zip(trace_bytes).unwrap();

    for context in &model.contexts {
        // Find actions with parent relationships
        let children: Vec<_> = context
            .actions
            .iter()
            .filter(|a| a.parent_id.is_some())
            .collect();

        for child in children {
            let parent_id = child.parent_id.as_ref().unwrap();
            let parent = context.actions.iter().find(|a| &a.call_id == parent_id);

            assert!(
                parent.is_some(),
                "Parent action {} not found for child {}",
                parent_id,
                child.call_id
            );
        }
    }
}
