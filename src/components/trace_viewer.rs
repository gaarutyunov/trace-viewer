use yew::prelude::*;
use crate::models::{TraceModel, ActionEntry};
use super::{ActionList, ActionDetails};

#[derive(Properties, PartialEq)]
pub struct TraceViewerProps {
    pub model: TraceModel,
}

pub struct TraceViewer {
    selected_action: Option<ActionEntry>,
}

pub enum TraceViewerMsg {
    SelectAction(ActionEntry),
}

impl Component for TraceViewer {
    type Message = TraceViewerMsg;
    type Properties = TraceViewerProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            selected_action: None,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TraceViewerMsg::SelectAction(action) => {
                self.selected_action = Some(action);
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
                    {
                        if let Some(ctx) = context {
                            html! {
                                <>
                                    <h2>
                                        { ctx.title.as_ref().map(|s| s.as_str()).unwrap_or("Trace") }
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
                                </>
                            }
                        } else {
                            html! { <h2>{ "No trace data" }</h2> }
                        }
                    }
                </div>

                {
                    if let Some(ctx) = context {
                        let on_action_selected = link.callback(TraceViewerMsg::SelectAction);

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
