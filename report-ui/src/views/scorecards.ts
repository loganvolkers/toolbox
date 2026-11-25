import type { ReportViews } from "../report-view";
import type { Summary } from "../lib/summary";
import { noFacts } from "../lib/summary";
import type { ReportData } from "../types";
import { badge, barEl, grow, legendEl, levelLegend, levelOrder, levelPill, medal, splitUnassessed, tallyLevels, unrunIn } from "../lib/bars";
import { BASIS, basisBadge, basisCounts, basisIcon, basisInline, basisKey, basisOf, lockIcon, refs, whyEl } from "../lib/basis";
import { levelName, moverBits, nextLevel, scCaption, verifiedMover } from "../lib/campaigns";
import { cardName, judgedMark, keepWithLast } from "../lib/compare";
import { el, enc, KNOWN, link } from "../lib/dom";
import { kindCount, kindTitle } from "../lib/kinds";
import { ONLY_LEVEL, fill } from "../lib/levels";
import { capsRepo, isTop, shortId, title } from "../lib/rings";
import { trackPath } from "../lib/routes";
import { LADDER, floorOf, floorPill, isComparative, memberLinks, ruleLink, ruleTitle } from "../lib/rules";
import { blockers, isFailing, isPassing } from "../lib/status";
import { rich, richCards } from "../lib/text";
import { trackStats, trackTitle, tracksSorted, unassessedIn, unassessedNote } from "../lib/tracks";
import { fmt, plural, scPurpose, whyFirst } from "../lib/words";

export const scorecardsViews = {
  scorecardIndex(D: ReportData, S: Summary, query) {
    var self = this;
    var standards = S.rows.filter(function (r) {
      return !r.compare;
    });
    var factors = S.rows.length - standards.length;
    var groups = tracksSorted(D, S)
      .filter(function (t) {
        return !t.compare && t.cards.length;
      })
      .map(function (t) {
        return { name: t.name, title: t.def.title || t.name, rows: t.cards.map(function (c) { return c.row; }) };
      });
    var placed = {};
    groups.forEach(function (g) {
      g.rows.forEach(function (r) {
        placed[r.name] = true;
      });
    });
    var loose = standards.filter(function (r) {
      return !placed[r.name];
    });
    if (loose.length) groups.push({ name: null, title: "Not in a track", rows: loose });
    var search = el("input", { type: "search", placeholder: "Find a scorecard…", title: "Matches a scorecard's name, id, kind, or how it's decided", ariaLabel: "Find a scorecard by name, id, kind, or how it's decided", value: (query || {}).q || "" });
    var only = function (b) {
      return el(
        "button",
        {
          className: "link",
          onclick: function () {
            search.value = BASIS[b].label.toLowerCase();
            search.dispatchEvent(new Event("input"));
          },
        },
        basisIcon(b),
        " " + BASIS[b].label,
      );
    };
    var mixedBases = standards.some(function (r) {
      return basisCounts(r.sc.rules).judged;
    }) && standards.some(function (r) {
      return basisCounts(r.sc.rules).measured;
    });
    var anyScored = standards.some(function (r) {
      return r.scored > 0;
    });
    var basisPick = el("p", { className: "sub", style: "margin:6px 0 0" }, "Show only: ", only("judged"), " · ", only("measured"));
    var list = el("div");
    var row = function (r) {
      var floor = floorOf(D, r.name);
      var purpose = scPurpose(r.sc);
      return link("scorecard/" + enc(r.name), "sc-row", [
        medal(floor || "not scored", 26),
        el("div", { className: "sc-name" }, ...cardName(D, r.name)),
        el("div", { className: "sc-dist" }, barEl(splitUnassessed(D, r.name, r.parts)), el("small", {}, ...scCaption(D, r))),
        el("div", { className: "sc-weak" }, floor ? floorPill(D, r.name) : el("span", { className: "sub", textContent: "not scored" }), r.none ? el("small", { className: "bad", textContent: fmt(r.none) + " at none" }) : floor && capsRepo(D, r.name, floor) ? el("small", { className: "capnote" }, lockIcon(), " at its top level, so it caps the repository") : floor && isTop(D, r.name, floor) && !ONLY_LEVEL.has(floor) ? el("small", { className: "topmark", textContent: "✓ top level" }) : null),
        purpose ? el("span", { className: "sc-why", textContent: purpose }) : null,
      ]);
    };
    var draw = function () {
      var q = search.value.toLowerCase();
      var hit = function (r) {
        var basis: keyof typeof BASIS = basisCounts(r.sc.rules).judged ? "judged" : "measured";
        var basisWords = BASIS[basis].label.toLowerCase() + (basis === "judged" ? " judged by an agent assessed" : " measured deterministic mechanical");
        return !q || r.name.toLowerCase().indexOf(q) >= 0 || title(D, r.name).toLowerCase().indexOf(q) >= 0 || kindTitle(D, r.sc.kind || "").toLowerCase().indexOf(q) >= 0 || basisWords.indexOf(q) >= 0;
      };
      var shown = groups
        .map(function (g) {
          return { g: g, rows: g.rows.filter(hit) };
        })
        .filter(function (x) {
          return x.rows.length;
        });
      var want = "#scorecards" + (search.value ? "?q=" + encodeURIComponent(search.value) : "");
      if (location.hash !== want && history.replaceState) history.replaceState(history.state, "", want);
      if (!shown.length) {
        list.replaceChildren(
          el("div", { className: "empty" }, "No scorecards match “" + search.value + "”. ", el("button", {
            className: "link",
            textContent: "Clear search",
            onclick: function () {
              search.value = "";
              draw();
              search.focus();
            },
          })),
        );
        return;
      }
      // Without a search, scorecards whose every entity is at the top level fold under one row, so the
      // work left leads each group. One that caps the repository stays out: its top level is the blocker.
      var doneOf = function (r) {
        return !q && r.scored > 0 && r.topShare === 100 && !capsRepo(D, r.name, floorOf(D, r.name));
      };
      list.replaceChildren(
        ...shown.map(function (x) {
          var done = x.rows.filter(doneOf);
          if (done.length < 2) done = [];
          var open = x.rows.filter(function (r) {
            return done.indexOf(r) < 0;
          });
          // A long group shows its weakest few; the rest open in place, so each track's debt scans first.
          var SHOWN = 4;
          var rest = !q && open.length > SHOWN + 1 ? open.slice(SHOWN) : [];
          if (rest.length) open = open.slice(0, SHOWN);
          var restRows = el("div", { className: "sc-rest", hidden: true });
          var restBtn = rest.length
            ? el("button", {
                className: "link sc-more",
                ariaExpanded: "false",
                textContent: "Show the other " + plural(rest.length, "scorecard") + " in " + x.g.title,
                onclick: function (this: HTMLElement) {
                  var opening = this.getAttribute("aria-expanded") !== "true";
                  if (opening && !restRows.childElementCount) restRows.append(...rest.map(row));
                  restRows.hidden = !opening;
                  this.setAttribute("aria-expanded", String(opening));
                  this.textContent = opening ? "Show only the weakest " + SHOWN : "Show the other " + plural(rest.length, "scorecard") + " in " + x.g.title;
                  var first = opening && restRows.querySelector("a");
                  if (first) first.focus();
                },
              })
            : null;
          var byLevel = {};
          var topAt = {};
          var levelOrderSeen: any[] = [];
          x.rows.forEach(function (r) {
            var f = floorOf(D, r.name);
            if (!f) return;
            if (!byLevel[f]) levelOrderSeen.push(f);
            byLevel[f] = (byLevel[f] || 0) + 1;
            if (isTop(D, r.name, f) && !ONLY_LEVEL.has(f) && LADDER.indexOf(f) < LADDER.length - 1) topAt[f] = (topAt[f] || 0) + 1;
          });
          var mix = q
            ? ""
            : levelOrderSeen
                .sort(function (a, b) {
                  var ra = LADDER.indexOf(a);
                  var rb = LADDER.indexOf(b);
                  return (ra < 0 ? 9 : ra) - (rb < 0 ? 9 : rb);
                })
                .map(function (l) {
                  return byLevel[l] + " at " + levelName(l) + (topAt[l] ? " (" + topAt[l] + " at its top level)" : "");
                })
                .join(" · ");
          var fold: HTMLDetailsElement | null = null;
          if (done.length) {
            var body = el("div", { className: "sc-done-rows" });
            fold = el(
              "details",
              {
                className: "sc-done",
                ontoggle: function (this: HTMLDetailsElement) {
                  if (this.open && !body.childElementCount) body.append(...done.map(row));
                },
              },
              el("summary", {}, el("span", { className: "topmark", textContent: "✓ " + plural(done.length, "scorecard") + " with every entity at the top level" }), el("small", { textContent: done.map(function (r) { return title(D, r.name); }).join(", ") })),
              body,
            );
          }
          var lead = x.g.name ? verifiedMover(D, trackStats(D, S, x.g.name).cards) : null;
          return el(
            "div",
            { className: "sc-group" },
            groups.length === 1 && !x.g.name ? null : el("h3", {}, x.g.name ? link(trackPath(D, x.g.name), "", x.g.title) : x.g.title, el("span", { className: "sub", textContent: " · " + plural(x.rows.length, "scorecard") + (x.g.name ? ", weakest first" : "") + (mix ? "; by lowest level, " + mix : "") })),
            lead ? el("div", { className: "tnext", style: "margin:0 0 8px" }, el("b", { textContent: "Next: " }), ...moverBits(D, lead)) : null,
            el("div", { className: "sc-rows" }, el("div", { className: "sc-head", ariaHidden: "true" }, el("span"), el("span", { textContent: "Scorecard" }), el("span", { textContent: "Entities by level" }), el("span", { textContent: "Lowest level" })), ...open.map(row), restRows, restBtn ? el("div", { className: "more" }, restBtn) : null, fold),
          );
        }),
      );
    };
    search.oninput = draw;
    draw();
    const highestDebt = groups
      .map(group => ({ group, atNone: group.rows.filter(row => row.none > 0).length }))
      .sort((a, b) => b.atNone - a.atNone)[0];
    const debtSummary = groups.length > 1 && highestDebt?.atNone
      ? el("p", { className: "sc-debt" },
          el("button", {
            type: "button",
            className: "link",
            textContent: highestDebt.group.title,
            onclick: () => {
              search.value = "";
              draw();
              const group = list.querySelectorAll(".sc-group")[groups.indexOf(highestDebt.group)];
              const heading = group?.querySelector("h3");
              group?.scrollIntoView({ block: "start" });
              if (heading) {
                heading.tabIndex = -1;
                heading.focus({ preventScroll: true });
              }
            },
          }),
          " has the most scorecards with an entity at none (" +
            highestDebt.atNone + " of " + highestDebt.group.rows.length + ").")
      : null;
    return el(
      "section",
      { className: "sc-index" },
      el("div", { className: "head" }, el("h2", { textContent: "Scorecards" }), el("span", { className: "sub", textContent: plural(standards.length, "scorecard") + (groups.length > 1 ? " in " + plural(groups.length, "group") : "") })),
      debtSummary,
      el("p", { className: "lede" }, "Each scorecard rates every entity of one kind against levels of rules; the medal is its weakest entity's level, and the bar shows where the rest stand. " + (Object.keys(D.tracks || {}).length ? "They are grouped by track, the line of work each one serves. " : "") + "Open one for its failures, entities and what each level needs.", factors ? el("span", {}, " The " + plural(factors, "comparison factor") + " against a rival are on ", link("compare", "", "the Compare tab"), ".") : null),
      noFacts(S) ? el("div", { className: "next", style: "margin:8px 0" }, el("b", { textContent: "Nothing is scored yet. Next: " }), "run ", el("code", { textContent: "toolbox doctor" }), " to see which connector or tool is missing, then ", el("code", { textContent: "toolbox report" }), " again.") : null,
      anyScored ? el("details", { className: "keyd", open: !matchMedia("(max-width:640px)").matches }, el("summary", { textContent: "How to read the bars" }), levelLegend(D, standards)) : null,
      el("label", { className: "sc-search" }, el("span", { className: "sub" }, "Filter scorecards"), search),
      mixedBases ? basisPick : null,
      list,
    );
  },

  scorecard(D: ReportData, S: Summary, name, query) {
    query = query || {};
    var ranked = S.rows;
    var active = name && D.scorecards[name] ? name : ranked[0] && ranked[0].name;
    if (!active) return el("section", {}, el("div", { className: "empty", textContent: "No scorecards in this report." }));
    var self = this;
    var sc = D.scorecards[active];
    var rows = Object.entries(D.scores[active] || {});
    var order = levelOrder(sc.levels);
    // How far off an entity is: the measures of the rules it fails (lines over a limit, findings).
    var offBy = function (r) {
      return Object.keys(r.rules || {}).reduce(function (n, rule) {
        var v = (r.measures || {})[rule];
        return n + (isFailing(r.rules[rule]) && typeof v === "number" ? v : 0);
      }, 0);
    };
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
      var nStd = shown.filter(function (r) {
        return !r.compare;
      }).length;
      var items: Node[] = [];
      shown.forEach(function (r, i) {
        if (i === 0 && !r.compare) items.push(el("div", { className: "rank-head", textContent: plural(nStd, "scorecard") }));
        if (r.compare && (i === 0 || !shown[i - 1].compare)) items.push(el("div", { className: "rank-head" }, plural(shown.length - nStd, "comparison factor") + ", on ", link("compare", "", "the Compare tab →")));
        var c = r.compare ? (D.comparisons || {})[r.name] || {} : null;
        var verdict = c ? (c.verdict || {})[(c.peers || [])[0]] || "unknown" : null;
        var debt = c ? (verdict === "unknown" ? "not measured" : verdict) : r.none > 0 ? r.none + " at none" : r.bronze > 0 ? r.bronze + " at bronze" : r.scored && r.topShare === 100 ? "✓ top level" : r.scored ? "✓ clear" : "not scored";
        var capped = !c && !r.none && !r.bronze && r.scored && r.topShare === 100 && capsRepo(D, r.name, floorOf(D, r.name));
        var a = link("scorecard/" + enc(r.name), "", [el("span", {}, ...keepWithLast(title(D, r.name), judgedMark(D, r.name, true))), el("span", { className: "debt " + (c ? (verdict === "behind" ? "bad" : verdict === "ahead" ? "ok" : "") : r.none > 0 ? "bad" : r.bronze > 0 || !r.scored ? "" : capped ? "capnote" : "ok") }, ...(capped ? [lockIcon(), " caps the repository"] : [debt]))]);
        a.title = r.name + " · " + r.sc.kind;
        if (r.name === active) a.setAttribute("aria-current", "true");
        items.push(a);
      });
      cardNav.replaceChildren.apply(cardNav, items);
    };
    cardSearch.oninput = function () {
      drawCards();
      edges();
    };
    drawCards();
    var toTop = el("button", {
      className: "link rank-top",
      hidden: true,
      textContent: "↑ Back to the top of the list",
      onclick: function () {
        cardNav.scrollTop = 0;
        var first = cardNav.querySelector("a");
        if (first) first.focus({ preventScroll: true });
      },
    });
    var edges = function () {
      cardNav.classList.toggle("more", cardNav.scrollTop + cardNav.clientHeight < cardNav.scrollHeight - 4);
      cardNav.classList.toggle("less", cardNav.scrollTop > 4);
      toTop.hidden = cardNav.scrollTop <= 4;
    };
    cardNav.onscroll = edges;
    requestAnimationFrame(function () {
      var cur = cardNav.querySelector<HTMLElement>("[aria-current]");
      if (cur) {
        var at = cur.getBoundingClientRect().top - cardNav.getBoundingClientRect().top + cardNav.scrollTop;
        if (at + cur.offsetHeight > cardNav.clientHeight) {
          var want = at - cardNav.clientHeight / 3;
          var base = cardNav.getBoundingClientRect().top - cardNav.scrollTop;
          var row = Array.from(cardNav.querySelectorAll("a")).find(function (a) {
            return a.getBoundingClientRect().top - base >= want;
          });
          cardNav.scrollTop = row ? row.getBoundingClientRect().top - base - 4 : want;
        }
      }
      edges();
    });

    var levelFilter = query.level === "not-in-scope" || query.level === "skipped" ? "skipped" : query.level && order.indexOf(query.level) >= 0 ? query.level : "all";
    var outcomeFilter = query.outcome === "failing" || query.outcome === "passing" ? query.outcome : "all";
    var ruleFilter = query.rule && (sc.rules || {})[query.rule] ? query.rule : null;
    var search = el("input", { type: "search", placeholder: "Filter by entity or rule…", ariaLabel: "Filter entities", value: query.q || "" });
    var keepInUrl = function () {
      var qs = new URLSearchParams();
      if (levelFilter !== "all") qs.set("level", levelFilter === "skipped" ? "not-in-scope" : levelFilter);
      if (outcomeFilter !== "all") qs.set("outcome", outcomeFilter);
      if (ruleFilter) qs.set("rule", ruleFilter);
      if (search.value) qs.set("q", search.value);
      var tail = qs.toString();
      if (!tail && location.hash.indexOf("?") < 0) return;
      var want = "#scorecard/" + enc(active) + (tail ? "?" + tail : "");
      if (location.hash !== want && history.replaceState) history.replaceState(history.state, "", want);
    };
    var filters = el("div", { className: "filters" });
    var list = el("div");
    var limit = 12;
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
      ruleFilter = null;
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
              return { label: l === "skipped" ? "not in scope" : l, value: l };
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
        ruleFilter
          ? el(
              "button",
              {
                className: "chip-filter",
                ariaLabel: "Remove filter: not passing " + ruleFilter,
                onclick: function () {
                  ruleFilter = null;
                  redrawFilters();
                  draw();
                },
              },
              "Not passing ",
              el("b", { textContent: ruleFilter }),
              el("span", { ariaHidden: "true", textContent: " ×" }),
            )
          : "",
      );
    };
    var draw = function () {
      keepInUrl();
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
          if (!ruleFilter) return true;
          var o = (pair[1].rules || {})[ruleFilter];
          return isFailing(o) || o === "unknown";
        })
        .filter(function (pair) {
          if (outcomeFilter === "all") return true;
          var outs = Object.values(pair[1].rules || {});
          return outcomeFilter === "failing"
            ? outs.some(isFailing)
            : outs.every(function (o) {
                return isPassing(o) || o === "skipped";
              });
        })
        .sort(function (a, b) {
          return order.indexOf(a[1].level) - order.indexOf(b[1].level) || offBy(b[1]) - offBy(a[1]) || a[0].localeCompare(b[0]);
        });
      var sized = shown.some(function (pair) {
        return offBy(pair[1]) > 0;
      });
      if (!rows.length) {
        list.replaceChildren(el("div", { className: "empty" }, "No " + kindTitle(D, sc.kind).toLowerCase() + " has been collected yet. Run ", el("code", { textContent: "toolbox doctor" }), " to see whether a connector for it is listed under ", el("code", { textContent: "connectors:" }), " in the portal's ", el("code", { textContent: "toolbox.yaml" }), " and installed, then ", el("code", { textContent: "toolbox report" }), " again."));
        return;
      }
      if (!shown.length) {
        list.replaceChildren(
          el("div", { className: "empty", textContent: "No entities match these filters." }),
          el("button", { className: "link", textContent: "Clear filters", onclick: clearFilters }),
        );
        return;
      }
      var outOfScope = rows.filter(function (pair) {
        return pair[1].level === "skipped";
      }).length;
      list.replaceChildren(el("div", { className: "sub", style: "margin:6px 0", textContent: plural(shown.length, "entity", "entities") + (outcomeFilter === "failing" ? " failing a rule" : outcomeFilter === "passing" ? " passing every rule" : "") + (outOfScope && levelFilter === "all" && outcomeFilter === "all" && !ruleFilter && !q ? " in scope, of " + kindCount(D, sc.kind, rows.length) : "") + ", lowest level first" + (sized ? ", then furthest off" : "") }));
      var facts = function (pair) {
        var r = pair[1];
        var outs = Object.entries(r.rules || {});
        var next = nextLevel(sc, r.level);
        var need = next ? blockers(sc, r, next) : [];
        return [r.level, unassessedIn(D, active, pair[0]) ? 1 : 0, unrunIn(sc, r).length, need.join(","), outs.filter(function (ro) {
          return isFailing(ro[1]);
        }).length, outs.filter(function (ro) {
          return isPassing(ro[1]);
        }).length, outs.length].join("|");
      };
      // The furthest-off member of a group (members are sorted that way), with the evidence that sizes it.
      var worstOf = function (g) {
        var r = g[0][1];
        if (!(offBy(r) > 0)) return "";
        var rule = Object.keys(r.rules || {}).find(function (n) {
          return isFailing(r.rules[n]) && typeof (r.measures || {})[n] === "number";
        });
        var ev = rule && (r.evidence || {})[rule];
        return shortId(g[0][0]) + (ev ? ", " + String(Array.isArray(ev) ? ev[0] : ev).split("\n")[0] : "");
      };
      var groups: any[] = [];
      var byFacts = new Map();
      shown.forEach(function (pair) {
        var key = facts(pair);
        var g = byFacts.get(key);
        if (!g) {
          g = [];
          byFacts.set(key, g);
          groups.push(g);
        }
        g.push(pair);
      });
      var entRow = function (pair) {
        var id = pair[0];
        var r = pair[1];
        var outs = Object.entries(r.rules || {});
        var failing = outs.filter(function (ro) {
          return isFailing(ro[1]);
        });
        var passing = outs.filter(function (ro) {
          return isPassing(ro[1]);
        });
        var unrun = outs.filter(function (ro) {
          return ro[1] === "unknown";
        }).length;
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
            r.level === "skipped" ? badge(r.level) : levelPill(r.level, unassessedIn(D, active, id), unrunIn(sc, r), !unassessedIn(D, active, id)),
            el("span", { className: "name" }, short.replace(/[^\s\-/]+$/, ""), el("span", { className: (/[^\s\-/]{25,}$/.test(short) ? "tail long" : "tail") }, (/[^\s\-/]+$/.exec(short) || [""])[0], el("span", { className: "chev", textContent: "›" })), short !== id ? el("small", { textContent: id, title: id }) : null),
            r.level === "skipped"
              ? el("span", { className: "tally sub", textContent: "not rated: outside this scorecard's scope" })
              : el(
              "span",
              { className: "tally" },
              need.length
                ? el(
                    "b",
                    { className: r.level === "none" ? "bad" : "" },
                    (KNOWN.indexOf(next) >= 0 ? next.charAt(0).toUpperCase() + next.slice(1) : levelName(next)) + " needs: ",
                    ...need.slice(0, 2).map(function (n, i) {
                      return el("span", { className: "nr" }, ...keepWithLast(ruleTitle(active, n), basisBadge(basisOf((sc.rules || {})[n]), true)), i < Math.min(need.length, 2) - 1 ? ", " : "");
                    }),
                    need.length > 2 ? " +" + (need.length - 2) : "",
                  )
                : failing.length
                  ? ""
                  : "✓ all pass",
              later > 0 ? (need.length ? " · " : "") + later + " more for higher levels" : "",
              " · " + passing.length + " of " + outs.length + " pass",
              unrun ? el("span", { className: "provisional" }, " · " + unrun + " not run") : "",
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
              self.rulesFor(sc, r, active, unassessedIn(D, active, id), id),
              el("p", { style: "margin:12px 0 0" }, link("entity/" + enc(id), "", "Open " + short + " across every scorecard →")),
            ),
          );
        });
        return d;
      };
      var rowsShown = 0;
      var entsShown = 0;
      groups.forEach(function (g) {
        if (rowsShown >= limit) return;
        if (g.length < 3) {
          g.forEach(function (pair) {
            if (rowsShown >= limit) return;
            list.append(entRow(pair));
            rowsShown++;
            entsShown++;
          });
          return;
        }
        rowsShown++;
        entsShown += g.length;
        var first = entRow(g[0]).querySelector("summary") as HTMLElement;
        var grp = el(
          "details",
          { className: "ent ent-group" },
          el(
            "summary",
            {},
            first.children[0],
            el("span", { className: "name" }, kindCount(D, sc.kind, g.length) + " at " + (g[0][1].level === "none" ? "none" : g[0][1].level), el("span", { className: "tail" }, el("span", { className: "chev", textContent: "›" })), worstOf(g) ? el("small", { textContent: "one of them: " + worstOf(g) }) : null),
            first.children[2],
          ),
        );
        var built = false;
        grp.addEventListener("toggle", function () {
          if (!grp.open || built) return;
          built = true;
          grp.append(el("div", { className: "ent-members" }, ...g.map(entRow)));
        });
        list.append(
          grp,
          memberLinks(g.map(function (pair) {
            return pair[0];
          })),
        );
      });
      if (entsShown < shown.length) {
        list.append(
          el(
            "div",
            { className: "more" },
            el("button", {
              className: "link",
              textContent: "Show all " + fmt(shown.length) + " (" + fmt(shown.length - entsShown) + " more)",
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
    var failCounts: Record<string, number> = {};
    var unrunCounts = {};
    var provisionalN = 0;
    scored.forEach(function (pair) {
      Object.entries(pair[1].rules || {}).forEach(function (ro) {
        if (isFailing(ro[1])) failCounts[ro[0]] = (failCounts[ro[0]] || 0) + 1;
        else if (ro[1] === "unknown") unrunCounts[ro[0]] = (unrunCounts[ro[0]] || 0) + 1;
      });
      if (unrunIn(sc, pair[1]).length) provisionalN++;
    });
    var unrunNames = Object.keys(unrunCounts).sort();
    var unrunNote = unrunNames.length
      ? el(
          "p",
          { className: "sub unrun-note" },
          el("b", { className: "provisional", textContent: "Not run in this build: " }),
          ...unrunNames.map(function (n, i) {
            return el("span", { className: "nr" }, i ? ", " : "", ruleLink(active, n), " ", basisBadge(basisOf((sc.rules || {})[n]), "tip"), " (" + unrunCounts[n] + " of " + scored.length + ")");
          }),
          ". A check that was not run is neither a pass nor a failure, so it is left out of the bars above and of which rule to fix first. Levels that need one are provisional: they hold only if it passes.",
        )
      : null;
    var provisionalNote = provisionalN
      ? el("p", { className: "sub", style: "margin:6px 0 0" }, el("b", { className: "provisional", textContent: "Provisional: " }), plural(provisionalN, "entity's level rests", "entities' levels rest") + " on checks not run in this build (drawn dotted in the bar above and the list below). ", el("span", { textContent: "They hold only if those checks pass." }))
      : null;
    // The rule that holds the most entities at the lowest occupied level below the top.
    var lever: HTMLElement | null = null;
    var leverRule: string | null = null;
    var low = order.find(function (l) {
      return l !== "skipped" && nextLevel(sc, l) && parts.some(function (p) {
        return p.level === l;
      });
    });
    if (low) {
      var held: Record<string, number> = {};
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
      if (best) leverRule = best[0];
      var close = scored.filter(function (pair) {
        var nx = pair[1].level !== "skipped" && nextLevel(sc, pair[1].level);
        return nx && blockers(sc, pair[1], nx).length === 1;
      }).length;
      if (best)
        lever = el(
          "div",
          { className: "next", style: "margin:0 0 10px" },
          el("b", { textContent: "Biggest lever" }),
          el("span", { className: "sub", textContent: " (the rule that lifts the most entities a level)" }),
          el("br"),
          ruleLink(active, best[0]),
          " ",
          basisBadge(basisOf((sc.rules || {})[best[0]])),
          " keeps " + plural(best[1], "entity", "entities") + " at " + low + "; they need it for " + up + ". ",
          ...rich(((sc.rules || {})[best[0]] || {}).fix),
          close ? el("span", { className: "close-note" }, el("br"), el("b", { textContent: "Within reach: " }), plural(close, "entity is", "entities are") + " one rule from " + (close === 1 ? "its" : "their") + " next level.") : null,
        );
    }
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
                  search.value = "";
                  levelFilter = "all";
                  if (outcomeFilter === "passing") outcomeFilter = "all";
                  ruleFilter = f[0];
                  limit = 25;
                  redrawFilters();
                  draw();
                  filters.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
                  var chip = filters.querySelector<HTMLElement>(".chip-filter");
                  if (chip) chip.focus({ preventScroll: true });
                },
              },
              el("span", { className: "rn" }, el("span", { textContent: ruleTitle(active, f[0]) }), basisBadge(basisOf((sc.rules || {})[f[0]]), false, true), rl ? badge(rl) : null, lever && f[0] === leverRule ? el("span", { className: "pill", title: "Fixing it lifts the most entities a level", textContent: "fix first" }) : null),
              el("span", { className: "bar xs" }, fillSpan, el("span", { style: "flex-grow:" + (100 - share) + ";background:transparent" })),
              el("span", { className: "fc" }, f[1] + " of " + scored.length, el("span", { className: "sr", textContent: " fail it. Show them." })),
            );
            return b;
          }),
        )
      : !scored.length
        ? el("p", { className: "empty", textContent: "No entity is scored yet, so no rule has failed or passed." })
        : el("p", { className: "empty good", textContent: unrunNames.length ? "✓ No rule fails on any entity; some checks were not run." : "✓ No rule fails on any entity." });

    var needs = el("div", { className: "levels-need" });
    (sc.levels || []).forEach(function (l) {
      var names = Object.keys(sc.rules || {})
        .filter(function (n) {
          return sc.rules[n].level === l;
        })
        .sort();
      var bases = names.map(function (n) {
        return basisOf(sc.rules[n]);
      });
      var shared = bases.length && bases.every(function (b) {
        return b === bases[0];
      })
        ? bases[0]
        : null;
      needs.append(
        el(
          "div",
          { className: "need" },
          el("div", { className: "need-head" }, badge(l), shared ? el("span", { className: "need-basis" }, names.length > 1 ? "All " + names.length + " rules" : "1 rule", basisBadge(shared, "tip")) : null),
          names.length
            ? el(
                "ul",
                {},
                ...names.map(function (n) {
                  var rd = sc.rules[n];
              return el("li", {}, el("span", { style: "display:inline-flex;flex-wrap:wrap;gap:4px 6px;align-items:center" }, el("b", {}, ruleLink(active, n, null, true)), shared ? null : basisBadge(basisOf(rd), "tip")), rd.rationale || rd.fix ? whyFirst(rd.rationale || rd.fix) : null, refs(rd.links));
                }),
              )
            : el("div", { className: "sub", textContent: "No rules at this level." }),
        ),
      );
    });
    // Reasons are clamped to three lines so the levels read side by side; the full text stays in the
    // accessibility tree and one toggle opens them all.
    needs.classList.add("clamped");
    var needsToggle = el("button", {
      className: "link need-toggle",
      textContent: "Show full reasons, with thresholds and sources",
      ariaExpanded: "false",
      onclick: function () {
        var open = needs.classList.toggle("clamped") === false;
        needsToggle.textContent = open ? "Show shorter reasons" : "Show full reasons, with thresholds and sources";
        needsToggle.setAttribute("aria-expanded", String(open));
      },
    });
    var picker = el(
      "select",
      {
        ariaLabel: "Scorecard",
        onchange: function () {
          location.hash = "scorecard/" + enc(picker.value);
        },
      },
      ...[false, true]
        .map(function (cmp) {
          var group = ranked.filter(function (r) {
            return !!r.compare === cmp;
          });
          if (!group.length) return null;
          return el(
            "optgroup",
            { label: cmp ? plural(group.length, "comparison factor") : plural(group.length, "scorecard") },
            ...group.map(function (r) {
              var c = cmp ? (D.comparisons || {})[r.name] || {} : null;
              var debt = c ? (c.verdict || {})[(c.peers || [])[0]] || "not measured" : r.none > 0 ? r.none + " at none" : r.bronze > 0 ? r.bronze + " at bronze" : r.scored && r.topShare === 100 ? "top level" : r.scored ? "clear" : "not scored";
              return el("option", { value: r.name, selected: r.name === active, textContent: title(D, r.name) + " (" + debt + ")" });
            }),
          );
        })
        .filter(Boolean),
    );
    var owner = sc.owner ? sc.owner.replace(/^group:/, "") : "";
    var hStand = el("h3", { textContent: "What holds it where it is", tabIndex: -1 });
    var hFail = el("h3", { textContent: scored.length ? "Most common failures, at any level" : "Nothing has failed or passed", tabIndex: -1 });
    var hNeeds = el("h3", { textContent: "What each level needs (each level also needs the ones below)", tabIndex: -1 });
    var hList = el("h3", { textContent: "Entities", tabIndex: -1 });
    var jump = function (label, target) {
      return el("button", {
        className: "link",
        textContent: label,
        onclick: function () {
          target.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
          target.focus({ preventScroll: true });
        },
      });
    };
    var standNotes = [unassessedNote(D, active, scored.length), provisionalNote, lever ? el("div", { style: "margin-top:12px" }, lever) : null].filter(Boolean);
    var jumpBar = el("nav", { className: "jumpbar", ariaLabel: "On this page" }, el("span", { className: "sub", textContent: "On this page:" }), standNotes.length ? jump("What holds it", hStand) : null, jump(scored.length ? "Failures" : "Results", hFail), jump("Filter entities (" + fmt(scored.length) + ")", hList), jump("What each level needs", hNeeds));
    var floorHere = floorOf(D, active);
    var topHere = order.filter(function (l) {
      return l !== "skipped" && l !== "none";
    }).pop();
    var atTop = scored.filter(function (pair) {
      return pair[1].level === topHere;
    }).length;
    var split = splitUnassessed(D, active, parts);
    var statusLegend = legendEl(parts);
    if (split.some(function (p) { return p.hollow; })) statusLegend.append(el("span", {}, el("span", { className: "dot hatch", style: "--seg:" + fill("bronze") }), "hatched: not assessed"));
    if (split.some(function (p) { return p.unrun; })) statusLegend.append(el("span", {}, el("span", { className: "dot dots", style: "--seg:" + fill("silver") }), "◌ dotted: provisional, checks not run"));
    var status = el(
      "div",
      { className: "sc-status" },
      medal(floorHere || (scored.length ? "none" : "not scored"), 44),
      el(
        "div",
        { className: "sc-status-b" },
        el(
          "div",
          { className: "sc-status-t" },
          el("span", { className: "sub", textContent: scored.length ? "Lowest level " : "" }),
          floorHere ? floorPill(D, active) : el("b", { textContent: scored.length ? "not scored" : "No entities collected yet" }),
          floorHere && isTop(D, active, floorHere) && !ONLY_LEVEL.has(floorHere) ? el("span", { className: "topmark", textContent: " ✓ top level" }) : null,
          topHere && scored.length ? el("span", { className: "sep", ariaHidden: "true", textContent: "·" }) : null,
          topHere && scored.length ? el("span", { className: "fact", textContent: fmt(atTop) + " of " + fmt(scored.length) + " at the top level (" + levelName(topHere) + ")" }) : null,
          leverRule ? el("span", { className: "sep", ariaHidden: "true", textContent: "·" }) : null,
          leverRule ? el("span", { className: "fact" }, "fix first: ", el("button", { className: "link", textContent: ruleTitle(active, leverRule), onclick: function () {
            hStand.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
            hStand.focus({ preventScroll: true });
          } })) : null,
        ),
        barEl(split, "lg"),
        statusLegend,
      ),
    );
    var whyEl: HTMLElement | null = null;
    if (sc.rationale || (sc.links || []).length) {
      var whyText = sc.rationale ? el("div", { className: "why-text" }, ...richCards(D, sc.rationale, active)) : null;
      var long = String(sc.rationale || "").length > 260;
      if (long && whyText) whyText.classList.add("clamped");
      whyEl = el(
        "div",
        { className: "rationale" },
        el("b", { textContent: "Why this scorecard" }),
        whyText,
        long
          ? el("button", {
              className: "link why-more",
              textContent: "Read the full why",
              ariaExpanded: "false",
              onclick: function (e) {
                var open = (whyText as HTMLElement).classList.toggle("clamped") === false;
                e.currentTarget.textContent = open ? "Show less" : "Read the full why";
                e.currentTarget.setAttribute("aria-expanded", String(open));
              },
            })
          : null,
        refs(sc.links),
      );
    }
    var anyJudged = ranked.some(function (r) {
      return basisCounts(((D.scorecards || {})[r.name] || {}).rules).judged;
    });
    return el(
      "div",
      { className: "split" },
      el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "Scorecards" })),
        el("p", { className: "sub", style: "margin:0" }, ranked.some(function (r) {
          return r.scored;
        }) ? "Most entities at none first" : "Nothing is scored yet", anyJudged ? el("span", {}, " · ", el("span", { className: "bicon judged" }, basisIcon("judged")), " has agent-judged rules · ", el("span", { className: "bicon" }, basisIcon("measured")), " tool checks only") : null),
        cardSearch,
        toTop,
        cardNav,
      ),
      el(
        "section",
        { className: basisCounts(sc.rules).judged || !basisCounts(sc.rules).measured ? "" : "allmeasured" },
        el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("scorecards", "", "Scorecards"), " / " + title(D, active)),
        el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Scorecard" }), picker),
        el("div", { className: "head" }, el("h2", { textContent: title(D, active) }), el("code", { className: "sub", textContent: active })),
        status,
        jumpBar,
        el("p", { className: "lede" }, ...richCards(D, sc.description, active)),
        whyEl,
        el(
          "div",
          { className: "sub" },
          "Rates every " + kindTitle(D, sc.kind).replace(/^[A-Z](?=[a-z]|$)/, function (c) {
            return c.toLowerCase();
          }) + (owner ? " · owned by " + owner : ""),
          sc.track ? el("span", {}, " · track ", link(trackPath(D, sc.track), "", trackTitle(D, sc.track))) : null,
          basisInline(basisCounts(sc.rules)),
        ),
        isComparative(D, active) ? el("div", { className: "callout" }, el("span", {}, "This scorecard compares products rather than levelling them. ", link("compare/" + enc(active), "", "See how they stand →"))) : null,
        basisInline(basisCounts(sc.rules)) ? null : basisKey(basisCounts(sc.rules)),
        standNotes.length ? hStand : null,
        ...standNotes,
        hFail,
        failChart && rows.length ? el("p", { className: "sub", style: "margin:0 0 6px", textContent: "Each bar is the share of entities failing the rule; the pill beside it names the level the rule is needed for." }) : null,
        failChart,
        unrunNote,
        hList,
        rows.length ? filters : null,
        rows.length ? search : null,
        list,
        hNeeds,
        needsToggle,
        needs,
      ),
    );
  },
} satisfies ThisType<ReportViews>;
