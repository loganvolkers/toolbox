// <toolbox-report>: Overview, Catalog, Campaigns, and Scorecards as a custom element.
//
// Data is a child <script type="application/json">, or the src attribute (a URL of that JSON).
// A host page projects its own chrome into the slots header, nav, and footer. Those children
// stay in light DOM; this file does not read or concatenate them. Mode chrome and scorecard
// navigation live inside the shadow tree and are not slots.
(function () {
  if (customElements.get("toolbox-report")) return;

  var template = document.createElement("template");
  template.innerHTML = [
    "<style>",
    ":host{display:block;color:var(--fg);font:15px/1.5 system-ui,-apple-system,'Segoe UI',sans-serif}",
    ":host{--bg:#fbfbfa;--fg:#1d1d1b;--muted:#6b6b66;--line:#e4e3df;--card:#fff;--accent:#2f5d8a;",
    "--none:#b4433a;--bronze:#9a6a2f;--silver:#6f7a86;--gold:#a8861d;--skipped:#9a9a94;--pass:#2f7d4f;",
    "--unknown:#8a7a4a}",
    "@media (prefers-color-scheme:dark){:host{--bg:#161615;--fg:#ecebe7;--muted:#a3a29c;--line:#2e2e2b;--card:#1e1e1c;",
    "--accent:#86b3e0;--none:#e0786f;--bronze:#d5a463;--silver:#b3bdc8;--gold:#e2c25a;--skipped:#7d7d77;--pass:#6fc493;",
    "--unknown:#c4b56a}}",
    "main{max-width:1100px;margin:0 auto;padding:24px 16px 64px}",
    "h1{font-size:22px;margin:0 0 4px}h2{font-size:17px;margin:0}h3{font-size:14px;margin:16px 0 8px;color:var(--muted);font-weight:600;text-transform:uppercase;letter-spacing:.04em}",
    ".meta{color:var(--muted);font-size:13px;margin-bottom:16px}",
    ".verdict{font-size:16px;line-height:1.45;margin:0 0 12px}",
    ".modes{display:flex;flex-wrap:wrap;gap:6px;margin-bottom:16px}",
    ".modes button,.filters button,nav.cards button,nav.rank button{border:1px solid var(--line);background:var(--card);color:var(--fg);border-radius:6px;padding:5px 12px;font:inherit;font-size:13px;cursor:pointer}",
    ".modes button[aria-selected=true],.filters button[aria-pressed=true],nav.cards button[aria-pressed=true],nav.rank button[aria-pressed=true]{border-color:var(--accent);color:var(--accent);font-weight:600}",
    "nav.cards{display:flex;flex-wrap:wrap;gap:6px;margin:0 0 12px}",
    "nav.rank{display:flex;flex-direction:column;gap:4px;margin:0 0 12px;max-height:280px;overflow:auto}",
    "nav.rank button{display:flex;align-items:baseline;justify-content:space-between;gap:12px;width:100%;text-align:left}",
    "nav.rank .debt{color:var(--muted);font-size:12px;white-space:nowrap}",
    "nav.rank .debt.has-none{color:var(--none);font-weight:600}",
    "section{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:16px;margin-bottom:16px}",
    ".head{display:flex;flex-wrap:wrap;gap:8px 16px;align-items:baseline;justify-content:space-between}",
    ".sub{color:var(--muted);font-size:13px}",
    ".bar{display:flex;height:10px;border-radius:5px;overflow:hidden;margin:12px 0 4px;background:var(--line)}",
    ".bar.sm{height:8px;margin:6px 0 0}",
    ".legend{display:flex;flex-wrap:wrap;gap:12px;font-size:12px;color:var(--muted)}",
    ".dot{display:inline-block;width:8px;height:8px;border-radius:50%;margin-right:4px;vertical-align:middle}",
    "input[type=search]{width:100%;margin:12px 0 8px;padding:6px 10px;border:1px solid var(--line);border-radius:6px;background:var(--bg);color:var(--fg);font:inherit;box-sizing:border-box}",
    ".filters{display:flex;flex-wrap:wrap;gap:6px;margin:8px 0 4px}",
    ".stats{display:flex;flex-wrap:wrap;gap:16px 28px;margin:0 0 8px}",
    ".stat strong{display:block;font-size:22px;line-height:1.2}.stat span{color:var(--muted);font-size:12px}",
    "table.fleet{width:100%;border-collapse:collapse;font-size:13px}",
    "table.fleet th{text-align:left;font-weight:600;color:var(--muted);padding:6px 8px 6px 0;border-bottom:1px solid var(--line)}",
    "table.fleet td{padding:8px 8px 8px 0;border-bottom:1px solid var(--line);vertical-align:middle}",
    "table.fleet tr[tabindex]{cursor:pointer}table.fleet tr[tabindex]:hover td{background:var(--bg)}",
    "table.fleet tr[tabindex]:focus{outline:2px solid var(--accent);outline-offset:2px}",
    "a.link,button.link{background:none;border:0;padding:0;color:var(--accent);font:inherit;cursor:pointer;text-decoration:underline;text-underline-offset:2px}",
    "details{border-top:1px solid var(--line);padding:6px 0}",
    "summary{cursor:pointer;display:flex;gap:10px;align-items:baseline;list-style:none}",
    "summary::-webkit-details-marker{display:none}",
    ".lvl{font-size:12px;font-weight:600;min-width:58px}",
    ".lvl::before{content:attr(data-label) ': ';font-weight:500;color:var(--muted)}",
    "code{font:13px ui-monospace,SFMono-Regular,Menlo,monospace;overflow-wrap:anywhere}",
    ".id-short{font:13px ui-monospace,SFMono-Regular,Menlo,monospace}",
    ".id-full{color:var(--muted);font-size:12px;margin-left:6px;overflow-wrap:anywhere}",
    ".fails{color:var(--muted);font-size:13px;overflow-wrap:anywhere}",
    ".rule{margin:8px 0 0 68px}.rule b{font-weight:600}",
    ".rule ul{margin:4px 0 0;padding-left:18px}.rule li{overflow-wrap:anywhere}",
    ".fix{color:var(--muted);font-size:13px}",
    ".outcome{font-size:12px;font-weight:600}",
    ".outcome.pass{color:var(--pass)}.outcome.fail{color:var(--none)}.outcome.unknown{color:var(--unknown)}.outcome.skipped{color:var(--skipped)}",
    ".kind-grid{display:grid;grid-template-columns:repeat(auto-fill,minmax(160px,1fr));gap:8px}",
    ".kind-grid button{text-align:left;border:1px solid var(--line);background:var(--bg);border-radius:8px;padding:10px 12px;font:inherit;cursor:pointer;color:var(--fg)}",
    ".kind-grid button[aria-pressed=true]{border-color:var(--accent)}",
    ".kind-grid strong{display:block;font-size:14px}.kind-grid span{color:var(--muted);font-size:12px}",
    ".crumb{font-size:13px;color:var(--muted);margin-bottom:8px}",
    ".empty{color:var(--muted);font-size:13px;padding:8px 0}",
    ".empty-actions{margin-top:8px}",
    "::slotted(*){display:block;max-width:1100px;margin:0 auto;padding:12px 16px 0;box-sizing:border-box}",
    "@media (max-width:600px){.rule{margin-left:0}summary{flex-wrap:wrap}.fails{flex-basis:100%}table.fleet .hide-sm{display:none}.id-full{display:block;margin-left:0}}",
    "</style>",
    '<slot name="header"></slot>',
    '<slot name="nav"></slot>',
    "<main>",
    '<h1 id="title"></h1>',
    '<div class="meta" id="meta"></div>',
    '<div class="modes" id="modes" role="tablist" aria-label="Report views"></div>',
    '<div id="body"></div>',
    "</main>",
    '<slot name="footer"></slot>',
  ].join("");

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

  function color(l) {
    return "var(--" + (["none", "bronze", "silver", "gold", "skipped"].indexOf(l) >= 0 ? l : "accent") + ")";
  }

  function levelOrder(levels) {
    return ["none"].concat(levels || [], ["skipped"]);
  }

  function tallyLevels(rows, order) {
    var tally = {};
    rows.forEach(function (pair) {
      tally[pair[1].level] = (tally[pair[1].level] || 0) + 1;
    });
    return order.filter(function (l) {
      return tally[l];
    }).map(function (l) {
      return { level: l, n: tally[l] };
    });
  }

  function barEl(parts, cls) {
    return el(
      "div",
      { className: "bar" + (cls ? " " + cls : "") },
      ...parts.map(function (p) {
        return el("span", {
          style: "flex:" + p.n + ";background:" + color(p.level),
          title: p.level + ": " + p.n,
        });
      }),
    );
  }

  function legendEl(parts) {
    return el(
      "div",
      { className: "legend" },
      ...parts.map(function (p) {
        return el(
          "span",
          {},
          el("span", { className: "dot", style: "background:" + color(p.level) }),
          p.level + " " + p.n,
        );
      }),
    );
  }

  function shortId(id) {
    var s = String(id || "");
    var slash = s.lastIndexOf("/");
    var colon = s.lastIndexOf(":");
    var i = Math.max(slash, colon);
    return i >= 0 && i < s.length - 1 ? s.slice(i + 1) : s;
  }

  function idLabel(id) {
    var short = shortId(id);
    if (short === id) return el("code", { textContent: id, title: id });
    return el(
      "span",
      { title: id },
      el("span", { className: "id-short", textContent: short }),
      el("span", { className: "id-full", textContent: id }),
    );
  }

  function makeLevelBadge(level) {
    var span = el("span", {
      className: "lvl",
      textContent: level,
      style: "color:" + color(level),
    });
    span.setAttribute("data-label", "level");
    return span;
  }

  function clickableRow(path, go) {
    var kids = Array.prototype.slice.call(arguments, 2);
    return el(
      "tr",
      {
        tabIndex: 0,
        role: "link",
        title: "Open",
        onclick: function () {
          go(path);
        },
        onkeydown: function (ev) {
          if (ev.key === "Enter" || ev.key === " ") {
            ev.preventDefault();
            go(path);
          }
        },
      },
      ...kids,
    );
  }

  function scorecardRows(D) {
    var cards = Object.keys(D.scorecards || {});
    return cards
      .map(function (name) {
        var sc = D.scorecards[name];
        var ents = Object.entries(D.scores[name] || {});
        var order = levelOrder(sc.levels);
        var parts = tallyLevels(ents, order);
        var none = parts.find(function (p) {
          return p.level === "none";
        });
        var bronze = parts.find(function (p) {
          return p.level === "bronze";
        });
        return {
          name: name,
          sc: sc,
          parts: parts,
          none: none ? none.n : 0,
          bronze: bronze ? bronze.n : 0,
          scored: ents.filter(function (p) {
            return p[1].level !== "skipped";
          }).length,
          weight: (none ? none.n : 0) * 1000 + (bronze ? bronze.n : 0),
        };
      })
      .sort(function (a, b) {
        return b.weight - a.weight || a.name.localeCompare(b.name);
      });
  }

  function campaignNamesSorted(D) {
    var campaigns = D.campaigns || {};
    return Object.keys(campaigns)
      .filter(function (n) {
        var st = campaigns[n].status;
        return st === "active" || st === "scheduled" || st === "ended";
      })
      .sort(function (a, b) {
        return (campaigns[a].percent || 0) - (campaigns[b].percent || 0) || a.localeCompare(b);
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

  function healthVerdict(D, campaignNames, rows) {
    var campaigns = D.campaigns || {};
    var off = campaignNames.filter(function (n) {
      return !campaigns[n].onTarget;
    });
    var debt = rows.filter(function (r) {
      return r.none > 0;
    });
    if (!campaignNames.length && !rows.length) {
      return "No campaigns or scorecards in this snapshot.";
    }
    if (!off.length && !debt.length) {
      return "On track: every active campaign meets its target, and no scorecard has entities at none.";
    }
    var bits = [];
    if (off.length) {
      bits.push(
        off.length +
          " campaign" +
          (off.length === 1 ? "" : "s") +
          " below target (" +
          off
            .slice(0, 3)
            .map(function (n) {
              return campaigns[n].title || n;
            })
            .join(", ") +
          (off.length > 3 ? "…" : "") +
          ")",
      );
    } else if (campaignNames.length) {
      bits.push("campaigns on target");
    }
    if (debt.length) {
      bits.push(
        debt.length +
          " scorecard" +
          (debt.length === 1 ? "" : "s") +
          " with entities at none (worst: " +
          debt
            .slice(0, 3)
            .map(function (r) {
              return r.name + " ×" + r.none;
            })
            .join(", ") +
          (debt.length > 3 ? "…" : "") +
          ")",
      );
    } else {
      bits.push("no scorecard debt at none");
    }
    return "Needs attention: " + bits.join("; ") + ".";
  }

  function parseHash() {
    var h = (location.hash || "").replace(/^#/, "");
    if (!h || h === "overview") return { mode: "overview" };
    if (h === "catalog") return { mode: "catalog" };
    if (h.indexOf("catalog/") === 0) return { mode: "catalog", kind: decodeURIComponent(h.slice(8)) };
    if (h.indexOf("entity/") === 0) return { mode: "entity", id: decodeURIComponent(h.slice(7)) };
    if (h === "campaigns") return { mode: "campaigns" };
    if (h.indexOf("campaign/") === 0) return { mode: "campaign", name: decodeURIComponent(h.slice(9)) };
    if (h.indexOf("scorecard/") === 0) return { mode: "scorecard", name: decodeURIComponent(h.slice(10)) };
    if (h === "scorecards") return { mode: "scorecard" };
    return { mode: "scorecard", name: decodeURIComponent(h) };
  }

  function setHash(path) {
    location.hash = path;
  }

  class ToolboxReport extends HTMLElement {
    connectedCallback() {
      if (this._booted) return;
      this._booted = true;
      var root = this.attachShadow({ mode: "open" });
      root.append(template.content.cloneNode(true));
      var self = this;
      var inline = this.querySelector('script[type="application/json"]');
      var ownPage = !this.hasAttribute("src");
      var load = inline
        ? Promise.resolve(JSON.parse(inline.textContent))
        : fetch(this.getAttribute("src"))
            .then(function (r) {
              if (!r.ok) throw new Error("report JSON HTTP " + r.status);
              return r.json();
            });
      load
        .then(function (data) {
          self.mount(root, data, ownPage);
        })
        .catch(function (e) {
          root.getElementById("body").replaceChildren(
            el("section", {}, el("div", { className: "empty", textContent: "Failed to load report: " + e.message })),
          );
        });
    }

    mount(root, D, ownPage) {
      var self = this;
      var catalog = D.catalog || {};
      var entityCount = Object.keys(catalog).reduce(function (n, k) {
        return n + catalog[k].length;
      }, 0);
      var cardCount = Object.keys(D.scorecards || {}).length;
      root.getElementById("title").textContent = D.portal || "Toolbox report";
      root.getElementById("meta").textContent =
        entityCount +
        " entities · " +
        Object.keys(catalog).length +
        " kinds · " +
        cardCount +
        " scorecards · snapshot " +
        D.snapshot +
        " · toolbox report";
      var draw = function () {
        self.render(root, D, ownPage);
      };
      window.addEventListener("hashchange", draw);
      draw();
    }

    render(root, D, ownPage) {
      var route = parseHash();
      var modes = root.getElementById("modes");
      var body = root.getElementById("body");
      var self = this;
      var go = function (path) {
        var next = "#" + path;
        if (location.hash === next) self.render(root, D, ownPage);
        else setHash(path);
      };
      var modeOf = function (r) {
        if (r.mode === "entity") return "catalog";
        if (r.mode === "campaign") return "campaigns";
        return r.mode === "scorecard" ? "scorecards" : r.mode;
      };
      var activeMode = modeOf(route);
      var tabs = [
        { label: "Overview", path: "overview", id: "overview" },
        { label: "Catalog", path: "catalog", id: "catalog" },
        { label: "Campaigns", path: "campaigns", id: "campaigns" },
        {
          label: "Scorecards",
          id: "scorecards",
          path: (function () {
            var worst = scorecardRows(D)[0];
            return worst ? "scorecard/" + encodeURIComponent(worst.name) : "overview";
          })(),
        },
      ];
      modes.replaceChildren(
        ...tabs.map(function (t) {
          var selected = activeMode === t.id;
          return el("button", {
            role: "tab",
            id: "tab-" + t.id,
            textContent: t.label,
            ariaSelected: String(selected),
            tabIndex: selected ? 0 : -1,
            onclick: function () {
              go(t.path);
            },
          });
        }),
      );
      modes.onkeydown = function (ev) {
        var keys = { ArrowLeft: -1, ArrowRight: 1, Home: "home", End: "end" };
        var dir = keys[ev.key];
        if (dir == null) return;
        ev.preventDefault();
        var buttons = Array.prototype.slice.call(modes.querySelectorAll('[role="tab"]'));
        var i = buttons.indexOf(root.activeElement);
        if (i < 0) i = buttons.findIndex(function (b) {
          return b.getAttribute("aria-selected") === "true";
        });
        var next = 0;
        if (dir === "home") next = 0;
        else if (dir === "end") next = buttons.length - 1;
        else next = (i + dir + buttons.length) % buttons.length;
        buttons[next].focus();
        buttons[next].click();
      };
      if (route.mode === "overview") body.replaceChildren(this.overview(D, go));
      else if (route.mode === "catalog") body.replaceChildren(this.catalog(D, route.kind, go));
      else if (route.mode === "entity") body.replaceChildren(this.entity(D, route.id, go));
      else if (route.mode === "campaigns" || route.mode === "campaign")
        body.replaceChildren(this.campaigns(D, route.name, go));
      else body.replaceChildren(this.scorecard(D, route.name, go));
    }

    overview(D, go) {
      var catalog = D.catalog || {};
      var kinds = Object.keys(catalog);
      var entityCount = kinds.reduce(function (n, k) {
        return n + catalog[k].length;
      }, 0);
      var campaigns = D.campaigns || {};
      var campaignNames = campaignNamesSorted(D);
      var rows = scorecardRows(D);
      var offTarget = campaignNames.filter(function (n) {
        return !campaigns[n].onTarget;
      }).length;
      var noneCards = rows.filter(function (r) {
        return r.none > 0;
      }).length;
      var campaignTable = null;
      if (campaignNames.length) {
        campaignTable = el("table", { className: "fleet" });
        campaignTable.append(
          el(
            "thead",
            {},
            el(
              "tr",
              {},
              el("th", { textContent: "Campaign" }),
              el("th", { textContent: "Progress" }),
              el("th", { className: "hide-sm", textContent: "Status" }),
              el("th", { textContent: "Meeting" }),
            ),
          ),
        );
        var cbody = el("tbody");
        campaignNames.forEach(function (name) {
          var c = campaigns[name];
          var label = c.title || name;
          var pct = c.percent == null ? 0 : c.percent;
          var target = c.targetPercent == null ? 100 : c.targetPercent;
          cbody.append(
            clickableRow(
              "campaign/" + encodeURIComponent(name),
              go,
              el("td", {}, el("code", { textContent: label })),
              el(
                "td",
                {},
                el(
                  "div",
                  { className: "bar sm" },
                  el("span", {
                    style: "flex:" + pct + ";background:" + (c.onTarget ? "var(--pass)" : "var(--accent)"),
                  }),
                  el("span", { style: "flex:" + Math.max(0, 100 - pct) + ";background:transparent" }),
                ),
                el("span", { className: "sub", textContent: pct + "% of " + target + "% target" }),
              ),
              el("td", { className: "hide-sm", textContent: c.status || "" }),
              el("td", { textContent: (c.meeting || 0) + " / " + (c.inScope || 0) }),
            ),
          );
        });
        campaignTable.append(cbody);
      }
      var table = el("table", { className: "fleet" });
      table.append(
        el(
          "thead",
          {},
          el(
            "tr",
            {},
            el("th", { textContent: "Scorecard" }),
            el("th", { className: "hide-sm", textContent: "Kind" }),
            el("th", { textContent: "Distribution" }),
            el("th", { textContent: "At none" }),
            el("th", { textContent: "Scored" }),
          ),
        ),
      );
      var tbody = el("tbody");
      rows.forEach(function (r) {
        tbody.append(
          clickableRow(
            "scorecard/" + encodeURIComponent(r.name),
            go,
            el("td", {}, el("code", { textContent: r.name })),
            el("td", { className: "hide-sm", textContent: r.sc.kind }),
            el("td", {}, barEl(r.parts, "sm"), legendEl(r.parts)),
            el("td", { textContent: String(r.none) }),
            el("td", { textContent: String(r.scored) }),
          ),
        );
      });
      table.append(tbody);
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Fleet health" })),
          el("p", { className: "verdict", textContent: healthVerdict(D, campaignNames, rows) }),
          el(
            "div",
            { className: "stats" },
            el(
              "div",
              { className: "stat" },
              el("strong", { textContent: String(offTarget) }),
              el("span", { textContent: "campaigns below target" }),
            ),
            el(
              "div",
              { className: "stat" },
              el("strong", { textContent: String(noneCards) }),
              el("span", { textContent: "scorecards with none" }),
            ),
            el(
              "div",
              { className: "stat" },
              el("strong", { textContent: String(rows.length) }),
              el("span", { textContent: "scorecards" }),
            ),
            el(
              "div",
              { className: "stat" },
              el("strong", { textContent: String(campaignNames.length) }),
              el("span", { textContent: "campaigns" }),
            ),
          ),
          el("p", {
            className: "sub",
            textContent:
              "Campaigns are time-bound focus; scorecards are the continuous bar. " +
              entityCount +
              " entities across " +
              kinds.length +
              " kinds — browse them in Catalog. Activate a row with Enter or Space.",
          }),
        ),
        campaignTable
          ? el(
              "section",
              {},
              el(
                "div",
                { className: "head" },
                el("h2", { textContent: "Campaigns" }),
                el("span", { className: "sub", textContent: "Lowest progress first" }),
              ),
              campaignTable,
            )
          : null,
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: "Scorecards" }),
            el("span", { className: "sub", textContent: "Needs attention first" }),
          ),
          table,
        ),
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: "Catalog" }),
            el(
              "button",
              {
                className: "link",
                textContent: "Open catalog by kind",
                onclick: function () {
                  go("catalog");
                },
              },
            ),
          ),
          el("p", {
            className: "sub",
            textContent: "Inventory browse lives in Catalog so Overview stays a health judgement.",
          }),
        ),
      );
    }

    campaigns(D, name, go) {
      var all = D.campaigns || {};
      var names = campaignNamesSorted(D);
      if (!names.length) {
        names = Object.keys(all).sort(function (a, b) {
          return (all[a].percent || 0) - (all[b].percent || 0) || a.localeCompare(b);
        });
      }
      if (!names.length) {
        return el("section", {}, el("div", { className: "empty", textContent: "No campaigns in this portal." }));
      }
      var selected = name && all[name] ? name : names[0];
      var nav = el("nav", { className: "cards", ariaLabel: "Campaigns" });
      names.forEach(function (n) {
        nav.append(
          el("button", {
            textContent: (all[n].title || n) + " (" + (all[n].percent || 0) + "%)",
            ariaPressed: String(n === selected),
            onclick: function () {
              go("campaign/" + encodeURIComponent(n));
            },
          }),
        );
      });
      var c = all[selected];
      var pct = c.percent == null ? 0 : c.percent;
      var target = c.targetPercent == null ? 100 : c.targetPercent;
      var includes = (c.includes || [])
        .map(function (i) {
          var bits = [i.scorecard];
          if (i.atLeast) bits.push("≥ " + i.atLeast);
          if (i.rules && i.rules.length) bits.push("rules " + i.rules.join(", "));
          return bits.join(" · ");
        })
        .join("; ");
      var milestones = (c.milestones || [])
        .map(function (m) {
          return m.name + " @" + m.targetPercent + "%" + (m.met ? " (met)" : " (open)");
        })
        .join(", ");
      var gaps = Object.entries(c.entities || {}).filter(function (pair) {
        return !pair[1].meeting;
      });
      gaps.sort(function (a, b) {
        return a[0].localeCompare(b[0]);
      });
      var gapList = el("div");
      if (!gaps.length) {
        gapList.append(el("div", { className: "empty", textContent: "Every in-scope entity meets the goal." }));
      } else {
        gaps.forEach(function (pair) {
          gapList.append(
            el(
              "div",
              { className: "rule", style: "margin-left:0" },
              el(
                "div",
                {},
                el("a", {
                  className: "link",
                  href: "#entity/" + encodeURIComponent(pair[0]),
                  textContent: shortId(pair[0]),
                  title: pair[0],
                  onclick: function (ev) {
                    ev.preventDefault();
                    go("entity/" + encodeURIComponent(pair[0]));
                  },
                }),
                el("span", { className: "id-full", textContent: pair[0] }),
              ),
              el("div", { className: "fails", textContent: (pair[1].gaps || []).join("; ") }),
            ),
          );
        });
      }
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: "Campaigns" }),
            el("span", { className: "sub", textContent: "Lowest progress first" }),
          ),
          nav,
        ),
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: c.title || selected }),
            el("span", { className: "sub", textContent: (c.status || "") + (c.track ? " · track " + c.track : "") }),
          ),
          el("p", { className: "sub", textContent: c.description || "" }),
          el(
            "div",
            { className: "stats" },
            el("div", { className: "stat" }, el("strong", { textContent: pct + "%" }), el("span", { textContent: "of " + target + "% target" })),
            el("div", { className: "stat" }, el("strong", { textContent: String(c.meeting || 0) }), el("span", { textContent: "meeting" })),
            el("div", { className: "stat" }, el("strong", { textContent: String(c.inScope || 0) }), el("span", { textContent: "in scope" })),
          ),
          el(
            "div",
            { className: "bar" },
            el("span", { style: "flex:" + pct + ";background:" + (c.onTarget ? "var(--pass)" : "var(--accent)") }),
            el("span", { style: "flex:" + Math.max(0, 100 - pct) + ";background:transparent" }),
          ),
          el("p", { className: "sub", textContent: "Includes: " + (includes || "—") }),
          milestones ? el("p", { className: "sub", textContent: "Milestones: " + milestones }) : null,
          el("h3", { textContent: "Not meeting" }),
          gapList,
        ),
      );
    }

    catalog(D, kind, go) {
      var catalog = D.catalog || {};
      var kinds = catalogKindsSorted(D);
      var selected = kind && catalog[kind] ? kind : kinds[0];
      var search = el("input", {
        type: "search",
        placeholder: "Filter entities…",
        ariaLabel: "Filter entities",
      });
      var list = el("div");
      var drawList = function () {
        var q = search.value.toLowerCase();
        var ids = (catalog[selected] || []).filter(function (id) {
          return !q || id.toLowerCase().indexOf(q) >= 0 || shortId(id).toLowerCase().indexOf(q) >= 0;
        });
        if (!ids.length) {
          list.replaceChildren(
            el("div", { className: "empty", textContent: "No entities match." }),
            el(
              "div",
              { className: "empty-actions" },
              el("button", {
                className: "link",
                textContent: "Clear filter",
                onclick: function () {
                  search.value = "";
                  drawList();
                },
              }),
            ),
          );
          return;
        }
        list.replaceChildren.apply(
          list,
          ids.map(function (id) {
            return el(
              "details",
              {},
              el(
                "summary",
                {},
                idLabel(id),
                el("a", {
                  className: "link",
                  href: "#entity/" + encodeURIComponent(id),
                  textContent: "Open",
                  onclick: function (ev) {
                    ev.preventDefault();
                    ev.stopPropagation();
                    go("entity/" + encodeURIComponent(id));
                  },
                }),
              ),
              this.entityScores(D, id, go),
            );
          }, this),
        );
      }.bind(this);
      search.oninput = drawList;
      var grid = el(
        "div",
        { className: "kind-grid" },
        ...kinds.map(function (k) {
          return el(
            "button",
            {
              ariaPressed: String(k === selected),
              onclick: function () {
                go("catalog/" + encodeURIComponent(k));
              },
            },
            el("strong", { textContent: k }),
            el("span", {
              textContent: catalog[k].length + " entities" + (scoredKinds(D)[k] ? " · scored" : ""),
            }),
          );
        }),
      );
      drawList();
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: "Catalog" }),
            el("span", { className: "sub", textContent: "Scored kinds first, then by size" }),
          ),
          grid,
        ),
        selected
          ? el(
              "section",
              {},
              el(
                "div",
                { className: "head" },
                el("h2", { textContent: selected }),
                el("span", { className: "sub", textContent: (catalog[selected] || []).length + " entities" }),
              ),
              search,
              list,
            )
          : el("section", {}, el("div", { className: "empty", textContent: "Catalog is empty in this report." })),
      );
    }

    entityScores(D, id, go) {
      var wrap = el("div");
      var cards = Object.keys(D.scorecards || {}).filter(function (name) {
        return D.scores[name] && D.scores[name][id];
      });
      if (!cards.length) {
        wrap.append(el("div", { className: "empty", textContent: "Not scored by any scorecard in this snapshot." }));
        return wrap;
      }
      cards.forEach(function (name) {
        var sc = D.scorecards[name];
        var score = D.scores[name][id];
        wrap.append(
          el(
            "div",
            { className: "rule", style: "margin-left:0" },
            el(
              "div",
              {},
              el("button", {
                className: "link",
                textContent: name,
                onclick: function () {
                  go("scorecard/" + encodeURIComponent(name));
                },
              }),
              " · ",
              makeLevelBadge(score.level),
              " · ",
              el("span", { className: "sub", textContent: sc.kind }),
            ),
            this.rulesBlock(sc, score, "all"),
          ),
        );
      }, this);
      return wrap;
    }

    entity(D, id, go) {
      var kind = null;
      var catalog = D.catalog || {};
      Object.keys(catalog).some(function (k) {
        if (catalog[k].indexOf(id) >= 0) {
          kind = k;
          return true;
        }
        return false;
      });
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el(
            "div",
            { className: "crumb" },
            el("button", {
              className: "link",
              textContent: "Catalog",
              onclick: function () {
                go("catalog");
              },
            }),
            kind
              ? el(
                  "span",
                  {},
                  " / ",
                  el("button", {
                    className: "link",
                    textContent: kind,
                    onclick: function () {
                      go("catalog/" + encodeURIComponent(kind));
                    },
                  }),
                )
              : "",
            " / entity",
          ),
          el(
            "div",
            { className: "head" },
            el("h2", {}, idLabel(id)),
            el("span", { className: "sub", textContent: kind || "unknown kind" }),
          ),
          this.entityScores(D, id, go),
        ),
      );
    }

    rulesBlock(sc, score, outcomeFilter) {
      var rules = Object.entries(score.rules || {});
      if (outcomeFilter === "failing") {
        rules = rules.filter(function (ro) {
          return ro[1] !== "pass";
        });
      } else if (outcomeFilter === "passing") {
        rules = rules.filter(function (ro) {
          return ro[1] === "pass";
        });
      }
      rules.sort(function (a, b) {
        var rank = function (o) {
          return o === "fail" ? 0 : o === "unknown" ? 1 : o === "skipped" ? 2 : 3;
        };
        return rank(a[1]) - rank(b[1]) || a[0].localeCompare(b[0]);
      });
      if (!rules.length) {
        return el("div", { className: "empty", textContent: "No rules match this filter." });
      }
      var frag = el("div");
      rules.forEach(function (ro) {
        var n = ro[0];
        var o = ro[1];
        var rule = (sc.rules && sc.rules[n]) || {};
        var ev = (score.evidence || {})[n];
        var items = ev == null ? [] : Array.isArray(ev) ? ev : [ev];
        var showDetail = o !== "pass";
        frag.append(
          el(
            "div",
            { className: "rule" },
            el(
              "div",
              {},
              el("b", { textContent: n }),
              " ",
              el("span", {
                className: "outcome " + (o === "pass" || o === "fail" || o === "unknown" || o === "skipped" ? o : "unknown"),
                textContent: o,
              }),
              rule.level ? el("span", { className: "sub", textContent: " · " + rule.level }) : "",
            ),
            showDetail && rule.fix ? el("div", { className: "fix", textContent: rule.fix }) : "",
            showDetail && items.length
              ? el(
                  "ul",
                  {},
                  ...items.slice(0, 50).map(function (i) {
                    return el("li", {}, el("code", { textContent: typeof i === "string" ? i : JSON.stringify(i) }));
                  }),
                )
              : "",
          ),
        );
      });
      return frag;
    }

    scorecard(D, name, go) {
      var ranked = scorecardRows(D);
      var cards = ranked.map(function (r) {
        return r.name;
      });
      var active = name && D.scorecards[name] ? name : cards[0];
      if (!active) {
        return el("section", {}, el("div", { className: "empty", textContent: "No scorecards in this report." }));
      }
      var sc = D.scorecards[active];
      var rows = Object.entries(D.scores[active] || {});
      var order = levelOrder(sc.levels);
      var parts = tallyLevels(rows, order);
      var cardSearch = el("input", {
        type: "search",
        placeholder: "Find a scorecard…",
        ariaLabel: "Find a scorecard",
      });
      var cardNav = el("nav", { className: "rank", ariaLabel: "Scorecards ranked by debt" });
      var drawCards = function () {
        var q = cardSearch.value.toLowerCase();
        var shown = ranked.filter(function (r) {
          return (
            !q ||
            r.name.toLowerCase().indexOf(q) >= 0 ||
            (r.sc.kind || "").toLowerCase().indexOf(q) >= 0
          );
        });
        if (!shown.length) {
          cardNav.replaceChildren(
            el("div", { className: "empty", textContent: "No scorecards match." }),
            el(
              "div",
              { className: "empty-actions" },
              el("button", {
                className: "link",
                textContent: "Clear search",
                onclick: function () {
                  cardSearch.value = "";
                  drawCards();
                },
              }),
            ),
          );
          return;
        }
        cardNav.replaceChildren.apply(
          cardNav,
          shown.map(function (r) {
            var debt =
              r.none > 0
                ? r.none + " at none"
                : r.bronze > 0
                  ? r.bronze + " at bronze"
                  : "clear";
            var btn = el(
              "button",
              {
                ariaPressed: String(r.name === active),
                onclick: function () {
                  go("scorecard/" + encodeURIComponent(r.name));
                },
              },
              el("span", {}, el("code", { textContent: r.name }), el("span", { className: "sub", textContent: " · " + r.sc.kind })),
              el("span", { className: "debt" + (r.none > 0 ? " has-none" : ""), textContent: debt }),
            );
            return btn;
          }),
        );
      };
      cardSearch.oninput = drawCards;
      drawCards();

      var levelFilter = "all";
      var outcomeFilter = "all";
      var search = el("input", {
        type: "search",
        placeholder: "Filter by entity or rule…",
        ariaLabel: "Filter entities",
      });
      var filters = el("div", { className: "filters" });
      var list = el("div");
      var self = this;

      var clearFilters = function () {
        levelFilter = "all";
        outcomeFilter = "all";
        search.value = "";
        redrawFilters();
        draw();
      };

      var redrawFilters = function () {
        var levelBtns = ["all"].concat(
          order.filter(function (l) {
            return parts.some(function (p) {
              return p.level === l;
            });
          }),
        );
        filters.replaceChildren(
          ...levelBtns.map(function (l) {
            return el("button", {
              textContent: l === "all" ? "All levels" : "level: " + l,
              ariaPressed: String(levelFilter === l),
              onclick: function () {
                levelFilter = l;
                redrawFilters();
                draw();
              },
            });
          }),
          el("button", {
            textContent: "All outcomes",
            ariaPressed: String(outcomeFilter === "all"),
            onclick: function () {
              outcomeFilter = "all";
              redrawFilters();
              draw();
            },
          }),
          el("button", {
            textContent: "Failing",
            ariaPressed: String(outcomeFilter === "failing"),
            onclick: function () {
              outcomeFilter = "failing";
              redrawFilters();
              draw();
            },
          }),
          el("button", {
            textContent: "Passing",
            ariaPressed: String(outcomeFilter === "passing"),
            onclick: function () {
              outcomeFilter = "passing";
              redrawFilters();
              draw();
            },
          }),
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
          .sort(function (a, b) {
            return order.indexOf(a[1].level) - order.indexOf(b[1].level) || a[0].localeCompare(b[0]);
          })
          .filter(function (pair) {
            if (!q) return true;
            if (pair[0].toLowerCase().indexOf(q) >= 0) return true;
            if (shortId(pair[0]).toLowerCase().indexOf(q) >= 0) return true;
            return Object.keys(pair[1].rules || {}).some(function (rn) {
              return rn.toLowerCase().indexOf(q) >= 0;
            });
          })
          .filter(function (pair) {
            if (outcomeFilter === "all") return true;
            var rules = Object.entries(pair[1].rules || {});
            if (outcomeFilter === "failing") {
              return rules.some(function (ro) {
                return ro[1] !== "pass";
              });
            }
            return rules.some(function (ro) {
              return ro[1] === "pass";
            });
          });
        if (!shown.length) {
          list.replaceChildren(
            el("div", { className: "empty", textContent: "No entities match these filters." }),
            el(
              "div",
              { className: "empty-actions" },
              el("button", {
                className: "link",
                textContent: "Clear filters",
                onclick: clearFilters,
              }),
            ),
          );
          return;
        }
        list.replaceChildren.apply(
          list,
          shown.map(function (pair) {
            var id = pair[0];
            var r = pair[1];
            var failing = Object.entries(r.rules || {}).filter(function (ro) {
              return ro[1] !== "pass";
            });
            var passing = Object.entries(r.rules || {}).filter(function (ro) {
              return ro[1] === "pass";
            });
            var summaryBits = [];
            if (failing.length) summaryBits.push(failing.length + " failing");
            if (passing.length) summaryBits.push(passing.length + " passing");
            return el(
              "details",
              {},
              el(
                "summary",
                {},
                makeLevelBadge(r.level),
                el("a", {
                  className: "link",
                  href: "#entity/" + encodeURIComponent(id),
                  title: id,
                  textContent: shortId(id),
                  onclick: function (ev) {
                    ev.preventDefault();
                    ev.stopPropagation();
                    go("entity/" + encodeURIComponent(id));
                  },
                }),
                el("span", { className: "id-full", textContent: id }),
                el("span", {
                  className: "fails",
                  textContent:
                    summaryBits.join(" · ") +
                    (failing.length
                      ? ": " +
                        failing
                          .map(function (ro) {
                            return ro[0];
                          })
                          .join(", ")
                      : ""),
                }),
              ),
              self.rulesBlock(sc, r, outcomeFilter === "all" ? "all" : outcomeFilter),
            );
          }),
        );
      };
      search.oninput = draw;
      redrawFilters();
      draw();
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: "Scorecards" }),
            el("span", { className: "sub", textContent: "Highest none debt first" }),
          ),
          cardSearch,
          cardNav,
        ),
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: active }),
            el("span", {
              className: "sub",
              textContent: sc.kind + " · levels " + (sc.levels || []).join(" < ") + " · owner " + sc.owner,
            }),
          ),
          barEl(parts),
          legendEl(parts),
          filters,
          search,
          list,
        ),
      );
    }
  }

  customElements.define("toolbox-report", ToolboxReport);
})();
