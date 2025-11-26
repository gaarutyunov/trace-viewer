use gloo::file::{callbacks::FileReader, File as GlooFile};
use std::collections::HashMap;
use wasm_bindgen::prelude::*;
use web_sys::File;
use yew::html::Scope;
use yew::prelude::*;

mod ansi_parser;
mod components;
pub mod markdown_exporter;
pub mod models;
pub mod test_case_loader;
pub mod trace_loader;

use components::{FileDropZone, TestCaseList, TraceViewer};
use models::{TestCaseCollection, TraceModel};

#[derive(Clone, PartialEq)]
pub enum LoadingState {
    Idle,
    Loading { progress: f32 },
    LoadedTrace { model: TraceModel },
    LoadedTestCases { test_cases: TestCaseCollection },
    Error { message: String },
}

pub enum AppMessage {
    FilesDropped(Vec<File>),
    FileSelected(File),
    LoadingProgress(f32),
    TraceLoaded(TraceModel),
    TestCasesLoaded(TestCaseCollection),
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
                    self.load_file(ctx, file.clone());
                }
                true
            }
            AppMessage::FileSelected(file) => {
                self.load_file(ctx, file);
                true
            }
            AppMessage::LoadingProgress(progress) => {
                self.state = LoadingState::Loading { progress };
                true
            }
            AppMessage::TraceLoaded(model) => {
                self.state = LoadingState::LoadedTrace { model };
                true
            }
            AppMessage::TestCasesLoaded(test_cases) => {
                self.state = LoadingState::LoadedTestCases { test_cases };
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
                        <h2>{ "Loading..." }</h2>
                        <div class="progress-bar">
                            <div class="progress-fill" style={format!("width: {}%", progress * 100.0)}></div>
                        </div>
                        <p>{ format!("{:.0}%", progress * 100.0) }</p>
                    </div>
                }
            }
            LoadingState::LoadedTrace { model } => {
                html! {
                    <TraceViewer model={model.clone()} />
                }
            }
            LoadingState::LoadedTestCases { test_cases } => {
                html! {
                    <TestCaseList test_cases={test_cases.clone()} />
                }
            }
            LoadingState::Error { message } => {
                let on_retry = link.callback(|_| AppMessage::FilesDropped(vec![]));

                html! {
                    <div class="error-container">
                        <h2>{ "Error Loading File" }</h2>
                        <p class="error-message">{ message }</p>
                        <button onclick={on_retry}>{ "Try Again" }</button>
                    </div>
                }
            }
        }
    }

    fn load_file(&mut self, ctx: &Context<Self>, file: File) {
        let link = ctx.link().clone();
        let file_name = file.name();

        log::info!("Loading file: {}", file_name);

        self.state = LoadingState::Loading { progress: 0.0 };

        let gloo_file = GlooFile::from(file);
        let task = {
            let link = link.clone();
            gloo::file::callbacks::read_as_bytes(&gloo_file, move |result| {
                match result {
                    Ok(bytes) => {
                        log::info!("File read successfully, {} bytes", bytes.len());
                        link.send_message(AppMessage::LoadingProgress(0.3));

                        // Try loading as test cases first
                        match test_case_loader::load_test_cases_from_zip(&bytes) {
                            Ok(test_cases) if !test_cases.test_cases.is_empty() => {
                                log::info!(
                                    "Test cases loaded successfully: {} test cases",
                                    test_cases.test_cases.len()
                                );
                                link.send_message(AppMessage::TestCasesLoaded(test_cases));
                                return;
                            }
                            Ok(_) => {
                                log::info!("No test cases found, trying to load as trace...");
                            }
                            Err(e) => {
                                log::info!(
                                    "Not a test case archive ({}), trying to load as trace...",
                                    e
                                );
                            }
                        }

                        // If not test cases, try loading as a trace
                        match trace_loader::load_trace_from_zip(&bytes) {
                            Ok(model) => {
                                log::info!("Trace loaded successfully");
                                link.send_message(AppMessage::TraceLoaded(model));
                            }
                            Err(e) => {
                                log::error!("Error loading file: {}", e);
                                link.send_message(AppMessage::LoadError(format!(
                                    "Could not load file as trace or test cases: {}",
                                    e
                                )));
                            }
                        }
                    }
                    Err(e) => {
                        log::error!("Error reading file: {:?}", e);
                        link.send_message(AppMessage::LoadError(format!(
                            "Error reading file: {:?}",
                            e
                        )));
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
