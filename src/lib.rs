use yew::prelude::*;
use yew::html::Scope;
use web_sys::File;
use wasm_bindgen::prelude::*;
use gloo::file::{callbacks::FileReader, File as GlooFile};
use std::collections::HashMap;

mod models;
mod trace_loader;
mod components;

use components::{FileDropZone, TraceViewer};
use models::TraceModel;

#[derive(Clone, PartialEq)]
pub enum LoadingState {
    Idle,
    Loading { progress: f32 },
    Loaded { model: TraceModel },
    Error { message: String },
}

pub enum AppMessage {
    FilesDropped(Vec<File>),
    FileSelected(File),
    LoadingProgress(f32),
    TraceLoaded(TraceModel),
    LoadError(String),
}

pub struct App {
    state: LoadingState,
    file_readers: HashMap<String, FileReader>,
}

impl Component for App {
    type Message = AppMessage;
    type Properties = ();

    fn create(_ctx: &Context<Self>) -> Self {
        wasm_logger::init(wasm_logger::Config::default());
        log::info!("Playwright Trace Viewer initialized");

        Self {
            state: LoadingState::Idle,
            file_readers: HashMap::new(),
        }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            AppMessage::FilesDropped(files) => {
                if let Some(file) = files.first() {
                    self.load_trace_file(ctx, file.clone());
                }
                true
            }
            AppMessage::FileSelected(file) => {
                self.load_trace_file(ctx, file);
                true
            }
            AppMessage::LoadingProgress(progress) => {
                self.state = LoadingState::Loading { progress };
                true
            }
            AppMessage::TraceLoaded(model) => {
                self.state = LoadingState::Loaded { model };
                true
            }
            AppMessage::LoadError(message) => {
                self.state = LoadingState::Error { message };
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        html! {
            <div class="app">
                <header class="header">
                    <div class="logo">
                        <h1>{ "Playwright Trace Viewer" }</h1>
                        <span class="subtitle">{ "Rust Edition" }</span>
                    </div>
                </header>
                <main class="main-content">
                    { self.render_content(link) }
                </main>
            </div>
        }
    }
}

impl App {
    fn render_content(&self, link: &Scope<Self>) -> Html {
        match &self.state {
            LoadingState::Idle => {
                let on_files_dropped = link.callback(AppMessage::FilesDropped);
                let on_file_selected = link.callback(AppMessage::FileSelected);

                html! {
                    <FileDropZone
                        {on_files_dropped}
                        {on_file_selected}
                    />
                }
            }
            LoadingState::Loading { progress } => {
                html! {
                    <div class="loading-container">
                        <div class="loading-spinner"></div>
                        <h2>{ "Loading Playwright Trace..." }</h2>
                        <div class="progress-bar">
                            <div class="progress-fill" style={format!("width: {}%", progress * 100.0)}></div>
                        </div>
                        <p>{ format!("{:.0}%", progress * 100.0) }</p>
                    </div>
                }
            }
            LoadingState::Loaded { model } => {
                html! {
                    <TraceViewer model={model.clone()} />
                }
            }
            LoadingState::Error { message } => {
                let on_retry = link.callback(|_| AppMessage::FilesDropped(vec![]));

                html! {
                    <div class="error-container">
                        <h2>{ "Error Loading Trace" }</h2>
                        <p class="error-message">{ message }</p>
                        <button onclick={on_retry}>{ "Try Again" }</button>
                    </div>
                }
            }
        }
    }

    fn load_trace_file(&mut self, ctx: &Context<Self>, file: File) {
        let link = ctx.link().clone();
        let file_name = file.name();

        log::info!("Loading trace file: {}", file_name);

        self.state = LoadingState::Loading { progress: 0.0 };

        let gloo_file = GlooFile::from(file);
        let task = {
            let link = link.clone();
            gloo::file::callbacks::read_as_bytes(&gloo_file, move |result| {
                match result {
                    Ok(bytes) => {
                        log::info!("File read successfully, {} bytes", bytes.len());
                        link.send_message(AppMessage::LoadingProgress(0.3));

                        // Parse the ZIP file
                        match trace_loader::load_trace_from_zip(&bytes) {
                            Ok(model) => {
                                log::info!("Trace loaded successfully");
                                link.send_message(AppMessage::TraceLoaded(model));
                            }
                            Err(e) => {
                                log::error!("Error loading trace: {}", e);
                                link.send_message(AppMessage::LoadError(e.to_string()));
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading file: {:?}", e);
                        link.send_message(AppMessage::LoadError(format!("Error reading file: {:?}", e)));
                    }
                }
            })
        };

        self.file_readers.insert(file_name, task);
    }
}

#[wasm_bindgen(start)]
pub fn run_app() {
    yew::Renderer::<App>::new().render();
}
