use wasm_bindgen::{closure::Closure, JsCast};
use web_sys::{DragEvent, Event, File, HtmlInputElement};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct FileDropZoneProps {
    pub on_files_dropped: Callback<Vec<File>>,
    pub on_file_selected: Callback<File>,
}

pub struct FileDropZone {
    drag_over: bool,
}

pub enum FileDropZoneMsg {
    DragOver,
    DragLeave,
    Drop(Vec<File>),
    FileSelected(File),
}

impl Component for FileDropZone {
    type Message = FileDropZoneMsg;
    type Properties = FileDropZoneProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { drag_over: false }
    }

    fn update(&mut self, ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            FileDropZoneMsg::DragOver => {
                self.drag_over = true;
                true
            }
            FileDropZoneMsg::DragLeave => {
                self.drag_over = false;
                true
            }
            FileDropZoneMsg::Drop(files) => {
                self.drag_over = false;
                ctx.props().on_files_dropped.emit(files);
                true
            }
            FileDropZoneMsg::FileSelected(file) => {
                ctx.props().on_file_selected.emit(file);
                false
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let link = ctx.link();

        let ondragover = link.callback(|e: DragEvent| {
            e.prevent_default();
            FileDropZoneMsg::DragOver
        });

        let ondragleave = link.callback(|_: DragEvent| FileDropZoneMsg::DragLeave);

        let ondrop = link.callback(|e: DragEvent| {
            e.prevent_default();
            let files = e
                .data_transfer()
                .and_then(|dt| dt.files())
                .map(|file_list| {
                    let mut files = Vec::new();
                    for i in 0..file_list.length() {
                        if let Some(file) = file_list.get(i) {
                            files.push(file);
                        }
                    }
                    files
                })
                .unwrap_or_default();

            FileDropZoneMsg::Drop(files)
        });

        let onclick = {
            let link = link.clone();
            Callback::from(move |_| {
                if let Some(window) = web_sys::window() {
                    if let Some(document) = window.document() {
                        let input = document.create_element("input").unwrap();
                        let input: HtmlInputElement = input.dyn_into().unwrap();
                        input.set_type("file");
                        input.set_accept(".zip");

                        let link = link.clone();
                        let onchange = Closure::wrap(Box::new(move |e: Event| {
                            let input: HtmlInputElement = e.target().unwrap().dyn_into().unwrap();
                            if let Some(files) = input.files() {
                                if let Some(file) = files.get(0) {
                                    link.send_message(FileDropZoneMsg::FileSelected(file));
                                }
                            }
                        })
                            as Box<dyn FnMut(_)>);

                        input.set_onchange(Some(onchange.as_ref().unchecked_ref()));
                        onchange.forget();

                        input.click();
                    }
                }
            })
        };

        let class = if self.drag_over {
            "drop-zone drag-over"
        } else {
            "drop-zone"
        };

        html! {
            <div
                class={class}
                {ondragover}
                {ondragleave}
                {ondrop}
            >
                <div class="drop-zone-content">
                    <div class="icon">{"📁"}</div>
                    <h2>{ "Drop Playwright Trace to load" }</h2>
                    <p>{ "or" }</p>
                    <button {onclick} class="select-file-button">
                        { "Select File" }
                    </button>
                    <p class="info">
                        { "Drop a Playwright trace .zip file here to view the test execution timeline, screenshots, and logs." }
                    </p>
                    <p class="privacy">
                        { "Your trace data is processed locally in your browser and never sent to any server." }
                    </p>
                </div>
            </div>
        }
    }
}
