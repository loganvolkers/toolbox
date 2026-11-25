//! Executable connectors and tools (D90), end to end through the binary: `toolbox install`
//! downloads a released connector from a local release server, pins it in toolbox.lock, refuses a
//! tampered archive, and `toolbox doctor` reports connectors, executor packages and tools.

use std::io::{Read, Write};
use std::path::Path;
use std::process::{Command, Output};

/// A minimal HTTP/1.0 server for files under `root` (GET only), for the test's lifetime.
fn serve(root: std::path::PathBuf) -> String {
    let l = std::net::TcpListener::bind("127.0.0.1:0").unwrap();
    let url = format!("http://{}", l.local_addr().unwrap());
    std::thread::spawn(move || {
        for mut s in l.incoming().flatten() {
            let mut buf = [0u8; 4096];
            let n = s.read(&mut buf).unwrap_or(0);
            let req = String::from_utf8_lossy(&buf[..n]);
            let path = req
                .split_whitespace()
                .nth(1)
                .unwrap_or("/")
                .trim_start_matches('/');
            match std::fs::read(root.join(path)) {
                Ok(body) => {
                    let _ = write!(
                        s,
                        "HTTP/1.0 200 OK\r\nContent-Length: {}\r\n\r\n",
                        body.len()
                    );
                    let _ = s.write_all(&body);
                }
                Err(_) => {
                    let _ = s.write_all(b"HTTP/1.0 404 Not Found\r\nContent-Length: 0\r\n\r\n");
                }
            }
        }
    });
    url
}

fn toolbox(args: &[&str], envs: &[(&str, &str)]) -> Output {
    let mut c = Command::new(env!("CARGO_BIN_EXE_toolbox"));
    c.args(args);
    for (k, v) in envs {
        c.env(k, v);
    }
    c.output().unwrap()
}

fn text(o: &Output) -> String {
    format!(
        "{}{}",
        String::from_utf8_lossy(&o.stdout),
        String::from_utf8_lossy(&o.stderr)
    )
}

/// A release of `fake` (a shell script connector) at `<root>/acme/conns/releases/download/v1.0.0/`.
fn release(root: &Path, target: &str, body: &str) {
    let dir = root.join("acme/conns/releases/download/v1.0.0");
    let stage = root.join(format!("stage/fake-{target}"));
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&stage).unwrap();
    std::fs::write(stage.join("fake"), body).unwrap();
    let archive = dir.join(format!("fake-{target}.tar.xz"));
    assert!(
        Command::new("tar")
            .arg("-cJf")
            .arg(&archive)
            .arg("-C")
            .arg(root.join("stage"))
            .arg(format!("fake-{target}"))
            .status()
            .unwrap()
            .success()
    );
    let sum = Command::new("sha256sum").arg(&archive).output().unwrap();
    let hash = String::from_utf8_lossy(&sum.stdout)
        .split_whitespace()
        .next()
        .unwrap()
        .to_owned();
    std::fs::write(
        dir.join(format!("fake-{target}.tar.xz.sha256")),
        format!("{hash} *fake-{target}.tar.xz\n"),
    )
    .unwrap();
}

#[test]
#[cfg(target_os = "linux")]
fn install_pins_verifies_and_doctor_reports() {
    let base = std::env::temp_dir().join(format!("toolbox-install-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (rel, portal, cache) = (base.join("rel"), base.join("portal"), base.join("cache"));
    std::fs::create_dir_all(portal.join("fixtures")).unwrap();
    let target = toolbox_api::artifacts::target();
    release(&rel, target, "#!/bin/sh\necho fake\n");
    std::fs::write(
        portal.join("toolbox.yaml"),
        r#"
apiVersion: toolbox.dev/v1alpha1
portal: {name: demo, title: Demo, owner: group:maintainers, toolbox: "0.1"}
connectors:
  - {name: fake, package: "github:acme/conns/fake@1.0.0", runtime: in-image, records: [thing]}
kinds:
  - name: Thing
    identityFrom: [connector:fake]
    fields: {title: {type: string}}
    sources: {default: [connector:fake]}
    map: [{from: fake/thing, id: "'thing:' + record.name", set: "{'title': record.name}"}]
actions:
  - name: poke
    kind: Thing
    description: Poke a thing.
    unverified: true
    inputs: {type: object}
    executor: {type: in-image, package: "github:acme/conns/fake@1.0.0"}
tools:
  sh-tool: {version: "9.9.9", mise: "aqua:acme/sh-tool"}
"#,
    )
    .unwrap();
    let url = serve(rel.clone());
    let env = [
        ("TOOLBOX_RELEASES_URL", url.as_str()),
        ("TOOLBOX_CACHE_DIR", cache.to_str().unwrap()),
    ];
    let p = portal.to_str().unwrap();

    // Not pinned yet: --frozen refuses; a plain install pins it.
    assert_eq!(
        toolbox(&["install", p, "--frozen"], &env).status.code(),
        Some(2)
    );
    let o = toolbox(&["install", p], &env);
    assert!(o.status.success(), "{}", text(&o));
    let lock: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(portal.join("toolbox.lock")).unwrap())
            .unwrap();
    let pinned = lock["connectors"]["github:acme/conns/fake@1.0.0"][target]
        .as_str()
        .unwrap()
        .to_owned();
    assert_eq!(pinned.len(), 64);
    assert_eq!(
        lock["executors"]["github:acme/conns/fake@1.0.0"][target].as_str(),
        Some(pinned.as_str())
    );
    let bin = cache.join(format!("connectors/acme/conns/1.0.0/{target}/fake"));
    assert!(bin.is_file(), "{}", bin.display());

    // Doctor: the connector resolves; the tool is missing; --mise prints how to install it.
    let o = toolbox(&["doctor", p], &env);
    assert_eq!(o.status.code(), Some(1), "{}", text(&o));
    assert!(text(&o).contains("ok      connector fake"), "{}", text(&o));
    assert!(
        text(&o).contains("ok      executor github:acme/conns/fake@1.0.0 (action poke)"),
        "{}",
        text(&o)
    );
    assert!(
        text(&o).contains("MISSING tool sh-tool 9.9.9"),
        "{}",
        text(&o)
    );
    let o = toolbox(&["doctor", p, "--mise"], &env);
    assert!(
        text(&o).contains("\"aqua:acme/sh-tool\" = \"9.9.9\""),
        "{}",
        text(&o)
    );

    // A re-published (tampered) archive no longer matches the lock: install refuses it.
    release(&rel, target, "#!/bin/sh\necho evil\n");
    let o = toolbox(&["install", p, "--frozen"], &env);
    assert_eq!(o.status.code(), Some(2));
    assert!(
        text(&o).contains("doesn't match") && text(&o).contains("toolbox.lock"),
        "{}",
        text(&o)
    );
    let _ = std::fs::remove_dir_all(&base);
}

/// A bare git host under `host/acme/pkg.git` with tag `v0.1.0`, branch `main`, and a package tree.
fn git_package_host(base: &Path) -> (String, String) {
    let work = base.join("work");
    let bare = base.join("host/acme/pkg.git");
    std::fs::create_dir_all(bare.parent().unwrap()).unwrap();
    assert!(
        Command::new("git")
            .args(["init", "-q", "--bare"])
            .arg(&bare)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["init", "-q"])
            .arg(&work)
            .status()
            .unwrap()
            .success()
    );
    let pkg = work.join("packages/demo");
    std::fs::create_dir_all(pkg.join("fragments")).unwrap();
    std::fs::write(
        pkg.join("toolbox-package.yaml"),
        "apiVersion: toolbox.dev/package/v0\nname: demo\nversion: \"0.1.0\"\n",
    )
    .unwrap();
    // No kinds/connectors: only prove the tree is cloned and hashed.
    std::fs::write(pkg.join("fragments/empty.yaml"), "{}\n").unwrap();
    assert!(
        Command::new("git")
            .args(["-C"])
            .arg(&work)
            .args(["config", "user.email", "t@example.com"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["-C"])
            .arg(&work)
            .args(["config", "user.name", "t"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["-C"])
            .arg(&work)
            .args(["add", "."])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["-C"])
            .arg(&work)
            .args(["commit", "-q", "-m", "pkg"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["-C"])
            .arg(&work)
            .args(["branch", "-M", "main"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["-C"])
            .arg(&work)
            .args(["tag", "v0.1.0"])
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["-C"])
            .arg(&work)
            .args(["remote", "add", "origin"])
            .arg(&bare)
            .status()
            .unwrap()
            .success()
    );
    assert!(
        Command::new("git")
            .args(["-C"])
            .arg(&work)
            .args(["push", "-q", "origin", "main", "v0.1.0"])
            .status()
            .unwrap()
            .success()
    );
    let sha = String::from_utf8(
        Command::new("git")
            .args(["-C"])
            .arg(&work)
            .args(["rev-parse", "HEAD"])
            .output()
            .unwrap()
            .stdout,
    )
    .unwrap()
    .trim()
    .to_owned();
    let url = format!("file://{}", base.join("host").display());
    (url, sha)
}

fn write_portal_with_package(portal: &Path, package: &str) {
    std::fs::create_dir_all(portal).unwrap();
    std::fs::write(
        portal.join("toolbox.yaml"),
        format!(
            r#"
apiVersion: toolbox.dev/v1alpha1
portal: {{name: demo, title: Demo, owner: group:maintainers, toolbox: "0.1"}}
include:
  - package: {package}
connectors:
  - {{name: workspace, package: builtin:workspace, runtime: in-image, records: [repo]}}
kinds:
  - name: Repository
    identityFrom: [connector:workspace]
    fields: {{name: {{type: string}}}}
    sources: {{default: [connector:workspace]}}
    map: [{{from: workspace/repo, id: "'repo:' + record.name", set: "{{'name': record.name}}"}}]
"#
        ),
    )
    .unwrap();
}

#[test]
fn install_git_package_tag_branch_and_commit() {
    let base = std::env::temp_dir().join(format!("toolbox-pkg-install-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&base);
    let (git_url, sha) = git_package_host(&base);
    let cache = base.join("cache");

    // Semver tag → v0.1.0
    let portal_tag = base.join("portal-tag");
    write_portal_with_package(&portal_tag, "github:acme/pkg/packages/demo@0.1.0");
    let env = [
        ("TOOLBOX_GIT_URL", git_url.as_str()),
        ("TOOLBOX_CACHE_DIR", cache.to_str().unwrap()),
    ];
    let o = toolbox(&["install", portal_tag.to_str().unwrap()], &env);
    assert!(o.status.success(), "{}", text(&o));
    assert!(
        cache
            .join("packages/acme/pkg/0.1.0/packages/demo/toolbox-package.yaml")
            .is_file()
    );

    // Branch main (must not become --branch vmain)
    let portal_main = base.join("portal-main");
    write_portal_with_package(&portal_main, "github:acme/pkg/packages/demo@main");
    let cache_main = base.join("cache-main");
    let env_main = [
        ("TOOLBOX_GIT_URL", git_url.as_str()),
        ("TOOLBOX_CACHE_DIR", cache_main.to_str().unwrap()),
    ];
    let o = toolbox(&["install", portal_main.to_str().unwrap()], &env_main);
    assert!(o.status.success(), "{}", text(&o));
    assert!(
        cache_main
            .join("packages/acme/pkg/main/packages/demo/toolbox-package.yaml")
            .is_file()
    );

    // Full commit SHA (must not become --branch v<sha>)
    let portal_sha = base.join("portal-sha");
    write_portal_with_package(&portal_sha, &format!("github:acme/pkg/packages/demo@{sha}"));
    let cache_sha = base.join("cache-sha");
    let env_sha = [
        ("TOOLBOX_GIT_URL", git_url.as_str()),
        ("TOOLBOX_CACHE_DIR", cache_sha.to_str().unwrap()),
    ];
    let o = toolbox(&["install", portal_sha.to_str().unwrap()], &env_sha);
    assert!(o.status.success(), "{}", text(&o));
    assert!(
        cache_sha
            .join(format!(
                "packages/acme/pkg/{sha}/packages/demo/toolbox-package.yaml"
            ))
            .is_file(),
        "{}",
        text(&o)
    );
    let lock: serde_json::Value =
        serde_json::from_str(&std::fs::read_to_string(portal_sha.join("toolbox.lock")).unwrap())
            .unwrap();
    let pin = lock["packages"][&format!("github:acme/pkg/packages/demo@{sha}")]
        .as_str()
        .unwrap();
    assert_ne!(
        pin,
        "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"
    );
    let o = toolbox(
        &["install", portal_sha.to_str().unwrap(), "--frozen"],
        &env_sha,
    );
    assert!(o.status.success(), "{}", text(&o));

    let _ = std::fs::remove_dir_all(&base);
}
