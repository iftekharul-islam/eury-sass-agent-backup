use similar::TextDiff;

#[derive(Debug, serde::Serialize)]
pub struct DiffHunk {
    pub old_start: usize,
    pub old_lines: usize,
    pub new_start: usize,
    pub new_lines: usize,
    pub lines: Vec<String>,
}

#[must_use]
pub fn generate_unified_diff(old_content: &str, new_content: &str, file_name: &str) -> String {
    let diff = TextDiff::from_lines(old_content, new_content);
    diff.unified_diff().context_radius(3).header(file_name, file_name).to_string()
}
