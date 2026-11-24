// <toolbox-report>: Overview, Catalog, Campaigns, Scorecards, and Compare as a custom element.
//
// Data is a child <script type="application/json">, or the src attribute (a URL of that JSON).
// A host page projects its own chrome into the slots header, nav, and footer. Those children
// stay in light DOM; this file does not read or concatenate them. Mode chrome and scorecard
// navigation live inside the shadow tree and are not slots.
(function () {
  if (customElements.get("toolbox-report")) return;

  var CSS = [
    ":host{display:block;color:var(--fg);background:var(--bg);font:15px/1.55 system-ui,-apple-system,'Segoe UI',sans-serif;",
    "--bg:#f7f6f2;--fg:#1d1d1b;--muted:#5f5f5a;--faint:#8a8a84;--line:#e4e2dc;--card:#fff;--tint:#f1efe9;",
    "--accent:#2f5d8a;--accent-bg:#e8eff6;--pass:#21683e;--fail:#b23a2f;--unknown:#7a6420;",
    "--none:#d0493d;--none-ink:#a8352a;--bronze:#bf7b3a;--bronze-ink:#86501b;",
    "--silver:#9aa6b2;--silver-ink:#525d69;--gold:#e1b030;--gold-ink:#7a5a00;--skipped:#c9c8c2;--skipped-ink:#5f5f5a;",
    "--shadow:0 1px 2px rgba(30,30,25,.06),0 4px 14px rgba(30,30,25,.05)}",
    "@media (prefers-color-scheme:dark){:host{--bg:#141413;--fg:#ecebe7;--muted:#b0afa8;--faint:#8b8a84;--line:#2f2f2c;",
    "--card:#1d1d1b;--tint:#252523;--accent:#8fbbe6;--accent-bg:#1e2a36;--pass:#6fc493;--fail:#f08a80;--unknown:#d6c177;",
    "--none:#e0665b;--none-ink:#f39a91;--bronze:#d39a5c;--bronze-ink:#e9b985;--silver:#aab5c1;--silver-ink:#cbd4dd;",
    "--gold:#e6c04c;--gold-ink:#f0d57e;--skipped:#5c5c58;--skipped-ink:#b0afa8;--shadow:none}}",
    "*{box-sizing:border-box}",
    "main{max-width:1120px;margin:0 auto;padding:28px 20px 72px}",
    "a{color:var(--accent)}",
    "a:focus-visible,button:focus-visible,summary:focus-visible,input:focus-visible{outline:2px solid var(--accent);outline-offset:2px;border-radius:6px}",
    ".top{display:flex;align-items:center;gap:14px;margin-bottom:6px}",
    ".top svg{flex:none}",
    "h1{font-size:26px;line-height:1.2;margin:0;letter-spacing:-.01em}",
    "h2{font-size:19px;line-height:1.3;margin:0;letter-spacing:-.005em}",
    "h3{font-size:12px;margin:18px 0 8px;color:var(--muted);font-weight:650;text-transform:uppercase;letter-spacing:.06em}",
    ".meta{color:var(--muted);font-size:13px}",
    ".modes{display:flex;gap:4px;margin:20px 0 22px;border-bottom:1px solid var(--line);overflow-x:auto}",
    ".modes button{border:0;background:none;color:var(--muted);padding:10px 14px;font:inherit;font-weight:600;cursor:pointer;border-bottom:3px solid transparent;margin-bottom:-1px;white-space:nowrap}",
    ".modes button:hover{color:var(--fg)}",
    ".modes button[aria-selected=true]{color:var(--fg);border-bottom-color:var(--accent)}",
    ".modes .count{display:inline-block;min-width:20px;margin-left:6px;padding:0 6px;border-radius:10px;background:var(--tint);color:var(--muted);font-size:12px;font-weight:600}",
    "section{background:var(--card);border:1px solid var(--line);border-radius:14px;padding:20px;margin-bottom:18px;box-shadow:var(--shadow)}",
    ".head{display:flex;flex-wrap:wrap;gap:6px 16px;align-items:baseline;justify-content:space-between;margin-bottom:10px}",
    ".sub{color:var(--muted);font-size:13px}",
    ".lede{color:var(--muted);margin:4px 0 12px;max-width:72ch}",
    ".hero{display:grid;grid-template-columns:minmax(0,5fr) minmax(0,7fr);gap:24px;align-items:start;padding:24px;",
    "background:linear-gradient(135deg,color-mix(in srgb,var(--lvl) 16%,var(--card)) 0%,var(--card) 55%)}",
    ".hero .badge-big{display:flex;align-items:center;gap:16px}",
    ".hero .kicker{font-size:12px;font-weight:650;text-transform:uppercase;letter-spacing:.08em;color:var(--muted)}",
    ".hero .levelword{font-size:42px;line-height:1.05;font-weight:750;color:var(--lvl-ink);text-transform:capitalize;letter-spacing:-.02em}",
    ".hero .verdict{font-size:17px;line-height:1.5;margin:16px 0 0}",
    ".next{margin-top:14px;padding:12px 14px;border-radius:10px;background:var(--tint);font-size:14px}",
    ".next b{font-weight:650}",
    ".ladder{display:grid;grid-template-columns:repeat(var(--steps),minmax(0,1fr));gap:8px}",
    ".rung{border-radius:10px;padding:10px;background:var(--tint);min-height:120px;border:1px solid transparent}",
    ".rung.here{border-color:var(--lvl);background:color-mix(in srgb,var(--lvl) 12%,var(--card))}",
    ".rung .rh{display:flex;align-items:center;gap:6px;font-size:13px;font-weight:650;text-transform:capitalize;margin-bottom:8px}",
    ".rung .rh .n{margin-left:auto;color:var(--muted);font-weight:600}",
    ".rung a{display:block;font-size:13px;line-height:1.3;padding:5px 7px;margin-top:4px;border-radius:6px;background:var(--card);color:var(--fg);text-decoration:none;border:1px solid var(--line)}",
    ".rung a:hover{border-color:var(--accent)}",
    ".rung .empty{font-size:12px}",
    ".tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(190px,1fr));gap:12px;margin-bottom:18px}",
    ".tile{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:14px 16px;display:flex;gap:12px;align-items:center;box-shadow:var(--shadow);text-decoration:none;color:var(--fg)}",
    "a.tile:hover{border-color:var(--accent)}",
    ".tile strong{display:block;font-size:24px;line-height:1.15;font-variant-numeric:tabular-nums}",
    ".tile span{color:var(--muted);font-size:13px;line-height:1.3;display:block}",
    ".tile.bad strong{color:var(--fail)}.tile.good strong{color:var(--pass)}",
    ".bar{display:flex;height:10px;border-radius:5px;overflow:hidden;background:var(--line);gap:2px}",
    ".bar span{display:block;height:100%;transition:flex-grow .6s cubic-bezier(.2,.7,.2,1)}",
    ".bar.lg{height:16px;border-radius:8px}",
    ".bar.xs{height:6px;gap:1px}",
    ".legend{display:flex;flex-wrap:wrap;gap:6px 14px;font-size:12px;color:var(--muted);margin-top:6px}",
    ".legend b{color:var(--fg);font-weight:650;font-variant-numeric:tabular-nums}",
    ".dot{display:inline-block;width:9px;height:9px;border-radius:50%;margin-right:5px;vertical-align:-1px}",
    ".lvl{display:inline-flex;align-items:center;gap:5px;padding:1px 8px 1px 6px;border-radius:999px;font-size:12px;font-weight:650;text-transform:capitalize;white-space:nowrap;",
    "color:var(--ink);background:color-mix(in srgb,var(--fill) 18%,transparent);border:1px solid color-mix(in srgb,var(--fill) 50%,transparent)}",
    ".lvl i{width:8px;height:8px;border-radius:50%;background:var(--fill);display:inline-block}",
    ".lvl.none i{background:transparent;border:1.5px solid var(--fill)}",
    ".rows{display:flex;flex-direction:column}",
    "a.row{display:grid;grid-template-columns:minmax(0,2.2fr) minmax(0,2.4fr) 90px;gap:16px;align-items:center;padding:11px 10px;border-top:1px solid var(--line);color:var(--fg);text-decoration:none;border-radius:8px}",
    "a.row:hover{background:var(--tint)}",
    ".row .t{font-weight:600;line-height:1.3}",
    ".row .c{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);margin-top:2px;overflow-wrap:anywhere}",
    ".row .num{text-align:right;font-variant-numeric:tabular-nums}",
    ".row .num strong{display:block;font-size:18px;line-height:1.1}",
    ".row .num.bad strong{color:var(--fail)}.row .num.ok strong{color:var(--pass)}",
    ".row .num span{font-size:11px;color:var(--muted)}",
    ".rows-head{display:grid;grid-template-columns:minmax(0,2.2fr) minmax(0,2.4fr) 90px;gap:16px;padding:0 10px 6px;font-size:12px;color:var(--muted);font-weight:600}",
    ".rows-head span:last-child{text-align:right}",
    ".cards{display:grid;grid-template-columns:repeat(auto-fill,minmax(250px,1fr));gap:12px}",
    "a.card{display:flex;gap:14px;align-items:center;padding:14px;border:1px solid var(--line);border-radius:12px;background:var(--card);color:var(--fg);text-decoration:none;transition:transform .15s,border-color .15s}",
    "a.card:hover{border-color:var(--accent);transform:translateY(-1px)}",
    "a.card[aria-current=true]{border-color:var(--accent);box-shadow:0 0 0 1px var(--accent)}",
    "a.card .t{font-weight:600;line-height:1.3}",
    "a.card .s{font-size:12px;color:var(--muted);margin-top:3px}",
    ".ring{flex:none}",
    ".ring circle{transition:stroke-dashoffset .8s cubic-bezier(.2,.7,.2,1)}",
    ".ring text{font:700 13px system-ui,sans-serif;fill:var(--fg)}",
    ".pill{display:inline-block;padding:1px 8px;border-radius:999px;font-size:12px;font-weight:600;background:var(--tint);color:var(--muted)}",
    ".pill.good{background:color-mix(in srgb,var(--pass) 16%,transparent);color:var(--pass)}",
    ".pill.bad{background:color-mix(in srgb,var(--fail) 14%,transparent);color:var(--fail)}",
    "input[type=search]{width:100%;margin:10px 0;padding:9px 12px;border:1px solid var(--line);border-radius:8px;background:var(--bg);color:var(--fg);font:inherit}",
    ".filters{display:flex;flex-wrap:wrap;gap:8px 16px;margin:12px 0 4px;align-items:center}",
    ".seg{display:inline-flex;border:1px solid var(--line);border-radius:8px;overflow:hidden}",
    ".seg button{border:0;border-right:1px solid var(--line);background:var(--card);color:var(--fg);padding:6px 12px;font:inherit;font-size:13px;cursor:pointer;min-height:32px;text-transform:capitalize}",
    ".seg button:last-child{border-right:0}",
    ".seg button[aria-pressed=true]{background:var(--accent-bg);color:var(--accent);font-weight:650}",
    "details.ent{border-top:1px solid var(--line)}",
    "details.ent>summary{cursor:pointer;display:grid;grid-template-columns:86px minmax(0,1fr) auto;gap:12px;align-items:center;padding:10px 6px;list-style:none;border-radius:8px}",
    "details.ent>summary:hover{background:var(--tint)}",
    "details.ent>summary::-webkit-details-marker{display:none}",
    "details.ent>summary .name{font-weight:600;overflow-wrap:anywhere}",
    "details.ent>summary .name small{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);font-weight:400}",
    "details.ent>summary .tally{font-size:12px;color:var(--muted);text-align:right;white-space:nowrap}",
    "details.ent>summary .tally b{color:var(--fg);font-weight:650}details.ent>summary .tally b.bad{color:var(--fail)}",
    "details.ent[open]>summary{background:var(--tint)}",
    ".ent-body{padding:6px 6px 14px 104px}",
    ".chev{display:inline-block;transition:transform .15s;color:var(--faint);margin-left:6px}",
    "details[open]>summary .chev{transform:rotate(90deg)}",
    ".rule{padding:10px 12px;border-radius:8px;border:1px solid var(--line);border-left:4px solid var(--fail);margin-top:8px;background:var(--card)}",
    ".rule.unknown{border-left-color:var(--unknown)}.rule.waived{border-left-color:var(--bronze)}",
    ".rule .rn{font-weight:650}",
    ".rule .fix{margin-top:4px;font-size:14px}",
    ".rule ul{margin:6px 0 0;padding-left:18px;font-size:13px;color:var(--muted)}",
    ".rule li{overflow-wrap:anywhere}",
    ".rule code,.evid code{font:12.5px ui-monospace,SFMono-Regular,Menlo,monospace;overflow-wrap:anywhere}",
    ".outcome{font-size:12px;font-weight:650;margin-left:6px}",
    ".outcome.fail{color:var(--fail)}.outcome.unknown{color:var(--unknown)}.outcome.waived{color:var(--bronze-ink)}.outcome.skipped{color:var(--muted)}",
    ".passes{display:flex;flex-wrap:wrap;align-items:center;gap:6px;margin-top:8px}",
    ".passes>span:not(.lvl){font-size:12px;padding:2px 8px 2px 6px;border-radius:999px;background:color-mix(in srgb,var(--pass) 12%,transparent);color:var(--pass);font-weight:600}",
    ".passes>span:not(.lvl)::before{content:'\\2713  '}",
    ".passes em{font-style:normal;font-size:12px;color:var(--muted);margin-right:4px}",
    ".callout{display:flex;gap:10px;align-items:flex-start;padding:10px 12px;border-radius:10px;background:var(--accent-bg);font-size:14px;margin:8px 0}",
    ".callout.done{background:color-mix(in srgb,var(--pass) 12%,var(--card))}",
    ".fails{display:flex;flex-direction:column;gap:2px}",
    ".fail-row{display:grid;grid-template-columns:minmax(0,2fr) minmax(0,3fr) 70px;gap:12px;align-items:center;padding:7px 8px;border:0;border-radius:8px;background:none;color:var(--fg);font:inherit;font-size:13px;text-align:left;cursor:pointer}",
    ".fail-row:hover{background:var(--tint)}",
    ".fail-row .rn{display:flex;gap:8px;align-items:center;font-weight:600;overflow-wrap:anywhere}",
    ".fail-row .fc{text-align:right;color:var(--muted);font-variant-numeric:tabular-nums}",
    ".levels-need{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:10px;margin:8px 0 4px}",
    ".need{padding:10px 12px;border-radius:10px;background:var(--tint);font-size:13px}",
    ".need .lvl{margin-bottom:6px}",
    ".need ul{margin:0;padding-left:16px}",
    ".need li{margin:6px 0}",
    ".need li .why{display:block;color:var(--muted);font-size:12.5px;line-height:1.4}",
    "code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.92em}",
    ".picker{display:none}",
    ".picker select{width:100%;padding:9px 10px;border:1px solid var(--line);border-radius:8px;background:var(--bg);color:var(--fg);font:inherit;min-height:40px}",
    ".kinds{display:grid;grid-template-columns:repeat(auto-fill,minmax(170px,1fr));gap:10px}",
    "a.kind{display:block;padding:12px;border:1px solid var(--line);border-radius:10px;background:var(--card);color:var(--fg);text-decoration:none}",
    "a.kind:hover{border-color:var(--accent)}",
    "a.kind[aria-current=true]{border-color:var(--accent);box-shadow:0 0 0 1px var(--accent)}",
    "a.kind strong{display:block;font-size:14px}",
    "a.kind span{display:block;color:var(--muted);font-size:12px;margin-bottom:6px}",
    ".box{display:flex;flex-wrap:wrap;gap:6px}",
    ".box a{font-size:13px;padding:4px 10px;border-radius:999px;background:var(--tint);color:var(--fg);text-decoration:none;border:1px solid transparent}",
    ".box a:hover{border-color:var(--accent)}",
    ".box a b{font-variant-numeric:tabular-nums;margin-left:4px;color:var(--muted);font-weight:600}",
    "a.erow{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:12px;align-items:center;padding:10px 8px;border-top:1px solid var(--line);color:var(--fg);text-decoration:none;border-radius:8px}",
    "a.erow:hover{background:var(--tint)}",
    "a.erow .name{font-weight:600;overflow-wrap:anywhere}",
    "a.erow .name small{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);font-weight:400}",
    "a.erow .badges{display:flex;flex-wrap:wrap;gap:4px;justify-content:flex-end}",
    ".crumb{font-size:13px;color:var(--muted);margin-bottom:12px}",
    ".crumb a{color:var(--muted)}",
    ".ehead{display:flex;flex-wrap:wrap;gap:16px;align-items:center;justify-content:space-between}",
    ".ehead h2{font-size:24px}",
    ".ehead code{font:12.5px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);overflow-wrap:anywhere}",
    ".sc-block{border-top:1px solid var(--line);padding-top:14px;margin-top:14px}",
    ".sc-block .head a{font-weight:650;font-size:16px;text-decoration:none}",
    ".sc-block .head a:hover{text-decoration:underline}",
    ".rank{position:relative;display:flex;flex-direction:column;max-height:min(62vh,560px);overflow:auto;margin-top:4px}",
    ".rank a{display:flex;justify-content:space-between;gap:12px;align-items:baseline;padding:8px 10px;border-radius:8px;color:var(--fg);text-decoration:none}",
    ".rank a:hover{background:var(--tint)}",
    ".rank a[aria-current=true]{background:var(--accent-bg);font-weight:650}",
    ".rank .debt{font-size:12px;color:var(--muted);white-space:nowrap}",
    ".rank .debt.bad{color:var(--fail);font-weight:650}.rank .debt.ok{color:var(--pass)}",
    ".split{display:grid;grid-template-columns:300px minmax(0,1fr);gap:18px;align-items:start}",
    ".split>section:first-child{position:sticky;top:12px}",
    ".gap{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,2fr);gap:12px;padding:9px 6px;border-top:1px solid var(--line);font-size:14px}",
    ".gap a{font-weight:600}",
    ".gap .why{color:var(--muted);font-size:13px;overflow-wrap:anywhere}",
    ".empty{color:var(--muted);font-size:14px;padding:10px 0}",
    ".empty.good{color:var(--pass);font-weight:600}",
    "button.link{background:none;border:0;padding:0;color:var(--accent);font:inherit;cursor:pointer;text-decoration:underline;text-underline-offset:2px}",
    ".more{margin-top:10px}",
    ".milestones{position:relative;height:18px;margin-top:2px}",
    ".milestones span{position:absolute;top:0;transform:translateX(-50%);font-size:11px;color:var(--muted);white-space:nowrap}",
    ".milestones span::before{content:'';display:block;width:2px;height:6px;background:var(--faint);margin:0 auto 1px}",
    ".milestones span.met{color:var(--pass);font-weight:650}",
    ".tablewrap{overflow-x:auto}",
    "table.cmp{width:100%;border-collapse:collapse;font-size:14px}",
    "table.cmp th{text-align:left;font-size:12px;font-weight:650;color:var(--muted);padding:6px 10px 6px 0;border-bottom:1px solid var(--line)}",
    "table.cmp td{padding:10px 10px 10px 0;border-top:1px solid var(--line);vertical-align:top}",
    "table.cmp td.n{font-variant-numeric:tabular-nums;color:var(--muted)}",
    "table.cmp a{font-weight:600}",
    ".standing{display:inline-block;padding:1px 8px;border-radius:999px;font-size:12px;font-weight:650;background:var(--tint);color:var(--muted)}",
    ".standing.ahead{background:color-mix(in srgb,var(--pass) 16%,transparent);color:var(--pass)}",
    ".standing.behind{background:color-mix(in srgb,var(--fail) 14%,transparent);color:var(--fail)}",
    ".standing.parity{background:var(--accent-bg);color:var(--accent)}",
    ".value{display:block;font-size:12px;color:var(--muted);margin-top:2px}",
    "::slotted(*){display:block;max-width:1120px;margin:0 auto;padding:12px 20px 0;box-sizing:border-box}",
    "@media (max-width:820px){.hero{grid-template-columns:1fr}.split{grid-template-columns:1fr}.split>section:first-child{display:none}.picker{display:flex;flex-direction:column;gap:4px;margin:0 0 16px}}",
    ".ering{display:flex;flex-direction:column;align-items:center;gap:4px;flex:none}",
    "@media (max-width:600px){main{padding:18px 12px 56px}.hero .levelword{font-size:34px}",
    ".modes{gap:0}.modes button{padding:10px 9px}.modes .count{display:none}",
    ".kinds{grid-template-columns:repeat(2,minmax(0,1fr));gap:8px}a.kind{padding:9px}.fail-row{grid-template-columns:minmax(0,1fr) 64px}.fail-row .bar{display:none}",
    "a.row,.rows-head{grid-template-columns:minmax(0,1fr) 64px}.row .dist,.rows-head .dist{display:none}",
    ".ladder{grid-template-columns:1fr 1fr}.rung{min-height:0}",
    "details.ent>summary{grid-template-columns:76px minmax(0,1fr)}details.ent>summary .tally{grid-column:2;text-align:left;white-space:normal}",
    ".ent-body{padding-left:6px}.gap{grid-template-columns:1fr}.hide-sm{display:none}}",
    "@media (prefers-reduced-motion:reduce){*{transition:none!important}}",
  ].join("");

  var template = document.createElement("template");
  template.innerHTML =
    "<style>" +
    CSS +
    "</style>" +
    '<slot name="header"></slot><slot name="nav"></slot>' +
    '<main><div class="top" id="top"></div><div class="meta" id="meta"></div>' +
    '<div class="modes" id="modes" role="tablist" aria-label="Report views"></div>' +
    '<div id="body"></div></main><slot name="footer"></slot>';

  var SVGNS = "http://www.w3.org/2000/svg";
  var KNOWN = ["none", "bronze", "silver", "gold", "skipped"];

  function el(t, a) {
    var e = document.createElement(t);
    var kids = Array.prototype.slice.call(arguments, 2);
    Object.assign(e, a || {});
    kids.forEach(function (k) {
      if (k === "" || k == null || k === false) return;
      e.append(k);
    });
    return e;
  }

  function svg(t, attrs) {
    var e = document.createElementNS(SVGNS, t);
    Object.keys(attrs || {}).forEach(function (k) {
      e.setAttribute(k, attrs[k]);
    });
    Array.prototype.slice.call(arguments, 2).forEach(function (k) {
      if (k) e.append(k);
    });
    return e;
  }

  function link(path, cls, kids) {
    var a = el("a", { href: "#" + path, className: cls || "" });
    (Array.isArray(kids) ? kids : [kids]).forEach(function (k) {
      if (k != null && k !== "") a.append(k);
    });
    return a;
  }

  function enc(s) {
    return encodeURIComponent(s);
  }

  // Manifest prose (a rule's fix) marks code with backticks.
  function rich(s) {
    return String(s || "")
      .split("`")
      .map(function (part, i) {
        return i % 2 ? el("code", { textContent: part }) : part;
      });
  }

  // Custom level names (a one-level scorecard's "ten", say) borrow the colour of the
  // standard level at the same height, counted from the top.
  var ALIAS = {};
  function aliasLevels(D) {
    Object.values(D.scorecards || {}).forEach(function (sc) {
      var levels = sc.levels || [];
      levels.forEach(function (l, i) {
        if (KNOWN.indexOf(l) < 0 && !ALIAS[l]) ALIAS[l] = ["gold", "silver", "bronze"][Math.min(2, levels.length - 1 - i)];
      });
    });
  }

  function known(l) {
    return KNOWN.indexOf(l) >= 0 ? l : ALIAS[l] || null;
  }

  function fill(l) {
    return "var(--" + (known(l) || "accent") + ")";
  }

  function ink(l) {
    return known(l) ? "var(--" + known(l) + "-ink)" : "var(--accent)";
  }

  function plural(n, word, many) {
    return fmt(n) + " " + (n === 1 ? word : many || word + "s");
  }

  function fmt(n) {
    return Number(n).toLocaleString("en-US");
  }

  function levelOrder(levels) {
    return ["none"].concat(levels || [], ["skipped"]);
  }

  function tallyLevels(rows, order) {
    var tally = {};
    rows.forEach(function (pair) {
      tally[pair[1].level] = (tally[pair[1].level] || 0) + 1;
    });
    return order
      .filter(function (l) {
        return tally[l];
      })
      .map(function (l) {
        return { level: l, n: tally[l] };
      });
  }

  // Grow from zero on the next frame so a re-render reads as movement, not a jump.
  function grow(nodes, targets) {
    requestAnimationFrame(function () {
      requestAnimationFrame(function () {
        nodes.forEach(function (n, i) {
          n.style.flexGrow = targets[i];
        });
      });
    });
  }

  // Skipped entities are outside a scorecard's scope, so they take no length in a bar.
  function barEl(all, cls, label) {
    var parts = all.filter(function (p) {
      return p.level !== "skipped";
    });
    var spans = parts.map(function (p) {
      return el("span", { style: "flex-grow:0;flex-basis:0;background:" + fill(p.level), title: p.level + ": " + p.n });
    });
    grow(
      spans,
      parts.map(function (p) {
        return p.n;
      }),
    );
    var bar = el("div", { className: "bar" + (cls ? " " + cls : "") }, ...spans);
    bar.setAttribute("role", "img");
    bar.setAttribute(
      "aria-label",
      label ||
        parts
          .map(function (p) {
            return p.n + " " + p.level;
          })
          .join(", "),
    );
    return bar;
  }

  function legendEl(parts) {
    return el(
      "div",
      { className: "legend" },
      ...parts.map(function (p) {
        if (p.level === "skipped") return el("span", {}, el("b", { textContent: fmt(p.n) }), " not in scope");
        return el(
          "span",
          {},
          el("span", { className: "dot", style: "background:" + fill(p.level) }),
          el("b", { textContent: fmt(p.n) }),
          " " + p.level,
        );
      }),
    );
  }

  function badge(level) {
    var span = el("span", { className: "lvl " + level, style: "--fill:" + fill(level) + ";--ink:" + ink(level) }, el("i"), level);
    span.setAttribute("aria-label", "level " + level);
    return span;
  }

  function medal(level, size) {
    var s = size || 56;
    var none = level === "none" || !known(level);
    var c = fill(level);
    var g = svg(
      "svg",
      { width: s, height: s, viewBox: "0 0 56 56", "aria-hidden": "true" },
      svg("path", { d: "M18 4h8l-4 22h-8z", fill: none ? "var(--line)" : "var(--accent)", opacity: ".85" }),
      svg("path", { d: "M30 4h8l2 22h-8z", fill: none ? "var(--line)" : "var(--accent)", opacity: ".6" }),
      svg("circle", {
        cx: 28,
        cy: 34,
        r: 17,
        fill: none ? "var(--card)" : c,
        stroke: c,
        "stroke-width": none ? 2.5 : 0,
        "stroke-dasharray": none ? "4 3" : "",
      }),
      none ? null : svg("circle", { cx: 28, cy: 34, r: 12, fill: "none", stroke: "#fff", "stroke-opacity": ".55", "stroke-width": 1.5 }),
      none
        ? null
        : svg("path", {
            d: "M28 26.5l2.3 4.7 5.2.8-3.8 3.7.9 5.1-4.6-2.4-4.6 2.4.9-5.1-3.8-3.7 5.2-.8z",
            fill: "#fff",
            "fill-opacity": ".8",
          }),
    );
    return g;
  }

  function boxLogo() {
    return svg(
      "svg",
      { width: 34, height: 34, viewBox: "0 0 34 34", "aria-hidden": "true" },
      svg("rect", { x: 3, y: 11, width: 28, height: 19, rx: 4, fill: "var(--accent)" }),
      svg("path", { d: "M12 11V8a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v3", fill: "none", stroke: "var(--accent)", "stroke-width": 2.4 }),
      svg("rect", { x: 3, y: 17, width: 28, height: 3, fill: "var(--card)", opacity: ".35" }),
      svg("rect", { x: 14.5, y: 15.5, width: 5, height: 6, rx: 1.2, fill: "var(--gold)" }),
    );
  }

  function ring(pct, opts) {
    var o = opts || {};
    var s = o.size || 52;
    var r = s / 2 - 5;
    var circ = 2 * Math.PI * r;
    var color = o.color || "var(--accent)";
    var arc = svg("circle", {
      cx: s / 2,
      cy: s / 2,
      r: r,
      fill: "none",
      stroke: color,
      "stroke-width": 6,
      "stroke-linecap": "round",
      "stroke-dasharray": circ,
      "stroke-dashoffset": circ,
      transform: "rotate(-90 " + s / 2 + " " + s / 2 + ")",
    });
    var label = String(o.text != null ? o.text : pct == null ? "—" : pct + "%");
    var target = null;
    if (o.target != null && o.target < 100) {
      var a = (o.target / 100) * 2 * Math.PI - Math.PI / 2;
      target = svg("line", {
        x1: s / 2 + (r - 5) * Math.cos(a),
        y1: s / 2 + (r - 5) * Math.sin(a),
        x2: s / 2 + (r + 5) * Math.cos(a),
        y2: s / 2 + (r + 5) * Math.sin(a),
        stroke: "var(--fg)",
        "stroke-width": 2,
      });
    }
    var g = svg(
      "svg",
      { class: "ring", width: s, height: s, viewBox: "0 0 " + s + " " + s, role: "img", "aria-label": o.label || pct + "%" },
      svg("circle", { cx: s / 2, cy: s / 2, r: r, fill: "none", stroke: "var(--line)", "stroke-width": 6 }),
      pct == null ? null : arc,
      target,
      svg("text", { x: "50%", y: "50%", "text-anchor": "middle", "dominant-baseline": "central", style: s > 70 ? "font-size:18px" : label.length > 3 ? "font-size:11px" : "" }, label),
    );
    if (pct != null) {
      requestAnimationFrame(function () {
        requestAnimationFrame(function () {
          arc.setAttribute("stroke-dashoffset", circ * (1 - Math.max(0, Math.min(100, pct)) / 100));
        });
      });
    }
    return g;
  }

  function shortId(id) {
    var s = String(id || "");
    var slash = s.lastIndexOf("/");
    var colon = s.lastIndexOf(":");
    var i = Math.max(slash, colon);
    return i >= 0 && i < s.length - 1 ? s.slice(i + 1) : s;
  }

  function title(D, name) {
    var sc = (D.scorecards || {})[name] || {};
    return sc.title || name;
  }

  function gapText(D, g, id) {
    var m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
    if (m) {
      var sc = (D.scorecards || {})[m[1]];
      var score = ((D.scores || {})[m[1]] || {})[id];
      var need = sc && score ? blockers(sc, score, m[2]) : [];
      return title(D, m[1]) + " is " + m[3] + "; the goal is " + m[2] + "." + (need.length ? " Needs " + need.join(", ") + "." : "");
    }
    m = /^(.+?):rule (.+) \((\S+)\)$/.exec(g);
    if (m) return title(D, m[1]) + ": rule " + m[2] + " is " + m[3] + ".";
    m = /^(.+?): unknown scorecard$/.exec(g);
    if (m) return "Scorecard " + m[1] + " is unknown.";
    return g;
  }

  function cardName(D, name) {
    var t = title(D, name);
    return t === name ? [el("span", { className: "t", textContent: name })] : [el("span", { className: "t", textContent: t }), el("code", { className: "c", textContent: name })];
  }

  function daysLeft(end) {
    if (!end) return null;
    var ms = Date.parse(end + "T23:59:59Z") - Date.now();
    return isNaN(ms) ? null : Math.ceil(ms / 86400000);
  }

  function isPassing(o) {
    return o === "pass" || o === "waived";
  }

  // The rules an entity still fails at or below `level` (what stands between it and that level).
  function blockers(sc, score, level) {
    var order = sc.levels || [];
    var cap = order.indexOf(level);
    return Object.keys(score.rules || {})
      .filter(function (n) {
        var rl = ((sc.rules || {})[n] || {}).level;
        return !isPassing(score.rules[n]) && score.rules[n] !== "skipped" && order.indexOf(rl) <= cap;
      })
      .sort();
  }

  function nextLevel(sc, level) {
    var order = sc.levels || [];
    if (level === "none") return order[0] || null;
    var i = order.indexOf(level);
    return i >= 0 && i < order.length - 1 ? order[i + 1] : null;
  }

  function campaignState(c) {
    if (!c.inScope && c.measure == null) return "empty";
    return c.onTarget ? "on" : "off";
  }

  function scorecardRows(D) {
    return Object.keys(D.scorecards || {})
      .map(function (name) {
        var sc = D.scorecards[name];
        var ents = Object.entries((D.scores || {})[name] || {});
        var parts = tallyLevels(ents, levelOrder(sc.levels));
        var count = function (l) {
          var p = parts.find(function (x) {
            return x.level === l;
          });
          return p ? p.n : 0;
        };
        var top = (sc.levels || [])[(sc.levels || []).length - 1];
        var scored = ents.filter(function (p) {
          return p[1].level !== "skipped";
        }).length;
        return {
          name: name,
          sc: sc,
          parts: parts,
          none: count("none"),
          bronze: count("bronze"),
          top: top,
          topShare: scored ? Math.round((count(top) / scored) * 100) : 0,
          scored: scored,
          weight: count("none") * 1000 + count("bronze"),
        };
      })
      .sort(function (a, b) {
        return b.weight - a.weight || a.name.localeCompare(b.name);
      });
  }

  function campaignNamesSorted(D) {
    var campaigns = D.campaigns || {};
    var rank = { off: 0, on: 1, empty: 2 };
    return Object.keys(campaigns)
      .filter(function (n) {
        var st = campaigns[n].status;
        return st === "active" || st === "scheduled" || st === "ended";
      })
      .sort(function (a, b) {
        var ca = campaigns[a];
        var cb = campaigns[b];
        return rank[campaignState(ca)] - rank[campaignState(cb)] || (ca.percent || 0) - (cb.percent || 0) || a.localeCompare(b);
      });
  }

  function scoredKinds(D) {
    var set = {};
    Object.keys(D.scorecards || {}).forEach(function (name) {
      var k = D.scorecards[name].kind;
      if (k) set[k] = true;
    });
    return set;
  }

  function catalogKindsSorted(D) {
    var catalog = D.catalog || {};
    var scored = scoredKinds(D);
    return Object.keys(catalog).sort(function (a, b) {
      var as = scored[a] ? 0 : 1;
      var bs = scored[b] ? 0 : 1;
      if (as !== bs) return as - bs;
      return catalog[b].length - catalog[a].length || a.localeCompare(b);
    });
  }

  // Facts every view shares, computed once per load.
  function summarize(D) {
    var rows = scorecardRows(D);
    var byEntity = {};
    var checks = { pass: 0, fail: 0, unknown: 0 };
    rows.forEach(function (r) {
      var order = levelOrder(r.sc.levels);
      Object.entries((D.scores || {})[r.name] || {}).forEach(function (pair) {
        var id = pair[0];
        var s = pair[1];
        if (s.level === "skipped") return;
        var cur = byEntity[id];
        var rank = order.indexOf(s.level);
        var norm = ["none", "bronze", "silver", "gold"].indexOf(s.level) >= 0 ? s.level : rank <= 0 ? "none" : "gold";
        if (!cur || ["none", "bronze", "silver", "gold"].indexOf(norm) < ["none", "bronze", "silver", "gold"].indexOf(cur)) byEntity[id] = norm;
        Object.values(s.rules || {}).forEach(function (o) {
          if (isPassing(o)) checks.pass++;
          else if (o === "fail") checks.fail++;
          else if (o === "unknown") checks.unknown++;
        });
      });
    });
    var catalog = D.catalog || {};
    var entityCount = Object.keys(catalog).reduce(function (n, k) {
      return n + catalog[k].length;
    }, 0);
    var kindOf = {};
    Object.keys(catalog).forEach(function (k) {
      catalog[k].forEach(function (id) {
        kindOf[id] = k;
      });
    });
    return { rows: rows, worst: byEntity, checks: checks, entityCount: entityCount, kindOf: kindOf, repo: repoLevel(D) };
  }

  function repoLevel(D) {
    var rollup = Array.isArray(D.repoRollup) ? D.repoRollup : [];
    if (!rollup.length) return null;
    var ladder = ["none", "bronze", "silver", "gold"];
    var constituents = rollup
      .filter(function (name) {
        return (D.scorecards || {})[name];
      })
      .map(function (name) {
        var sc = D.scorecards[name];
        var order = ["none"].concat(sc.levels || []);
        var best = null;
        Object.keys((D.scores || {})[name] || {}).forEach(function (id) {
          if (id.indexOf("/demo/") >= 0) return;
          var row = D.scores[name][id];
          if (!row || row.level === "skipped") return;
          var rules = row.rules || {};
          var keys = Object.keys(rules);
          if (
            keys.length &&
            keys.every(function (k) {
              return rules[k] === "unknown";
            })
          )
            return;
          var r = order.indexOf(row.level);
          if (best == null || r < best.rank) best = { level: row.level, rank: r };
        });
        return { name: name, level: best ? best.level : "none" };
      });
    if (!constituents.length) return null;
    var floor = constituents.reduce(function (m, c) {
      return ladder.indexOf(c.level) < ladder.indexOf(m) ? c.level : m;
    }, "gold");
    var next = ladder[ladder.indexOf(floor) + 1] || null;
    return {
      ladder: ladder,
      floor: floor,
      next: next,
      constituents: constituents,
      holding: constituents.filter(function (c) {
        return c.level === floor;
      }),
    };
  }

  function healthVerdict(D, S, campaignNames) {
    var campaigns = D.campaigns || {};
    var off = campaignNames.filter(function (n) {
      return campaignState(campaigns[n]) === "off";
    });
    var debt = S.rows.filter(function (r) {
      return r.none > 0;
    });
    if (!campaignNames.length && !S.rows.length) return "No campaigns or scorecards in this snapshot.";
    if (!off.length && !debt.length) return "On track: every active campaign meets its target, and no scorecard has entities at none.";
    var bits = [];
    if (debt.length) bits.push(plural(debt.length, "scorecard") + " with entities at none");
    if (off.length) bits.push(plural(off.length, "campaign") + " behind target");
    return "Needs attention: " + bits.join(" and ") + ".";
  }

  function parseHash() {
    var h = (location.hash || "").replace(/^#/, "");
    if (!h || h === "overview") return { mode: "overview" };
    if (h === "catalog") return { mode: "catalog" };
    if (h.indexOf("catalog/") === 0) return { mode: "catalog", kind: decodeURIComponent(h.slice(8)) };
    if (h.indexOf("entity/") === 0) return { mode: "entity", id: decodeURIComponent(h.slice(7)) };
    if (h === "campaigns") return { mode: "campaigns" };
    if (h.indexOf("campaign/") === 0) return { mode: "campaign", name: decodeURIComponent(h.slice(9)) };
    if (h === "compare") return { mode: "compare" };
    if (h.indexOf("compare/") === 0) return { mode: "compare", name: decodeURIComponent(h.slice(8)) };
    if (h.indexOf("scorecard/") === 0) return { mode: "scorecard", name: decodeURIComponent(h.slice(10)) };
    if (h === "scorecards") return { mode: "scorecard" };
    return { mode: "scorecard", name: decodeURIComponent(h) };
  }

  function missingTarget(D, route) {
    if (route.mode === "scorecard" && route.name && !(D.scorecards || {})[route.name]) return "No scorecard is named “" + route.name + "”.";
    if (route.mode === "campaign" && !(D.campaigns || {})[route.name]) return "No campaign is named “" + route.name + "”.";
    if (route.mode === "compare" && route.name && !(D.comparisons || {})[route.name]) return "No comparison is named “" + route.name + "”.";
    if (route.mode === "catalog" && route.kind && !(D.catalog || {})[route.kind]) return "No kind is named “" + route.kind + "”.";
    if (route.mode === "entity") {
      var known = Object.values(D.catalog || {}).some(function (ids) {
        return ids.indexOf(route.id) >= 0;
      });
      if (!known) return "No entity has the id “" + route.id + "” in this snapshot.";
    }
    return null;
  }

  function notFound(message) {
    return el(
      "section",
      {},
      el("h2", { textContent: "Not found" }),
      el("p", { className: "lede", textContent: message + " The link may be from an older report." }),
      el("p", {}, link("overview", "", "Go to the overview"), " · ", link("catalog", "", "Browse the catalog")),
    );
  }

  class ToolboxReport extends HTMLElement {
    connectedCallback() {
      if (this._booted) return;
      this._booted = true;
      var root = this.attachShadow({ mode: "open" });
      root.append(template.content.cloneNode(true));
      var self = this;
      var inline = this.querySelector('script[type="application/json"]');
      var load = inline
        ? Promise.resolve(JSON.parse(inline.textContent))
        : fetch(this.getAttribute("src")).then(function (r) {
            if (!r.ok) throw new Error("report JSON HTTP " + r.status);
            return r.json();
          });
      load
        .then(function (data) {
          self.mount(root, data);
        })
        .catch(function (e) {
          root.getElementById("body").replaceChildren(el("section", {}, el("div", { className: "empty", textContent: "Failed to load report: " + e.message })));
        });
    }

    mount(root, D) {
      var self = this;
      aliasLevels(D);
      var S = summarize(D);
      var catalog = D.catalog || {};
      root.getElementById("top").replaceChildren(boxLogo(), el("h1", { textContent: D.portal || "Toolbox report" }));
      root.getElementById("meta").textContent =
        fmt(S.entityCount) +
        " entities in the box · " +
        plural(Object.keys(D.scorecards || {}).length, "scorecard") +
        " · snapshot " +
        D.snapshot +
        " · generated by toolbox report";
      var draw = function () {
        self.render(root, D, S);
      };
      window.addEventListener("hashchange", draw);
      draw();
    }

    render(root, D, S) {
      var route = parseHash();
      var modes = root.getElementById("modes");
      var body = root.getElementById("body");
      var self = this;
      var go = function (path) {
        var next = "#" + path;
        if (location.hash === next) self.render(root, D, S);
        else location.hash = path;
      };
      var modeOf = function (r) {
        if (r.mode === "entity") return "catalog";
        if (r.mode === "campaign") return "campaigns";
        return r.mode === "scorecard" ? "scorecards" : r.mode;
      };
      var activeMode = modeOf(route);
      var campaigns = campaignNamesSorted(D);
      var tabs = [
        { label: "Overview", path: "overview", id: "overview" },
        { label: "Catalog", path: "catalog", id: "catalog", count: S.entityCount },
        { label: "Campaigns", path: "campaigns", id: "campaigns", count: campaigns.length },
        {
          label: "Scorecards",
          id: "scorecards",
          count: S.rows.length,
          path: S.rows[0] ? "scorecard/" + enc(S.rows[0].name) : "overview",
        },
      ];
      var comparisons = Object.keys(D.comparisons || {});
      if (comparisons.length) tabs.push({ label: "Compare", path: "compare", id: "compare", count: comparisons.length });
      modes.replaceChildren(
        ...tabs.map(function (t) {
          var selected = activeMode === t.id;
          return el(
            "button",
            {
              role: "tab",
              id: "tab-" + t.id,
              ariaSelected: String(selected),
              tabIndex: selected ? 0 : -1,
              onclick: function () {
                go(t.path);
              },
            },
            t.label,
            t.count != null ? el("span", { className: "count", textContent: fmt(t.count) }) : null,
          );
        }),
      );
      modes.onkeydown = function (ev) {
        var keys = { ArrowLeft: -1, ArrowRight: 1, Home: "home", End: "end" };
        var dir = keys[ev.key];
        if (dir == null) return;
        ev.preventDefault();
        var buttons = Array.prototype.slice.call(modes.querySelectorAll('[role="tab"]'));
        var i = buttons.indexOf(root.activeElement);
        if (i < 0)
          i = buttons.findIndex(function (b) {
            return b.getAttribute("aria-selected") === "true";
          });
        var next = dir === "home" ? 0 : dir === "end" ? buttons.length - 1 : (i + dir + buttons.length) % buttons.length;
        buttons[next].focus();
        buttons[next].click();
      };
      var view;
      var missing = missingTarget(D, route);
      if (missing) view = notFound(missing);
      else if (route.mode === "overview") view = this.overview(D, S);
      else if (route.mode === "catalog") view = this.catalog(D, S, route.kind);
      else if (route.mode === "entity") view = this.entity(D, S, route.id);
      else if (route.mode === "campaigns" || route.mode === "campaign") view = this.campaigns(D, route.name);
      else if (route.mode === "compare") view = this.compare(D, route.name);
      else view = this.scorecard(D, S, route.name);
      body.replaceChildren(view);
      if (route.mode !== "overview" && this._lastMode !== undefined && this._lastRoute !== location.hash) {
        root.host.scrollIntoView({ block: "start" });
      }
      this._lastMode = route.mode;
      this._lastRoute = location.hash;
    }

    hero(D, S, campaignNames) {
      var R = S.repo;
      var verdict = healthVerdict(D, S, campaignNames);
      if (!R) {
        return el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Fleet health" })), el("p", { className: "verdict", textContent: verdict }));
      }
      var level = R.floor;
      var section = el("section", { className: "hero", style: "--lvl:" + fill(level) + ";--lvl-ink:" + ink(level) });
      section.setAttribute("aria-label", "Fleet health");
      var holdingNames = R.holding.map(function (c) {
        return title(D, c.name);
      });
      var sentence =
        (D.portal || "This repository") +
        " is at " +
        level +
        ": the lowest level among " +
        plural(R.constituents.length, "repository standard") +
        ". " +
        (R.next ? plural(R.holding.length, "standard holds", "standards hold") + " it there." : "Every standard is at the top level.");
      var nextBox = R.next
        ? el(
            "div",
            { className: "next" },
            el("b", { textContent: "Next: " }),
            "reach ",
            badge(R.next),
            " by lifting " + holdingNames.join(", ") + " to " + R.next + ".",
          )
        : el("div", { className: "next" }, el("b", { textContent: "Top of the ladder. " }), "Every repository standard is at its highest level.");
      var left = el(
        "div",
        {},
        el(
          "div",
          { className: "badge-big" },
          medal(level, 72),
          el("div", {}, el("div", { className: "kicker", textContent: "Repository level" }), el("h2", { className: "levelword", textContent: level })),
        ),
        el("p", { className: "verdict", textContent: sentence }),
        nextBox,
        el("p", { className: "sub", style: "margin:12px 0 0", textContent: verdict }),
      );
      var ladder = el("div", { className: "ladder", style: "--steps:" + R.ladder.length });
      ladder.setAttribute("aria-label", "Repository standards by lowest level");
      R.ladder.forEach(function (l) {
        var here = R.constituents.filter(function (c) {
          return c.level === l;
        });
        var rung = el(
          "div",
          { className: "rung" + (l === level ? " here" : ""), style: "--lvl:" + fill(l) },
          el("div", { className: "rh" }, medal(l, 22), l, el("span", { className: "n", textContent: String(here.length) })),
        );
        if (!here.length) rung.append(l === "none" ? el("div", { className: "empty good", textContent: "✓ No standard at none" }) : el("div", { className: "empty", textContent: "—" }));
        here.forEach(function (c) {
          var a = link("scorecard/" + enc(c.name), "", title(D, c.name));
          a.title = c.name;
          rung.append(a);
        });
        ladder.append(rung);
      });
      section.append(
        left,
        el(
          "div",
          {},
          el("h3", { style: "margin-top:0", textContent: "Where each repository standard stands" }),
          ladder,
          el("p", { className: "sub", style: "margin:10px 0 0", textContent: "Only these standards set the repository level. The other scorecards rate single entities, such as one rule or one skill, and are listed below." }),
        ),
      );
      return section;
    }

    tiles(D, S, campaignNames) {
      var campaigns = D.campaigns || {};
      var c = S.checks;
      var total = c.pass + c.fail + c.unknown;
      var pct = total ? Math.round((c.pass / total) * 100) : 0;
      var on = campaignNames.filter(function (n) {
        return campaignState(campaigns[n]) === "on";
      }).length;
      var debt = S.rows.filter(function (r) {
        return r.none > 0;
      }).length;
      var scoredEntities = Object.keys(S.worst).length;
      var golden = Object.keys(S.worst).filter(function (id) {
        return S.worst[id] === "gold";
      }).length;
      var tile = function (path, cls, lead, big, text) {
        return link(path, "tile " + (cls || ""), [lead, el("div", {}, el("strong", { textContent: big }), el("span", { textContent: text }))]);
      };
      return el(
        "div",
        { className: "tiles" },
        tile("scorecards", "good", ring(pct, { size: 46, color: "var(--pass)", text: "", label: pct + "% of checks pass" }), pct + "%", fmt(c.pass) + " of " + fmt(total) + " checks pass" + (c.unknown ? " (" + fmt(c.unknown) + " unknown)" : "")),
        tile("catalog", "", medal("gold", 40), fmt(golden), "of " + fmt(scoredEntities) + " scored entities are gold on every scorecard"),
        tile("scorecards", debt ? "bad" : "good", medal(debt ? "none" : "gold", 40), String(debt), debt ? "scorecards have entities at none" : "scorecards with entities at none"),
        tile("campaigns", on === campaignNames.length ? "good" : "", ring(campaignNames.length ? Math.round((on / campaignNames.length) * 100) : null, { size: 46, color: "var(--pass)", text: "", label: on + " of " + campaignNames.length + " campaigns on target" }), on + " of " + campaignNames.length, "campaigns on target"),
      );
    }

    campaignCard(D, name, current) {
      var c = (D.campaigns || {})[name];
      var state = campaignState(c);
      var pct = c.percent == null ? 0 : c.percent;
      var target = c.targetPercent == null ? 100 : c.targetPercent;
      var days = daysLeft(c.end);
      var color = state === "on" ? "var(--pass)" : state === "empty" ? "var(--faint)" : "var(--accent)";
      var status =
        state === "empty"
          ? el("span", { className: "pill", textContent: "scope is empty" })
          : state === "on"
            ? el("span", { className: "pill good", textContent: "on target" })
            : el("span", { className: "pill bad", textContent: (c.inScope || 0) - (c.meeting || 0) + " to go" });
      var a = link("campaign/" + enc(name), "card", [
        ring(state === "empty" ? null : pct, { color: color, target: target, label: state === "empty" ? "no entities in scope" : pct + "% of " + target + "% target" }),
        el(
          "div",
          {},
          el("div", { className: "t", textContent: c.title || name }),
          el("div", { className: "s" }, status, days != null && days >= 0 && c.status !== "ended" ? " · " + plural(days, "day") + " left" : c.status === "ended" ? " · ended" : ""),
        ),
      ]);
      if (current) a.setAttribute("aria-current", "true");
      return a;
    }

    scorecardTable(D, rows) {
      var wrap = el("div", { className: "rows" });
      wrap.append(el("div", { className: "rows-head" }, el("span", { textContent: "Standard" }), el("span", { className: "dist", textContent: "Entities by level" }), el("span", { textContent: "At none" })));
      rows.forEach(function (r) {
        wrap.append(
          link("scorecard/" + enc(r.name), "row", [
            el("div", {}, ...cardName(D, r.name)),
            el("div", { className: "dist" }, barEl(r.parts, "", r.parts.map(function (p) { return p.n + " " + p.level; }).join(", ")), el("div", { className: "sub", style: "font-size:12px;margin-top:4px", textContent: r.scored + " " + (r.sc.kind || "entities") + " · " + r.topShare + "% " + (r.top || "top") })),
            el("div", { className: "num " + (r.none ? "bad" : "ok") }, el("strong", { textContent: String(r.none) }), el("span", { textContent: r.none ? "at none" : "all clear" })),
          ]),
        );
      });
      return wrap;
    }

    overview(D, S) {
      var catalog = D.catalog || {};
      var campaignNames = campaignNamesSorted(D);
      var self = this;
      var attention = S.rows.filter(function (r) {
        return r.none > 0;
      });
      var clear = S.rows.filter(function (r) {
        return r.none === 0;
      });
      var showAll = false;
      var clearWrap = el("div");
      var drawClear = function () {
        clearWrap.replaceChildren(
          showAll
            ? self.scorecardTable(D, clear)
            : el(
                "p",
                { className: "empty good" },
                "✓ " + plural(clear.length, "scorecard has", "scorecards have") + " no entity at none. ",
                el("button", {
                  className: "link",
                  textContent: "Show them",
                  onclick: function () {
                    showAll = true;
                    drawClear();
                  },
                }),
              ),
        );
      };
      drawClear();
      var legendParts = ["none", "bronze", "silver", "gold"].map(function (l) {
        return el("span", {}, el("span", { className: "dot", style: "background:" + fill(l) }), l);
      });
      var kinds = catalogKindsSorted(D);
      return el(
        "div",
        {},
        this.hero(D, S, campaignNames),
        this.tiles(D, S, campaignNames),
        campaignNames.length
          ? el(
              "section",
              {},
              el("div", { className: "head" }, el("h2", { textContent: "Campaigns" }), el("span", { className: "sub", textContent: "Time-bound goals · behind target first" })),
              el(
                "div",
                { className: "cards" },
                ...campaignNames.map(function (n) {
                  return self.campaignCard(D, n);
                }),
              ),
            )
          : null,
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: attention.length ? "Scorecards that need attention" : "Scorecards" }),
            el("span", { className: "sub", textContent: "Needs attention first" }),
          ),
          el("p", { className: "lede", textContent: "Each scorecard rates every entity of one kind against levels of rules. An entity is at none until it passes every rule of the first level." }),
          el("div", { className: "legend", style: "margin-bottom:8px" }, ...legendParts),
          attention.length ? this.scorecardTable(D, attention) : null,
          clearWrap,
        ),
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "What's in the box" }), link("catalog", "", "Browse the catalog →")),
          el(
            "div",
            { className: "box" },
            ...kinds.map(function (k) {
              return link("catalog/" + enc(k), "", [k, el("b", { textContent: fmt(catalog[k].length) })]);
            }),
          ),
        ),
      );
    }

    campaigns(D, name) {
      var all = D.campaigns || {};
      var names = campaignNamesSorted(D);
      if (!names.length) names = Object.keys(all).sort();
      if (!names.length) return el("section", {}, el("div", { className: "empty", textContent: "No campaigns in this portal." }));
      var self = this;
      var selected = name && all[name] ? name : names[0];
      var c = all[selected];
      var state = campaignState(c);
      var pct = c.percent == null ? 0 : c.percent;
      var target = c.targetPercent == null ? 100 : c.targetPercent;
      var days = daysLeft(c.end);
      var includes = el("ul", { style: "margin:4px 0 0;padding-left:18px" });
      (c.includes || []).forEach(function (i) {
        includes.append(
          el(
            "li",
            {},
            link("scorecard/" + enc(i.scorecard), "", title(D, i.scorecard)),
            i.atLeast ? el("span", {}, " at ", badge(i.atLeast), " or above") : "",
            i.rules && i.rules.length ? el("span", { className: "sub", textContent: " · rules " + i.rules.join(", ") }) : "",
          ),
        );
      });
      var bar = el("div", { className: "bar lg" });
      var fillSpan = el("span", { style: "flex-grow:0;flex-basis:0;background:" + (state === "on" ? "var(--pass)" : "var(--accent)") });
      bar.append(fillSpan, el("span", { style: "flex-grow:" + Math.max(0, 100 - pct) + ";background:transparent" }));
      grow([fillSpan], [pct]);
      var marks = el("div", { className: "milestones" });
      (c.milestones || []).forEach(function (m) {
        marks.append(el("span", { className: m.met ? "met" : "", style: "left:" + m.targetPercent + "%", textContent: (m.met ? "✓ " : "") + m.name + " " + m.targetPercent + "%" }));
      });
      if (target < 100) marks.append(el("span", { style: "left:" + target + "%", textContent: "target " + target + "%" }));
      var gaps = Object.entries(c.entities || {})
        .filter(function (pair) {
          return !pair[1].meeting;
        })
        .sort(function (a, b) {
          return a[0].localeCompare(b[0]);
        });
      var gapList = el("div");
      if (state === "empty") {
        gapList.append(
          el(
            "div",
            { className: "empty" },
            "No entity is in scope, so progress can't be measured yet. A campaign's scope is the entities scored by every included scorecard; check that they score the same kind.",
          ),
        );
      } else if (!gaps.length) {
        gapList.append(el("div", { className: "empty good", textContent: "✓ Every in-scope entity meets the goal." }));
      } else {
        var needCounts = {};
        var needFix = {};
        gaps.forEach(function (pair) {
          (pair[1].gaps || []).forEach(function (g) {
            var m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
            var gsc = m && (D.scorecards || {})[m[1]];
            var score = m && ((D.scores || {})[m[1]] || {})[pair[0]];
            if (gsc && score)
              blockers(gsc, score, m[2]).forEach(function (r) {
                needCounts[r] = (needCounts[r] || 0) + 1;
                needFix[r] = ((gsc.rules || {})[r] || {}).fix || "";
              });
          });
        });
        var common = Object.entries(needCounts).sort(function (a, b) {
          return b[1] - a[1] || a[0].localeCompare(b[0]);
        })[0];
        if (common)
          gapList.append(
            el(
              "div",
              { className: "next", style: "margin:0 0 10px" },
              el("b", { textContent: "Biggest lever: " }),
              common[0] + " blocks " + common[1] + " of " + gaps.length + (gaps.length === 1 ? " entity" : " entities") + ". ",
              ...rich(needFix[common[0]]),
            ),
          );
        gaps.forEach(function (pair) {
          gapList.append(
            el(
              "div",
              { className: "gap" },
              el("div", {}, link("entity/" + enc(pair[0]), "", shortId(pair[0])), el("div", { className: "sub", style: "font-size:12px", textContent: pair[0] })),
              el("div", { className: "why", textContent: (pair[1].gaps || []).map(function (g) { return gapText(D, g, pair[0]); }).join("; ") }),
            ),
          );
        });
      }
      var track = c.track && D.tracks && D.tracks[c.track] ? D.tracks[c.track].title || c.track : c.track;
      var picker = el(
        "select",
        {
          ariaLabel: "Campaign",
          onchange: function () {
            location.hash = "campaign/" + enc(picker.value);
          },
        },
        ...names.map(function (n) {
          var cn = all[n];
          return el("option", { value: n, selected: n === selected, textContent: (cn.title || n) + " (" + (campaignState(cn) === "empty" ? "scope is empty" : (cn.percent || 0) + "%") + ")" });
        }),
      );
      return el(
        "div",
        { className: "split" },
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Campaigns" })),
          el("p", { className: "sub", style: "margin-top:0", textContent: "Behind target first" }),
          el(
            "div",
            { className: "cards", style: "grid-template-columns:1fr" },
            ...names.map(function (n) {
              return self.campaignCard(D, n, n === selected);
            }),
          ),
        ),
        el(
          "section",
          {},
          el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Campaign" }), picker),
          el(
            "div",
            { className: "ehead" },
            el("div", {}, el("h2", { textContent: c.title || selected }), el("div", { className: "sub", textContent: [c.status, track ? "track: " + track : null, c.owner ? "owned by " + c.owner.replace(/^group:/, "") : null].filter(Boolean).join(" · ") })),
            ring(state === "empty" ? null : pct, { size: 84, color: state === "on" ? "var(--pass)" : "var(--accent)", target: target, label: state === "empty" ? "no entities in scope" : pct + "% of " + target + "% target" }),
          ),
          c.description ? el("p", { className: "lede", textContent: c.description }) : null,
          state === "empty"
            ? null
            : el(
                "p",
                { style: "margin:8px 0 6px" },
                el("b", { textContent: (c.meeting || 0) + " of " + (c.inScope || 0) }),
                " entities meet the goal · target " + target + "%" + (c.end ? " by " + c.end : "") + (days != null && days >= 0 ? " (" + plural(days, "day") + " left)" : ""),
              ),
          state === "empty" ? null : bar,
          state === "empty" ? null : marks,
          el("h3", { textContent: "Goal" }),
          includes,
          el("h3", { textContent: state === "empty" ? "Scope" : "Not meeting yet" + (gaps.length ? " (" + gaps.length + ")" : "") }),
          gapList,
        ),
      );
    }

    catalog(D, S, kind) {
      var catalog = D.catalog || {};
      var kinds = catalogKindsSorted(D);
      var selected = kind && catalog[kind] ? kind : kinds[0];
      if (!selected) return el("section", {}, el("div", { className: "empty", textContent: "Catalog is empty in this report." }));
      var scored = scoredKinds(D);
      var self = this;
      var grid = el(
        "div",
        { className: "kinds" },
        ...kinds.map(function (k) {
          var ids = catalog[k];
          var a = link("catalog/" + enc(k), "kind", [el("strong", { textContent: k }), el("span", { textContent: plural(ids.length, "entity", "entities") + (scored[k] ? "" : " · not scored") })]);
          if (scored[k]) {
            var parts = tallyLevels(
              ids
                .filter(function (id) {
                  return S.worst[id];
                })
                .map(function (id) {
                  return [id, { level: S.worst[id] }];
                }),
              ["none", "bronze", "silver", "gold"],
            );
            if (parts.length) a.append(barEl(parts, "xs"));
          }
          if (k === selected) a.setAttribute("aria-current", "true");
          return a;
        }),
      );
      var search = el("input", { type: "search", placeholder: "Filter " + selected + " by name…", ariaLabel: "Filter entities" });
      var list = el("div");
      var limit = 60;
      var cardsFor = function (id) {
        return Object.keys(D.scorecards || {}).filter(function (n) {
          return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
        });
      };
      var drawList = function () {
        var q = search.value.toLowerCase();
        var ids = (catalog[selected] || []).filter(function (id) {
          return !q || id.toLowerCase().indexOf(q) >= 0;
        });
        if (!ids.length) {
          list.replaceChildren(
            el("div", { className: "empty", textContent: "No entities match “" + search.value + "”." }),
            el("button", {
              className: "link",
              textContent: "Clear filter",
              onclick: function () {
                search.value = "";
                drawList();
              },
            }),
          );
          return;
        }
        var shown = ids.slice(0, limit);
        list.replaceChildren.apply(
          list,
          shown.map(function (id) {
            var cards = cardsFor(id);
            var short = shortId(id);
            return link("entity/" + enc(id), "erow", [
              el("div", { className: "name" }, short, short !== id ? el("small", { textContent: id }) : null),
              el(
                "div",
                { className: "badges" },
                ...cards.slice(0, 4).map(function (n) {
                  var b = badge(D.scores[n][id].level);
                  b.title = title(D, n);
                  return b;
                }),
                cards.length > 4 ? el("span", { className: "sub", textContent: "+" + (cards.length - 4) }) : null,
              ),
            ]);
          }),
        );
        if (ids.length > limit) {
          list.append(
            el(
              "div",
              { className: "more" },
              el("button", {
                className: "link",
                textContent: "Show all " + fmt(ids.length) + " (" + fmt(ids.length - limit) + " more)",
                onclick: function () {
                  limit = Infinity;
                  drawList();
                },
              }),
            ),
          );
        }
      };
      search.oninput = drawList;
      drawList();
      var levels = tallyLevels(
        (catalog[selected] || [])
          .filter(function (id) {
            return S.worst[id];
          })
          .map(function (id) {
            return [id, { level: S.worst[id] }];
          }),
        ["none", "bronze", "silver", "gold"],
      );
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Catalog" }), el("span", { className: "sub", textContent: fmt(S.entityCount) + " entities in " + kinds.length + " kinds · scored kinds first" })),
          el("p", { className: "lede", textContent: "Everything the portal derived from this repository. The bar under a kind shows each entity's lowest level across the scorecards that rate it." }),
          grid,
        ),
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: selected }), el("span", { className: "sub", textContent: plural((catalog[selected] || []).length, "entity", "entities") })),
          levels.length ? el("div", {}, barEl(levels), legendEl(levels)) : el("p", { className: "sub", textContent: "No scorecard rates this kind." }),
          search,
          list,
        ),
      );
    }

    rulesFor(sc, score) {
      var wrap = el("div");
      var rules = Object.entries(score.rules || {});
      var rank = function (o) {
        return o === "fail" ? 0 : o === "unknown" ? 1 : o === "waived" ? 2 : o === "skipped" ? 3 : 4;
      };
      var order = sc.levels || [];
      rules.sort(function (a, b) {
        var la = order.indexOf(((sc.rules || {})[a[0]] || {}).level);
        var lb = order.indexOf(((sc.rules || {})[b[0]] || {}).level);
        return rank(a[1]) - rank(b[1]) || la - lb || a[0].localeCompare(b[0]);
      });
      var passed = {};
      var npass = 0;
      rules.forEach(function (ro) {
        var n = ro[0];
        var o = ro[1];
        var rule = (sc.rules && sc.rules[n]) || {};
        if (o === "pass") {
          var lv = rule.level || "";
          (passed[lv] = passed[lv] || []).push(el("span", { title: rule.fix || rule.check || "", textContent: n }));
          npass++;
          return;
        }
        var ev = (score.evidence || {})[n];
        var items = ev == null ? [] : Array.isArray(ev) ? ev : [ev];
        wrap.append(
          el(
            "div",
            { className: "rule " + o },
            el("div", {}, el("span", { className: "rn", textContent: n }), el("span", { className: "outcome " + o, textContent: o }), rule.level ? el("span", { className: "sub", textContent: " · needed for " + rule.level }) : ""),
            rule.fix ? el("div", { className: "fix" }, el("b", { textContent: "Fix: " }), ...rich(rule.fix)) : "",
            items.length
              ? el(
                  "ul",
                  {},
                  ...items.slice(0, 20).map(function (i) {
                    return el("li", {}, el("code", { textContent: typeof i === "string" ? i : JSON.stringify(i) }));
                  }),
                  items.length > 20 ? el("li", { textContent: "… and " + (items.length - 20) + " more" }) : null,
                )
              : "",
          ),
        );
      });
      if (npass) {
        wrap.append(el("div", { className: "sub", style: "margin-top:10px", textContent: plural(npass, "check passes", "checks pass") }));
        order.concat([""]).forEach(function (lv) {
          if (!passed[lv]) return;
          var all = Object.keys(sc.rules || {}).filter(function (n) {
            return (sc.rules[n].level || "") === lv && score.rules[n] !== "skipped";
          }).length;
          wrap.append(
            el(
              "div",
              { className: "passes" },
              lv ? badge(lv) : null,
              el("em", { textContent: passed[lv].length === all ? "all " + all + " met" : passed[lv].length + " of " + all }),
              ...passed[lv],
            ),
          );
        });
      }
      return wrap;
    }

    nextStep(sc, score) {
      var next = nextLevel(sc, score.level);
      if (!next) return el("div", { className: "callout done" }, "✓ Top level reached: every rule passes.");
      var need = blockers(sc, score, next);
      if (!need.length) return null;
      return el("div", { className: "callout" }, el("span", {}, el("b", { textContent: "Next: " }), badge(next), " needs " + need.join(", ") + "."));
    }

    entity(D, S, id) {
      var kind = S.kindOf[id];
      var self = this;
      var cards = Object.keys(D.scorecards || {}).filter(function (name) {
        return D.scores[name] && D.scores[name][id];
      });
      var total = { pass: 0, all: 0 };
      cards.forEach(function (n) {
        Object.values(D.scores[n][id].rules || {}).forEach(function (o) {
          if (o === "skipped") return;
          total.all++;
          if (isPassing(o)) total.pass++;
        });
      });
      var pct = total.all ? Math.round((total.pass / total.all) * 100) : null;
      var blocks = cards.map(function (name) {
        var sc = D.scorecards[name];
        var score = D.scores[name][id];
        return el(
          "div",
          { className: "sc-block" },
          el("div", { className: "head" }, el("div", {}, link("scorecard/" + enc(name), "", title(D, name)), " ", badge(score.level)), el("span", { className: "sub", textContent: name })),
          score.level === "skipped" ? el("div", { className: "sub", textContent: "Not in this scorecard's scope." }) : self.nextStep(sc, score),
          self.rulesFor(sc, score),
        );
      });
      var short = shortId(id);
      return el(
        "section",
        {},
        el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("catalog", "", "Catalog"), kind ? el("span", {}, " / ", link("catalog/" + enc(kind), "", kind)) : "", " / " + short),
        el(
          "div",
          { className: "ehead" },
          el("div", {}, el("h2", { textContent: short }), short !== id ? el("code", { textContent: id }) : null, el("div", { className: "sub", textContent: (kind || "unknown kind") + " · " + plural(cards.length, "scorecard") })),
          pct == null
            ? null
            : el(
                "div",
                { className: "ering" },
                ring(pct, { size: 76, color: pct === 100 ? "var(--pass)" : "var(--accent)", label: total.pass + " of " + total.all + " checks pass" }),
                el("span", { className: "sub", textContent: total.pass + " of " + total.all + " checks" }),
              ),
        ),
        cards.length ? el("div", {}, ...blocks) : el("div", { className: "empty", textContent: "Not scored by any scorecard in this snapshot." }),
      );
    }

    compare(D, name) {
      var all = D.comparisons || {};
      var names = Object.keys(all).sort();
      if (!names.length) return el("section", {}, el("div", { className: "empty", textContent: "No comparative scorecards in this portal." }));
      var who = function (c, id) {
        return (c.titles || {})[id] || shortId(id);
      };
      var standing = function (s) {
        return el("span", { className: "standing " + (s || "unknown"), textContent: s || "unknown" });
      };
      var standings = ["ahead", "parity", "behind", "unknown"];
      var count = function (c, peer, s) {
        return Object.keys(c.rules || {}).filter(function (r) {
          return (c.rules[r] || {})[peer] === s;
        }).length;
      };
      var th = function (text, cls) {
        return el("th", { textContent: text, className: cls || "" });
      };
      var first = all[names[0]];
      var summary = el(
        "table",
        { className: "cmp" },
        el("thead", {}, el("tr", {}, th("Factor"), th("Against"), th("Standing"), ...standings.map(function (s) {
          return th(s, "hide-sm");
        }))),
        el(
          "tbody",
          {},
          ...names.flatMap(function (n) {
            var c = all[n];
            var def = (D.scorecards || {})[n] || {};
            return (c.peers || []).map(function (peer) {
              return el(
                "tr",
                {},
                el(
                  "td",
                  {},
                  link("compare/" + enc(n), n === name ? "here" : "", def.title || n),
                  def.title ? el("span", { className: "value", textContent: n }) : null,
                ),
                el("td", { textContent: who(c, peer) }),
                el("td", {}, standing((c.verdict || {})[peer])),
                ...standings.map(function (s) {
                  return el("td", { className: "n hide-sm", textContent: fmt(count(c, peer, s)) });
                }),
              );
            });
          }),
        ),
      );
      var head = el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "Where " + who(first, first.subject) + " stands, factor by factor" })),
        el("p", {
          className: "lede",
          textContent:
            "Each factor is a comparative scorecard: the same rules score every product, and a rule's measured value decides when both sides have one. Behind on any rule is behind on the factor.",
        }),
        el("div", { className: "tablewrap" }, summary),
      );
      if (!name) return el("div", {}, head);
      var c = all[name];
      var def = (D.scorecards || {})[name] || {};
      var ids = [c.subject].concat(c.peers || []);
      var cell = function (r, rd, id) {
        var s = (c.scores || {})[id];
        var outcome = s && s.rules ? s.rules[r] || "unknown" : "unknown";
        var v = s && s.values ? s.values[r] : undefined;
        return el(
          "td",
          {},
          outcome === "pass" ? el("span", { className: "standing ahead", textContent: "pass" }) : el("span", { className: "outcome " + outcome, textContent: outcome }),
          v == null ? null : el("span", { className: "value", textContent: v.toLocaleString("en-US", { maximumFractionDigits: 2 }) + (rd.unit ? " " + rd.unit : "") }),
        );
      };
      var detail = el(
        "table",
        { className: "cmp" },
        el("thead", {}, el("tr", {}, th("Rule"), ...ids.map(function (id) {
          return th(who(c, id));
        }), ...(c.peers || []).map(function (p) {
          return th("vs " + who(c, p));
        }))),
        el(
          "tbody",
          {},
          ...Object.keys(def.rules || {}).map(function (r) {
            var rd = def.rules[r];
            return el(
              "tr",
              {},
              el("td", {}, el("strong", { textContent: r }), el("span", { className: "value" }, el("code", { textContent: rd.check }), rd.better ? " · " + rd.better + " is better" : "")),
              ...ids.map(function (id) {
                return cell(r, rd, id);
              }),
              ...(c.peers || []).map(function (p) {
                return el("td", {}, standing((c.rules[r] || {})[p]));
              }),
            );
          }),
        ),
      );
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el("div", { className: "crumb" }, link("compare", "", "Compare"), " / ", def.title || name),
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: def.title || name }),
            el("span", { className: "sub" }, ...(c.peers || []).map(function (p) {
              return el("span", {}, "vs " + who(c, p) + " ", standing((c.verdict || {})[p]), " ");
            })),
          ),
          def.description ? el("p", { className: "lede", textContent: def.description }) : null,
          el("div", { className: "tablewrap" }, detail),
        ),
        head,
      );
    }

    scorecard(D, S, name) {
      var ranked = S.rows;
      var active = name && D.scorecards[name] ? name : ranked[0] && ranked[0].name;
      if (!active) return el("section", {}, el("div", { className: "empty", textContent: "No scorecards in this report." }));
      var self = this;
      var sc = D.scorecards[active];
      var rows = Object.entries(D.scores[active] || {});
      var order = levelOrder(sc.levels);
      var parts = tallyLevels(rows, order);

      var cardSearch = el("input", { type: "search", placeholder: "Find a scorecard by name or id…", ariaLabel: "Find a scorecard" });
      var cardNav = el("nav", { className: "rank", ariaLabel: "Scorecards, most debt first" });
      var drawCards = function () {
        var q = cardSearch.value.toLowerCase();
        var shown = ranked.filter(function (r) {
          return !q || r.name.toLowerCase().indexOf(q) >= 0 || title(D, r.name).toLowerCase().indexOf(q) >= 0 || (r.sc.kind || "").toLowerCase().indexOf(q) >= 0;
        });
        if (!shown.length) {
          cardNav.replaceChildren(
            el("div", { className: "empty", textContent: "No scorecards match." }),
            el("button", {
              className: "link",
              textContent: "Clear search",
              onclick: function () {
                cardSearch.value = "";
                drawCards();
              },
            }),
          );
          return;
        }
        cardNav.replaceChildren.apply(
          cardNav,
          shown.map(function (r) {
            var debt = r.none > 0 ? r.none + " at none" : r.bronze > 0 ? r.bronze + " at bronze" : "✓ clear";
            var a = link("scorecard/" + enc(r.name), "", [el("span", { textContent: title(D, r.name) }), el("span", { className: "debt " + (r.none > 0 ? "bad" : r.bronze > 0 ? "" : "ok"), textContent: debt })]);
            a.title = r.name + " · " + r.sc.kind;
            if (r.name === active) a.setAttribute("aria-current", "true");
            return a;
          }),
        );
      };
      cardSearch.oninput = drawCards;
      drawCards();
      requestAnimationFrame(function () {
        var cur = cardNav.querySelector("[aria-current]");
        if (cur && cardNav.scrollHeight > cardNav.clientHeight) cardNav.scrollTop = cur.offsetTop - cardNav.clientHeight / 3;
      });

      var levelFilter = "all";
      var outcomeFilter = "all";
      var search = el("input", { type: "search", placeholder: "Filter by entity or rule…", ariaLabel: "Filter entities" });
      var filters = el("div", { className: "filters" });
      var list = el("div");
      var limit = 80;
      var seg = function (label, options, current, set) {
        var g = el(
          "div",
          { className: "seg" },
          ...options.map(function (o) {
            return el("button", {
              textContent: o.label,
              ariaPressed: String(current === o.value),
              onclick: function () {
                set(o.value);
                redrawFilters();
                draw();
              },
            });
          }),
        );
        g.setAttribute("role", "group");
        g.setAttribute("aria-label", label);
        return g;
      };
      var clearFilters = function () {
        levelFilter = "all";
        outcomeFilter = "all";
        search.value = "";
        redrawFilters();
        draw();
      };
      var redrawFilters = function () {
        var present = order.filter(function (l) {
          return parts.some(function (p) {
            return p.level === l;
          });
        });
        filters.replaceChildren(
          seg(
            "Level",
            [{ label: "All levels", value: "all" }].concat(
              present.map(function (l) {
                return { label: l, value: l };
              }),
            ),
            levelFilter,
            function (v) {
              levelFilter = v;
            },
          ),
          seg(
            "Outcome",
            [
              { label: "All outcomes", value: "all" },
              { label: "Failing", value: "failing" },
              { label: "Passing", value: "passing" },
            ],
            outcomeFilter,
            function (v) {
              outcomeFilter = v;
            },
          ),
        );
      };
      var draw = function () {
        var q = search.value.toLowerCase();
        var shown = rows
          .filter(function (pair) {
            return pair[1].level !== "skipped" || levelFilter === "skipped";
          })
          .filter(function (pair) {
            return levelFilter === "all" || pair[1].level === levelFilter;
          })
          .filter(function (pair) {
            if (!q) return true;
            if (pair[0].toLowerCase().indexOf(q) >= 0) return true;
            return Object.keys(pair[1].rules || {}).some(function (rn) {
              return rn.toLowerCase().indexOf(q) >= 0;
            });
          })
          .filter(function (pair) {
            if (outcomeFilter === "all") return true;
            var outs = Object.values(pair[1].rules || {});
            return outcomeFilter === "failing"
              ? outs.some(function (o) {
                  return !isPassing(o) && o !== "skipped";
                })
              : outs.every(function (o) {
                  return isPassing(o) || o === "skipped";
                });
          })
          .sort(function (a, b) {
            return order.indexOf(a[1].level) - order.indexOf(b[1].level) || a[0].localeCompare(b[0]);
          });
        if (!shown.length) {
          list.replaceChildren(
            el("div", { className: "empty", textContent: "No entities match these filters." }),
            el("button", { className: "link", textContent: "Clear filters", onclick: clearFilters }),
          );
          return;
        }
        list.replaceChildren(el("div", { className: "sub", style: "margin:6px 0", textContent: plural(shown.length, "entity", "entities") + ", lowest level first" }));
        shown.slice(0, limit).forEach(function (pair) {
          var id = pair[0];
          var r = pair[1];
          var outs = Object.entries(r.rules || {});
          var failing = outs.filter(function (ro) {
            return !isPassing(ro[1]) && ro[1] !== "skipped";
          });
          var passing = outs.filter(function (ro) {
            return isPassing(ro[1]);
          });
          var short = shortId(id);
          var next = nextLevel(sc, r.level);
          var need = next ? blockers(sc, r, next) : [];
          var later = failing.length - need.length;
          var d = el(
            "details",
            { className: "ent" },
            el(
              "summary",
              {},
              badge(r.level),
              el("span", { className: "name" }, short, el("span", { className: "chev", textContent: "›" }), short !== id ? el("small", { textContent: id }) : null),
              el(
                "span",
                { className: "tally" },
                need.length
                  ? el("b", {
                      className: r.level === "none" ? "bad" : "",
                      textContent: next.charAt(0).toUpperCase() + next.slice(1) + " needs " + need.slice(0, 2).join(", ") + (need.length > 2 ? " +" + (need.length - 2) : ""),
                    })
                  : failing.length
                    ? ""
                    : "✓ all pass",
                later > 0 ? (need.length ? " · " : "") + later + " more for higher levels" : "",
                " · " + passing.length + " of " + outs.length + " pass",
              ),
            ),
          );
          var bodyBuilt = false;
          d.addEventListener("toggle", function () {
            if (!d.open || bodyBuilt) return;
            bodyBuilt = true;
            d.append(
              el(
                "div",
                { className: "ent-body" },
                self.nextStep(sc, r),
                self.rulesFor(sc, r),
                el("p", { style: "margin:12px 0 0" }, link("entity/" + enc(id), "", "Open " + short + " across every scorecard →")),
              ),
            );
          });
          list.append(d);
        });
        if (shown.length > limit) {
          list.append(
            el(
              "div",
              { className: "more" },
              el("button", {
                className: "link",
                textContent: "Show all " + shown.length,
                onclick: function () {
                  limit = Infinity;
                  draw();
                },
              }),
            ),
          );
        }
      };
      search.oninput = draw;
      redrawFilters();
      draw();

      var scored = rows.filter(function (pair) {
        return pair[1].level !== "skipped";
      });
      var failCounts = {};
      scored.forEach(function (pair) {
        Object.entries(pair[1].rules || {}).forEach(function (ro) {
          if (!isPassing(ro[1]) && ro[1] !== "skipped") failCounts[ro[0]] = (failCounts[ro[0]] || 0) + 1;
        });
      });
      var topFails = Object.entries(failCounts)
        .sort(function (a, b) {
          return b[1] - a[1] || a[0].localeCompare(b[0]);
        })
        .slice(0, 5);
      var failChart = topFails.length
        ? el(
            "div",
            { className: "fails" },
            ...topFails.map(function (f) {
              var rl = ((sc.rules || {})[f[0]] || {}).level;
              var share = Math.round((f[1] / scored.length) * 100);
              var fillSpan = el("span", { style: "flex-grow:0;background:var(--fail)" });
              grow([fillSpan], [share]);
              var b = el(
                "button",
                {
                  className: "fail-row",
                  title: ((sc.rules || {})[f[0]] || {}).fix || "",
                  onclick: function () {
                    search.value = f[0];
                    outcomeFilter = "failing";
                    redrawFilters();
                    draw();
                    list.scrollIntoView({ block: "start", behavior: "smooth" });
                  },
                },
                el("span", { className: "rn" }, f[0], rl ? badge(rl) : null),
                el("span", { className: "bar xs" }, fillSpan, el("span", { style: "flex-grow:" + (100 - share) + ";background:transparent" })),
                el("span", { className: "fc", textContent: f[1] + " of " + scored.length }),
              );
              b.setAttribute("aria-label", f[0] + ": fails on " + f[1] + " of " + scored.length + ". Show them.");
              return b;
            }),
          )
        : el("p", { className: "empty good", textContent: "✓ No rule fails on any entity." });

      var needs = el("div", { className: "levels-need" });
      (sc.levels || []).forEach(function (l) {
        var names = Object.keys(sc.rules || {})
          .filter(function (n) {
            return sc.rules[n].level === l;
          })
          .sort();
        needs.append(
          el(
            "div",
            { className: "need" },
            badge(l),
            names.length
              ? el(
                  "ul",
                  {},
                  ...names.map(function (n) {
                    return el("li", {}, el("b", { textContent: n }), sc.rules[n].fix ? el("span", { className: "why" }, ...rich(sc.rules[n].fix)) : null);
                  }),
                )
              : el("div", { className: "sub", textContent: "No rules at this level." }),
          ),
        );
      });
      // The rule that holds the most entities at the lowest occupied level below the top.
      var lever = null;
      var low = order.find(function (l) {
        return l !== "skipped" && nextLevel(sc, l) && parts.some(function (p) {
          return p.level === l;
        });
      });
      if (low) {
        var held = {};
        var up = nextLevel(sc, low);
        scored.forEach(function (pair) {
          if (pair[1].level !== low) return;
          blockers(sc, pair[1], up).forEach(function (n) {
            held[n] = (held[n] || 0) + 1;
          });
        });
        var best = Object.entries(held).sort(function (a, b) {
          return b[1] - a[1] || a[0].localeCompare(b[0]);
        })[0];
        if (best)
          lever = el(
            "div",
            { className: "next", style: "margin:0 0 10px" },
            el("b", { textContent: "Biggest lever: " }),
            best[0] + " keeps " + plural(best[1], "entity", "entities") + " at " + low + "; they need it for " + up + ". ",
            ...rich(((sc.rules || {})[best[0]] || {}).fix),
          );
      }
      var picker = el(
        "select",
        {
          ariaLabel: "Scorecard",
          onchange: function () {
            location.hash = "scorecard/" + enc(picker.value);
          },
        },
        ...ranked.map(function (r) {
          var debt = r.none > 0 ? r.none + " at none" : r.bronze > 0 ? r.bronze + " at bronze" : "clear";
          return el("option", { value: r.name, selected: r.name === active, textContent: title(D, r.name) + " (" + debt + ")" });
        }),
      );
      var owner = sc.owner ? sc.owner.replace(/^group:/, "") : "";
      return el(
        "div",
        { className: "split" },
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Scorecards" })),
          el("p", { className: "sub", style: "margin:0", textContent: "Most entities at none first" }),
          cardSearch,
          cardNav,
        ),
        el(
          "section",
          {},
          el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Scorecard" }), picker),
          el("div", { className: "head" }, el("h2", { textContent: title(D, active) }), el("code", { className: "sub", textContent: active })),
          el("p", { className: "lede", textContent: sc.description || "" }),
          el("div", { className: "sub", textContent: "Rates every " + sc.kind + (owner ? " · owned by " + owner : "") + (sc.track ? " · track " + sc.track : "") }),
          el("h3", { textContent: "What each level needs (each level also needs the ones below)" }),
          needs,
          el("h3", { textContent: "Where entities stand" }),
          barEl(parts, "lg"),
          legendEl(parts),
          el("h3", { textContent: "Most common failures" }),
          lever,
          failChart,
          el("h3", { textContent: "Entities" }),
          filters,
          search,
          list,
        ),
      );
    }
  }

  customElements.define("toolbox-report", ToolboxReport);
})();
