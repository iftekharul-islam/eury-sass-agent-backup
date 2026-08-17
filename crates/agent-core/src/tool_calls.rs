//! Extraction of tool calls from assistant text.
//!
//! # Why this exists
//!
//! The cloud API contract ([`docs/04-specs/06-cloud-api-contract.md`]) defines
//! a typed `{ type: "tool_call", … }` stream event, and
//! [`crate::providers::gateway::ToolCallAccumulator`] consumes it. **No server
//! in this stack emits that event today.** The upstream LLM service emits only
//! `meta | activity | citations | delta | done | error`, and every prompt in
//! the stack instructs the model to emit a ` ```tool_call ` fence inside its
//! text instead. The web frontend and the deprecated desktop client both parse
//! those fences.
//!
//! So this module is the transcoder: it recovers tool calls from assistant
//! text for the path that actually works, while the typed-event path stays in
//! place for when the server implements it. Typed events take precedence.
//!
//! # Safety
//!
//! Parsing tool calls out of prose is inherently weaker than a typed channel —
//! a fenced *example* in an explanation is indistinguishable from a real call.
//! The mitigation, which the rest of the stack also relies on, is that a call
//! only executes if its name matches a **registered tool**. An unknown or
//! malformed name is dropped rather than guessed at.

use serde_json::{Map, Value};

/// A tool call recovered from assistant text.
#[derive(Debug, Clone, PartialEq)]
pub struct ParsedToolCall {
    pub name: String,
    pub arguments: Value,
}

/// A half-open span of the source text occupied by a tool call, so it can be
/// stripped from what the user sees.
#[derive(Debug, Clone, Copy)]
struct Span {
    start: usize,
    end: usize,
}

/// Extracts tool calls from `text`, keeping only those whose name satisfies
/// `is_known_tool`, and returns them alongside `text` with the matched regions
/// removed.
///
/// Recognized forms, matching what the rest of the stack emits:
/// - ` ```tool_call ` fence containing `{"name": …, "arguments": {…}}`
/// - a fence whose language *is* the tool name, containing just the arguments
/// - `[tool_call name=…]` followed by a JSON object
pub fn extract_tool_calls(
    text: &str,
    is_known_tool: &dyn Fn(&str) -> bool,
) -> (String, Vec<ParsedToolCall>) {
    let mut calls = Vec::new();
    let mut spans: Vec<Span> = Vec::new();

    collect_fenced(text, is_known_tool, &mut calls, &mut spans);
    collect_bracketed(text, is_known_tool, &mut calls, &mut spans);

    let calls = calls
        .into_iter()
        .map(|call| {
            let (name, arguments) = repair_tool_call(&call.name, call.arguments);
            ParsedToolCall { name, arguments }
        })
        .collect();

    (strip_spans(text, &mut spans), calls)
}

/// Scans ```` ```lang\n body ``` ```` blocks.
fn collect_fenced(
    text: &str,
    is_known_tool: &dyn Fn(&str) -> bool,
    calls: &mut Vec<ParsedToolCall>,
    spans: &mut Vec<Span>,
) {
    let bytes = text.as_bytes();
    let mut i = 0;

    while let Some(open) = find_from(text, "```", i) {
        let after_open = open + 3;
        // The fence language runs to end of line.
        let Some(nl) = find_from(text, "\n", after_open) else { break };
        let lang = text[after_open..nl].trim();
        let body_start = nl + 1;

        let Some(close) = find_from(text, "```", body_start) else { break };
        let body = &text[body_start..close];
        let end = (close + 3).min(bytes.len());

        let fenced = call_from_fence(lang, body, is_known_tool);
        if !fenced.is_empty() && !overlaps(spans, open, end) {
            spans.push(Span { start: open, end });
            calls.extend(fenced);
        }

        i = end;
    }
}

/// Scans `[tool_call name=…]` followed by a JSON object.
fn collect_bracketed(
    text: &str,
    is_known_tool: &dyn Fn(&str) -> bool,
    calls: &mut Vec<ParsedToolCall>,
    spans: &mut Vec<Span>,
) {
    const MARKER: &str = "[tool_call ";
    let mut i = 0;

    while let Some(start) = find_from(text, MARKER, i) {
        let after = start + MARKER.len();
        let Some(close_bracket) = find_from(text, "]", after) else { break };
        let attrs = &text[after..close_bracket];

        let name = attrs
            .split_whitespace()
            .find_map(|kv| kv.strip_prefix("name="))
            .map(|n| n.trim_matches('"').trim());

        i = close_bracket + 1;

        let Some(name) = name else { continue };
        if !is_known_tool(name) {
            continue;
        }

        let rest = &text[close_bracket + 1..];
        let Some((json, consumed)) = first_json_object(rest) else { continue };
        let Some(arguments) = arguments_from(&json, name) else { continue };

        let end = close_bracket + 1 + consumed;
        if !overlaps(spans, start, end) {
            spans.push(Span { start, end });
            calls.push(ParsedToolCall { name: name.to_string(), arguments });
        }
        i = end;
    }
}

fn call_from_fence(
    lang: &str,
    body: &str,
    is_known_tool: &dyn Fn(&str) -> bool,
) -> Vec<ParsedToolCall> {
    let normalized = lang.trim().to_lowercase();

    // ```tool_call / ```json / ```tool → body carries {"name": …, "arguments": …},
    // or an array of them: models batch calls that way unprompted.
    if normalized == "tool_call" || normalized == "json" || normalized == "tool" {
        return named_calls(body, is_known_tool);
    }

    // ```<tool_name> → body carries the arguments directly.
    if is_known_tool(&normalized) {
        match parse_json(body) {
            Some(Value::Object(map)) if !map.contains_key("name") => {
                return vec![ParsedToolCall {
                    name: normalized,
                    arguments: Value::Object(map),
                }];
            }
            Some(Value::Array(items)) => {
                return items
                    .into_iter()
                    .filter_map(|item| match item {
                        Value::Object(map) if !map.contains_key("name") => Some(ParsedToolCall {
                            name: normalized.clone(),
                            arguments: Value::Object(map),
                        }),
                        other => named_call_value(&other, is_known_tool),
                    })
                    .collect();
            }
            _ => {}
        }
        let body_trimmed = body.trim();
        if normalized == "read_file"
            && !body_trimmed.starts_with('{')
            && !body_trimmed.starts_with('[')
            && looks_like_shell_command(body_trimmed)
        {
            let (name, arguments) =
                repair_tool_call("read_file", Value::String(body_trimmed.to_string()));
            return vec![ParsedToolCall { name, arguments }];
        }
        return named_calls(body, is_known_tool);
    }

    // An unlabeled fence may still carry a well-formed named call.
    if normalized.is_empty() {
        return named_calls(body, is_known_tool);
    }

    Vec::new()
}

/// Parses `{"name": …, "arguments": {…}}` — or an array of them — rejecting
/// unregistered names.
fn named_calls(raw: &str, is_known_tool: &dyn Fn(&str) -> bool) -> Vec<ParsedToolCall> {
    match parse_json(raw) {
        Some(Value::Array(items)) => {
            items.iter().filter_map(|item| named_call_value(item, is_known_tool)).collect()
        }
        Some(value) => named_call_value(&value, is_known_tool).into_iter().collect(),
        None => Vec::new(),
    }
}

fn named_call_value(value: &Value, is_known_tool: &dyn Fn(&str) -> bool) -> Option<ParsedToolCall> {
    let Value::Object(map) = value else { return None };

    let name = map
        .get("name")
        .or_else(|| map.get("tool"))
        .or_else(|| map.get("tool_name"))
        .and_then(Value::as_str)?
        .trim()
        .to_string();

    if !is_known_tool(&name) {
        return None;
    }

    let arguments = map
        .get("arguments")
        .or_else(|| map.get("args"))
        .or_else(|| map.get("parameters"))
        .or_else(|| map.get("input"))
        .cloned()
        .unwrap_or_else(|| Value::Object(Map::new()));

    Some(ParsedToolCall { name, arguments })
}

/// For the bracket form the JSON may be either the arguments themselves or a
/// full named object.
fn arguments_from(raw: &str, expected_name: &str) -> Option<Value> {
    let Some(Value::Object(map)) = parse_json(raw) else { return None };

    if let Some(name) = map.get("name").and_then(Value::as_str) {
        if name.trim() != expected_name {
            return None;
        }
        return Some(
            map.get("arguments")
                .or_else(|| map.get("args"))
                .cloned()
                .unwrap_or_else(|| Value::Object(Map::new())),
        );
    }

    Some(Value::Object(map))
}

fn parse_json(raw: &str) -> Option<Value> {
    let trimmed = raw.trim();
    if trimmed.is_empty() {
        return None;
    }
    serde_json::from_str(trimmed).ok()
}

/// Returns the first balanced `{…}` object in `text` plus how many bytes of
/// `text` it consumed (including leading whitespace).
fn first_json_object(text: &str) -> Option<(String, usize)> {
    let start = text.find('{')?;
    let bytes = text.as_bytes();
    let mut depth = 0usize;
    let mut in_string = false;
    let mut escaped = false;

    for (offset, &byte) in bytes.iter().enumerate().skip(start) {
        if in_string {
            if escaped {
                escaped = false;
            } else if byte == b'\\' {
                escaped = true;
            } else if byte == b'"' {
                in_string = false;
            }
            continue;
        }
        match byte {
            b'"' => in_string = true,
            b'{' => depth += 1,
            b'}' => {
                depth -= 1;
                if depth == 0 {
                    let end = offset + 1;
                    return Some((text[start..end].to_string(), end));
                }
            }
            _ => {}
        }
    }
    None
}

fn find_from(haystack: &str, needle: &str, from: usize) -> Option<usize> {
    if from >= haystack.len() {
        return None;
    }
    // Only search from a character boundary; a mid-codepoint index would panic.
    let mut start = from;
    while start < haystack.len() && !haystack.is_char_boundary(start) {
        start += 1;
    }
    haystack[start..].find(needle).map(|i| start + i)
}

fn overlaps(spans: &[Span], start: usize, end: usize) -> bool {
    spans.iter().any(|s| (start >= s.start && start < s.end) || (end > s.start && end <= s.end))
}

/// Removes the matched regions so the user doesn't see raw tool JSON.
fn strip_spans(text: &str, spans: &mut [Span]) -> String {
    if spans.is_empty() {
        return text.to_string();
    }
    spans.sort_by_key(|s| s.start);

    let mut out = String::with_capacity(text.len());
    let mut cursor = 0;
    for span in spans.iter() {
        if span.start > cursor {
            out.push_str(&text[cursor..span.start]);
        }
        cursor = cursor.max(span.end);
    }
    if cursor < text.len() {
        out.push_str(&text[cursor..]);
    }
    out.trim().to_string()
}

/// Formats a tool result in the `[tool_result …]` shape the rest of the stack
/// already produces and the prompts tell the model to expect.
#[must_use]
pub fn format_tool_result(name: &str, content: &str) -> String {
    format!("[tool_result name={name}]\n{content}\n[/tool_result]")
}

/// Fixes common model mistakes before execution — e.g. passing a shell command
/// to `read_file` instead of `run_command`.
#[must_use]
pub fn repair_tool_call(name: &str, arguments: Value) -> (String, Value) {
    if name == "read_file" {
        if let Some(command) = shell_command_misrouted_to_read_file(&arguments) {
            return ("run_command".to_string(), serde_json::json!({ "command": command }));
        }
    }
    (name.to_string(), arguments)
}

fn shell_command_misrouted_to_read_file(arguments: &Value) -> Option<String> {
    if let Some(text) = arguments.as_str() {
        let trimmed = text.trim();
        if looks_like_shell_command(trimmed) {
            return Some(trimmed.to_string());
        }
    }

    let map = arguments.as_object()?;

    if let Some(command) = map.get("command").and_then(Value::as_str) {
        return Some(command.trim().to_string());
    }

    if let Some(path) = map.get("path").and_then(Value::as_str) {
        let trimmed = path.trim();
        if looks_like_shell_command(trimmed) {
            return Some(trimmed.to_string());
        }
    }

    None
}

fn looks_like_shell_command(text: &str) -> bool {
    if text.is_empty() {
        return false;
    }
    // File paths the read tool should handle — not shell commands.
    if text.contains('/') && !text.contains(' ') && !text.contains("--") {
        return false;
    }
    if text.ends_with(".rs")
        || text.ends_with(".ts")
        || text.ends_with(".tsx")
        || text.ends_with(".js")
        || text.ends_with(".json")
        || text.ends_with(".md")
        || text.ends_with(".html")
        || text.ends_with(".css")
    {
        return false;
    }

    if text.contains("--")
        || text.contains('|')
        || text.contains('>')
        || text.contains("&&")
        || text.contains("||")
    {
        return true;
    }

    const PREFIXES: &[&str] = &[
        "node ",
        "npm ",
        "pnpm ",
        "yarn ",
        "npx ",
        "python",
        "python3",
        "pip ",
        "ruby ",
        "go ",
        "rustc ",
        "cargo ",
        "docker ",
        "git ",
        "find ",
        "echo ",
        "which ",
        "env",
        "printenv",
        "uname ",
        "java ",
        "mvn ",
        "gradle ",
    ];
    let lower = text.to_ascii_lowercase();
    PREFIXES.iter().any(|prefix| lower.starts_with(prefix))
}

#[cfg(test)]
mod tests {
    use super::{extract_tool_calls, format_tool_result};
    use serde_json::json;

    /// The tools a real registry would report; anything else must be dropped.
    fn known(name: &str) -> bool {
        matches!(name, "read_file" | "write_file" | "edit_file" | "run_command" | "list_dir")
    }

    #[test]
    fn parses_the_tool_call_fence_the_prompts_instruct() {
        let text = "Let me look.\n\n```tool_call\n{\"name\":\"read_file\",\"arguments\":{\"path\":\"README.md\"}}\n```";
        let (cleaned, calls) = extract_tool_calls(text, &known);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_file");
        assert_eq!(calls[0].arguments, json!({"path": "README.md"}));
        assert_eq!(cleaned, "Let me look.", "the fence must not be shown to the user");
    }

    #[test]
    fn parses_a_json_fence_holding_an_array_of_calls() {
        // What a model actually emitted in the app: a ```json fence with the
        // calls batched into an array. Parsing only objects dropped the lot,
        // and the run ended with the raw JSON shown to the user.
        let text = concat!(
            "I'll look around first.\n\n```json\n",
            "[{\"name\":\"run_command\",\"arguments\":{\"command\":\"find . -maxdepth 2\"}},",
            "{\"name\":\"list_dir\",\"arguments\":{\"path\":\".\"}}]\n```",
        );
        let (cleaned, calls) = extract_tool_calls(text, &known);

        assert_eq!(calls.len(), 2);
        assert_eq!(calls[0].name, "run_command");
        assert_eq!(calls[0].arguments, json!({"command": "find . -maxdepth 2"}));
        assert_eq!(calls[1].name, "list_dir");
        assert_eq!(cleaned, "I'll look around first.");
    }

    #[test]
    fn parses_an_array_of_arguments_under_a_tool_named_fence() {
        let text = "```read_file\n[{\"path\":\"a.rs\"},{\"path\":\"b.rs\"}]\n```";
        let (_, calls) = extract_tool_calls(text, &known);

        assert_eq!(calls.len(), 2);
        assert!(calls.iter().all(|c| c.name == "read_file"));
    }

    #[test]
    fn still_drops_unregistered_names_inside_an_array() {
        let text = "```json\n[{\"name\":\"rm_rf\",\"arguments\":{}},{\"name\":\"list_dir\",\"arguments\":{}}]\n```";
        let (_, calls) = extract_tool_calls(text, &known);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "list_dir");
    }

    #[test]
    fn parses_a_fence_whose_language_is_the_tool_name() {
        let text = "```read_file\n{\"path\":\"src/main.rs\"}\n```";
        let (_, calls) = extract_tool_calls(text, &known);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "read_file");
        assert_eq!(calls[0].arguments, json!({"path": "src/main.rs"}));
    }

    #[test]
    fn parses_the_bracket_form() {
        let text = "[tool_call name=run_command]\n{\"command\":\"ls -la\"}";
        let (_, calls) = extract_tool_calls(text, &known);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "run_command");
        assert_eq!(calls[0].arguments, json!({"command": "ls -la"}));
    }

    #[test]
    fn parses_multiple_calls_in_emission_order() {
        let text = concat!(
            "```tool_call\n{\"name\":\"read_file\",\"arguments\":{\"path\":\"a\"}}\n```\n",
            "```tool_call\n{\"name\":\"read_file\",\"arguments\":{\"path\":\"b\"}}\n```"
        );
        let (_, calls) = extract_tool_calls(text, &known);

        let paths: Vec<_> =
            calls.iter().map(|c| c.arguments["path"].as_str().unwrap_or_default()).collect();
        assert_eq!(paths, vec!["a", "b"]);
    }

    #[test]
    fn drops_unregistered_tool_names() {
        // This is the safety property fence-parsing can actually guarantee:
        // a name the registry doesn't know never executes.
        let text = "```tool_call\n{\"name\":\"rm_rf_everything\",\"arguments\":{}}\n```";
        let (_, calls) = extract_tool_calls(text, &known);
        assert!(calls.is_empty());
    }

    #[test]
    fn drops_malformed_json() {
        let text = "```tool_call\n{\"name\":\"read_file\", \"arguments\": {oops\n```";
        let (_, calls) = extract_tool_calls(text, &known);
        assert!(calls.is_empty());
    }

    #[test]
    fn ignores_prose_and_unrelated_code_fences() {
        let text = "Here is how you would call it:\n\n```bash\nrm -rf /\n```\n\nBut I won't.";
        let (cleaned, calls) = extract_tool_calls(text, &known);

        assert!(calls.is_empty(), "a bash fence is not a tool call");
        assert_eq!(cleaned, text, "unmatched text passes through untouched");
    }

    #[test]
    fn handles_an_unterminated_fence_without_panicking() {
        let text = "```tool_call\n{\"name\":\"read_file\",\"arguments\":{\"path\":\"a\"}}";
        let (_, calls) = extract_tool_calls(text, &known);
        assert!(calls.is_empty(), "an unclosed fence is incomplete, not a call");
    }

    #[test]
    fn handles_multibyte_text_without_panicking() {
        // Byte-index scanning must not split a codepoint.
        let text = "Héllo — ünïcode 🎉\n```tool_call\n{\"name\":\"read_file\",\"arguments\":{\"path\":\"é.md\"}}\n```";
        let (_, calls) = extract_tool_calls(text, &known);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments["path"], "é.md");
    }

    #[test]
    fn nested_braces_in_arguments_are_parsed_whole() {
        let text = "[tool_call name=write_file]\n{\"path\":\"a.json\",\"content\":\"{\\\"k\\\":{\\\"n\\\":1}}\"}";
        let (_, calls) = extract_tool_calls(text, &known);

        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].arguments["content"], "{\"k\":{\"n\":1}}");
    }

    #[test]
    fn tool_result_uses_the_format_the_stack_expects() {
        assert_eq!(
            format_tool_result("read_file", "hello"),
            "[tool_result name=read_file]\nhello\n[/tool_result]"
        );
    }

    #[test]
    fn repairs_shell_commands_misrouted_to_read_file() {
        let text = "```tool_call\n{\"name\":\"read_file\",\"arguments\":{\"command\":\"node --version\"}}\n```";
        let (_, calls) = extract_tool_calls(text, &known);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "run_command");
        assert_eq!(calls[0].arguments, json!({"command": "node --version"}));
    }

    #[test]
    fn repairs_plain_text_shell_in_read_file_fence() {
        let text = "```read_file\nnode --version\n```";
        let (_, calls) = extract_tool_calls(text, &known);
        assert_eq!(calls.len(), 1);
        assert_eq!(calls[0].name, "run_command");
        assert_eq!(calls[0].arguments, json!({"command": "node --version"}));
    }
}
