use serde_json::{json, Value};
use spanda_core::{run_debug, DebugOptions, DebugPause, SpandaError};
use std::collections::HashSet;
use std::io::{self, BufRead, Write};

fn read_message(reader: &mut dyn BufRead) -> io::Result<Option<Value>> {
    let mut line = String::new();
    let mut content_length = 0usize;
    loop {
        line.clear();
        if reader.read_line(&mut line)? == 0 {
            return Ok(None);
        }
        if let Some(rest) = line.strip_prefix("Content-Length:") {
            content_length = rest.trim().parse().unwrap_or(0);
        } else if line.trim().is_empty() && content_length > 0 {
            break;
        }
    }
    let mut body = vec![0u8; content_length];
    reader.read_exact(&mut body)?;
    Ok(Some(serde_json::from_slice(&body)?))
}

fn write_message(writer: &mut dyn Write, msg: &Value) -> io::Result<()> {
    let body = serde_json::to_string(msg)?;
    write!(writer, "Content-Length: {}\r\n\r\n{}", body.len(), body)?;
    writer.flush()
}

fn respond(writer: &mut dyn Write, req: &Value, body: Value) -> io::Result<()> {
    write_message(
        writer,
        &json!({
            "seq": req.get("seq").cloned().unwrap_or(json!(0)),
            "type": "response",
            "request_seq": req.get("seq"),
            "success": true,
            "command": req.get("command"),
            "body": body,
        }),
    )
}

fn pause_for_frame(pauses: &[DebugPause], frame_id: i64) -> Option<&DebugPause> {
    if pauses.is_empty() {
        return None;
    }
    let index = if frame_id <= 0 {
        pauses.len() - 1
    } else {
        (frame_id as usize).saturating_sub(1).min(pauses.len() - 1)
    };
    pauses.get(index)
}

pub fn serve(source: &str, reader: &mut dyn BufRead, writer: &mut dyn Write) -> io::Result<()> {
    let mut breakpoints: HashSet<u32> = HashSet::new();
    let mut running = false;
    let mut last_pauses: Vec<DebugPause> = Vec::new();

    while let Some(req) = read_message(reader)? {
        let command = req
            .get("command")
            .and_then(|v| v.as_str())
            .unwrap_or_default();
        match command {
            "initialize" => {
                respond(
                    writer,
                    &req,
                    json!({
                        "capabilities": {
                            "supportsConfigurationDoneRequest": true,
                            "supportsSetVariable": false,
                            "supportsStepBack": false,
                            "supportsRestartRequest": false,
                        }
                    }),
                )?;
            }
            "launch" => {
                running = true;
                last_pauses.clear();
                respond(writer, &req, json!({}))?;
            }
            "setBreakpoints" => {
                breakpoints.clear();
                if let Some(bps) = req
                    .pointer("/arguments/breakpoints")
                    .and_then(|v| v.as_array())
                {
                    for bp in bps {
                        if let Some(line) = bp.get("line").and_then(|l| l.as_u64()) {
                            breakpoints.insert(line as u32);
                        }
                    }
                }
                let verified: Vec<Value> = breakpoints
                    .iter()
                    .map(|line| json!({ "verified": true, "line": line }))
                    .collect();
                respond(writer, &req, json!({ "breakpoints": verified }))?;
            }
            "configurationDone" => {
                respond(writer, &req, json!({}))?;
            }
            "continue" | "next" | "stepIn" | "stepOut" | "pause" => {
                if running {
                    let step = matches!(command, "next" | "stepIn" | "stepOut");
                    let session = run_debug(
                        source,
                        DebugOptions {
                            breakpoints: breakpoints.clone(),
                            step,
                        },
                    )
                    .unwrap_or_else(|e: SpandaError| {
                        spanda_core::DebugSession {
                            pauses: vec![spanda_core::DebugPause {
                                line: 1,
                                reason: e.to_string(),
                                variables: Default::default(),
                            }],
                        }
                    });
                    last_pauses = session.pauses.clone();
                    for pause in session.pauses {
                        write_message(
                            writer,
                            &json!({
                                "type": "event",
                                "event": "stopped",
                                "body": {
                                    "reason": if step { "step" } else { "breakpoint" },
                                    "threadId": 1,
                                    "text": pause.reason,
                                    "line": pause.line,
                                }
                            }),
                        )?;
                    }
                }
                respond(writer, &req, json!({ "allThreadsContinued": true }))?;
            }
            "threads" => {
                respond(
                    writer,
                    &req,
                    json!({ "threads": [{ "id": 1, "name": "spanda-main" }] }),
                )?;
            }
            "stackTrace" => {
                let line = last_pauses
                    .last()
                    .map(|pause| pause.line)
                    .unwrap_or(1);
                respond(
                    writer,
                    &req,
                    json!({
                        "stackFrames": [{
                            "id": 1,
                            "name": "main",
                            "line": line,
                            "column": 1,
                        }],
                        "totalFrames": 1,
                    }),
                )?;
            }
            "scopes" => {
                respond(
                    writer,
                    &req,
                    json!({
                        "scopes": [{
                            "name": "Locals",
                            "variablesReference": 1,
                            "expensive": false,
                        }]
                    }),
                )?;
            }
            "variables" => {
                let frame_id = req
                    .pointer("/arguments/frameId")
                    .and_then(|v| v.as_i64())
                    .unwrap_or(1);
                let variables: Vec<Value> = pause_for_frame(&last_pauses, frame_id)
                    .map(|pause| {
                        pause
                            .variables
                            .iter()
                            .enumerate()
                            .map(|(index, (name, value))| {
                                json!({
                                    "name": name,
                                    "value": value,
                                    "type": "String",
                                    "variablesReference": 0,
                                    "evaluateName": name,
                                    "indexedVariables": index,
                                })
                            })
                            .collect()
                    })
                    .unwrap_or_default();
                respond(writer, &req, json!({ "variables": variables }))?;
            }
            "disconnect" => {
                respond(writer, &req, json!({}))?;
                break;
            }
            _ => {
                respond(writer, &req, json!({}))?;
            }
        }
    }
    Ok(())
}

fn main() {
    let source = std::env::args().nth(1).unwrap_or_else(|| {
        eprintln!("Usage: spanda-dap <file.sd>");
        std::process::exit(1);
    });
    let text = std::fs::read_to_string(&source).unwrap_or_else(|e| {
        eprintln!("Error reading {source}: {e}");
        std::process::exit(1);
    });
    let stdin = io::stdin();
    let mut reader = stdin.lock();
    let mut stdout = io::stdout();
    if let Err(e) = serve(&text, &mut reader, &mut stdout) {
        eprintln!("DAP server error: {e}");
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::Cursor;

    #[test]
    fn scopes_and_variables_after_stop() {
        let source = r#"
robot R {
  actuator wheels: DifferentialDrive;
  behavior run() {
    let speed = 0.5 m/s;
    wheels.stop();
  }
}
"#;
        let init = json!({
            "seq": 1,
            "type": "request",
            "command": "initialize",
            "arguments": {}
        });
        let launch = json!({
            "seq": 2,
            "type": "request",
            "command": "launch",
            "arguments": {}
        });
        let set_bps = json!({
            "seq": 3,
            "type": "request",
            "command": "setBreakpoints",
            "arguments": { "breakpoints": [{ "line": 5 }] }
        });
        let cont = json!({
            "seq": 4,
            "type": "request",
            "command": "continue",
            "arguments": { "threadId": 1 }
        });
        let scopes = json!({
            "seq": 5,
            "type": "request",
            "command": "scopes",
            "arguments": { "frameId": 1 }
        });
        let variables = json!({
            "seq": 6,
            "type": "request",
            "command": "variables",
            "arguments": { "variablesReference": 1, "frameId": 1 }
        });

        let mut input = String::new();
        for msg in [init, launch, set_bps, cont, scopes, variables] {
            let body = serde_json::to_string(&msg).unwrap();
            input.push_str(&format!("Content-Length: {}\r\n\r\n{}", body.len(), body));
        }
        let mut reader = Cursor::new(input.into_bytes());
        let mut output = Vec::new();
        serve(source, &mut reader, &mut output).expect("serve");

        let text = String::from_utf8(output).expect("utf8");
        assert!(text.contains("\"command\":\"scopes\""));
        assert!(text.contains("\"name\":\"Locals\""));
        assert!(text.contains("\"command\":\"variables\""));
    }
}
