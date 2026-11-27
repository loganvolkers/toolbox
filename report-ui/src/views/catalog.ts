import type { ReportViews } from "../report-view";
import type { Summary } from "../lib/summary";
import type { ReportData } from "../types";
import { entitySource, sourceLabel } from "../lib/federation";
import { barEl, kindParts, legendEl, levelPill, unrunIn } from "../lib/bars";
import { basisBadge, basisCounts } from "../lib/basis";
import { judgedMark } from "../lib/compare";
import { el, enc, link } from "../lib/dom";
import { kindCount, kindDescription, kindGroupsOf, kindTitle, scorecardSplit } from "../lib/kinds";
import { fill, ownLevel } from "../lib/levels";
import { shortId, title } from "../lib/rings";
import { memberLinks } from "../lib/rules";
import { isFailing } from "../lib/status";
import { scoredKinds } from "../lib/summary";
import { rich } from "../lib/text";
import { unassessedIn } from "../lib/tracks";
import { firstSentence, fmt, plural, unscoredWhy } from "../lib/words";

// What sizes an entity's failure: the first line of evidence on its first failing measured rule.
function worstEvidence(D: ReportData, cols: string[], id: string) {
  for (var c of cols) {
    var s = ((D.scores || {})[c] || {})[id];
    if (!s) continue;
    for (var rule of Object.keys(s.rules || {})) {
      var ev = (s.evidence || {})[rule];
      if (isFailing(s.rules[rule]) && typeof (s.measures || {})[rule] === "number" && ev) return String(Array.isArray(ev) ? ev[0] : ev).split("\n")[0];
    }
  }
  return "";
}

export const catalogViews = {
  catalog(D: ReportData, S: Summary, kind: string | undefined, query: import("../lib/routes").Query, catalogIds?: ReadonlySet<string>) {
    query = query || {};
    var catalog = D.catalog || {};
    var groups = kindGroupsOf(D);
    var kinds = ([] as string[]).concat(
      ...groups.map(function (g) {
        return g.kinds;
      }),
    );
    if (!kinds.length) return el("section", {}, el("div", { className: "empty", textContent: "Catalog is empty in this report." }));
    const selected = kind && catalog[kind] ? kind : null;
    if (selected && D.sources) return federatedList(D, selected, query, catalogIds);
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
      el("p", { className: "lede", textContent: D.sources ? "Browse entities from the local portal and federated repositories by kind." : "Everything the portal derived from this repository, grouped by what it is about. The bar under a kind shows each entity's lowest level across the scorecards that rate it; pick a kind to list its entities." }),
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
    var search = el("input", { type: "search", placeholder: "Filter by name…", ariaLabel: "Filter " + kindTitle(D, selected) + " entities by name", value: query.q || "" });
    var list = el("div");
    var limit = 60;
    var drawList = function () {
      var q = search.value.toLowerCase();
      var ids = (catalog[selected] || []).filter(function (id) {
        return (!catalogIds || catalogIds.has(id)) && (!q || id.toLowerCase().includes(q));
      });
      if (!ids.length) {
        list.replaceChildren(
          el("div", { className: "empty", textContent: search.value ? "No entities match “" + search.value + "”." : "No entities match the filter." }),
          search.value ? el("button", {
            className: "link",
            textContent: "Clear filter",
            onclick: function () {
              search.value = "";
              search.dispatchEvent(new Event("input"));
            },
          }) : "",
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
      var rowsOut: { nodes: Node[]; n: number; rank: number }[] = [];
      // Furthest behind first, as on a scorecard page: rows at none, then rows below their top level.
      var rankOf = function (id) {
        return cols.reduce(function (n, c) {
          var s = ((D.scores || {})[c] || {})[id];
          if (!s || s.level === "skipped") return n;
          var lv = ((D.scorecards || {})[c] || {}).levels || [];
          return n + (s.level === "none" ? 1000 : 0) + (lv.length && s.level !== lv[lv.length - 1] ? 1 : 0);
        }, 0);
      };
      built.forEach(function (b) {
        if (!cols.length || sigCount[b.sig] < 5) {
          rowsOut.push({ nodes: [b.r], n: 1, rank: rankOf(b.id) });
          return;
        }
        if (grouped[b.sig]) return;
        grouped[b.sig] = true;
        var members = built.filter(function (x) {
          return x.sig === b.sig;
        });
        // A group that fails a measured rule (lines over a limit, findings) lists its furthest-off member first.
        var off = function (id) {
          return cols.reduce(function (n, c) {
            var s = ((D.scores || {})[c] || {})[id];
            return n + Object.keys((s && s.rules) || {}).reduce(function (m, rule) {
              var v = (s!.measures || {})[rule];
              return m + (isFailing(s!.rules[rule]) && typeof v === "number" ? v : 0);
            }, 0);
          }, 0);
        };
        var sized = members.some(function (x) {
          return off(x.id) > 0;
        });
        if (sized)
          members.sort(function (x, y) {
            return off(y.id) - off(x.id);
          });
        var worst = sized ? worstEvidence(D, cols, members[0].id) : "";
        var sum = el(
          "summary",
          { className: "erow cols", style: "grid-template-columns:" + tpl },
          el("div", { className: "name" }, kindCount(D, selected, members.length) + (cols.every(function (c) {
            var s = ((D.scores || {})[c] || {})[members[0].id];
            return !s || s.level === "skipped" || s.level === "none";
          }) && cols.some(function (c) {
            var s = ((D.scores || {})[c] || {})[members[0].id];
            return s && s.level === "none";
          }) ? " at none " : " at the same levels "), el("span", { className: "chev", ariaHidden: "true", textContent: "›" })),
          ...Array.from(b.r.querySelectorAll(".ecell"), function (c) {
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
            }), sized ? { order: "one file is furthest off", first: worst } : undefined),
          ],
          n: members.length,
          rank: rankOf(b.id),
        });
      });
      rowsOut.sort(function (a, b) {
        return b.rank - a.rank;
      });
      var paged = rowsOut.slice(0, limit);
      var hidden = rowsOut.slice(paged.length).reduce(function (n, o) {
        return n + o.n;
      }, 0);
      list.replaceChildren(
        ...(header ? [header] : []),
        ...paged.flatMap(function (o) {
          return o.nodes;
        }),
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
      const params = new URLSearchParams();
      if (search.value) params.set("q", search.value);
      if (query.filter) params.set("filter", query.filter);
      var want = "#catalog/" + enc(selected) + (params.size ? "?" + params : "");
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
  scopeNote(D: ReportData, kind, n) {
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
} satisfies ThisType<ReportViews>;

function federatedList(D: ReportData, kind: string, query: import("../lib/routes").Query, catalogIds?: ReadonlySet<string>) {
  const ids = D.catalog[kind] ?? [];
  const search = el("input", { type: "search", ariaLabel: "Filter entities by name", value: query.q ?? "" });
  const group = el("select", { ariaLabel: "Group entities" },
    el("option", { value: "", textContent: "Full list" }),
    el("option", { value: "repository", textContent: "By repository" }));
  group.value = query.group === "repository" ? query.group : "";
  const sources = [...new Set(ids.map(id => D.entitySources?.[id]?.source ?? ""))];
  const source = el("select", { ariaLabel: "Source repository" },
    el("option", { value: "", textContent: "All repositories" }),
    ...sources.filter(Boolean).map(key => el("option", { value: key, textContent: sourceLabel(D, key) })));
  source.value = query.source ?? "";
  const list = el("div");
  const kindCards = Object.keys(D.scorecards).filter(name => D.scorecards[name].kind === kind);
  const standard = kindCards.length === 1 ? kindCards[0] : undefined;
  const pageSize = 40;
  let limit = pageSize;
  const openGroups = new Set<string>();
  const coverage = new Map<string, number>();
  if (standard) for (const id of ids) {
    const level = typeof D.entities?.[id]?.fields.auditError === "string" ? "skipped"
      : D.scores[standard]?.[id]?.level ?? "skipped";
    coverage.set(level, (coverage.get(level) ?? 0) + 1);
  }
  const render = () => {
    const params = new URLSearchParams();
    if (search.value) params.set("q", search.value);
    if (group.value) params.set("group", group.value);
    if (source.value) params.set("source", source.value);
    if (query.filter) params.set("filter", query.filter);
    if (params.size || location.hash) {
      history.replaceState(history.state, "", "#catalog/" + enc(kind) + (params.size ? "?" + params : ""));
    }
    const q = search.value.trim().toLowerCase();
    const filtered = ids.filter(id => {
      const locator = D.entitySources?.[id];
      return (!catalogIds || catalogIds.has(id))
        && (!source.value || locator?.source === source.value)
        && (!q || [locator?.original ?? id, D.entityOverview?.[id]?.title, entitySource(D, id)]
          .some(value => typeof value === "string" && value.toLowerCase().includes(q)));
    });
    const row = (id: string) => {
      const raw = D.entitySources?.[id]?.original ?? id;
      const titleText = (D.entityOverview?.[id]?.title as string | undefined) ?? raw;
      const unavailable = kind === "Workflow" && typeof D.entities?.[id]?.fields.auditError === "string";
      return el("div", { className: "federated-catalog-row" },
        el("div", {}, link("entity/" + enc(id), "", titleText), el("small", { className: "sub", textContent: entitySource(D, id) })),
        el("ul", { className: standard ? "federated-catalog-score" : "" }, ...kindCards.flatMap(name => {
          const score = D.scores[name]?.[id];
          return score ? [el("li", {},
            standard ? null : link("scorecard/" + enc(name), "", title(D, name)), " ",
            unavailable ? el("span", { className: "sub", textContent: "Audit unavailable" })
              : levelPill(score.level, false, unrunIn(D.scorecards[name], score)),
          )] : [];
        })));
    };
    if (!filtered.length) {
      list.replaceChildren(el("p", { className: "empty", textContent: "No entities match." }),
        search.value || source.value ? el("button", { type: "button", textContent: "Clear name and source filters", onclick: () => {
          search.value = "";
          source.value = "";
          render();
        } }) : "");
      return;
    }
    let shownCount = Math.min(limit, filtered.length);
    if (group.value) {
      const groups = new Map<string, string[]>();
      for (const id of filtered) {
        const key = D.entitySources?.[id]?.source ?? "";
        if (!groups.has(key)) groups.set(key, []);
        groups.get(key)!.push(id);
      }
      shownCount = 0;
      list.replaceChildren(...Array.from(groups, ([key, members]) =>
        {
          const visible = members.slice(0, limit);
          shownCount += visible.length;
          return el("details", { open: openGroups.has(key), className: "federated-catalog-group",
            ontoggle: (event: Event) => {
              if ((event.currentTarget as HTMLDetailsElement).open) openGroups.add(key);
              else openGroups.delete(key);
            } },
            el("summary", { textContent: sourceLabel(D, key) + " · " + members.length }),
            ...visible.map(row));
        }));
    } else {
      list.replaceChildren(...filtered.slice(0, limit).map(row));
    }
    if (shownCount < filtered.length) list.append(el("button", {
      type: "button", textContent: "Show more (" + shownCount + " of " + filtered.length + ")",
      onclick: () => { limit += pageSize; render(); },
    }));
  };
  search.oninput = group.onchange = source.onchange = () => { limit = pageSize; render(); };
  render();
  const section = el("section", { id: "kind-list" },
    link("catalog", "crumb", "Catalog"),
    el("h2", { textContent: kindCount(D, kind, ids.length) }),
    el("p", { className: "sub", textContent: kindDescription(D, kind) }),
    standard ? el("div", { className: "sub" },
      link("scorecard/" + enc(standard), "", title(D, standard)),
      el("p", { textContent: ["none", ...D.scorecards[standard].levels, "skipped"]
        .filter(level => coverage.has(level)).map(level => fmt(coverage.get(level)) + " "
        + (level === "none" ? "below " + (D.scorecards[standard].levels[0] ?? "the first level")
          : level === "skipped" ? "not scored" : level)).join(" · ") })) : null,
    el("div", { className: "report-filters" },
      el("label", {}, "Filter entities", search),
      el("label", {}, "Source", source),
      el("label", {}, "View", group)),
    list);
  section.dataset.focus = "";
  return section;
}
