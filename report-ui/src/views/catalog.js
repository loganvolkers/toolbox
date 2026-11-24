// catalog, scopeNote: methods of <toolbox-report>, installed on its prototype by element.js.
import { barEl, kindParts, legendEl, levelPill, unrunIn } from "../lib/bars.js";
import { basisBadge, basisCounts } from "../lib/basis.js";
import { judgedMark } from "../lib/compare.js";
import { el, enc, link } from "../lib/dom.js";
import { kindCount, kindDescription, kindGroupsOf, kindTitle, scorecardSplit } from "../lib/kinds.js";
import { fill, ownLevel } from "../lib/levels.js";
import { shortId, title } from "../lib/rings.js";
import { memberLinks } from "../lib/rules.js";
import { template } from "../lib/styles.js";
import { scoredKinds } from "../lib/summary.js";
import { rich } from "../lib/text.js";
import { unassessedIn } from "../lib/tracks.js";
import { firstSentence, fmt, plural, unscoredWhy } from "../lib/words.js";

export const catalogViews = {
  catalog(D, S, kind, query) {
    query = query || {};
    var catalog = D.catalog || {};
    var groups = kindGroupsOf(D);
    var kinds = [].concat.apply(
      [],
      groups.map(function (g) {
        return g.kinds;
      }),
    );
    if (!kinds.length) return el("section", {}, el("div", { className: "empty", textContent: "Catalog is empty in this report." }));
    var selected = kind && catalog[kind] ? kind : null;
    var scored = scoredKinds(D);
    var compared = {};
    Object.keys(D.comparisons || {}).forEach(function (n) {
      var k = ((D.scorecards || {})[n] || {}).kind;
      if (k) compared[k] = true;
    });
    var groupOf = {};
    groups.forEach(function (g) {
      g.kinds.forEach(function (k) {
        groupOf[k] = g;
      });
    });
    var tile = function (k) {
        var ids = catalog[k];
        var unscored = scored[k]
          ? ids.filter(function (id) {
              return !S.worst[id];
            }).length
          : 0;
        var a = link("catalog/" + enc(k), "kind", [
          el("strong", {}, kindTitle(D, k)),
          kindDescription(D, k) ? el("small", {}, ...rich(firstSentence(kindDescription(D, k)))) : null,
          el("span", { textContent: plural(ids.length, "entity", "entities") + scorecardSplit(D, k) + (scored[k] ? (unscored ? " · " + fmt(unscored) + " not scored" : "") : compared[k] ? " · compared" : " · not scored") }),
          unscored * 10 > ids.length ? unscoredWhy(kindDescription(D, k)) : null,
        ]);
        if (kindDescription(D, k)) a.title = kindDescription(D, k).replace(/`/g, "");
        if (scored[k]) {
          var parts = kindParts(S, ids);
          if (parts.length) a.append(barEl(parts, "xs"));
          var byLevel = {};
          parts.forEach(function (p) {
            if (p.level !== "unscored") byLevel[p.level] = (byLevel[p.level] || 0) + p.n;
          });
          var mix = Object.keys(byLevel).map(function (l) {
            return fmt(byLevel[l]) + " " + ownLevel(D, k, l);
          });
          if (mix.length) a.append(el("small", { className: "kmix", textContent: mix.join(" · "), title: "Each entity counted at its weakest level" }));
        }
        if (k === selected) a.setAttribute("aria-current", "true");
        return a;
    };
    var grid = el(
      "div",
      {},
      ...groups.map(function (g) {
        return el(
          "div",
          { className: "kgroup" },
          el("div", { className: "kgh" }, el("h3", { textContent: g.title }), el("span", { className: "sub", textContent: plural(g.count, "entity", "entities") + (g.kinds.length > 1 ? ": " + g.kinds.map(function (k) { return fmt(((D.catalog || {})[k] || []).length) + " " + kindTitle(D, k); }).join(" · ") : "") })),
          g.description ? el("p", { className: "sub kgd", textContent: g.description }) : null,
          el("div", { className: "kinds" }, ...g.kinds.map(tile)),
        );
      }),
    );
    var top = el(
      "section",
      {},
      el("div", { className: "head" }, el("h2", { textContent: "Catalog" }), el("span", { className: "sub", textContent: fmt(S.entityCount) + " entities in " + kinds.length + " kinds and " + plural(groups.length, "group") })),
      el("p", { className: "lede", textContent: "Everything the portal derived from this repository, grouped by what it is about. The bar under a kind shows each entity's lowest level across the scorecards that rate it; pick a kind to list its entities." }),
      el(
        "div",
        { className: "legend", style: "margin:0 0 12px" },
        ...["none", "bronze", "silver", "gold", "unscored"].map(function (l) {
          return el("span", {}, l === "unscored" ? el("span", { className: "dot unscored" }) : el("span", { className: "dot", style: "background:" + fill(l) }), l === "unscored" ? "not scored: out of every scorecard's scope, or none has run on it" : l);
        }),
        [...S.worstState.values()].indexOf("hollow") >= 0 ? el("span", {}, el("span", { className: "dot hatch", style: "--seg:" + fill("bronze") }), "hatched: not assessed by an agent yet") : null,
        [...S.worstState.values()].indexOf("unrun") >= 0 ? el("span", {}, el("span", { className: "dot dots", style: "--seg:" + fill("silver") }), "◌ dotted: provisional, checks not run in this build") : null,
      ),
      grid,
    );
    if (!selected) return top;
    var search = el("input", { type: "search", placeholder: "Filter " + selected + " by name…", ariaLabel: "Filter entities", value: query.q || "" });
    var list = el("div");
    var limit = 60;
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
              search.oninput();
            },
          }),
        );
        return;
      }
      var shown = ids;
      var kindCards = Object.keys(D.scorecards || {})
        .filter(function (n) {
          return (catalog[selected] || []).some(function (id) {
            return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
          });
        })
        .sort(function (a, b) {
          return title(D, a).localeCompare(title(D, b));
        });
      var cols = kindCards.slice(0, 4);
      var allUnrun = {};
      cols.forEach(function (n) {
        var inScope = (catalog[selected] || []).filter(function (id) {
          return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
        });
        allUnrun[n] = inScope.length > 1 && inScope.every(function (id) {
          return unrunIn(D.scorecards[n], D.scores[n][id]).length > 0;
        });
      });
      var tpl = "minmax(0,1fr)" + (cols.length ? " repeat(" + cols.length + ",minmax(96px,176px))" : " auto");
      var header = cols.length
        ? el(
            "div",
            { className: "ehdr", style: "grid-template-columns:" + tpl, ariaHidden: "true" },
            el("span", { textContent: kindTitle(D, selected) }),
            ...cols.map(function (n) {
              var mark = judgedMark(D, n);
              return el("span", { title: n }, title(D, n), mark ? " " : null, mark, allUnrun[n] ? el("span", { className: "unassessed-tag unrun", style: "display:block", title: "Every level in this column rests on checks not run in this build: it holds only if they pass" }, "all provisional: not run") : null);
            }),
          )
        : null;
      var rowOf = function (id) {
        var short = shortId(id);
        var r = link("entity/" + enc(id), "erow cols", [
          el("div", { className: "name" }, short, short !== id ? el("small", { textContent: id, title: id }) : null),
          ...(cols.length
            ? cols.map(function (n) {
                var sc = D.scores[n] && D.scores[n][id];
                var lv = sc && sc.level !== "skipped" ? sc.level : null;
                var cell = el(
                  "div",
                  { className: "ecell" },
                  lv ? levelPill(lv, unassessedIn(D, n, id), unrunIn(D.scorecards[n], sc), allUnrun[n]) : el("span", { className: "sub", title: "Not in this scorecard's scope", textContent: "not in scope" }),
                );
                cell.dataset.sc = title(D, n);
                if (basisCounts(((D.scorecards || {})[n] || {}).rules).judged) cell.dataset.judged = "";
                return cell;
              })
            : [el("div", { className: "badges" }, el("span", { className: "sub", textContent: "not scored" }))]),
        ]);
        r.style.gridTemplateColumns = tpl;
        var lvls = cols
          .filter(function (n) {
            return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
          })
          .map(function (n) {
            return title(D, n) + ": " + D.scores[n][id].level;
          });
        // The cells repeat what the column headers say; a screen reader hears one sentence instead.
        if (lvls.length) {
          r.querySelectorAll(".ecell").forEach(function (c) {
            c.setAttribute("aria-hidden", "true");
          });
          r.append(el("span", { className: "sr", textContent: ". " + lvls.join(", ") }));
        }
        return r;
      };
      var built = shown.map(function (id) {
        var r = rowOf(id);
        return {
          id: id,
          r: r,
          sig: Array.prototype.map
            .call(r.querySelectorAll(".ecell"), function (c) {
              return c.innerHTML;
            })
            .join("|"),
        };
      });
      var sigCount = {};
      built.forEach(function (b) {
        sigCount[b.sig] = (sigCount[b.sig] || 0) + 1;
      });
      var grouped = {};
      var rowsOut = [];
      built.forEach(function (b) {
        if (!cols.length || sigCount[b.sig] < 5) {
          rowsOut.push({ nodes: [b.r], n: 1 });
          return;
        }
        if (grouped[b.sig]) return;
        grouped[b.sig] = true;
        var members = built.filter(function (x) {
          return x.sig === b.sig;
        });
        var sum = el(
          "summary",
          { className: "erow cols", style: "grid-template-columns:" + tpl },
          el("div", { className: "name" }, kindCount(D, selected, members.length) + " with the same result ", el("span", { className: "chev", ariaHidden: "true", textContent: "›" })),
          ...Array.prototype.map.call(b.r.querySelectorAll(".ecell"), function (c) {
            return c.cloneNode(true);
          }),
        );
        rowsOut.push({
          nodes: [
            el("details", { className: "egroup" }, sum, el("div", { className: "egroup-members" }, ...members.map(function (x) {
              return x.r;
            }))),
            memberLinks(members.map(function (x) {
              return x.id;
            })),
          ],
          n: members.length,
        });
      });
      var paged = rowsOut.slice(0, limit);
      var hidden = rowsOut.slice(paged.length).reduce(function (n, o) {
        return n + o.n;
      }, 0);
      list.replaceChildren.apply(
        list,
        (header ? [header] : []).concat(
          paged.flatMap(function (o) {
            return o.nodes;
          }),
        ),
      );
      var judgedCols = cols.filter(function (n) {
        return basisCounts(((D.scorecards || {})[n] || {}).rules).judged > 0;
      });
      var unrunCols = cols.filter(function (n) {
        return allUnrun[n];
      });
      var dashed = unrunCols.length ? "Every level on the " + unrunCols.map(function (n) { return title(D, n); }).join(" and ") + (unrunCols.length > 1 ? " scorecards" : " scorecard") + " is provisional here: checks it needs were not run in this build." : "";
      var hollowJ = list.querySelector(".lvl.vacuous.unjudged");
      var hollowR = list.querySelector(".lvl.vacuous.unrun");
      if ((hollowJ || hollowR) && header)
        header.after(
          el(
            "div",
            { className: "hollow-key" },
            el("b", { textContent: "Levels that may not hold:" }),
            hollowJ ? el("span", {}, levelPill("bronze", true, null).firstChild, " no agent has reviewed it yet") : null,
            hollowR ? el("span", {}, levelPill("bronze", false, ["a check"], true).firstChild, " checks it needs were not run in this build") : null,
          ),
        );
      if (judgedCols.length || dashed)
        list.prepend(
          el(
            "div",
            { className: "sub basis-note" },
            judgedCols.length
              ? el(
                  "p",
                  {},
                  el("b", { textContent: "How it's decided: " }),
                  ...cols.flatMap(function (n, i) {
                    var bc = basisCounts(((D.scorecards || {})[n] || {}).rules);
                    var all = bc.measured + bc.judged;
                    return [i ? " · " : "", title(D, n) + " ", basisBadge(bc.judged ? "judged" : "measured"), " " + (bc.judged === all || !bc.judged ? "all " + plural(all, "rule") : bc.judged + " of " + plural(all, "rule") + ", the rest tool-checked")];
                  }),
                )
              : null,
            dashed ? el("p", {}, el("b", { textContent: "Has it run: " }), dashed) : null,
          ),
        );
      if (kindCards.length > cols.length)
        list.append(el("p", { className: "sub", textContent: plural(kindCards.length - cols.length, "more scorecard") + " rate " + selected + "; open an entity to see them." }));
      if (hidden) {
        list.append(
          el(
            "div",
            { className: "more" },
            el("button", {
              className: "link",
              textContent: "Show all " + fmt(ids.length) + " (" + fmt(hidden) + " more)",
              onclick: function () {
                limit = Infinity;
                drawList();
              },
            }),
          ),
        );
      }
    };
    search.oninput = function () {
      var want = "#catalog/" + enc(selected) + (search.value ? "?" + new URLSearchParams({ q: search.value }) : "");
      if (location.hash !== want && history.replaceState) history.replaceState(history.state, "", want);
      drawList();
    };
    drawList();
    var levels = (scored[selected] ? kindParts(S, catalog[selected] || []) : []).map(function (p) {
      return Object.assign({}, p, { label: ownLevel(D, selected, p.level) });
    });
    var nScored = (catalog[selected] || []).filter(function (id) {
      return S.worst[id];
    }).length;
    var nAll = (catalog[selected] || []).length;
    var kpick = el(
      "select",
      {
        ariaLabel: "Kind",
        onchange: function () {
          location.hash = "catalog/" + enc(kpick.value);
        },
      },
      ...groups.map(function (g) {
        return el(
          "optgroup",
          { label: g.title },
          ...g.kinds.map(function (k) {
            return el("option", { value: k, selected: k === selected, textContent: kindTitle(D, k) + " (" + fmt((catalog[k] || []).length) + ")" });
          }),
        );
      }),
    );
    var listSection = el(
      "section",
      { id: "kind-list" },
      el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Kind" }), kpick),
      el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("catalog", "", "Catalog"), " / " + (groupOf[selected] && groupOf[selected].name ? groupOf[selected].title + " / " : "") + kindTitle(D, selected)),
      el("div", { className: "head" }, el("div", {}, el("h2", { textContent: kindTitle(D, selected), tabIndex: -1 }), kindTitle(D, selected) !== selected ? el("code", { className: "kid", textContent: selected }) : null), el("span", { className: "sub", textContent: (groupOf[selected] && groupOf[selected].name ? groupOf[selected].title + " · " : "") + plural(nAll, "entity", "entities") + scorecardSplit(D, selected) + (scored[selected] && nScored < nAll ? " · " + fmt(nScored) + " scored · " + fmt(nAll - nScored) + " not scored" : "") })),
      kindDescription(D, selected) ? el("p", { className: "lede" }, ...rich(kindDescription(D, selected))) : null,
      scored[selected] && nScored < nAll ? this.scopeNote(D, selected, nAll - nScored) : null,
      levels.length
        ? el("div", {}, barEl(levels), legendEl(levels))
        : compared[selected]
          ? el("p", { className: "sub" }, "Compared rather than levelled: ", link("compare", "", "see how they stand →"))
          : el("p", { className: "sub", textContent: "No scorecard rates this kind." }),
      search,
      list,
    );
    listSection.dataset.focus = "";
    // With a kind open, the kind grid folds to one chip row per group so the list starts near the top.
    var chips = el(
      "section",
      { className: "kchips-wrap" },
      el("div", { className: "head" }, el("h2", { textContent: "Catalog" }), el("span", { className: "sub" }, fmt(S.entityCount) + " entities in " + kinds.length + " kinds · ", link("catalog", "", "all kinds, with descriptions"))),
      ...groups.map(function (g) {
        return el(
          "div",
          { className: "kchips" },
          el("span", { className: "kchips-g", textContent: g.title }),
          el(
            "div",
            { className: "box" },
            ...g.kinds.map(function (k) {
              var a = link("catalog/" + enc(k), "", [kindTitle(D, k), el("b", { textContent: fmt((catalog[k] || []).length) })]);
              if (k === selected) a.setAttribute("aria-current", "true");
              return a;
            }),
          ),
        );
      }),
    );
    return el("div", {}, chips, listSection);
  },

  // Why some entities of a rated kind carry no level: each scorecard on the kind that limits itself with appliesTo.
  scopeNote(D, kind, n) {
    var scoped = Object.entries(D.scorecards || {}).filter(function (e) {
      return e[1].kind === kind && e[1].appliesTo;
    });
    var who = plural(n, "entity is", "entities are");
    if (!scoped.length) return el("p", { className: "sub", textContent: fmt(n) + " " + who.replace(/^\d+ /, "") + " not scored: no scorecard on this kind has produced a level for them in this build." });
    return el(
      "p",
      { className: "sub scope-note" },
      el("b", { textContent: who.replace(/^\d+/, fmt(n)) + " not scored" }),
      ": they fall outside the scope that ",
      ...scoped.flatMap(function (e, i) {
        return [i ? (i === scoped.length - 1 ? " and " : ", ") : "", link("scorecard/" + enc(e[0]), "", e[1].title || e[0])];
      }),
      " sets for itself. ",
      el(
        "details",
        { className: "scope-rule" },
        el("summary", { textContent: "Show the scope rule" }),
        ...scoped.map(function (e) {
          return el("div", {}, el("span", { textContent: (e[1].title || e[0]) + " applies where " }), el("code", { textContent: e[1].appliesTo }));
        }),
      ),
    );
  },
};
