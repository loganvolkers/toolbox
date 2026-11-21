// <toolbox-report>: Overview, Catalog, and Scorecards as a custom element.
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
    ".modes{display:flex;flex-wrap:wrap;gap:6px;margin-bottom:16px}",
    ".modes button,.filters button,nav.cards button{border:1px solid var(--line);background:var(--card);color:var(--fg);border-radius:6px;padding:5px 12px;font:inherit;font-size:13px;cursor:pointer}",
    ".modes button[aria-pressed=true],.filters button[aria-pressed=true],nav.cards button[aria-pressed=true]{border-color:var(--accent);color:var(--accent);font-weight:600}",
    "nav.cards{display:flex;flex-wrap:wrap;gap:6px;margin:0 0 12px}",
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
    "table.fleet tr{cursor:pointer}table.fleet tr:hover td{background:var(--bg)}",
    "a.link,button.link{background:none;border:0;padding:0;color:var(--accent);font:inherit;cursor:pointer;text-decoration:underline;text-underline-offset:2px}",
    "details{border-top:1px solid var(--line);padding:6px 0}",
    "summary{cursor:pointer;display:flex;gap:10px;align-items:baseline;list-style:none}",
    "summary::-webkit-details-marker{display:none}",
    ".lvl{font-size:12px;font-weight:600;min-width:58px}",
    "code{font:13px ui-monospace,SFMono-Regular,Menlo,monospace;overflow-wrap:anywhere}",
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
    "::slotted(*){display:block;max-width:1100px;margin:0 auto;padding:12px 16px 0;box-sizing:border-box}",
    "@media (max-width:600px){.rule{margin-left:0}summary{flex-wrap:wrap}.fails{flex-basis:100%}table.fleet .hide-sm{display:none}}",
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

  function parseHash() {
    var h = (location.hash || "#").slice(1);
    if (!h || h === "overview") return { mode: "overview" };
    if (h === "catalog") return { mode: "catalog" };
    if (h.indexOf("catalog/") === 0) return { mode: "catalog", kind: decodeURIComponent(h.slice(8)) };
    if (h.indexOf("entity/") === 0) return { mode: "entity", id: decodeURIComponent(h.slice(7)) };
    if (h.indexOf("scorecard/") === 0) return { mode: "scorecard", name: decodeURIComponent(h.slice(10)) };
    // Legacy: bare scorecard name from older pages.
    return { mode: "scorecard", name: decodeURIComponent(h) };
  }

  function setHash(path) {
    var next = "#" + path;
    if (location.hash !== next) location.hash = path;
  }

  class ToolboxReport extends HTMLElement {
    connectedCallback() {
      if (this.shadowRoot) return;
      var root = this.attachShadow({ mode: "open" });
      root.append(template.content.cloneNode(true));
      var inline = this.querySelector('script[type="application/json"]');
      if (inline) {
        this.boot(root, JSON.parse(inline.textContent), true);
        return;
      }
      var src = this.getAttribute("src");
      if (!src) return;
      var self = this;
      fetch(src)
        .then(function (r) {
          if (!r.ok) throw new Error("report " + src + ": " + r.status);
          return r.json();
        })
        .then(function (data) {
          self.boot(root, data, false);
        })
        .catch(function (e) {
          root.getElementById("meta").textContent = String(e);
        });
    }

    boot(root, D, ownPage) {
      var self = this;
      this._D = D;
      this._root = root;
      this._ownPage = ownPage;
      if (ownPage) document.title = D.portal + " — report";
      root.getElementById("title").textContent = D.portal;
      var entityCount = 0;
      var catalog = D.catalog || {};
      Object.keys(catalog).forEach(function (k) {
        entityCount += (catalog[k] || []).length;
      });
      var cardCount = Object.keys(D.scorecards || {}).length;
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
        return r.mode === "scorecard" ? "scorecards" : r.mode;
      };
      var activeMode = modeOf(route);
      modes.replaceChildren(
        el("button", {
          textContent: "Overview",
          ariaPressed: String(activeMode === "overview"),
          onclick: function () {
            go("overview");
          },
        }),
        el("button", {
          textContent: "Catalog",
          ariaPressed: String(activeMode === "catalog"),
          onclick: function () {
            go("catalog");
          },
        }),
        el("button", {
          textContent: "Scorecards",
          ariaPressed: String(activeMode === "scorecards"),
          onclick: function () {
            var first = Object.keys(D.scorecards || {})[0] || "";
            go(first ? "scorecard/" + encodeURIComponent(first) : "overview");
          },
        }),
      );
      if (route.mode === "overview") body.replaceChildren(this.overview(D, go));
      else if (route.mode === "catalog") body.replaceChildren(this.catalog(D, route.kind, go));
      else if (route.mode === "entity") body.replaceChildren(this.entity(D, route.id, go));
      else body.replaceChildren(this.scorecard(D, route.name, go));
    }

    overview(D, go) {
      var catalog = D.catalog || {};
      var kinds = Object.keys(catalog);
      var entityCount = kinds.reduce(function (n, k) {
        return n + catalog[k].length;
      }, 0);
      var cards = Object.keys(D.scorecards || {});
      var rows = cards.map(function (name) {
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
          scored: ents.filter(function (p) {
            return p[1].level !== "skipped";
          }).length,
          weight: (none ? none.n : 0) * 1000 + (bronze ? bronze.n : 0),
        };
      });
      rows.sort(function (a, b) {
        return b.weight - a.weight || a.name.localeCompare(b.name);
      });
      var kindButtons = kinds
        .slice()
        .sort(function (a, b) {
          return catalog[b].length - catalog[a].length || a.localeCompare(b);
        })
        .map(function (k) {
          return el(
            "button",
            {
              onclick: function () {
                go("catalog/" + encodeURIComponent(k));
              },
            },
            el("strong", { textContent: k }),
            el("span", { textContent: catalog[k].length + " entities" }),
          );
        });
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
            el("th", { textContent: "Scored" }),
          ),
        ),
      );
      var tbody = el("tbody");
      rows.forEach(function (r) {
        var tr = el(
          "tr",
          {
            onclick: function () {
              go("scorecard/" + encodeURIComponent(r.name));
            },
          },
          el("td", {}, el("code", { textContent: r.name })),
          el("td", { className: "hide-sm", textContent: r.sc.kind }),
          el("td", {}, barEl(r.parts, "sm"), legendEl(r.parts)),
          el("td", { textContent: String(r.scored) }),
        );
        tbody.append(tr);
      });
      table.append(tbody);
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Fleet health" })),
          el(
            "div",
            { className: "stats" },
            el("div", { className: "stat" }, el("strong", { textContent: String(entityCount) }), el("span", { textContent: "entities in catalog" })),
            el("div", { className: "stat" }, el("strong", { textContent: String(kinds.length) }), el("span", { textContent: "kinds" })),
            el("div", { className: "stat" }, el("strong", { textContent: String(cards.length) }), el("span", { textContent: "scorecards" })),
          ),
          el("p", { className: "sub", textContent: "Scorecards ordered by entities at none, then bronze. Click a row to open that standard." }),
        ),
        el("section", {}, el("h2", { textContent: "Catalog by kind" }), el("div", { className: "kind-grid" }, ...kindButtons)),
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Scorecards" }), el("span", { className: "sub", textContent: "Needs attention first" })),
          table,
        ),
      );
    }

    catalog(D, kind, go) {
      var catalog = D.catalog || {};
      var kinds = Object.keys(catalog).sort(function (a, b) {
        return catalog[b].length - catalog[a].length || a.localeCompare(b);
      });
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
          return !q || id.toLowerCase().indexOf(q) >= 0;
        });
        if (!ids.length) {
          list.replaceChildren(el("div", { className: "empty", textContent: "No entities match." }));
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
                el("code", { textContent: id }),
                el("button", {
                  className: "link",
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
            el("span", { textContent: catalog[k].length + " entities" }),
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
          el("div", { className: "head" }, el("h2", { textContent: "Catalog" }), el("span", { className: "sub", textContent: "Every derived entity, by kind" })),
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
              el("span", {
                className: "lvl",
                textContent: score.level,
                style: "color:" + color(score.level),
              }),
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
          el("div", { className: "head" }, el("h2", {}, el("code", { textContent: id })), el("span", { className: "sub", textContent: kind || "unknown kind" })),
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
      var cards = Object.keys(D.scorecards || {});
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
      var cardNav = el("nav", { className: "cards", ariaLabel: "Scorecards" });
      var drawCards = function () {
        var q = cardSearch.value.toLowerCase();
        var shown = cards.filter(function (c) {
          return !q || c.toLowerCase().indexOf(q) >= 0 || (D.scorecards[c].kind || "").toLowerCase().indexOf(q) >= 0;
        });
        cardNav.replaceChildren.apply(
          cardNav,
          shown.map(function (c) {
            return el("button", {
              textContent: c,
              ariaPressed: String(c === active),
              onclick: function () {
                go("scorecard/" + encodeURIComponent(c));
              },
            });
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

      var redrawFilters = function () {
        var levelBtns = ["all"].concat(order.filter(function (l) {
          return parts.some(function (p) {
            return p.level === l;
          });
        }));
        filters.replaceChildren(
          ...levelBtns.map(function (l) {
            return el("button", {
              textContent: l === "all" ? "All levels" : l,
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
          list.replaceChildren(el("div", { className: "empty", textContent: "No entities match these filters." }));
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
            var details = el(
              "details",
              {},
              el(
                "summary",
                {},
                el("span", {
                  className: "lvl",
                  textContent: r.level,
                  style: "color:" + color(r.level),
                }),
                el("button", {
                  className: "link",
                  textContent: id,
                  onclick: function (ev) {
                    ev.preventDefault();
                    ev.stopPropagation();
                    go("entity/" + encodeURIComponent(id));
                  },
                }),
                el("span", {
                  className: "fails",
                  textContent: summaryBits.join(" · ") + (failing.length ? ": " + failing.map(function (ro) {
                    return ro[0];
                  }).join(", ") : ""),
                }),
              ),
              self.rulesBlock(sc, r, outcomeFilter === "all" ? "all" : outcomeFilter),
            );
            return details;
          }),
        );
      };
      search.oninput = draw;
      redrawFilters();
      draw();
      return el(
        "div",
        {},
        el("section", {}, el("h2", { textContent: "Scorecards" }), cardSearch, cardNav),
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
