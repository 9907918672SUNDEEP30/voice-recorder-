//! # IPC Messages
//!
//! Zero-bloat command and response structures for communication across standard IO / pipes.

use std::collections::HashMap;

/// An inbound command received across the IPC pipeline.
#[derive(Debug, Clone, PartialEq, Eq)]
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
    /// Parses an incoming line in zero-bloat standard format:
    /// `VERB run_id [key=value key2=value2 ...]`
    pub fn parse(line: &str) -> Option<Self> {
        let trimmed = line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            return None;
        }

        let mut parts = trimmed.split_whitespace();
        let verb = parts.next()?.to_uppercase();
        let run_id = parts.next().unwrap_or("default").to_string();

        let mut arguments = HashMap::new();
        for token in parts {
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

/// Outbound message emitted back through stdout / pipe.
#[derive(Debug, Clone, PartialEq, Eq)]
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

    /// Formats the response for line-based stream transmission.
    pub fn format_line(&self) -> String {
        format!(
            "STATUS {} run_id={} duration_ms={} details=\"{}\"\n",
            self.status, self.run_id, self.duration_ms, self.details
        )
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
