//! Persistent ROS2 daemon subprocess (rclpy) for in-process I/O.
//!
use std::io::{BufRead, BufReader, Write};
use std::path::PathBuf;
use std::process::{Child, ChildStdin, Command, Stdio};
use std::sync::Mutex;

static DAEMON: Mutex<Option<Ros2Daemon>> = Mutex::new(None);

struct Ros2Daemon {
    child: Child,
    stdin: ChildStdin,
    reader: BufReader<std::process::ChildStdout>,
}

impl Ros2Daemon {
    fn start() -> Result<Self, String> {
        let script = daemon_script_path()?;
        let python = python_cmd().ok_or_else(|| "python3 not found for ROS2 daemon".to_string())?;
        let mut child = Command::new(&python)
            .arg(&script)
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|e| format!("failed to start ROS2 daemon: {e}"))?;
        let stdin = child
            .stdin
            .take()
            .ok_or_else(|| "daemon stdin unavailable".to_string())?;
        let stdout = child
            .stdout
            .take()
            .ok_or_else(|| "daemon stdout unavailable".to_string())?;
        Ok(Self {
            child,
            stdin,
            reader: BufReader::new(stdout),
        })
    }

    fn request(&mut self, op: &str, args: &[String]) -> bool {
        let payload = serde_json::json!({ "op": op, "args": args });
        let line = match serde_json::to_string(&payload) {
            Ok(text) => text,
            Err(_) => return false,
        };
        if writeln!(self.stdin, "{line}").is_err() {
            return false;
        }
        if self.stdin.flush().is_err() {
            return false;
        }
        let mut response = String::new();
        if self.reader.read_line(&mut response).is_err() {
            return false;
        }
        serde_json::from_str::<serde_json::Value>(&response)
            .ok()
            .and_then(|value| value.get("ok").and_then(|ok| ok.as_bool()))
            .unwrap_or(false)
    }
}

fn python_cmd() -> Option<String> {
    for cmd in ["python3", "python"] {
        if Command::new(cmd)
            .arg("-c")
            .arg("import sys")
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .status()
            .map(|s| s.success())
            .unwrap_or(false)
        {
            return Some(cmd.to_string());
        }
    }
    None
}

pub fn python_available() -> bool {
    python_cmd().is_some()
}

pub fn daemon_script_path() -> Result<PathBuf, String> {
    if let Ok(path) = std::env::var("SPANDA_ROS2_DAEMON_SCRIPT") {
        let path = PathBuf::from(path);
        if path.is_file() {
            return Ok(path);
        }
    }
    if let Ok(manifest) = std::env::var("CARGO_MANIFEST_DIR") {
        let path = PathBuf::from(manifest)
            .join("../../scripts/spanda_ros2_daemon.py")
            .canonicalize()
            .ok();
        if let Some(path) = path {
            if path.is_file() {
                return Ok(path);
            }
        }
    }
    let path = PathBuf::from("scripts/spanda_ros2_daemon.py");
    if path.is_file() {
        return Ok(path);
    }
    Err("spanda_ros2_daemon.py not found".into())
}

fn with_daemon<F>(f: F) -> bool
where
    F: FnOnce(&mut Ros2Daemon) -> bool,
{
    if !python_available() {
        return false;
    }
    let mut guard = match DAEMON.lock() {
        Ok(guard) => guard,
        Err(_) => return false,
    };
    if guard.is_none() {
        match Ros2Daemon::start() {
            Ok(daemon) => *guard = Some(daemon),
            Err(_) => return false,
        }
    }
    let daemon = guard.as_mut().expect("daemon");
    if daemon.child.try_wait().ok().flatten().is_some() {
        match Ros2Daemon::start() {
            Ok(restarted) => *daemon = restarted,
            Err(_) => {
                *guard = None;
                return false;
            }
        }
    }
    f(guard.as_mut().expect("daemon"))
}

pub fn daemon_publish(topic: &str, payload: &str) -> bool {
    with_daemon(|daemon| daemon.request("publish", &[topic.to_string(), payload.to_string()]))
}

pub fn daemon_subscribe(topic: &str) -> bool {
    with_daemon(|daemon| daemon.request("subscribe", &[topic.to_string()]))
}

pub fn daemon_service_call(service: &str, service_type: &str, request: &str) -> bool {
    with_daemon(|daemon| {
        daemon.request(
            "service_call",
            &[
                service.to_string(),
                service_type.to_string(),
                request.to_string(),
            ],
        )
    })
}
