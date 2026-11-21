#!/usr/bin/env python3
"""Render a static HTML aggregated portal from compose.py report output."""

from __future__ import annotations

import argparse
import html
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent


def render(report: dict) -> str:
    catalog = report.get("catalog") or {}
    entities = catalog.get("entities") or []
    campaigns = report.get("campaigns") or {}
    extend = report.get("extend") or {}
    push = report.get("push") or {}
    adoption = report.get("adoption") or []

    rows_entities = "\n".join(
        "<tr>"
        f"<td>{html.escape(e.get('kind', ''))}</td>"
        f"<td><code>{html.escape(e.get('id', ''))}</code></td>"
        f"<td>{html.escape(str(e.get('name', '')))}</td>"
        "</tr>"
        for e in entities
    )
    rows_campaigns = "\n".join(
        "<tr>"
        f"<td><code>{html.escape(name)}</code></td>"
        f"<td>{int(prog.get('meeting', 0))}/{int(prog.get('inScope', 0))}</td>"
        f"<td>{html.escape(', '.join(prog.get('nodes') or []))}</td>"
        "</tr>"
        for name, prog in sorted(campaigns.items())
    )
    leaf_cards = []
    for leaf, info in sorted((push.get("leaves") or {}).items()):
        scorecards = ", ".join(f"<code>{html.escape(s)}</code>" for s in info.get("scorecards") or [])
        ext = ", ".join(
            f"<code>{html.escape(s)}</code>"
            for s in (extend.get("leafScorecards") or {}).get(leaf, [])
        )
        leaf_cards.append(
            f"<section class='leaf'><h3>{html.escape(leaf)}</h3>"
            f"<p><strong>Pushed shared:</strong> {scorecards}</p>"
            f"<p><strong>Leaf extensions:</strong> {ext}</p></section>"
        )

    counts = catalog.get("counts") or {}
    count_bits = " · ".join(
        f"{html.escape(k)}: {v}" for k, v in sorted(counts.items())
    )
    adopted = ", ".join(
        f"<code>{html.escape(a['node'])}/{html.escape(str(a['package']))}</code>"
        for a in adoption
    )

    return f"""<!DOCTYPE html>
<html lang="en">
<head>
<meta charset="utf-8"/>
<meta name="viewport" content="width=device-width, initial-scale=1"/>
<title>OSS monorepo federation portal</title>
<style>
  :root {{
    --ink: #1a1f1c;
    --muted: #5a635c;
    --paper: #f3f0e8;
    --band: #d9e2d4;
    --accent: #2f5d3a;
    --line: #c4c9bf;
  }}
  * {{ box-sizing: border-box; }}
  body {{
    margin: 0;
    font-family: "IBM Plex Sans", "Source Sans 3", "Segoe UI", sans-serif;
    color: var(--ink);
    background:
      radial-gradient(1200px 500px at 10% -10%, #e7efe0 0%, transparent 55%),
      linear-gradient(180deg, #ebe6db 0%, var(--paper) 40%, #e4e8df 100%);
    min-height: 100vh;
  }}
  header {{
    padding: 2.5rem 1.5rem 1.5rem;
    border-bottom: 1px solid var(--line);
    background: linear-gradient(90deg, rgba(47,93,58,0.08), transparent);
  }}
  header h1 {{
    font-family: "IBM Plex Serif", "Source Serif 4", Georgia, serif;
    font-weight: 600;
    font-size: clamp(1.8rem, 3vw, 2.4rem);
    margin: 0 0 0.4rem;
    letter-spacing: -0.02em;
  }}
  header p {{ margin: 0; color: var(--muted); max-width: 52rem; }}
  main {{ padding: 1.5rem; max-width: 1100px; margin: 0 auto; }}
  h2 {{
    font-family: "IBM Plex Serif", Georgia, serif;
    font-size: 1.25rem;
    margin: 2rem 0 0.75rem;
    border-bottom: 2px solid var(--accent);
    display: inline-block;
    padding-bottom: 0.15rem;
  }}
  .stats {{
    display: flex; flex-wrap: wrap; gap: 0.75rem; margin: 1rem 0 0;
  }}
  .stat {{
    background: var(--band);
    padding: 0.65rem 0.9rem;
    border-radius: 2px;
    font-size: 0.95rem;
  }}
  .leaf {{
    background: rgba(255,255,255,0.55);
    border: 1px solid var(--line);
    padding: 1rem 1.1rem;
    margin: 0.75rem 0;
  }}
  .leaf h3 {{ margin: 0 0 0.5rem; color: var(--accent); }}
  .leaf p {{ margin: 0.35rem 0; font-size: 0.92rem; line-height: 1.45; }}
  table {{
    width: 100%;
    border-collapse: collapse;
    background: rgba(255,255,255,0.55);
    font-size: 0.92rem;
  }}
  th, td {{
    text-align: left;
    padding: 0.55rem 0.7rem;
    border-bottom: 1px solid var(--line);
    vertical-align: top;
  }}
  th {{ background: var(--band); font-weight: 600; }}
  code {{
    font-family: "IBM Plex Mono", ui-monospace, monospace;
    font-size: 0.85em;
  }}
  footer {{
    padding: 1.5rem;
    color: var(--muted);
    font-size: 0.85rem;
    border-top: 1px solid var(--line);
    margin-top: 2rem;
  }}
</style>
</head>
<body>
<header>
  <h1>OSS monorepo federation portal</h1>
  <p>
    Aggregated view of standards push / leaf extend / results-up across
    <strong>babel</strong>, <strong>jest</strong>, and <strong>pnpm</strong>.
    Conventions are mined from upstream fixtures — not invented.
  </p>
  <div class="stats">
    <div class="stat">Root: <code>{html.escape(str(report.get('root')))}</code></div>
    <div class="stat">Catalog — {count_bits}</div>
    <div class="stat">Operators: governance → leaf</div>
  </div>
</header>
<main>
  <h2>Push + extend</h2>
  {''.join(leaf_cards)}

  <h2>Adoption (results up)</h2>
  <p>{adopted or '—'}</p>

  <h2>Campaigns</h2>
  <table>
    <thead><tr><th>Campaign</th><th>Meeting / in scope</th><th>Nodes</th></tr></thead>
    <tbody>{rows_campaigns}</tbody>
  </table>

  <h2>Catalog</h2>
  <table>
    <thead><tr><th>Kind</th><th>Id</th><th>Name</th></tr></thead>
    <tbody>{rows_entities}</tbody>
  </table>
</main>
<footer>
  Generated by <code>experiments/oss-monorepo-federation/render_portal.py</code>.
  See ATTRIBUTION.md for upstream licenses.
</footer>
</body>
</html>
"""


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--report",
        type=Path,
        help="JSON report path (default: run compose report in-process)",
    )
    parser.add_argument(
        "-o",
        "--out",
        type=Path,
        default=ROOT / "out" / "portal.html",
    )
    args = parser.parse_args(argv)

    if args.report:
        report = json.loads(args.report.read_text(encoding="utf-8"))
    else:
        # Import sibling compose without requiring package install.
        sys.path.insert(0, str(ROOT))
        import compose  # type: ignore

        report = compose.build_report(ROOT)

    out = args.out
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(render(report), encoding="utf-8")
    print(f"wrote {out}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
