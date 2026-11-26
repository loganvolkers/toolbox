//! Start an in-image connector where it cannot read other processes' memory. Connectors run as the
//! server's user, beside OpenFGA, which holds the datastore password and its per-start key and
//! cannot be made non-dumpable. Same-user `/proc/<pid>/mem` and `environ` are then guarded only by
//! the host's Yama setting, which an image cannot choose. A Landlock domain closes that: a process
//! in one cannot ptrace, or read the memory or environment of, any process outside it.

use std::io;
use std::sync::Once;
use tokio::process::{Child, Command};

/// Whether an in-image connector may start without Landlock (`SEIKIKA_CONNECTOR_CONFINEMENT`,
/// D01M88668MR8K559A1NYSNQQTX0). `best-effort`, the default, starts it unconfined and says so,
/// because some container hosts (gVisor, a seccomp profile without the Landlock calls) have none.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum Confinement {
    #[default]
    BestEffort,
    Required,
}

impl Confinement {
    pub const VAR: &str = "SEIKIKA_CONNECTOR_CONFINEMENT";

    /// A value of the variable; unset or empty is `best-effort`.
    pub fn parse(v: &str) -> Result<Self, String> {
        match v.trim() {
            "" | "best-effort" => Ok(Self::BestEffort),
            "required" => Ok(Self::Required),
            other => Err(format!(
                "{}={other}: expected `required` or `best-effort`",
                Self::VAR
            )),
        }
    }

    /// The setting from the environment, or why its value is not one.
    pub fn from_env() -> Result<Self, String> {
        std::env::var(Self::VAR).map_or(Ok(Self::default()), |v| Self::parse(&v))
    }

    /// The value as the variable spells it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::BestEffort => "best-effort",
            Self::Required => "required",
        }
    }
}

/// How a connector's last start went.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Started {
    /// In its own Landlock domain.
    Confined,
    /// Without Landlock, under `best-effort`.
    Unconfined,
    /// Not started: no Landlock, under `required`.
    Refused,
}

impl Started {
    /// How `/v1/connectors`, `/readyz` and explain spell it.
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Confined => "landlock",
            Self::Unconfined => "unconfined",
            Self::Refused => "refused",
        }
    }
}

/// The error [`spawn`] returns when `required` confinement can't be had.
#[derive(Debug)]
pub struct Refused;

impl std::fmt::Display for Refused {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Landlock is unavailable and {}=required: refusing to start an unconfined connector",
            Confinement::VAR
        )
    }
}

impl std::error::Error for Refused {}

/// Whether `e` is [`spawn`] refusing an unconfined start, as opposed to the program failing.
pub fn is_refused(e: &io::Error) -> bool {
    e.get_ref().is_some_and(|i| i.is::<Refused>())
}

/// What to do once the launcher thread has tried to enter a domain.
fn decide(confined: bool, setting: Confinement) -> Result<Started, Refused> {
    match (confined, setting) {
        (true, _) => Ok(Started::Confined),
        (false, Confinement::BestEffort) => Ok(Started::Unconfined),
        (false, Confinement::Required) => Err(Refused),
    }
}

/// Spawn `cmd` from a short-lived thread that first enters a fresh Landlock domain, so the child
/// starts in it and the server's own threads stay outside. Each spawn gets its own domain, so
/// connectors cannot read each other either. Without Landlock in the kernel, `best-effort` starts
/// the child as before and the server warns once; `required` doesn't start it ([`Refused`]).
pub fn spawn(cmd: &mut Command, setting: Confinement) -> io::Result<(Child, Started)> {
    let handle = tokio::runtime::Handle::current();
    std::thread::scope(|scope| {
        scope
            .spawn(|| {
                let _runtime = handle.enter();
                let started = decide(confine_thread(), setting).map_err(io::Error::other)?;
                if started == Started::Unconfined {
                    warn_unconfined();
                }
                cmd.spawn().map(|c| (c, started))
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
            "Landlock is unavailable: in-image connectors start unconfined and can read same-user \
             processes' memory (including OpenFGA's credentials) wherever the host's ptrace \
             policy allows; set {}=required to refuse them instead",
            Confinement::VAR
        );
    });
}

#[cfg(test)]
mod setting_tests {
    use super::*;

    #[test]
    fn best_effort_is_the_default_and_only_required_refuses() {
        assert_eq!(Confinement::parse("").unwrap(), Confinement::BestEffort);
        assert_eq!(
            Confinement::parse(" required ").unwrap(),
            Confinement::Required
        );
        assert_eq!(
            Confinement::parse("best-effort").unwrap(),
            Confinement::BestEffort
        );
        assert!(
            Confinement::parse("strict")
                .unwrap_err()
                .contains("expected `required`")
        );
        assert_eq!(
            decide(true, Confinement::Required).unwrap(),
            Started::Confined
        );
        assert_eq!(
            decide(false, Confinement::BestEffort).unwrap(),
            Started::Unconfined
        );
        let refused = io::Error::other(decide(false, Confinement::Required).unwrap_err());
        assert!(is_refused(&refused));
        assert!(!is_refused(&io::Error::from(
            io::ErrorKind::PermissionDenied
        )));
    }
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
            spawn(&mut cmd, Confinement::Required).unwrap().0
        } else {
            cmd.spawn().unwrap()
        };
        child.wait().await.unwrap().success()
    }

    #[tokio::test]
    async fn a_confined_connector_cannot_read_a_sibling() {
        if !available() {
            assert!(
                std::env::var_os("SEIKIKA_REQUIRE_LANDLOCK").is_none(),
                "SEIKIKA_REQUIRE_LANDLOCK is set, but this kernel has no Landlock"
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
