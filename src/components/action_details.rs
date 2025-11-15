use super::AnsiText;
use crate::models::ActionEntry;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct ActionDetailsProps {
    pub action: ActionEntry,
}

#[function_component(ActionDetails)]
pub fn action_details(props: &ActionDetailsProps) -> Html {
    let action = &props.action;
    let duration = if action.end_time > 0.0 {
        action.end_time - action.start_time
    } else {
        0.0
    };

    html! {
        <div class="action-details">
            <div class="details-header">
                <h3>
                    {
                        if let Some(method) = &action.method {
                            method.clone()
                        } else {
                            action.action_type.clone()
                        }
                    }
                </h3>
                {
                    if action.error.is_some() {
                        html! { <span class="status-badge error">{ "Failed" }</span> }
                    } else {
                        html! { <span class="status-badge success">{ "Success" }</span> }
                    }
                }
            </div>

            {
                if let Some(title) = &action.title {
                    html! {
                        <div class="detail-section">
                            <div class="detail-label">{ "Description" }</div>
                            <div class="detail-value">{ title }</div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            <div class="detail-section">
                <div class="detail-row">
                    <div class="detail-column">
                        <div class="detail-label">{ "Duration" }</div>
                        <div class="detail-value">{ format!("{:.2}ms", duration) }</div>
                    </div>
                    <div class="detail-column">
                        <div class="detail-label">{ "Call ID" }</div>
                        <div class="detail-value code">{ &action.call_id }</div>
                    </div>
                </div>
            </div>

            <div class="detail-section">
                <div class="detail-row">
                    <div class="detail-column">
                        <div class="detail-label">{ "Start Time" }</div>
                        <div class="detail-value">{ format!("{:.2}ms", action.start_time) }</div>
                    </div>
                    <div class="detail-column">
                        <div class="detail-label">{ "End Time" }</div>
                        <div class="detail-value">{ format!("{:.2}ms", action.end_time) }</div>
                    </div>
                </div>
            </div>

            {
                if let Some(class) = &action.class {
                    html! {
                        <div class="detail-section">
                            <div class="detail-label">{ "Class" }</div>
                            <div class="detail-value code">{ class }</div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            {
                if !action.params.is_empty() {
                    html! {
                        <div class="detail-section">
                            <div class="detail-label">{ "Parameters" }</div>
                            <div class="params-list">
                                {
                                    action.params.iter().map(|(key, value)| {
                                        html! {
                                            <div class="param-item" key={key.clone()}>
                                                <span class="param-key">{ key }{ ": " }</span>
                                                <span class="param-value code">
                                                    { format!("{}", value) }
                                                </span>
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            {
                if let Some(error) = &action.error {
                    html! {
                        <div class="detail-section error-section">
                            <div class="detail-label">{ "Error" }</div>
                            {
                                if let Some(message) = &error.message {
                                    html! {
                                        <div class="error-message">
                                            <AnsiText text={message.clone()} />
                                        </div>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                            {
                                if let Some(stack) = &error.stack {
                                    html! {
                                        <details class="error-stack" open={true}>
                                            <summary>{ "Stack Trace" }</summary>
                                            <pre class="ansi-pre"><AnsiText text={stack.clone()} /></pre>
                                        </details>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </div>
                    }
                } else {
                    html! {}
                }
            }

            {
                if !action.log.is_empty() {
                    html! {
                        <div class="detail-section">
                            <div class="detail-label">{ "Logs" }</div>
                            <div class="log-list">
                                {
                                    action.log.iter().map(|log| {
                                        html! {
                                            <div class="log-entry">
                                                <span class="log-time">{ format!("{:.2}ms", log.time) }</span>
                                                <span class="log-message">{ &log.message }</span>
                                            </div>
                                        }
                                    }).collect::<Html>()
                                }
                            </div>
                        </div>
                    }
                } else {
                    html! {}
                }
            }
        </div>
    }
}
