//! sc — the sommerfluss control client (sommerfluss's `herbstclient`).
//!
//! Connects to the sfwm IPC socket, sends its arguments as one command, and
//! prints the reply. The config layer is a shell script that calls `sc` over and
//! over — a near-direct port of the hlwm `autostart` (`hc` becomes `sc`).
//!
//! Wire format: arguments are sent NUL-separated; the write side is then
//! half-closed so the server reads to EOF. The reply is plain text.

use std::io::{Read, Write};
use std::os::unix::net::UnixStream;
use std::path::PathBuf;
use std::process::ExitCode;

/// Resolve the IPC socket path. Kept in sync (by duplication) with sfwm's
/// `socket_path()` — both honor `SOMMERFLUSS_SOCKET` first.
fn socket_path() -> PathBuf {
    if let Ok(p) = std::env::var("SOMMERFLUSS_SOCKET") {
        return PathBuf::from(p);
    }
    let dir = std::env::var("XDG_RUNTIME_DIR").unwrap_or_else(|_| "/tmp".to_string());
    let display = std::env::var("WAYLAND_DISPLAY").unwrap_or_else(|_| "wayland-0".to_string());
    PathBuf::from(dir).join(format!("sfwm-{display}.sock"))
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() {
        eprintln!("usage: sc <command> [args...]");
        eprintln!("e.g.:  sc set_monitors 1440x2560+0+1876 3840x2160+1440+0");
        eprintln!("       sc add_monitor 3840x2160+1440+0 8 float1");
        eprintln!("       sc list_monitors");
        return ExitCode::from(2);
    }

    let path = socket_path();
    let mut stream = match UnixStream::connect(&path) {
        Ok(s) => s,
        Err(e) => {
            eprintln!("sc: cannot connect to sfwm at {}: {e}", path.display());
            eprintln!("    (is sfwm running, and is SOMMERFLUSS_SOCKET/WAYLAND_DISPLAY set?)");
            return ExitCode::from(1);
        }
    };

    // Send arguments NUL-separated, then half-close so the server sees EOF.
    let mut payload = Vec::new();
    for (i, a) in args.iter().enumerate() {
        if i > 0 {
            payload.push(0);
        }
        payload.extend_from_slice(a.as_bytes());
    }
    if let Err(e) = stream.write_all(&payload) {
        eprintln!("sc: write failed: {e}");
        return ExitCode::from(1);
    }
    let _ = stream.flush();
    let _ = stream.shutdown(std::net::Shutdown::Write);

    let mut reply = String::new();
    if let Err(e) = stream.read_to_string(&mut reply) {
        eprintln!("sc: read failed: {e}");
        return ExitCode::from(1);
    }

    // Errors come back prefixed with "error:"; route them to stderr / nonzero.
    if let Some(rest) = reply.strip_prefix("error:") {
        eprintln!("sc: error:{}", rest.trim_end());
        return ExitCode::from(1);
    }
    print!("{reply}");
    ExitCode::SUCCESS
}
