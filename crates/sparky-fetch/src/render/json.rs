use crate::domain::report::Report;

pub fn render(report: &Report) -> Result<String, String> {
    serde_json::to_string_pretty(report)
        .map(|text| text + "\n")
        .map_err(|error| error.to_string())
}
