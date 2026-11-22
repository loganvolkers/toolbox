(async function () {
  const app = document.getElementById("app");
  const tabs = document.querySelectorAll("#tabs a");
  let data;
  let report;
  try {
    const res = await fetch("../fixtures.json");
    data = await res.json();
  } catch (e) {
    app.innerHTML = "<p>Could not load fixtures.json</p>";
    return;
  }
  try {
    const res = await fetch("../report/report.json");
    if (res.ok) report = await res.json();
  } catch (e) {
    report = null;
  }

  function setActive(hash) {
    tabs.forEach((a) => a.classList.toggle("active", a.getAttribute("href") === hash));
  }

  function home() {
    const k = data.kpis;
    const catalogNote =
      report && report.catalog
        ? `<p class="muted">Catalog from Toolbox report: ${Object.entries(report.catalog)
            .map(([k, ids]) => `${ids.length} ${k}`)
            .join(", ")}.</p>`
        : "";
    return `
      <h1>Good day</h1>
      <div class="grid">
        <div class="card"><div class="n">${k.in_progress}</div><div class="l">In progress — features moving through the line</div></div>
        <div class="card warn"><div class="n">${k.waiting_on_person}</div><div class="l">Waiting on a person — queue building up</div></div>
        <div class="card bad"><div class="n">${k.stuck}</div><div class="l">Stuck — needs attention</div></div>
        <div class="card ok"><div class="n">${k.shipped}</div><div class="l">Shipped — reached production</div></div>
      </div>
      ${catalogNote}
      <h2>Main features line</h2>
      <div class="stages">
        ${data.stages.map((s) => `<div class="stage"><h3>${s.label}</h3><div class="n">${s.count}</div></div>`).join("")}
      </div>`;
  }

  function catalog() {
    const rows = data.factory_lines
      .map(
        (l) => `<tr>
        <td><a href="#entity">${l.title}</a></td>
        <td><span class="badge ${l.lifecycle}">${l.lifecycle}</span></td>
        <td>${l.owning_team}</td>
        <td>${l.started_by}</td>
        <td>${l.open_items}</td>
        <td>${l.stages}</td>
        <td>${l.cost_per_shipped}</td>
      </tr>`
      )
      .join("");
    const real =
      report && report.catalog && report.catalog.FactoryLine
        ? `<p class="muted">Backed by Toolbox catalog (${report.catalog.FactoryLine.length} FactoryLine entities in <a href="../report/">report</a>).</p>`
        : "";
    return `<h1>Factory lines</h1>
      ${real}
      <table><thead><tr>
        <th>Title</th><th>Lifecycle</th><th>Owning team</th><th>Started by</th>
        <th>Open</th><th>Stages</th><th>Cost / shipped</th>
      </tr></thead><tbody>${rows}</tbody></table>`;
  }

  function entity() {
    const l = data.factory_lines[0];
    return `<h1>${l.title} <span class="badge">Factory Line</span></h1>
      <p class="muted">${l.owning_team} · ${l.lifecycle}</p>
      <div class="grid">
        <div class="card"><div class="n">${l.open_items}</div><div class="l">Open items</div></div>
        <div class="card"><div class="n">${l.cycle_time_days}</div><div class="l">Cycle time (days)</div></div>
        <div class="card"><div class="n">${l.done_30d}</div><div class="l">Done (30 days)</div></div>
      </div>
      <h2>Work items by stage</h2>
      <div class="stages">
        ${data.stages.map((s) => `<div class="stage"><h3>${s.label}</h3><div class="n">${s.count}</div></div>`).join("")}
      </div>`;
  }

  function myWork() {
    const rows = data.hitl_tasks
      .map(
        (t) => `<tr>
        <td>${t.title}</td><td>${t.why}</td><td>${t.factory_line}</td>
        <td><span class="badge">${t.stage}</span></td><td>${t.risk_score}</td><td>${t.days_waiting}</td>
      </tr>`
      )
      .join("");
    const real =
      report && report.catalog && report.catalog.HitlTask
        ? `<p class="muted">${report.catalog.HitlTask.length} HitlTask entities in the Toolbox report.</p>`
        : "";
    return `<h1>My work (engineer)</h1>
      <p class="muted">What the factory is waiting on you for.</p>
      ${real}
      <table><thead><tr>
        <th>Task</th><th>Why</th><th>Line</th><th>Stage</th><th>Risk</th><th>Days</th>
      </tr></thead><tbody>${rows}</tbody></table>`;
  }

  function tierCounts(score, def) {
    const levels = (def && def.levels) || ["bronze", "silver", "gold"];
    const rules = (def && def.rules) || {};
    const out = {};
    for (const level of levels) {
      const names = Object.keys(rules).filter((n) => rules[n].level === level);
      const passed = names.filter((n) => score.rules && score.rules[n] === "pass").length;
      out[level] = { passed, total: names.length };
    }
    return out;
  }

  function governanceFromReport() {
    const card = "agent-governance";
    const scores = report.scores && report.scores[card];
    const def = report.scorecards && report.scorecards[card];
    if (!scores || !def) return null;
    const id = "agent:release-notes-drafter";
    const score = scores[id];
    if (!score) return null;
    const agent = (data.agents || []).find((a) => a.id === "release-notes-drafter") || data.agents[0];
    const tiers = tierCounts(score, def);
    const level = score.level || "none";
    const filled = level === "gold" ? 100 : level === "silver" ? 75 : level === "bronze" ? 40 : 10;
    const overBudget = Object.values(scores).filter(
      (s) => s.rules && s.rules["under-budget"] === "fail"
    ).length;
    return `<h1>Governance</h1>
      <p class="muted">Scorecard levels from <code>toolbox report</code> (<a href="../report/">report.json</a>), not hand-edited fixtures.</p>
      <div class="grid">
        <div class="card warn"><div class="n">${overBudget}</div><div class="l">Agents over budget (under-budget rule)</div></div>
        <div class="card"><div class="n">${Object.keys(scores).length}</div><div class="l">Agents scored</div></div>
        <div class="card"><div class="n">${(report.catalog && report.catalog.Agent && report.catalog.Agent.length) || 0}</div><div class="l">Agent catalog entities</div></div>
      </div>
      <h2>${(agent && agent.title) || id} — Agent Governance | ${level[0].toUpperCase() + level.slice(1)}</h2>
      <p class="muted">${(agent && (agent.kind || agent.kindLabel)) || "External AI Agent"} · evaluated by Toolbox</p>
      <div class="bar"><span style="width:${filled}%"></span></div>
      ${Object.entries(tiers)
        .map(
          ([name, t]) =>
            `<div class="tier"><strong>${name[0].toUpperCase() + name.slice(1)}</strong>${t.passed}/${t.total} rules</div>`
        )
        .join("")}`;
  }

  function governance() {
    const fromReport = report && governanceFromReport();
    if (fromReport) return fromReport;
    const g = data.governance;
    const a = data.agents[0];
    const sc = a.scorecard;
    const filled = 75;
    return `<h1>Governance</h1>
      <div class="grid">
        <div class="card warn"><div class="n">${g.agents_over_budget}</div><div class="l">Agents over budget</div></div>
        <div class="card bad"><div class="n">${g.risky_agents}</div><div class="l">Risky agents</div></div>
        <div class="card bad"><div class="n">${g.risky_skills}</div><div class="l">Risky skills</div></div>
      </div>
      <h2>${a.title} — ${sc.title} | ${sc.level}</h2>
      <p class="muted">${a.kind || "External AI Agent"} · scorecard pass ${a.scorecard_pass_pct}%</p>
      <div class="bar"><span style="width:${filled}%"></span></div>
      ${Object.entries(sc.tiers)
        .map(
          ([name, t]) =>
            `<div class="tier"><strong>${name[0].toUpperCase() + name.slice(1)}</strong>${t.passed}/${t.total} rules</div>`
        )
        .join("")}`;
  }

  function selfServe() {
    const gaps = (data.gaps_vs_port || [])
      .map((g) => `<li>${g}</li>`)
      .join("");
    return `<h1>Self-serve</h1>
      <p class="muted">Port’s action list is at <a href="https://demo.getport.io/self-serve">demo.getport.io/self-serve</a>. Toolbox does not fake execution (M5 / D64 paused).</p>
      <div class="card warn"><div class="l">Available in Toolbox today</div><div class="n" style="font-size:1.1rem">Entity refresh</div><p class="muted">POST /v1/entities/{id}/refresh in the minimal UI — not a form runner.</p></div>
      <h2>Not rebuilt here</h2>
      <ul>${gaps}</ul>
      <p><a href="/toolbox/demo/port/gaps/">Honest gaps (docs)</a></p>`;
  }

  function render() {
    const hash = location.hash || "#home";
    setActive(hash);
    // Explicit allowlist — never call a function looked up from a user-controlled key.
    let html;
    switch (hash) {
      case "#home":
        html = home();
        break;
      case "#catalog":
        html = catalog();
        break;
      case "#entity":
        html = entity();
        break;
      case "#my-work":
        html = myWork();
        break;
      case "#governance":
        html = governance();
        break;
      case "#self-serve":
        html = selfServe();
        break;
      default:
        html = home();
        break;
    }
    app.innerHTML = html;
  }

  window.addEventListener("hashchange", render);
  render();
})();
