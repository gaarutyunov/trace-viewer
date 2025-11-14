use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TraceModel {
    pub contexts: Vec<ContextEntry>,
}

impl TraceModel {
    pub fn new() -> Self {
        Self {
            contexts: Vec::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextEntry {
    pub start_time: f64,
    pub end_time: f64,
    pub browser_name: String,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub playwright_version: Option<String>,
    pub wall_time: f64,
    #[serde(default)]
    pub title: Option<String>,
    pub pages: Vec<PageEntry>,
    pub actions: Vec<ActionEntry>,
    #[serde(default)]
    pub resources: Vec<ResourceSnapshot>,
    #[serde(default)]
    pub events: Vec<TraceEvent>,
    #[serde(default)]
    pub errors: Vec<ErrorEvent>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PageEntry {
    pub page_id: String,
    #[serde(default)]
    pub screencast_frames: Vec<ScreencastFrame>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreencastFrame {
    pub sha1: String,
    pub timestamp: f64,
    pub width: u32,
    pub height: u32,
    #[serde(default)]
    pub frame_swap_wall_time: Option<f64>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ActionEntry {
    #[serde(rename = "type")]
    pub action_type: String,
    pub call_id: String,
    pub start_time: f64,
    #[serde(default)]
    pub end_time: f64,
    #[serde(default)]
    pub title: Option<String>,
    #[serde(default)]
    pub class: Option<String>,
    #[serde(default)]
    pub method: Option<String>,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub page_id: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub error: Option<SerializedError>,
    #[serde(default)]
    pub log: Vec<LogEntry>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LogEntry {
    pub time: f64,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SerializedError {
    #[serde(default)]
    pub message: Option<String>,
    #[serde(default)]
    pub stack: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type")]
pub enum TraceEvent {
    #[serde(rename = "before")]
    Before(BeforeActionEvent),
    #[serde(rename = "after")]
    After(AfterActionEvent),
    #[serde(rename = "input")]
    Input(InputActionEvent),
    #[serde(rename = "screencast-frame")]
    ScreencastFrame(ScreencastFrameEvent),
    #[serde(rename = "context-options")]
    ContextOptions(ContextOptionsEvent),
    #[serde(other)]
    Other,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BeforeActionEvent {
    pub call_id: String,
    pub start_time: f64,
    #[serde(default)]
    pub title: Option<String>,
    pub class: String,
    pub method: String,
    #[serde(default)]
    pub params: HashMap<String, serde_json::Value>,
    #[serde(default)]
    pub page_id: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AfterActionEvent {
    pub call_id: String,
    pub end_time: f64,
    #[serde(default)]
    pub error: Option<SerializedError>,
    #[serde(default)]
    pub result: Option<serde_json::Value>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct InputActionEvent {
    pub call_id: String,
    #[serde(default)]
    pub input_snapshot: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ScreencastFrameEvent {
    pub page_id: String,
    pub sha1: String,
    pub width: u32,
    pub height: u32,
    pub timestamp: f64,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ContextOptionsEvent {
    pub version: u32,
    pub browser_name: String,
    #[serde(default)]
    pub platform: Option<String>,
    #[serde(default)]
    pub playwright_version: Option<String>,
    pub wall_time: f64,
    pub monotonic_time: f64,
    #[serde(default)]
    pub title: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorEvent {
    pub message: String,
    #[serde(default)]
    pub stack: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceSnapshot {
    pub url: String,
    #[serde(default)]
    pub content_type: Option<String>,
    #[serde(default)]
    pub sha1: Option<String>,
}
