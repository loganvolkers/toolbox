//! `seikika serve` as a process: in-image connectors run as the server's user, so the server's
//! environment (`DATABASE_URL`, `SEIKIKA_SECRET_*`) must not be readable through `/proc`.

#![cfg(target_os = "linux")]

use std::path::Path;
use std::process::{Command, Stdio};
use std::time::{Duration, Instant};

fn free_port() -> u16 {
    std::net::TcpListener::bind("127.0.0.1:0")
        .unwrap()
        .local_addr()
        .unwrap()
        .port()
}

fn ready(addr: &str) -> bool {
    use std::io::{Read, Write};
    let Ok(mut s) = std::net::TcpStream::connect(addr) else {
        return false;
    };
    let _ = write!(s, "GET /readyz HTTP/1.0\r\nHost: {addr}\r\n\r\n");
    let mut out = String::new();
    let _ = s.read_to_string(&mut out);
    out.starts_with("HTTP/1.1 200") || out.starts_with("HTTP/1.0 200")
}

fn is_root() -> bool {
    std::fs::read_to_string("/proc/self/status")
        .unwrap()
        .lines()
        .any(|l| l.starts_with("Uid:") && l.split_whitespace().nth(1) == Some("0"))
}

#[test]
fn a_same_user_process_cannot_read_the_server_environment() {
    if is_root() {
        eprintln!("skipping: root reads every process's environment");
        return;
    }
    let portal = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/repo-portal-starter");
    let addr = format!("127.0.0.1:{}", free_port());
    let mut server = Command::new(env!("CARGO_BIN_EXE_seikika"))
        .args(["serve", portal.to_str().unwrap(), "--listen", &addr])
        .env("SEIKIKA_OPENFGA_BIN", "/nonexistent")
        .env("SEIKIKA_CONNECTORS", "off")
        .env("SEIKIKA_SECRET_PROBE", "not-for-connectors")
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .unwrap();
    let start = Instant::now();
    while !ready(&addr) {
        assert!(
            start.elapsed() < Duration::from_secs(30),
            "server not ready on {addr}"
        );
        std::thread::sleep(Duration::from_millis(100));
    }
    let pid = server.id();
    let environ = std::fs::read(format!("/proc/{pid}/environ"));
    let status = std::fs::read_to_string(format!("/proc/{pid}/status"));
    let _ = server.kill();
    let _ = server.wait();
    assert_eq!(
        environ
            .map(|e| String::from_utf8_lossy(&e).contains("not-for-connectors"))
            .map_err(|e| e.kind()),
        Err(std::io::ErrorKind::PermissionDenied)
    );
    assert!(
        status.unwrap().contains("VmRSS"),
        "resident memory stays observable"
    );
}

#[test]
fn a_detached_server_answers_once_started_and_is_gone_once_stopped() {
    let portal = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../examples/repo-portal-starter");
    let state = std::env::temp_dir().join(format!("seikika-detach-{}", std::process::id()));
    let addr = format!("127.0.0.1:{}", free_port());
    let seikika = |args: &[&str]| {
        Command::new(env!("CARGO_BIN_EXE_seikika"))
            .arg("serve")
            .arg(&portal)
            .args(args)
            .env("SEIKIKA_STATE_DIR", &state)
            .env("SEIKIKA_OPENFGA_BIN", "/nonexistent")
            .env("SEIKIKA_CONNECTORS", "off")
            .output()
            .unwrap()
    };
    struct Stop<F: Fn(&[&str]) -> std::process::Output>(F);
    impl<F: Fn(&[&str]) -> std::process::Output> Drop for Stop<F> {
        fn drop(&mut self) {
            (self.0)(&["--stop"]);
        }
    }
    let seikika = Stop(seikika);
    let seikika = |args: &[&str]| (seikika.0)(args);
    let started = seikika(&["--listen", &addr, "--detach"]);
    let out = String::from_utf8_lossy(&started.stdout);
    assert!(
        started.status.success(),
        "{out}{}",
        String::from_utf8_lossy(&started.stderr)
    );
    assert!(out.contains(&format!("http://{addr}")), "{out}");
    assert!(out.contains("run records: "), "{out}");
    assert!(ready(&addr), "it returns only once the server answers");
    let again = seikika(&["--listen", &addr, "--detach"]);
    assert_eq!(
        again.status.code(),
        Some(2),
        "one server per portal and host"
    );
    let stopped = seikika(&["--stop"]);
    assert!(
        stopped.status.success(),
        "{}",
        String::from_utf8_lossy(&stopped.stderr)
    );
    assert!(!ready(&addr), "--stop waits for the server to exit");
    assert!(
        seikika(&["--stop"]).status.success(),
        "stopping twice is not an error"
    );
    let _ = std::fs::remove_dir_all(&state);
}
