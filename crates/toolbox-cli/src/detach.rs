//! `toolbox serve --detach` and `--stop`: a server in the background, kept track of by a pid file
//! next to the portal's other server state, so a script or agent starts, restarts and stops it
//! with one command each and needs no terminal multiplexer or process manager.

use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::ExitCode;
use std::time::{Duration, Instant};

const READY_WITHIN: Duration = Duration::from_secs(60);

fn manifest(path: &Path) -> Result<toolbox_model::Manifest, ExitCode> {
    let report = toolbox_model::validate_portal(path).map_err(|e| {
        eprintln!("error: {e}");
        ExitCode::from(2)
    })?;
    if !report.ok() {
        for d in &report.errors {
            eprintln!("ERROR {d}");
        }
        return Err(ExitCode::from(1));
    }
    let src = toolbox_model::load::load(path).expect("validated above");
    Ok(crate::portal::manifest_from(&src).expect("validated above"))
}

fn files(m: &toolbox_model::Manifest) -> (PathBuf, PathBuf, PathBuf) {
    let dir = toolbox_api::actions::served_dir(&m.portal.name);
    (dir.join("serve.pid"), dir.join("serve.log"), dir)
}

/// The pid in `file`, while that process is alive and is a toolbox server.
fn running(file: &Path) -> Option<i32> {
    let pid: i32 = std::fs::read_to_string(file).ok()?.trim().parse().ok()?;
    let p = rustix::process::Pid::from_raw(pid)?;
    rustix::process::test_kill_process(p).ok()?;
    // A pid outlives its process and can be reused; on Linux, check it is still `toolbox serve`.
    if let Ok(cmd) = std::fs::read(format!("/proc/{pid}/cmdline")) {
        let args: Vec<&[u8]> = cmd.split(|b| *b == 0).collect();
        if !args.contains(&b"serve".as_slice()) {
            return None;
        }
    }
    Some(pid)
}

/// The address a client on this host reaches `listen` at.
fn reach(listen: &str) -> String {
    match listen.rsplit_once(':') {
        Some(("0.0.0.0" | "[::]" | "", port)) => format!("127.0.0.1:{port}"),
        _ => listen.to_owned(),
    }
}

/// Whether `GET <path>` on `addr` answers 200.
fn answers(addr: &str, path: &str) -> bool {
    let Ok(mut s) = std::net::TcpStream::connect(addr) else {
        return false;
    };
    let _ = s.set_read_timeout(Some(Duration::from_secs(2)));
    if s.write_all(format!("GET {path} HTTP/1.0\r\nHost: {addr}\r\n\r\n").as_bytes())
        .is_err()
    {
        return false;
    }
    let mut head = [0u8; 12];
    s.read_exact(&mut head).is_ok() && head.ends_with(b"200")
}

fn tail(log: &Path) -> String {
    let text = std::fs::read_to_string(log).unwrap_or_default();
    let lines: Vec<&str> = text.lines().collect();
    lines[lines.len().saturating_sub(20)..].join("\n")
}

/// Start `toolbox serve` on `path` in its own process group, writing to `log_file`; returns the
/// child and the log's length before it started.
fn spawn(
    path: &Path,
    listen: &str,
    database_url: Option<String>,
    fixtures: bool,
    log_file: &Path,
) -> Result<(std::process::Child, u64), String> {
    let log = std::fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(log_file)
        .map_err(|e| format!("{}: {e}", log_file.display()))?;
    let since = log.metadata().map(|m| m.len()).unwrap_or(0);
    let err = log
        .try_clone()
        .map_err(|e| format!("{}: {e}", log_file.display()))?;
    let exe = std::env::current_exe().unwrap_or_else(|_| "toolbox".into());
    let mut cmd = std::process::Command::new(exe);
    cmd.arg("serve").arg(path).arg("--listen").arg(listen);
    if fixtures {
        cmd.arg("--fixtures");
    }
    // The environment, not argv: every local user can read a process's argv.
    if let Some(url) = database_url {
        cmd.env("DATABASE_URL", url);
    }
    cmd.stdin(std::process::Stdio::null())
        .stdout(log)
        .stderr(err);
    std::os::unix::process::CommandExt::process_group(&mut cmd, 0);
    let child = cmd
        .spawn()
        .map_err(|e| format!("cannot start the server: {e}"))?;
    Ok((child, since))
}

/// Wait until the server answers `/readyz` (its first snapshot is in, so its actions can be
/// listed straight away), or say why it never will.
fn ready(
    child: &mut std::process::Child,
    addr: &str,
    log_file: &Path,
    since: u64,
) -> Result<(), String> {
    let deadline = Instant::now() + READY_WITHIN;
    loop {
        if let Ok(Some(status)) = child.try_wait() {
            let text = std::fs::read_to_string(log_file).unwrap_or_default();
            let new = text.get(since as usize..).unwrap_or(&text);
            return Err(format!(
                "the server exited ({status}) before it was ready:\n{}",
                new.trim_end()
            ));
        }
        if answers(addr, "/readyz") {
            return Ok(());
        }
        if Instant::now() > deadline {
            return Err(format!(
                "the server (pid {}) did not answer on {addr} within {}s; its log ends:\n{}",
                child.id(),
                READY_WITHIN.as_secs(),
                tail(log_file)
            ));
        }
        std::thread::sleep(Duration::from_millis(100));
    }
}

/// `toolbox serve --detach`: start the server in the background, return once it is ready, and
/// print its URL, pid, log and approval key. Exits 2 when one already serves this portal here.
pub fn start(path: &Path, listen: &str, database_url: Option<String>, fixtures: bool) -> ExitCode {
    let m = match manifest(path) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let (pid_file, log_file, dir) = files(&m);
    if let Some(pid) = running(&pid_file) {
        eprintln!(
            "error: {} is already served by pid {pid} (log {}); `toolbox serve {} --stop` stops it",
            m.portal.name,
            log_file.display(),
            path.display()
        );
        return ExitCode::from(2);
    }
    let addr = reach(listen);
    if answers(&addr, "/healthz") {
        eprintln!("error: another server already answers on {addr}; pick another --listen");
        return ExitCode::from(2);
    }
    if let Err(e) = std::fs::create_dir_all(&dir) {
        eprintln!("error: {}: {e}", dir.display());
        return ExitCode::from(2);
    }
    let in_memory = database_url.is_none();
    let (mut child, since) = match spawn(path, listen, database_url, fixtures, &log_file) {
        Ok(c) => c,
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(1);
        }
    };
    if let Err(e) = std::fs::write(&pid_file, format!("{}\n", child.id())) {
        eprintln!("error: {}: {e}", pid_file.display());
    }
    if let Err(e) = ready(&mut child, &addr, &log_file, since) {
        let _ = std::fs::remove_file(&pid_file);
        eprintln!("error: {e}");
        return ExitCode::from(1);
    }
    println!(
        "toolbox: serving {} on http://{addr} (pid {}, log {})",
        m.portal.name,
        child.id(),
        log_file.display()
    );
    if let Some(k) = toolbox_api::actions::approval_key_for(&m) {
        println!(
            "approval key: {k} (children pin `{{portal: {}, key: {k}}}` in federation.acceptApprovalsFrom)",
            m.portal.name
        );
    }
    if in_memory {
        println!("run records: {}/<run id>.json", dir.join("runs").display());
    }
    println!("stop it with: toolbox serve {} --stop", path.display());
    ExitCode::SUCCESS
}

/// `toolbox serve --stop`: SIGTERM the server `--detach` started for this portal and wait for it
/// to exit. Nothing to stop is not an error.
pub fn stop(path: &Path) -> ExitCode {
    let m = match manifest(path) {
        Ok(m) => m,
        Err(code) => return code,
    };
    let (pid_file, _, _) = files(&m);
    let Some(pid) = running(&pid_file) else {
        let _ = std::fs::remove_file(&pid_file);
        println!(
            "toolbox: {} is not being served from this host",
            m.portal.name
        );
        return ExitCode::SUCCESS;
    };
    let p = rustix::process::Pid::from_raw(pid).expect("running() checked it");
    if let Err(e) = rustix::process::kill_process(p, rustix::process::Signal::TERM) {
        eprintln!("error: cannot stop pid {pid}: {e}");
        return ExitCode::from(1);
    }
    let deadline = Instant::now() + Duration::from_secs(20);
    while rustix::process::test_kill_process(p).is_ok() && running(&pid_file).is_some() {
        if Instant::now() > deadline {
            eprintln!("error: pid {pid} is still running 20s after SIGTERM");
            return ExitCode::from(1);
        }
        std::thread::sleep(Duration::from_millis(100));
    }
    let _ = std::fs::remove_file(&pid_file);
    println!("toolbox: stopped {} (pid {pid})", m.portal.name);
    ExitCode::SUCCESS
}

#[cfg(test)]
mod tests {
    #[test]
    fn a_wildcard_listen_address_is_reached_on_loopback() {
        assert_eq!(super::reach("0.0.0.0:7100"), "127.0.0.1:7100");
        assert_eq!(super::reach("127.0.0.1:7100"), "127.0.0.1:7100");
    }
}
