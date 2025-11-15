use crate::ansi_parser::parse_ansi;
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct AnsiTextProps {
    pub text: String,
}

#[function_component(AnsiText)]
pub fn ansi_text(props: &AnsiTextProps) -> Html {
    let segments = parse_ansi(&props.text);

    html! {
        <>
            {
                segments.into_iter().map(|segment| {
                    let classes = segment.css_classes();
                    if classes.is_empty() {
                        html! { <span>{ segment.text }</span> }
                    } else {
                        html! { <span class={classes}>{ segment.text }</span> }
                    }
                }).collect::<Html>()
            }
        </>
    }
}
