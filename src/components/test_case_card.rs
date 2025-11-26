use crate::models::{TestCase, TestStatus};
use pulldown_cmark::{html, Options, Parser};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TestCaseCardProps {
    pub test_case: TestCase,
}

pub enum TestCaseCardMessage {
    ToggleExpanded,
}

pub struct TestCaseCard {
    expanded: bool,
}

impl Component for TestCaseCard {
    type Message = TestCaseCardMessage;
    type Properties = TestCaseCardProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self { expanded: false }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TestCaseCardMessage::ToggleExpanded => {
                self.expanded = !self.expanded;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let test_case = &ctx.props().test_case;
        let expanded = self.expanded;

        let status_class = match test_case.status {
            TestStatus::Passed => "status-passed",
            TestStatus::Failed => "status-failed",
            TestStatus::Skipped => "status-skipped",
            TestStatus::Pending => "status-pending",
        };

        let card_class = classes!(
            "test-case-card",
            status_class,
            expanded.then_some("expanded")
        );

        let onclick = ctx.link().callback(|_| TestCaseCardMessage::ToggleExpanded);

        html! {
            <div class={card_class}>
                <div class="test-case-header" {onclick}>
                    <div class="test-case-header-left">
                        <span class="expand-icon">
                            { if expanded { "▼" } else { "▶" } }
                        </span>
                        <span class={classes!("test-status-badge", status_class)}>
                            { test_case.status.to_string() }
                        </span>
                        <h3 class="test-case-name">{ &test_case.name }</h3>
                    </div>
                    <div class="test-case-header-right">
                        {
                            if let Some(duration) = test_case.duration_ms {
                                html! {
                                    <span class="test-duration">
                                        { format!("{:.0}ms", duration) }
                                    </span>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>

                {
                    if expanded {
                        html! {
                            <div class="test-case-content">
                                { self.render_error_message(test_case) }
                                { self.render_markdown(test_case) }
                                { self.render_screenshots(test_case) }
                                { self.render_video(test_case) }
                                { self.render_trace_link(test_case) }
                            </div>
                        }
                    } else {
                        html! {}
                    }
                }
            </div>
        }
    }
}

impl TestCaseCard {
    fn render_error_message(&self, test_case: &TestCase) -> Html {
        if let Some(error_msg) = &test_case.error_message {
            html! {
                <div class="test-error-message">
                    <strong>{ "Error: " }</strong>
                    <span>{ error_msg }</span>
                </div>
            }
        } else {
            html! {}
        }
    }

    fn render_markdown(&self, test_case: &TestCase) -> Html {
        if let Some(markdown_content) = &test_case.markdown_content {
            // Parse markdown to HTML
            let mut options = Options::empty();
            options.insert(Options::ENABLE_STRIKETHROUGH);
            options.insert(Options::ENABLE_TABLES);
            options.insert(Options::ENABLE_TASKLISTS);

            let parser = Parser::new_ext(markdown_content, options);
            let mut html_output = String::new();
            html::push_html(&mut html_output, parser);

            html! {
                <div class="test-markdown-content">
                    <div class="markdown-rendered">
                        { Html::from_html_unchecked(AttrValue::from(html_output)) }
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }

    fn render_screenshots(&self, test_case: &TestCase) -> Html {
        if test_case.screenshots.is_empty() {
            return html! {};
        }

        html! {
            <div class="test-screenshots">
                <h4>{ "Screenshots" }</h4>
                <div class="screenshot-gallery">
                    {
                        test_case.screenshots.iter().map(|screenshot| {
                            html! {
                                <div class="screenshot-item">
                                    <img
                                        src={screenshot.data_url.clone()}
                                        alt={screenshot.name.clone()}
                                        title={screenshot.name.clone()}
                                    />
                                    <div class="screenshot-name">
                                        { &screenshot.name }
                                    </div>
                                </div>
                            }
                        }).collect::<Html>()
                    }
                </div>
            </div>
        }
    }

    fn render_video(&self, test_case: &TestCase) -> Html {
        if let Some(video) = &test_case.video {
            html! {
                <div class="test-video">
                    <h4>{ "Video Recording" }</h4>
                    <div class="video-player">
                        <video controls={true} preload="metadata">
                            <source src={video.data_url.clone()} type={video.mime_type.clone()} />
                            { "Your browser does not support the video tag." }
                        </video>
                    </div>
                    <div class="video-info">
                        <span class="video-name">{ &video.name }</span>
                        {
                            if let Some(size) = video.size_bytes {
                                html! {
                                    <span class="video-size">
                                        { format!(" ({:.1} MB)", size as f64 / 1024.0 / 1024.0) }
                                    </span>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }

    fn render_trace_link(&self, test_case: &TestCase) -> Html {
        if let Some(trace) = &test_case.trace_file {
            html! {
                <div class="test-trace-link">
                    <h4>{ "Trace File" }</h4>
                    <div class="trace-download">
                        <a
                            href={trace.data_url.clone()}
                            download={trace.name.clone()}
                            class="trace-download-button"
                        >
                            { format!("📥 Download {} ", trace.name) }
                            {
                                if let Some(size) = trace.size_bytes {
                                    html! {
                                        <span class="trace-size">
                                            { format!("({:.1} KB)", size as f64 / 1024.0) }
                                        </span>
                                    }
                                } else {
                                    html! {}
                                }
                            }
                        </a>
                    </div>
                </div>
            }
        } else {
            html! {}
        }
    }
}
