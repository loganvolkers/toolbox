# Hosted report UX

*Status: decided 2026-11-21 (D01M7XY1QYKW2D0HMSG50R0XTQH). Amends the static page in
[repo-portals.md](./repo-portals.md) §4. The agent skill is
[`report-ux`](../../plugins/toolbox-operator/skills/report-ux/SKILL.md).*

---

## 1. Conclusion

A VP of engineering opening Toolbox's own report must be able to **judge quality in one scroll**,
then **explore the catalog and every check** without switching to `serve`. The page that shipped
under D92/D93 was a scorecard tab strip that expanded **failing rules only**. That is a work list,
not a quality judgement.

| Mode | Job | Primary content |
|---|---|---|
| **Overview** (default) | Fleet health | Entity and kind counts; per-scorecard level bars; "needs attention" ordered by `none` then bronze |
| **Catalog** | What exists | Kinds → entity ids → that entity's scores across every scorecard |
| **Scorecards** | Standards debt and proof | One scorecard; level and outcome filters; every rule outcome on expand |

## 2. Why the previous page failed the VP test

| Expectation | What the page did |
|---|---|
| "How healthy is Toolbox?" in the first viewport | Landed on the first scorecard name alphabetically |
| See the software inventory | No catalog; only entities that appear under the selected scorecard |
| See the bar we already clear | Expanded rows listed failing rules only; passing rules were invisible |
| Scan thirty-plus standards | Equal-weight pill buttons with no summary ranking |
| Share a deep link to a concern | Hash named a scorecard only |

## 3. Alternative products reviewed

Primary sources were product docs and the OpenSSF viewer (vendor marketing sites for Port/Cortex/OpsLevel
remain partially blocked in cloud sessions; docs pages and GitHub READMEs were used).

| Product | Pattern we keep | Pattern we skip on a static repo report |
|---|---|---|
| **OpsLevel Maturity Report** | Passing and failing checks on one ladder; fix path next to the fail | In-product "Fix this" mutations |
| **Cortex Bird's eye / All Scorecards** | Org-wide overview, drill-down, filters by level and failing rule | Domain/team hierarchy rollups |
| **Port scorecards** | Catalog as system of record; scorecards as overlay; distribution dashboards | Initiatives, alerts, deployment gates |
| **OpenSSF Scorecard viewer** | Every check listed with reason and remediation | Weighted 0–10 aggregate as the hero metric |
| **Dashboard design patterns** (Bach et al.) | Overview → detail-on-demand; URL-parameterized exploration | Multi-page personalization |

## 4. Skill adopted

No existing Cursor or Toolbox skill covered engineering-quality report UX. The new skill
`report-ux` encodes the reader jobs and the reject criteria above. Agents that change the HTML
element or `report_json` must follow it. `maintainer-skill` scores the file like every other skill.

## 5. Data contract

`report -o json|html|web` keeps `portal`, `snapshot`, `scorecards`, and `scores`, and adds:

```json
"catalog": { "Crate": ["crate:toolbox-cli", "..."], "Skill": ["..."] }
```

Ids are sorted. Full entity fields stay out of the static page (size and facet caveats in
repo-portals §4). Catalog browse and entity score rollups are computed in the element from
`catalog` + `scores`.

## 6. Interaction rules

- Default hash is Overview (`#` / `#overview`).
- Deep links: `#catalog`, `#catalog/<Kind>`, `#entity/<id>`, `#scorecard/<name>`.
- Scorecard entity rows sort worst-first (`none` → bronze → … → gold).
- Outcome filter defaults to **all**; "Failing" is one click for a work list.
- Passing rules render as a compact pass line; failing rules keep fix + evidence.

## Sources

- OpsLevel Maturity Report docs; Cortex Bird's eye and All Scorecards docs; Port scorecards guide;
  OpenSSF Scorecard viewer (`scorecard.dev/viewer`); Bach et al., *Dashboard Design Patterns*
  (arXiv:2205.00757); [repo-portals.md](./repo-portals.md); [minimal-ui.md](./minimal-ui.md).
