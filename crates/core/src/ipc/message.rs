//! # IPC Messages
//!
//! Zero-bloat command and response structures for communication across standard IO / pipes.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct InboundCommandJson {
    pub verb: String,
    #[serde(default = "default_run_id")]
    pub run_id: String,
    #[serde(default)]
    pub arguments: HashMap<String, String>,
}

fn default_run_id() -> String {
    "default".to_string()
}

/// An inbound command received across the IPC pipeline.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct InboundCommand {
    /// Command verb or action (e.g. "RUN", "PING", "STOP").
    pub verb: String,
    /// Associated run ID.
    pub run_id: String,
    /// Parsed key-value arguments.
    pub arguments: HashMap<String, String>,
    /// Full original raw line.
    pub raw: String,
}

impl InboundCommand {
    /// Parses an incoming line in zero-bloat standard format or structured JSON:
    /// `VERB run_id [key=value key2="value with spaces" ...]`
    /// or `{"verb": "RUN", "run_id": "123", "arguments": {...}}`
    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        // Support structured JSON framing (Fix M-04)
        if trimmed.starts_with('{') {
            if let Ok(cmd) = serde_json::from_str::<InboundCommandJson>(trimmed) {
                return Some(Self {
                    verb: cmd.verb.to_uppercase(),
                    run_id: cmd.run_id,
                    arguments: cmd.arguments,
                    raw: trimmed.to_string(),
                });
            }
        }

        // Robust quoted token tokenizer (Fix L-02)
        let tokens = tokenize_quoted(trimmed);
        if tokens.is_empty() {
            return None;
        }

        let verb = tokens[0].to_uppercase();
        let run_id = if tokens.len() > 1 {
            tokens[1].clone()
        } else {
            "default".to_string()
        };

        let mut arguments = HashMap::new();
        for token in tokens.iter().skip(2) {
            if let Some((k, v)) = token.split_once('=') {
                arguments.insert(k.to_string(), v.to_string());
            } else {
                arguments.insert(token.to_string(), "true".to_string());
            }
        }

        Some(Self {
            verb,
            run_id,
            arguments,
            raw: trimmed.to_string(),
        })
    }
}

/// Tokenizes command line strings with respect for double quotes (`"..."`)
fn tokenize_quoted(input: &str) -> Vec<String> {
    let mut tokens = Vec::new();
    let mut current = String::new();
    let mut in_quotes = false;
    let mut chars = input.chars().peekable();

    while let Some(c) = chars.next() {
        match c {
            '"' => {
                in_quotes = !in_quotes;
            }
            ' ' | '\t' if !in_quotes => {
                if !current.is_empty() {
                    tokens.push(std::mem::take(&mut current));
                }
            }
            '\\' if in_quotes => {
                if let Some(&next_c) = chars.peek() {
                    if next_c == '"' || next_c == '\\' {
                        current.push(chars.next().unwrap());
                        continue;
                    }
                }
                current.push('\\');
            }
            _ => {
                current.push(c);
            }
        }
    }

    if !current.is_empty() {
        tokens.push(current);
    }

    tokens
}

/// Outbound message emitted back through stdout / pipe.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboundMessage {
    pub run_id: String,
    pub status: String,
    pub duration_ms: u64,
    pub details: String,
}

impl OutboundMessage {
    pub fn new(
        run_id: impl Into<String>,
        status: impl Into<String>,
        duration_ms: u64,
        details: impl Into<String>,
    ) -> Self {
        Self {
            run_id: run_id.into(),
            status: status.into(),
            duration_ms,
            details: details.into(),
        }
    }

    /// Formats the response for line-based stream transmission with CRLF escaping (Fix M-04).
    pub fn format_line(&self) -> String {
        let clean_details = self
            .details
            .replace('\\', "\\\\")
            .replace('\n', "\\n")
            .replace('\r', "\\r")
            .replace('"', "\\\"");
        let clean_status = self.status.replace(['\n', '\r'], "");
        let clean_run_id = self.run_id.replace(['\n', '\r'], "");

        format!(
            "STATUS {} run_id={} duration_ms={} details=\"{}\"\n",
            clean_status, clean_run_id, self.duration_ms, clean_details
        )
    }

    /// Formats as robust JSON line transmission
    pub fn to_json_line(&self) -> Result<String, serde_json::Error> {
        let mut json = serde_json::to_string(self)?;
        json.push('\n');
        Ok(json)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_inbound_command() {
        let cmd = InboundCommand::parse("EXECUTE run_42 input=prompt timeout=30 dry_run")
            .expect("should parse");
        assert_eq!(cmd.verb, "EXECUTE");
        assert_eq!(cmd.run_id, "run_42");
        assert_eq!(cmd.arguments.get("input").map(|s| s.as_str()), Some("prompt"));
        assert_eq!(cmd.arguments.get("timeout").map(|s| s.as_str()), Some("30"));
        assert_eq!(cmd.arguments.get("dry_run").map(|s| s.as_str()), Some("true"));
    }

    #[test]
    fn skips_comments_and_empty_lines() {
        assert_eq!(InboundCommand::parse(""), None);
        assert_eq!(InboundCommand::parse("   "), None);
        assert_eq!(InboundCommand::parse("# comment line"), None);
    }
}
