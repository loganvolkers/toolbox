# Shell script standard

The package puts ShellCheck and shfmt results in Seikika's runner and ratchet. The
tools also run independently. Bats runs directly in CI so contributors see which
behavior test failed.

| Requirement | Command |
| --- | --- |
| ShellCheck 0.11.0, zero warnings and errors | `shellcheck --severity=warning -- path/to/script.sh` |
| shfmt 3.12.0, two spaces and indented case arms | `shfmt -i 2 -ci -d -- path/to/script.sh` |
| Bats 1.12.0, shell behavior tests | `bats tests/shell` |

Declare the intended shell with a shebang or a ShellCheck shell directive. Quote
expansions and preserve argument boundaries. Handle expected failures explicitly.
Keep ShellCheck suppressions narrow and explain why they are safe where they apply.

Add a Bats case for a bug fix or externally observable change when it can be
exercised in isolation. Cover the expected output or side effect and exit status.
Use Bats temporary directories and stub external commands as needed. Keep
credentials, network calls, and shared resources out of these tests. Format
`.bats` files with shfmt before running them.

## Use it

Choose a Git commit or release containing this directory. Include the package
and copy [check-shell.py](check-shell.py) from the same ref into your checkout.
The adapter uses Python 3 and Git. It checks tracked and unignored untracked
files matched by the supplied Git pathspecs. List extensionless shell launchers
explicitly where a pathspec would otherwise miss them.

```yaml
include:
  - package: github:seikika/seikika/packages/shell@<ref>
    with:
      command: [python3, tools/check-shell.py, "scripts/*.sh"]
```

From the checkout root, run `seikika install .seikika`, install the pinned tools
from their upstream releases or use `seikika tools config .seikika --for mise`
to generate pins for your mise configuration. With the tools on PATH, run `seikika doctor .seikika` and `seikika report .seikika --all`. The report
should show one `ShellScript` per selected source, with rules for lint and
formatting. Set `name` and `kind` to distinct values for multiple includes. Use
`track` and `owner` to place the scorecard in your portal.

In [CI](https://seikika.com/operators/ci/), require the portal ratchet and add a
step that runs `shfmt -i 2 -ci -d tests/shell/*.bats` and `bats tests/shell` from
the checkout root. Set the path to your Bats suite. Seikika uses these same checks
in its [maintainer workflow](../../.github/workflows/ci.yml).

## Verify changes to the package

Run `seikika validate packages/shell/tests` and `seikika test packages/shell/tests`
to check the package rules against fixtures. Run `bats tests/shell` in Seikika's
checkout to exercise the adapter with the pinned tools and test the launcher.

## Sources

- [ShellCheck CLI and diagnostics](https://github.com/koalaman/shellcheck)
- [shfmt and shell dialects](https://github.com/mvdan/sh)
- [Bats test authoring](https://bats-core.readthedocs.io/en/stable/writing-tests.html)
