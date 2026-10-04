//! # Zero-Bloat Runtime CLI
//!
//! Demonstrates the application-agnostic core runtime:
//! IPC Bridge -> ExecutionContext -> Orchestrator & Telemetry -> IPC Outbound.

use ai_core::{
    ExecutionContext, FnStep, IpcBridge, MillisecondLogger, Orchestrator, OutboundMessage,
    State, Step, StepOutcome,
};
use std::io::Cursor;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AI Agent Core Engine Runtime (Zero-Bloat) ===");

    // 1. Simulating an incoming IPC pipeline (in production: IpcBridge::stdio())
    let simulated_input = b"EXECUTE run_core_001 task=\"process_data\" batch_size=64 retry_test\n";
    let mut outbound_stream = Vec::new();

    let mut ipc = IpcBridge::new(Cursor::new(&simulated_input[..]), &mut outbound_stream);

    // 2. Read incoming payload from IPC
    let command = match ipc.read_command()? {
        Some(cmd) => cmd,
        None => {
            eprintln!("No command received on IPC bridge.");
            return Ok(());
        }
    };

    println!("[IPC] Inbound Command: verb='{}', run_id='{}'", command.verb, command.run_id);

    // 3. Initialize ExecutionContext & route IPC payload arguments
    let mut ctx = ExecutionContext::new(&command.run_id);
    ipc.route_into_context(&command, &mut ctx);

    // 4. Initialize Zero-Bloat Millisecond Logger (telemetry)
    let telemetry = MillisecondLogger::stdout();

    // 5. Initialize Universal State-Machine Orchestrator (The Master Runner)
    let mut orchestrator = Orchestrator::new(telemetry);

    // 6. Define generic, application-agnostic steps
    let mut retry_count = 0;
    let step1 = Box::new(FnStep::new("prepare_stage", |c: &mut ExecutionContext| {
        c.set("stage1_status", "ready");
        StepOutcome::Success
    }));

    let step2 = Box::new(
        FnStep::new("resilient_worker", move |_c: &mut ExecutionContext| {
            retry_count += 1;
            if retry_count == 1 {
                // Simulating a transient glitch on first try
                StepOutcome::Retry("transient I/O busy, retrying".to_string())
            } else {
                StepOutcome::Success
            }
        })
        .with_retries(2),
    );

    let step3 = Box::new(FnStep::new("finalize_stage", |c: &mut ExecutionContext| {
        c.set("result", "computed_successfully");
        StepOutcome::Success
    }));

    let mut steps: Vec<Box<dyn Step>> = vec![step1, step2, step3];

    // 7. Execute the blind run
    let start_time = std::time::Instant::now();
    let run_result = orchestrator.run(&mut ctx, &mut steps);
    let duration_ms = start_time.elapsed().as_millis() as u64;

    // 8. Emit IPC outbound response
    let (status_label, details) = match run_result {
        Ok(State::Success) => ("SUCCESS", "All steps completed successfully".to_string()),
        Ok(other) => ("UNEXPECTED", format!("Finished in state {:?}", other)),
        Err(e) => ("FAILED", format!("Run error: {}", e)),
    };

    let response = OutboundMessage::new(&command.run_id, status_label, duration_ms, details);
    ipc.send_message(&response)?;

    let formatted_outbound = String::from_utf8(outbound_stream)?;
    println!("\n[IPC] Outbound Response Transmitted:\n{}", formatted_outbound);

    Ok(())
}
