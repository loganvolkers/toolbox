// <toolbox-report>: the scorecard report as a custom element.
//
// Data is a child <script type="application/json">, or the src attribute (a URL of that JSON).
// A host page projects its own chrome into the slots header, nav, and footer. Those children
// stay in light DOM; this file does not read or concatenate them. The scorecard tabs are
// inside the shadow tree and are not a slot.
(function () {
  if (customElements.get("toolbox-report")) return;

  var template = document.createElement("template");
  template.innerHTML = [
    "<style>",
    ":host{display:block;color:var(--fg);font:15px/1.5 system-ui,-apple-system,'Segoe UI',sans-serif}",
    ":host{--bg:#fbfbfa;--fg:#1d1d1b;--muted:#6b6b66;--line:#e4e3df;--card:#fff;--accent:#2f5d8a;",
    "--none:#b4433a;--bronze:#9a6a2f;--silver:#6f7a86;--gold:#a8861d;--skipped:#9a9a94;--pass:#2f7d4f}",
    "@media (prefers-color-scheme:dark){:host{--bg:#161615;--fg:#ecebe7;--muted:#a3a29c;--line:#2e2e2b;--card:#1e1e1c;",
    "--accent:#86b3e0;--none:#e0786f;--bronze:#d5a463;--silver:#b3bdc8;--gold:#e2c25a;--skipped:#7d7d77;--pass:#6fc493}}",
    "main{max-width:1100px;margin:0 auto;padding:24px 16px 64px}",
    "h1{font-size:22px;margin:0 0 4px}h2{font-size:17px;margin:0}",
    ".meta{color:var(--muted);font-size:13px;margin-bottom:20px}",
    "nav{display:flex;flex-wrap:wrap;gap:6px;margin-bottom:16px}",
    "nav button{border:1px solid var(--line);background:var(--card);color:var(--fg);border-radius:999px;padding:4px 12px;font:inherit;font-size:13px;cursor:pointer}",
    "nav button[aria-pressed=true]{border-color:var(--accent);color:var(--accent);font-weight:600}",
    "section{background:var(--card);border:1px solid var(--line);border-radius:10px;padding:16px;margin-bottom:16px}",
    ".head{display:flex;flex-wrap:wrap;gap:8px 16px;align-items:baseline;justify-content:space-between}",
    ".sub{color:var(--muted);font-size:13px}",
    ".bar{display:flex;height:10px;border-radius:5px;overflow:hidden;margin:12px 0 4px;background:var(--line)}",
    ".legend{display:flex;flex-wrap:wrap;gap:12px;font-size:12px;color:var(--muted)}",
    ".dot{display:inline-block;width:8px;height:8px;border-radius:50%;margin-right:4px;vertical-align:middle}",
    "input[type=search]{width:100%;margin:12px 0 8px;padding:6px 10px;border:1px solid var(--line);border-radius:6px;background:var(--bg);color:var(--fg);font:inherit}",
    "details{border-top:1px solid var(--line);padding:6px 0}",
    "summary{cursor:pointer;display:flex;gap:10px;align-items:baseline;list-style:none}",
    "summary::-webkit-details-marker{display:none}",
    ".lvl{font-size:12px;font-weight:600;min-width:58px}",
    "code{font:13px ui-monospace,SFMono-Regular,Menlo,monospace;overflow-wrap:anywhere}",
    ".fails{color:var(--muted);font-size:13px;overflow-wrap:anywhere}",
    ".rule{margin:8px 0 0 68px}.rule b{font-weight:600}",
    ".rule ul{margin:4px 0 0;padding-left:18px}.rule li{overflow-wrap:anywhere}",
    ".fix{color:var(--muted);font-size:13px}",
    "::slotted(*){display:block;max-width:1100px;margin:0 auto;padding:12px 16px 0;box-sizing:border-box}",
    "@media (max-width:600px){.rule{margin-left:0}summary{flex-wrap:wrap}.fails{flex-basis:100%}}",
    "</style>",
    '<slot name="header"></slot>',
    '<slot name="nav"></slot>',
    "<main>",
    '<h1 id="title"></h1>',
    '<div class="meta" id="meta"></div>',
    '<nav id="tabs" aria-label="Scorecards"></nav>',
    '<div id="cards"></div>',
    "</main>",
    '<slot name="footer"></slot>',
  ].join("");

  function el(t, a) {
    var e = document.createElement(t);
    var kids = Array.prototype.slice.call(arguments, 2).filter(function (k) {
      return k !== "";
    });
    Object.assign(e, a || {});
    kids.forEach(function (k) {
      e.append(k);
    });
    return e;
  }

  class ToolboxReport extends HTMLElement {
    connectedCallback() {
      if (this.shadowRoot) return;
      var root = this.attachShadow({ mode: "open" });
      root.append(template.content.cloneNode(true));
      var inline = this.querySelector('script[type="application/json"]');
      if (inline) {
        this.render(root, JSON.parse(inline.textContent), true);
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
          self.render(root, data, false);
        })
        .catch(function (e) {
          root.getElementById("meta").textContent = String(e);
        });
    }

    render(root, D, ownPage) {
      var color = function (l) {
        return "var(--" + (["none", "bronze", "silver", "gold", "skipped"].indexOf(l) >= 0 ? l : "accent") + ")";
      };
      if (ownPage) document.title = D.portal + " — scorecards";
      root.getElementById("title").textContent = D.portal;
      root.getElementById("meta").textContent =
        "Scorecards at snapshot " + D.snapshot + " · generated by toolbox report";
      var cards = Object.keys(D.scorecards);
      var active = location.hash.slice(1) || cards[0];
      var tabs = root.getElementById("tabs");
      function drawCard() {
        tabs.replaceChildren.apply(
          tabs,
          cards.map(function (c) {
            return el("button", {
              textContent: c,
              ariaPressed: String(c === active),
              onclick: function () {
                active = c;
                location.hash = c;
                drawCard();
              },
            });
          }),
        );
        var sc = D.scorecards[active];
        var rows = Object.entries(D.scores[active] || {});
        var order = ["none"].concat(sc.levels, ["skipped"]);
        var tally = {};
        rows.forEach(function (pair) {
          tally[pair[1].level] = (tally[pair[1].level] || 0) + 1;
        });
        var bar = el(
          "div",
          { className: "bar" },
          ...order
            .filter(function (l) {
              return tally[l];
            })
            .map(function (l) {
              return el("span", {
                style: "flex:" + tally[l] + ";background:" + color(l),
                title: l + ": " + tally[l],
              });
            }),
        );
        var legend = el(
          "div",
          { className: "legend" },
          ...order
            .filter(function (l) {
              return tally[l];
            })
            .map(function (l) {
              return el(
                "span",
                {},
                el("span", { className: "dot", style: "background:" + color(l) }),
                l + " " + tally[l],
              );
            }),
        );
        var list = el("div");
        var search = el("input", {
          type: "search",
          placeholder: "Filter by entity or failing rule…",
          ariaLabel: "Filter",
        });
        var draw = function () {
          var q = search.value.toLowerCase();
          var shown = rows
            .filter(function (pair) {
              return pair[1].level !== "skipped";
            })
            .sort(function (a, b) {
              return order.indexOf(a[1].level) - order.indexOf(b[1].level) || a[0].localeCompare(b[0]);
            })
            .filter(function (pair) {
              if (!q) return true;
              if (pair[0].toLowerCase().indexOf(q) >= 0) return true;
              return Object.entries(pair[1].rules || {}).some(function (ro) {
                return ro[1] !== "pass" && ro[0].indexOf(q) >= 0;
              });
            });
          list.replaceChildren.apply(
            list,
            shown.map(function (pair) {
                var id = pair[0];
                var r = pair[1];
                var failing = Object.entries(r.rules || {}).filter(function (ro) {
                  return ro[1] !== "pass";
                });
                var summary = el(
                  "summary",
                  {},
                  el("span", {
                    className: "lvl",
                    textContent: r.level,
                    style: "color:" + color(r.level),
                  }),
                  el("code", { textContent: id }),
                  el("span", {
                    className: "fails",
                    textContent: failing
                      .map(function (ro) {
                        return ro[0];
                      })
                      .join(", "),
                  }),
                );
                var body = failing.map(function (ro) {
                  var n = ro[0];
                  var o = ro[1];
                  var ev = (r.evidence || {})[n];
                  var items = ev == null ? [] : Array.isArray(ev) ? ev : [ev];
                  var rule = sc.rules[n] || {};
                  return el(
                    "div",
                    { className: "rule" },
                    el("div", {}, el("b", { textContent: n }), " (" + (rule.level || "") + ", " + o + ")"),
                    rule.fix ? el("div", { className: "fix", textContent: rule.fix }) : "",
                    items.length
                      ? el(
                          "ul",
                          {},
                          ...items.slice(0, 50).map(function (i) {
                            return el("li", {}, el("code", { textContent: typeof i === "string" ? i : JSON.stringify(i) }));
                          }),
                        )
                      : "",
                  );
                });
                var details = el("details", {}, summary);
                body.forEach(function (node) {
                  details.append(node);
                });
                return details;
              }),
          );
        };
        search.oninput = draw;
        draw();
        root.getElementById("cards").replaceChildren(
          el(
            "section",
            {},
            el(
              "div",
              { className: "head" },
              el("h2", { textContent: active }),
              el("span", {
                className: "sub",
                textContent: sc.kind + " · levels " + sc.levels.join(" < ") + " · owner " + sc.owner,
              }),
            ),
            bar,
            legend,
            search,
            list,
          ),
        );
      }
      drawCard();
    }
  }

  customElements.define("toolbox-report", ToolboxReport);
})();
