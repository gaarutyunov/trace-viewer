use crate::components::test_case_card::TestCaseCard;
use crate::models::{TestCaseCollection, TestStatus};
use yew::prelude::*;

#[derive(Properties, PartialEq)]
pub struct TestCaseListProps {
    pub test_cases: TestCaseCollection,
}

pub enum TestCaseListMessage {
    FilterChanged(TestStatusFilter),
}

#[derive(Clone, PartialEq)]
pub enum TestStatusFilter {
    All,
    Failed,
    Passed,
    Skipped,
}

pub struct TestCaseList {
    filter: TestStatusFilter,
}

impl Component for TestCaseList {
    type Message = TestCaseListMessage;
    type Properties = TestCaseListProps;

    fn create(_ctx: &Context<Self>) -> Self {
        Self {
            filter: TestStatusFilter::All,
        }
    }

    fn update(&mut self, _ctx: &Context<Self>, msg: Self::Message) -> bool {
        match msg {
            TestCaseListMessage::FilterChanged(filter) => {
                self.filter = filter;
                true
            }
        }
    }

    fn view(&self, ctx: &Context<Self>) -> Html {
        let test_cases = &ctx.props().test_cases.test_cases;

        // Filter test cases based on current filter
        let filtered_cases: Vec<_> = test_cases
            .iter()
            .filter(|tc| match self.filter {
                TestStatusFilter::All => true,
                TestStatusFilter::Failed => tc.status == TestStatus::Failed,
                TestStatusFilter::Passed => tc.status == TestStatus::Passed,
                TestStatusFilter::Skipped => tc.status == TestStatus::Skipped,
            })
            .collect();

        // Count test cases by status
        let total_count = test_cases.len();
        let failed_count = test_cases
            .iter()
            .filter(|tc| tc.status == TestStatus::Failed)
            .count();
        let passed_count = test_cases
            .iter()
            .filter(|tc| tc.status == TestStatus::Passed)
            .count();
        let skipped_count = test_cases
            .iter()
            .filter(|tc| tc.status == TestStatus::Skipped)
            .count();

        html! {
            <div class="test-case-list">
                <div class="test-case-list-header">
                    <h2>{ "Test Results" }</h2>
                    <div class="test-summary">
                        <span class="test-summary-item">
                            { format!("Total: {}", total_count) }
                        </span>
                        <span class="test-summary-item status-failed">
                            { format!("Failed: {}", failed_count) }
                        </span>
                        <span class="test-summary-item status-passed">
                            { format!("Passed: {}", passed_count) }
                        </span>
                        {
                            if skipped_count > 0 {
                                html! {
                                    <span class="test-summary-item status-skipped">
                                        { format!("Skipped: {}", skipped_count) }
                                    </span>
                                }
                            } else {
                                html! {}
                            }
                        }
                    </div>
                </div>

                <div class="test-filter-bar">
                    <span class="filter-label">{ "Filter: " }</span>
                    { self.render_filter_button(ctx, TestStatusFilter::All, "All") }
                    { self.render_filter_button(ctx, TestStatusFilter::Failed, "Failed") }
                    { self.render_filter_button(ctx, TestStatusFilter::Passed, "Passed") }
                    {
                        if skipped_count > 0 {
                            self.render_filter_button(ctx, TestStatusFilter::Skipped, "Skipped")
                        } else {
                            html! {}
                        }
                    }
                </div>

                <div class="test-case-list-content">
                    {
                        if filtered_cases.is_empty() {
                            html! {
                                <div class="empty-state">
                                    <p>{ "No test cases match the current filter." }</p>
                                </div>
                            }
                        } else {
                            filtered_cases.iter().map(|test_case| {
                                html! {
                                    <TestCaseCard
                                        key={test_case.id.clone()}
                                        test_case={(*test_case).clone()}
                                    />
                                }
                            }).collect::<Html>()
                        }
                    }
                </div>
            </div>
        }
    }
}

impl TestCaseList {
    fn render_filter_button(
        &self,
        ctx: &Context<Self>,
        filter: TestStatusFilter,
        label: &str,
    ) -> Html {
        let is_active = self.filter == filter;
        let filter_clone = filter.clone();
        let onclick = ctx
            .link()
            .callback(move |_| TestCaseListMessage::FilterChanged(filter_clone.clone()));

        let class = classes!("filter-button", is_active.then_some("active"));

        html! {
            <button {class} {onclick}>
                { label }
            </button>
        }
    }
}
