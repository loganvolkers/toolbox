"""Verify the documented inventory of the installable skill files."""

import re
from pathlib import Path

root = Path("skills/security-audit")
readme = Path("README.md").read_text()
section = re.search(r"(?ms)^## Files\s*$\n(.*?)(?=^## |\Z)", readme)
table = re.search(r"(?m)^\|\s*File\s*\|.*\n\|[-: |]+\|\s*\n((?:\|[^\n]+\n?)*)", section[1]) if section else None
documented = [cell.strip().strip("`") for cell in re.findall(r"(?m)^\|([^|]+)\|", table[1])] if table else []
installed = {path.relative_to(root).as_posix() for path in root.rglob("*") if path.is_file()}

findings = []
if not documented or len(documented) != len(set(documented)):
    findings.append("README.md needs a nonempty Files table with unique file names")
findings.extend(
    f"README.md lists missing installed file {name}"
    for name in sorted(set(documented) - installed)
)
findings.extend(
    f"README.md does not list installed file {name}"
    for name in sorted(installed - set(documented))
)
if "SKILL.md" in installed:
    frontmatter = re.match(r"(?s)^---\n(.*?)\n---(?:\n|$)", (root / "SKILL.md").read_text())
    names = re.findall(r"(?m)^(?:name|'name'|\"name\"):[ \t]*([^\n#]+)", frontmatter[1]) if frontmatter else []
    if [name.strip().strip("'\"") for name in names] != [root.name]:
        findings.append(f"{root}/SKILL.md must declare exactly one name: {root.name}")
elif "SKILL.md" not in documented:
    findings.append(f"{root}/SKILL.md is required for installation")
if findings:
    raise SystemExit("\n".join(findings))
