//! Executable connectors and tools (D90), end to end through the binary: `toolbox install`
//! downloads a released connector from a local release server, pins it in toolbox.lock, refuses a
//! tampered archive, and `toolbox doctor` reports connectors and tools.

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
    let bin = cache.join(format!("connectors/acme/conns/1.0.0/{target}/fake"));
    assert!(bin.is_file(), "{}", bin.display());

    // Doctor: the connector resolves; the tool is missing; --mise prints how to install it.
    let o = toolbox(&["doctor", p], &env);
    assert_eq!(o.status.code(), Some(1), "{}", text(&o));
    assert!(text(&o).contains("ok      connector fake"), "{}", text(&o));
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
