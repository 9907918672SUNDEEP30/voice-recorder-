//! # Zero-Server IPC Bridge
//!
//! High-throughput, zero-dependency communication pipeline operating over standard IO / pipes.

use std::io::{BufRead, BufReader, Stdin, Stdout, Write};
use crate::engine::context::ExecutionContext;
use crate::ipc::message::{InboundCommand, OutboundMessage};

/// Bridge managing bidirectional stream communication.
pub struct IpcBridge<R: BufRead, W: Write> {
    reader: R,
    writer: W,
}

impl<R: BufRead, W: Write> IpcBridge<R, W> {
    /// Constructs a new bridge from any buffered reader and writer.
    pub fn new(reader: R, writer: W) -> Self {
        Self { reader, writer }
    }

    /// Reads the next command from the stream.
    /// Returns `Ok(None)` if EOF is reached.
    pub fn read_command(&mut self) -> std::io::Result<Option<InboundCommand>> {
        let mut line = String::new();
        loop {
            line.clear();
            let bytes_read = self.reader.read_line(&mut line)?;
            if bytes_read == 0 {
                return Ok(None);
            }
            if let Some(cmd) = InboundCommand::parse(&line) {
                return Ok(Some(cmd));
            }
            // If empty line or comment, continue reading
        }
    }

    /// Transmits an outbound message across the output stream.
    pub fn send_message(&mut self, message: &OutboundMessage) -> std::io::Result<()> {
        let formatted = message.format_line();
        self.writer.write_all(formatted.as_bytes())?;
        self.writer.flush()
    }

    /// Routes an incoming payload's arguments directly into an ExecutionContext.
    pub fn route_into_context(&self, command: &InboundCommand, ctx: &mut ExecutionContext) {
        ctx.set("ipc.verb", &command.verb);
        ctx.set("ipc.raw", &command.raw);
        for (k, v) in &command.arguments {
            ctx.set(k, v);
        }
    }
}

impl IpcBridge<BufReader<Stdin>, Stdout> {
    /// Creates an IPC bridge wired to standard Stdin and Stdout.
    pub fn stdio() -> Self {
        Self::new(BufReader::new(std::io::stdin()), std::io::stdout())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn bridge_reads_and_writes_payloads() {
        let input_data = b"RUN run_99 prompt=\"translate text\" mode=fast\n";
        let mut output_data = Vec::new();

        let mut bridge = IpcBridge::new(Cursor::new(&input_data[..]), &mut output_data);
        let cmd = bridge.read_command().unwrap().expect("valid command");

        assert_eq!(cmd.verb, "RUN");
        assert_eq!(cmd.run_id, "run_99");

        let mut ctx = ExecutionContext::new(&cmd.run_id);
        bridge.route_into_context(&cmd, &mut ctx);

        assert_eq!(ctx.get("ipc.verb"), Some("RUN"));
        assert_eq!(ctx.get("mode"), Some("fast"));

        let response = OutboundMessage::new(&cmd.run_id, "SUCCESS", 42, "Execution completed");
        bridge.send_message(&response).unwrap();

        let output_str = String::from_utf8(output_data).unwrap();
        assert!(output_str.contains("STATUS SUCCESS"));
        assert!(output_str.contains("run_id=run_99"));
        assert!(output_str.contains("duration_ms=42"));
    }
}
