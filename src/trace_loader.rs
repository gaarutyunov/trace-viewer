use crate::models::*;
use std::collections::HashMap;
use std::io::{Cursor, Read};
use zip::ZipArchive;

#[derive(Debug)]
pub enum LoadError {
    ZipError(String),
    IoError(String),
    #[allow(dead_code)]
    ParseError(String),
    MissingTraceFile,
}

impl std::fmt::Display for LoadError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            LoadError::ZipError(e) => write!(f, "ZIP error: {}", e),
            LoadError::IoError(e) => write!(f, "IO error: {}", e),
            LoadError::ParseError(e) => write!(f, "Parse error: {}", e),
            LoadError::MissingTraceFile => write!(f, "No .trace file found in archive"),
        }
    }
}

impl std::error::Error for LoadError {}

pub fn load_trace_from_zip(bytes: &[u8]) -> Result<TraceModel, LoadError> {
    log::info!("Parsing ZIP archive...");

    let cursor = Cursor::new(bytes);
    let mut archive = ZipArchive::new(cursor).map_err(|e| LoadError::ZipError(e.to_string()))?;

    log::info!("ZIP archive opened, {} entries found", archive.len());

    // Check if this is a report archive (contains data/ folder with nested ZIPs)
    let is_report_archive = (0..archive.len()).any(|i| {
        archive
            .by_index(i)
            .map(|f| {
                let name = f.name();
                name.starts_with("data/") && name.ends_with(".zip")
            })
            .unwrap_or(false)
    });

    if is_report_archive {
        log::info!("Detected report archive format");
        return load_report_archive(archive);
    }

    // Regular trace archive processing
    load_single_trace_archive(archive)
}

fn load_report_archive(mut archive: ZipArchive<Cursor<&[u8]>>) -> Result<TraceModel, LoadError> {
    let mut all_contexts = Vec::new();

    // Find all ZIP files in the data/ folder
    let mut nested_zips = Vec::new();
    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| LoadError::ZipError(e.to_string()))?;
        let name = file.name().to_string();

        if name.starts_with("data/") && name.ends_with(".zip") {
            nested_zips.push((i, name));
        }
    }

    if nested_zips.is_empty() {
        return Err(LoadError::MissingTraceFile);
    }

    log::info!("Found {} nested trace archives", nested_zips.len());

    // Process each nested trace archive
    for (index, name) in nested_zips {
        log::info!("Loading nested archive: {}", name);

        // Read the nested ZIP file
        let mut nested_file = archive
            .by_index(index)
            .map_err(|e| LoadError::ZipError(e.to_string()))?;

        let mut nested_bytes = Vec::new();
        nested_file
            .read_to_end(&mut nested_bytes)
            .map_err(|e| LoadError::IoError(e.to_string()))?;

        // Recursively load the nested trace
        let trace_model = load_trace_from_zip(&nested_bytes)?;
        all_contexts.extend(trace_model.contexts);
    }

    log::info!(
        "Loaded {} total contexts from report archive",
        all_contexts.len()
    );

    Ok(TraceModel {
        contexts: all_contexts,
    })
}

fn load_single_trace_archive(
    mut archive: ZipArchive<Cursor<&[u8]>>,
) -> Result<TraceModel, LoadError> {
    // Find all .trace files
    let mut trace_files = Vec::new();
    let mut network_files = HashMap::new();
    let mut resources = HashMap::new();

    for i in 0..archive.len() {
        let file = archive
            .by_index(i)
            .map_err(|e| LoadError::ZipError(e.to_string()))?;
        let name = file.name().to_string();

        if name.ends_with(".trace") {
            let ordinal = name.trim_end_matches(".trace");
            trace_files.push(ordinal.to_string());
        } else if name.ends_with(".network") {
            let ordinal = name.trim_end_matches(".network");
            network_files.insert(ordinal.to_string(), i);
        } else if name.starts_with("resources/") {
            resources.insert(name.clone(), i);
        }
    }

    if trace_files.is_empty() {
        return Err(LoadError::MissingTraceFile);
    }

    log::info!("Found {} trace file(s)", trace_files.len());

    let mut contexts = Vec::new();

    for ordinal in trace_files {
        log::info!("Processing trace: {}", ordinal);

        // Read the main trace file
        let trace_name = format!("{}.trace", ordinal);
        let trace_content = read_file_from_archive(&mut archive, &trace_name)?;

        // Read the network file if it exists
        let network_name = format!("{}.network", ordinal);
        let network_content = if archive.by_name(&network_name).is_ok() {
            Some(read_file_from_archive(&mut archive, &network_name)?)
        } else {
            None
        };

        // Parse the trace
        let context = parse_trace(&trace_content, network_content)?;
        contexts.push(context);
    }

    Ok(TraceModel { contexts })
}

fn read_file_from_archive(
    archive: &mut ZipArchive<Cursor<&[u8]>>,
    name: &str,
) -> Result<String, LoadError> {
    let mut file = archive
        .by_name(name)
        .map_err(|e| LoadError::ZipError(format!("Failed to read {}: {}", name, e)))?;

    let mut content = String::new();
    file.read_to_string(&mut content)
        .map_err(|e| LoadError::IoError(e.to_string()))?;

    Ok(content)
}

fn parse_trace(
    trace_content: &str,
    network_content: Option<String>,
) -> Result<ContextEntry, LoadError> {
    let mut actions_map: HashMap<String, ActionEntry> = HashMap::new();
    let mut pages: HashMap<String, PageEntry> = HashMap::new();
    let mut events = Vec::new();
    let errors = Vec::new();

    let mut context = ContextEntry {
        start_time: f64::MAX,
        end_time: 0.0,
        browser_name: String::new(),
        platform: None,
        playwright_version: None,
        wall_time: 0.0,
        title: None,
        pages: Vec::new(),
        actions: Vec::new(),
        resources: Vec::new(),
        events: Vec::new(),
        errors: Vec::new(),
    };

    // Parse main trace file (line-delimited JSON)
    for line in trace_content.lines() {
        let line = line.trim();
        if line.is_empty() {
            continue;
        }

        match serde_json::from_str::<TraceEvent>(line) {
            Ok(event) => {
                match &event {
                    TraceEvent::ContextOptions(ctx_opts) => {
                        context.browser_name = ctx_opts.browser_name.clone();
                        context.platform = ctx_opts.platform.clone();
                        context.playwright_version = ctx_opts.playwright_version.clone();
                        context.wall_time = ctx_opts.wall_time;
                        context.title = ctx_opts.title.clone();
                    }
                    TraceEvent::Before(before) => {
                        let action = ActionEntry {
                            action_type: "before".to_string(),
                            call_id: before.call_id.clone(),
                            start_time: before.start_time,
                            end_time: 0.0,
                            title: before.title.clone(),
                            class: Some(before.class.clone()),
                            method: Some(before.method.clone()),
                            params: before.params.clone(),
                            page_id: before.page_id.clone(),
                            parent_id: before.parent_id.clone(),
                            error: None,
                            log: Vec::new(),
                        };

                        if action.start_time < context.start_time {
                            context.start_time = action.start_time;
                        }

                        actions_map.insert(before.call_id.clone(), action);
                    }
                    TraceEvent::After(after) => {
                        if let Some(action) = actions_map.get_mut(&after.call_id) {
                            action.end_time = after.end_time;
                            action.error = after.error.clone();

                            if after.end_time > context.end_time {
                                context.end_time = after.end_time;
                            }
                        }
                    }
                    TraceEvent::ScreencastFrame(frame) => {
                        let page =
                            pages
                                .entry(frame.page_id.clone())
                                .or_insert_with(|| PageEntry {
                                    page_id: frame.page_id.clone(),
                                    screencast_frames: Vec::new(),
                                });

                        page.screencast_frames.push(ScreencastFrame {
                            sha1: frame.sha1.clone(),
                            timestamp: frame.timestamp,
                            width: frame.width,
                            height: frame.height,
                            frame_swap_wall_time: None,
                        });
                    }
                    _ => {}
                }
                events.push(event);
            }
            Err(e) => {
                log::warn!("Failed to parse trace event: {} - Line: {}", e, line);
            }
        }
    }

    // Parse network file if present
    if let Some(network) = network_content {
        for line in network.lines() {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }

            // Network events are also parsed as trace events
            if let Ok(event) = serde_json::from_str::<TraceEvent>(line) {
                events.push(event);
            }
        }
    }

    // Convert maps to vectors
    context.actions = actions_map.into_values().collect();

    context
        .actions
        .sort_by(|a, b| a.start_time.partial_cmp(&b.start_time).unwrap());

    context.pages = pages.into_values().collect();

    context.events = events;
    context.errors = errors;

    log::info!(
        "Parsed {} actions, {} pages",
        context.actions.len(),
        context.pages.len()
    );

    Ok(context)
}
