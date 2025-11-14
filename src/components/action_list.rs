use yew::prelude::*;
use crate::models::ActionEntry;

#[derive(Properties, PartialEq)]
pub struct ActionListProps {
    pub actions: Vec<ActionEntry>,
    pub on_action_selected: Callback<ActionEntry>,
    #[prop_or_default]
    pub selected_action: Option<ActionEntry>,
}

#[function_component(ActionList)]
pub fn action_list(props: &ActionListProps) -> Html {
    let selected_id = props.selected_action.as_ref().map(|a| a.call_id.as_str());

    html! {
        <div class="action-list">
            <div class="action-list-header">
                <h3>{ "Actions" }</h3>
                <span class="action-count">{ format!("{} actions", props.actions.len()) }</span>
            </div>
            <div class="action-list-content">
                {
                    props.actions.iter().map(|action| {
                        let action_clone = action.clone();
                        let on_action_selected = props.on_action_selected.clone();
                        let is_selected = selected_id == Some(action.call_id.as_str());
                        let has_error = action.error.is_some();

                        let onclick = Callback::from(move |_| {
                            on_action_selected.emit(action_clone.clone());
                        });

                        let class = classes!(
                            "action-item",
                            is_selected.then_some("selected"),
                            has_error.then_some("error"),
                        );

                        let duration = if action.end_time > 0.0 {
                            action.end_time - action.start_time
                        } else {
                            0.0
                        };

                        html! {
                            <div key={action.call_id.clone()} {class} {onclick}>
                                <div class="action-header">
                                    <span class="action-method">
                                        {
                                            if let Some(method) = &action.method {
                                                method.clone()
                                            } else {
                                                action.action_type.clone()
                                            }
                                        }
                                    </span>
                                    {
                                        if has_error {
                                            html! { <span class="error-indicator">{ "⚠" }</span> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                </div>
                                <div class="action-info">
                                    {
                                        if let Some(title) = &action.title {
                                            html! { <span class="action-title">{ title }</span> }
                                        } else {
                                            html! {}
                                        }
                                    }
                                    <span class="action-duration">
                                        { format!("{:.0}ms", duration) }
                                    </span>
                                </div>
                            </div>
                        }
                    }).collect::<Html>()
                }
            </div>
        </div>
    }
}
