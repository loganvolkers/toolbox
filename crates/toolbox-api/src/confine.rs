//! Start an in-image connector where it cannot read other processes' memory. Connectors run as the
//! server's user, beside OpenFGA, which holds the datastore password and its per-start key and
//! cannot be made non-dumpable. Same-user `/proc/<pid>/mem` and `environ` are then guarded only by
//! the host's Yama setting, which an image cannot choose. A Landlock domain closes that: a process
//! in one cannot ptrace, or read the memory or environment of, any process outside it.

use std::io;
use std::sync::Once;
use tokio::process::{Child, Command};

/// Spawn `cmd` from a short-lived thread that first enters a fresh Landlock domain, so the child
/// starts in it and the server's own threads stay outside. Each spawn gets its own domain, so
/// connectors cannot read each other either. Without Landlock in the kernel the child starts as
/// before, and the server warns once.
pub fn spawn(cmd: &mut Command) -> io::Result<Child> {
    let handle = tokio::runtime::Handle::current();
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let _runtime = handle.enter();
                if !confine_thread() {
                    warn_unconfined();
                }
                cmd.spawn()
            })
            .join()
            .unwrap_or_else(|_| Err(io::Error::other("the connector launcher panicked")))
    })
}

/// Whether this kernel enforces Landlock, checked from a throwaway thread.
pub fn available() -> bool {
    std::thread::spawn(confine_thread).join().unwrap_or(false)
}

#[cfg(target_os = "linux")]
fn confine_thread() -> bool {
    use landlock::{AccessFs, Ruleset, RulesetAttr, RulesetStatus};
    // Any domain restricts ptrace. Handling MakeBlock with no rule denies only creating block
    // devices, which the server's user cannot do anyway, so connectors keep every file they had.
    Ruleset::default()
        .handle_access(AccessFs::MakeBlock)
        .and_then(|r| r.create())
        .and_then(|r| r.restrict_self())
        .is_ok_and(|s| s.ruleset != RulesetStatus::NotEnforced)
}

#[cfg(not(target_os = "linux"))]
fn confine_thread() -> bool {
    false
}

fn warn_unconfined() {
    static ONCE: Once = Once::new();
    ONCE.call_once(|| {
        tracing::warn!(
            "Landlock is unavailable: in-image connectors can read same-user processes' memory \
             (including OpenFGA's credentials) wherever the host's ptrace policy allows"
        );
    });
}

#[cfg(all(test, target_os = "linux"))]
mod tests {
    use super::*;
    use std::process::Stdio;

    /// Whether a process can open `path`: the open is where ptrace access is checked.
    async fn opens(path: &str, confined: bool) -> bool {
        let mut cmd = Command::new("sh");
        cmd.args(["-c", "exec 3<\"$0\"", path])
            .stdout(Stdio::null())
            .stderr(Stdio::null());
        let mut child = if confined {
            spawn(&mut cmd).unwrap()
        } else {
            cmd.spawn().unwrap()
        };
        child.wait().await.unwrap().success()
    }

    #[tokio::test]
    async fn a_confined_connector_cannot_read_a_sibling() {
        if !available() {
            assert!(
                std::env::var_os("TOOLBOX_REQUIRE_LANDLOCK").is_none(),
                "TOOLBOX_REQUIRE_LANDLOCK is set, but this kernel has no Landlock"
            );
            eprintln!("skipped: this kernel has no Landlock");
            return;
        }
        // A dumpable same-user process, as OpenFGA is.
        let mut sibling = std::process::Command::new("sleep")
            .arg("30")
            .env("PROBE_SECRET", "s3cret")
            .spawn()
            .unwrap();
        let environ = format!("/proc/{}/environ", sibling.id());
        let mem = format!("/proc/{}/mem", sibling.id());
        let unconfined = opens(&environ, false).await;
        let (confined_environ, confined_mem) =
            (opens(&environ, true).await, opens(&mem, true).await);
        sibling.kill().unwrap();
        sibling.wait().unwrap();
        assert!(
            unconfined,
            "the control could not open a same-user environment"
        );
        assert!(
            !confined_environ,
            "a confined connector opened a sibling's environment"
        );
        assert!(
            !confined_mem,
            "a confined connector opened a sibling's memory"
        );
    }
}
