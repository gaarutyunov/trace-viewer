use super::{ActionDetails, ActionList};
use crate::markdown_exporter::{export_to_markdown, ExportOptions};
use crate::models::{ActionEntry, TraceModel};
use wasm_bindgen::closure::Closure;
use wasm_bindgen::{JsCast, JsValue};
use web_sys::{Blob, BlobPropertyBag, HtmlAnchorElement, Url};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TraceViewerProps {
    pub model: TraceModel,
}

pub struct TraceViewer {
    selected_action: Option<ActionEntry>,
    errors_only: bool,
    copy_success: bool,
}

pub enum TraceViewerMsg {
    SelectAction(Box<ActionEntry>),
    ToggleErrorsOnly,
    ExportMarkdown,
    CopyToClipboard,
    ResetCopySuccess,
}

impl Component for TraceViewer {
    type Message = TraceViewerMsg;
    type Properties = TraceViewerProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            selected_action: None,
            errors_only: false,
            copy_success: false,
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TraceViewerMsg::SelectAction(action) => {
                self.selected_action = Some(*action);
                true
            }
            TraceViewerMsg::ToggleErrorsOnly => {
                self.errors_only = !self.errors_only;
                true
            }
            TraceViewerMsg::ExportMarkdown => {
                self.export_markdown(ctx);
                false
            }
            TraceViewerMsg::CopyToClipboard => {
                self.copy_to_clipboard(ctx);
                false
            }
            TraceViewerMsg::ResetCopySuccess => {
                self.copy_success = false;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let model = &ctx.props().model;
        let link = ctx.link();

        // For now, show the first context if available
        let context = model.contexts.first();

        html! {
            <div class="trace-viewer">
                <div class="viewer-header">
                    <div class="header-content">
                        {
                            if let Some(ctx) = context {
                                html! {
                                    <>
                                        <div class="header-left">
                                            <h2>
                                                { ctx.title.as_deref().unwrap_or("Trace") }
                                            </h2>
                                            <div class="context-info">
                                                <span class="browser">{ &ctx.browser_name }</span>
                                                {
                                                    if let Some(platform) = &ctx.platform {
                                                        html! { <span class="platform">{ platform }</span> }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                                {
                                                    if let Some(version) = &ctx.playwright_version {
                                                        html! { <span class="version">{ format!("v{}", version) }</span> }
                                                    } else {
                                                        html! {}
                                                    }
                                                }
                                            </div>
                                        </div>
                                        <div class="header-right">
                                            <div class="export-controls">
                                                <label class="checkbox-label">
                                                    <input
                                                        type="checkbox"
                                                        checked={self.errors_only}
                                                        onchange={link.callback(|_| TraceViewerMsg::ToggleErrorsOnly)}
                                                    />
                                                    <span>{ "Errors only" }</span>
                                                </label>
                                                <button
                                                    class="copy-button"
                                                    onclick={link.callback(|_| TraceViewerMsg::CopyToClipboard)}
                                                    title="Copy trace to clipboard in markdown format"
                                                >
                                                    { if self.copy_success { "Copied!" } else { "Copy to Clipboard" } }
                                                </button>
                                                <button
                                                    class="export-button"
                                                    onclick={link.callback(|_| TraceViewerMsg::ExportMarkdown)}
                                                >
                                                    { "Export to Markdown" }
                                                </button>
                                            </div>
                                        </div>
                                    </>
                                }
                            } else {
                                html! { <h2>{ "No trace data" }</h2> }
                            }
                        }
                    </div>
                </div>

                {
                    if let Some(ctx) = context {
                        let on_action_selected = link.callback(|a| TraceViewerMsg::SelectAction(Box::new(a)));

                        html! {
                            <div class="viewer-content">
                                <div class="left-panel">
                                    <ActionList
                                        actions={ctx.actions.clone()}
                                        {on_action_selected}
                                        selected_action={self.selected_action.clone()}
                                    />
                                </div>
                                <div class="right-panel">
                                    {
                                        if let Some(action) = &self.selected_action {
                                            html! {
                                                <ActionDetails action={action.clone()} />
                                            }
                                        } else {
                                            html! {
                                                <div class="no-selection">
                                                    <p>{ "Select an action to view details" }</p>
                                                </div>
                                            }
                                        }
                                    }
                                </div>
                            </div>
                        }
                    } else {
                        html! {
                            <div class="no-data">
                                <p>{ "No trace contexts found" }</p>
                            </div>
                        }
                    }
                }
            </div>
        }
    }
}

impl TraceViewer {
    fn export_markdown(&self, ctx: &Context<Self>) {
        let model = &ctx.props().model;
        let options = ExportOptions {
            errors_only: self.errors_only,
        };

        let markdown = export_to_markdown(model, &options);

        // Create a blob with the markdown content
        let array = js_sys::Array::new();
        array.push(&wasm_bindgen::JsValue::from_str(&markdown));

        let blob_options = BlobPropertyBag::new();
        blob_options.set_type("text/markdown");

        let blob = match Blob::new_with_str_sequence_and_options(&array, &blob_options) {
            Ok(blob) => blob,
            Err(e) => {
                log::error!("Failed to create blob: {:?}", e);
                return;
            }
        };

        // Create a download link
        let url = match Url::create_object_url_with_blob(&blob) {
            Ok(url) => url,
            Err(e) => {
                log::error!("Failed to create object URL: {:?}", e);
                return;
            }
        };

        // Create and click an anchor element to trigger download
        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                log::error!("Failed to get window");
                return;
            }
        };

        let document = match window.document() {
            Some(doc) => doc,
            None => {
                log::error!("Failed to get document");
                return;
            }
        };

        let anchor = match document.create_element("a") {
            Ok(el) => el,
            Err(e) => {
                log::error!("Failed to create anchor element: {:?}", e);
                return;
            }
        };

        let anchor: HtmlAnchorElement = match anchor.dyn_into() {
            Ok(a) => a,
            Err(e) => {
                log::error!("Failed to cast to HtmlAnchorElement: {:?}", e);
                return;
            }
        };

        anchor.set_href(&url);

        // Generate filename based on trace title and whether it's errors only
        let filename = if let Some(context) = model.contexts.first() {
            let title = context
                .title
                .as_deref()
                .unwrap_or("trace")
                .replace(' ', "_")
                .to_lowercase();

            if self.errors_only {
                format!("{}_errors.md", title)
            } else {
                format!("{}.md", title)
            }
        } else if self.errors_only {
            "trace_errors.md".to_string()
        } else {
            "trace.md".to_string()
        };

        anchor.set_download(&filename);

        // Trigger the download
        anchor.click();

        // Clean up the object URL
        Url::revoke_object_url(&url).ok();
    }

    fn copy_to_clipboard(&mut self, ctx: &Context<Self>) {
        let model = &ctx.props().model;
        let options = ExportOptions {
            errors_only: self.errors_only,
        };

        let markdown = export_to_markdown(model, &options);

        // Get window and navigator
        let window = match web_sys::window() {
            Some(window) => window,
            None => {
                log::error!("Failed to get window");
                return;
            }
        };

        let navigator = window.navigator();
        let clipboard = navigator.clipboard();

        // Copy to clipboard
        let promise = clipboard.write_text(&markdown);

        let link = ctx.link().clone();
        let success_callback = Closure::wrap(Box::new(move |_: JsValue| {
            log::info!("Text copied to clipboard successfully");
            link.send_message(TraceViewerMsg::ResetCopySuccess);
        }) as Box<dyn FnMut(JsValue)>);

        let error_callback = Closure::wrap(Box::new(move |err: JsValue| {
            log::error!("Failed to copy to clipboard: {:?}", err);
        }) as Box<dyn FnMut(JsValue)>);

        let _ = promise.then2(&success_callback, &error_callback);

        success_callback.forget();
        error_callback.forget();

        // Set copy success state
        self.copy_success = true;
    }
}
