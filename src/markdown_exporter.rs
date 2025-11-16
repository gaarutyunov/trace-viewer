use crate::models::{ActionEntry, ContextEntry, TraceModel};
use chrono::{DateTime, Utc};

/// Options for exporting traces to markdown
#[derive(Debug, Clone, Default)]
pub struct ExportOptions {
    /// Only export actions with errors
    pub errors_only: bool,
}

/// Export a trace model to markdown format suitable for Claude Code
pub fn export_to_markdown(model: &TraceModel, options: &ExportOptions) -> String {
    let mut output = String::new();

    output.push_str("# Playwright Trace Report\n\n");

    // Export each context
    for (idx, context) in model.contexts.iter().enumerate() {
        if model.contexts.len() > 1 {
            output.push_str(&format!("## Context {}\n\n", idx + 1));
        }

        export_context(&mut output, context, options);

        if idx < model.contexts.len() - 1 {
            output.push_str("\n---\n\n");
        }
    }

    output
}

fn export_context(output: &mut String, context: &ContextEntry, options: &ExportOptions) {
    // Test information
    output.push_str("## Test Information\n\n");

    if let Some(title) = &context.title {
        output.push_str(&format!("- **Title**: {}\n", title));
    }

    output.push_str(&format!("- **Browser**: {}\n", context.browser_name));

    if let Some(platform) = &context.platform {
        output.push_str(&format!("- **Platform**: {}\n", platform));
    }

    if let Some(version) = &context.playwright_version {
        output.push_str(&format!("- **Playwright Version**: {}\n", version));
    }

    // Convert wall time to readable date
    let datetime = DateTime::from_timestamp_millis(context.wall_time as i64)
        .unwrap_or(DateTime::<Utc>::MIN_UTC);
    output.push_str(&format!(
        "- **Start Time**: {}\n",
        datetime.format("%Y-%m-%d %H:%M:%S UTC")
    ));

    let duration = (context.end_time - context.start_time) / 1000.0;
    output.push_str(&format!("- **Duration**: {:.2}s\n\n", duration));

    // Summary
    let actions_to_export: Vec<&ActionEntry> = if options.errors_only {
        context
            .actions
            .iter()
            .filter(|a| a.error.is_some())
            .collect()
    } else {
        context.actions.iter().collect()
    };

    let failed_actions = context.actions.iter().filter(|a| a.error.is_some()).count();

    output.push_str("## Summary\n\n");
    output.push_str(&format!("- **Total Actions**: {}\n", context.actions.len()));
    output.push_str(&format!("- **Failed Actions**: {}\n", failed_actions));

    if !context.errors.is_empty() {
        output.push_str(&format!("- **Context Errors**: {}\n", context.errors.len()));
    }

    if options.errors_only && failed_actions == 0 && context.errors.is_empty() {
        output.push_str("\n*No errors found in this trace.*\n\n");
        return;
    }

    output.push('\n');

    // Export actions
    if !actions_to_export.is_empty() {
        output.push_str("## Actions\n\n");

        for (idx, action) in actions_to_export.iter().enumerate() {
            export_action(output, action, idx + 1);
        }
    }

    // Export context-level errors
    if !context.errors.is_empty() {
        output.push_str("## Context Errors\n\n");

        for (idx, error) in context.errors.iter().enumerate() {
            output.push_str(&format!("### Error {}\n\n", idx + 1));
            output.push_str("```\n");
            output.push_str(&error.message);
            output.push('\n');

            if let Some(stack) = &error.stack {
                output.push_str("\nStack trace:\n");
                output.push_str(stack);
                output.push('\n');
            }

            output.push_str("```\n\n");
        }
    }
}

fn export_action(output: &mut String, action: &ActionEntry, index: usize) {
    let method = action
        .method
        .as_deref()
        .or(action.class.as_deref())
        .unwrap_or(&action.action_type);

    let status = if action.error.is_some() {
        " ⚠️ FAILED"
    } else {
        ""
    };

    output.push_str(&format!("### {}. {}{}\n\n", index, method, status));

    // Duration
    if action.end_time > 0.0 {
        let duration = action.end_time - action.start_time;
        output.push_str(&format!("**Duration**: {:.0}ms  \n", duration));
    }

    output.push_str(&format!("**Start**: {:.0}ms  \n", action.start_time));

    // Title if available
    if let Some(title) = &action.title {
        output.push_str(&format!("**Action**: {}  \n", title));
    }

    output.push('\n');

    // Parameters
    if !action.params.is_empty() {
        output.push_str("**Parameters**:\n\n");
        output.push_str("```json\n");

        match serde_json::to_string_pretty(&action.params) {
            Ok(json) => output.push_str(&json),
            Err(_) => output.push_str(&format!("{:?}", action.params)),
        }

        output.push_str("\n```\n\n");
    }

    // Error information
    if let Some(error) = &action.error {
        output.push_str("**Error**:\n\n");
        output.push_str("```\n");

        if let Some(message) = &error.message {
            output.push_str(message);
            output.push('\n');
        }

        if let Some(stack) = &error.stack {
            output.push_str("\nStack trace:\n");
            output.push_str(stack);
            output.push('\n');
        }

        output.push_str("```\n\n");
    }

    // Logs
    if !action.log.is_empty() {
        output.push_str("**Logs**:\n\n");

        for log in &action.log {
            output.push_str(&format!("- {:.0}ms: {}\n", log.time, log.message));
        }

        output.push('\n');
    }

    output.push_str("---\n\n");
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{ErrorEvent, LogEntry, SerializedError};

    #[test]
    fn test_export_empty_trace() {
        let model = TraceModel::new();
        let options = ExportOptions::default();
        let markdown = export_to_markdown(&model, &options);

        assert!(markdown.contains("# Playwright Trace Report"));
    }

    #[test]
    fn test_export_with_errors_only() {
        let mut model = TraceModel::new();

        let action_with_error = ActionEntry {
            action_type: "navigate".to_string(),
            call_id: "1".to_string(),
            start_time: 100.0,
            end_time: 600.0,
            title: Some("Navigate to page".to_string()),
            class: Some("Page".to_string()),
            method: Some("goto".to_string()),
            params: HashMap::new(),
            page_id: Some("page1".to_string()),
            parent_id: None,
            error: Some(SerializedError {
                message: Some("Navigation timeout".to_string()),
                stack: Some("at Page.goto".to_string()),
            }),
            log: vec![],
        };

        let action_without_error = ActionEntry {
            action_type: "click".to_string(),
            call_id: "2".to_string(),
            start_time: 700.0,
            end_time: 800.0,
            title: Some("Click button".to_string()),
            class: Some("Page".to_string()),
            method: Some("click".to_string()),
            params: HashMap::new(),
            page_id: Some("page1".to_string()),
            parent_id: None,
            error: None,
            log: vec![],
        };

        let context = ContextEntry {
            start_time: 0.0,
            end_time: 1000.0,
            browser_name: "chromium".to_string(),
            platform: Some("linux".to_string()),
            playwright_version: Some("1.40.0".to_string()),
            wall_time: 1700000000000.0,
            title: Some("Test".to_string()),
            pages: vec![],
            actions: vec![action_with_error, action_without_error],
            resources: vec![],
            events: vec![],
            errors: vec![],
        };

        model.contexts.push(context);

        let options = ExportOptions { errors_only: true };
        let markdown = export_to_markdown(&model, &options);

        assert!(markdown.contains("goto"));
        assert!(markdown.contains("Navigation timeout"));
        assert!(!markdown.contains("click"));
    }

    #[test]
    fn test_export_all_actions() {
        let mut model = TraceModel::new();

        let action = ActionEntry {
            action_type: "click".to_string(),
            call_id: "1".to_string(),
            start_time: 100.0,
            end_time: 150.0,
            title: Some("Click button".to_string()),
            class: Some("Page".to_string()),
            method: Some("click".to_string()),
            params: {
                let mut params = HashMap::new();
                params.insert("selector".to_string(), serde_json::json!("button"));
                params
            },
            page_id: Some("page1".to_string()),
            parent_id: None,
            error: None,
            log: vec![
                LogEntry {
                    time: 100.0,
                    message: "Starting click".to_string(),
                },
                LogEntry {
                    time: 150.0,
                    message: "Click complete".to_string(),
                },
            ],
        };

        let context = ContextEntry {
            start_time: 0.0,
            end_time: 200.0,
            browser_name: "chromium".to_string(),
            platform: Some("linux".to_string()),
            playwright_version: Some("1.40.0".to_string()),
            wall_time: 1700000000000.0,
            title: Some("Test".to_string()),
            pages: vec![],
            actions: vec![action],
            resources: vec![],
            events: vec![],
            errors: vec![],
        };

        model.contexts.push(context);

        let options = ExportOptions::default();
        let markdown = export_to_markdown(&model, &options);

        assert!(markdown.contains("click"));
        assert!(markdown.contains("Click button"));
        assert!(markdown.contains("selector"));
        assert!(markdown.contains("Starting click"));
        assert!(markdown.contains("Click complete"));
    }

    #[test]
    fn test_export_context_errors() {
        let mut model = TraceModel::new();

        let context = ContextEntry {
            start_time: 0.0,
            end_time: 1000.0,
            browser_name: "chromium".to_string(),
            platform: Some("linux".to_string()),
            playwright_version: Some("1.40.0".to_string()),
            wall_time: 1700000000000.0,
            title: Some("Test".to_string()),
            pages: vec![],
            actions: vec![],
            resources: vec![],
            events: vec![],
            errors: vec![ErrorEvent {
                message: "Uncaught exception".to_string(),
                stack: Some("at test.js:10".to_string()),
            }],
        };

        model.contexts.push(context);

        let options = ExportOptions::default();
        let markdown = export_to_markdown(&model, &options);

        assert!(markdown.contains("Context Errors"));
        assert!(markdown.contains("Uncaught exception"));
        assert!(markdown.contains("at test.js:10"));
    }
}
