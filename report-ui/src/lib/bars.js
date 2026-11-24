import { levelName } from "./campaigns.js";
import { el, svg } from "./dom.js";
import { LEVEL_NOTE, ONLY_LEVEL, fill, ink, known } from "./levels.js";
import { title } from "./rings.js";
import { ruleTitleOf } from "./rules.js";
import { andList, fmt, plural } from "./words.js";

export function levelOrder(levels) {
  return ["none"].concat(levels || [], ["skipped"]);
}

export function tallyLevels(rows, order) {
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
export function grow(nodes, targets) {
  requestAnimationFrame(function () {
    requestAnimationFrame(function () {
      nodes.forEach(function (n, i) {
        n.style.flexGrow = targets[i];
      });
    });
  });
}

// Skipped entities are outside a scorecard's scope, so they take no length in a bar.
export function barEl(all, cls, label) {
  var parts = all.filter(function (p) {
    return p.level !== "skipped";
  });
  var spans = parts.map(function (p) {
    return p.hollow
      ? el("span", { className: "hollow", style: "flex-grow:0;flex-basis:0;--seg:" + fill(p.level), title: p.level + ": " + p.n + ", not assessed by an agent yet" })
      : p.unrun
        ? el("span", { className: "unrun", style: "flex-grow:0;flex-basis:0;--seg:" + fill(p.level), title: p.level + ": " + p.n + ", provisional: checks it needs were not run in this build" })
        : p.level === "unscored"
          ? el("span", { className: "unscored", style: "flex-grow:0;flex-basis:0", title: "not scored: " + p.n })
          : el("span", { style: "flex-grow:0;flex-basis:0;background:" + fill(p.level), title: p.level + ": " + p.n });
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
          return p.n + " " + p.level + (p.hollow ? " not assessed" : p.unrun ? " provisional" : "");
        })
        .join(", "),
  );
  return bar;
}

// A scorecard's level parts with the entities no agent has assessed split out, so a bar draws them hollow.
// The level key above a scorecard table, with the hatched swatch when any row draws one.
export function levelLegend(D, rows) {
  var hollow = (rows || []).some(function (r) {
    return !r.compare && unassessedOf(D, r.name);
  });
  var unrun = (rows || []).some(function (r) {
    return !r.compare && provisionalOf(D, r.name);
  });
  return el(
    "div",
    { className: "legend", style: "margin-bottom:8px" },
    ...["none", "bronze", "silver", "gold"].map(function (l) {
      return el("span", {}, el("span", { className: "dot", style: "background:" + fill(l) }), l);
    }),
    hollow ? el("span", {}, el("span", { className: "dot hatch", style: "--seg:" + fill("bronze") }), "hatched, on a bar or a pill: not assessed by an agent yet") : null,
    unrun ? el("span", {}, el("span", { className: "dot dots", style: "--seg:" + fill("silver") }), "◌ dotted, or on a pill: provisional, checks not run in this build") : null,
  );
}

export function splitUnassessed(D, name, parts) {
  var ids = (D.unassessed || {})[name] || [];
  var sc = (D.scorecards || {})[name];
  var rows = (D.scores || {})[name] || {};
  var unassessed = new Set(ids);
  var by = new Map();
  var unrunBy = new Map();
  Object.keys(rows).forEach(function (id) {
    var s = rows[id];
    if (!s || s.level === "skipped") return;
    if (unassessed.has(id)) by.set(s.level, (by.get(s.level) || 0) + 1);
    else if (unrunIn(sc, s).length) unrunBy.set(s.level, (unrunBy.get(s.level) || 0) + 1);
  });
  var out = [];
  parts.forEach(function (p) {
    var u = Math.min(by.get(p.level) || 0, p.n);
    var r = Math.min(unrunBy.get(p.level) || 0, p.n - u);
    if (p.n - u - r) out.push({ level: p.level, n: p.n - u - r });
    if (r) out.push({ level: p.level, n: r, unrun: true });
    if (u) out.push({ level: p.level, n: u, hollow: true });
  });
  return out;
}

// A kind's entities by their weakest level, with the levels that may not hold split out as in a scorecard bar.
export function kindParts(S, ids) {
  var tally = new Map();
  ids.forEach(function (id) {
    var key = (S.worst[id] || "unscored") + "|" + (S.worst[id] ? S.worstState.get(id) || "" : "");
    tally.set(key, (tally.get(key) || 0) + 1);
  });
  var out = [];
  ["none", "bronze", "silver", "gold", "unscored"].forEach(function (l) {
    ["", "unrun", "hollow"].forEach(function (st) {
      var n = tally.get(l + "|" + st);
      if (n) out.push({ level: l, n: n, unrun: st === "unrun", hollow: st === "hollow" });
    });
  });
  return out;
}

export function unassessedOf(D, name) {
  var rows = (D.scores || {})[name] || {};
  return ((D.unassessed || {})[name] || []).filter(function (id) {
    return rows[id] && rows[id].level !== "skipped";
  }).length;
}

export function legendEl(parts) {
  return el(
    "div",
    { className: "legend" },
    ...parts.map(function (p) {
      if (p.level === "skipped") return el("span", {}, el("b", { textContent: fmt(p.n) }), " not in scope");
      if (p.level === "unscored") return el("span", {}, el("span", { className: "dot unscored" }), el("b", { textContent: fmt(p.n) }), " not scored");
      return el(
        "span",
        {},
        el("span", { className: "dot" + (p.hollow ? " hatch" : p.unrun ? " dots" : ""), style: p.hollow || p.unrun ? "--seg:" + fill(p.level) : "background:" + fill(p.level) }),
        el("b", { textContent: fmt(p.n) }),
        " " + p.level + (p.hollow ? " not assessed" : p.unrun ? " ◌ provisional" : ""),
      );
    }),
  );
}

export function badge(level) {
  var only = ONLY_LEVEL.has(level);
  var span = el("span", { className: "lvl " + level, style: "--fill:" + fill(level) + ";--ink:" + ink(level) }, el("i"), level === "skipped" ? "not in scope" : level, only ? el("span", { className: "lvl-top", ariaHidden: "true", textContent: " ✓ top" }) : null);
  span.setAttribute("aria-label", level === "skipped" ? "not in this scorecard's scope" : "level " + level + (only ? ", its scorecard's only and top level" : ""));
  if (LEVEL_NOTE[level]) span.title = LEVEL_NOTE[level];
  return span;
}

// An agent-judged level with no assessment behind it is drawn dashed, the same everywhere it is shown.
// Checks at or below an entity's level that were not run in this build: the level holds only if they pass.
export function unrunIn(sc, score) {
  var order = (sc && sc.levels) || [];
  var here = order.indexOf(score.level);
  return Object.keys(score.rules || {}).filter(function (r) {
    return score.rules[r] === "unknown" && order.indexOf(((sc.rules || {})[r] || {}).level) <= here;
  });
}

// How many in-scope entities of a scorecard hold a level that rests on checks not run in this build.
export function provisionalOf(D, name) {
  var sc = (D.scorecards || {})[name];
  var rows = (D.scores || {})[name] || {};
  return Object.keys(rows).filter(function (id) {
    return rows[id].level !== "skipped" && unrunIn(sc, rows[id]).length > 0;
  }).length;
}

// A level that may not hold is dashed everywhere it is drawn, with the reason beside it. `quiet`
// keeps the outline but drops the tag, for a column whose header already says it once.
// "2 checks were not run in this build: A and B, both needed for bronze."
export function unrunWords(sc, names) {
  var lv = names.map(function (n) {
    return ((sc.rules || {})[n] || {}).level;
  });
  var same = lv.every(function (l) {
    return l === lv[0];
  });
  var list = same
    ? andList(names.map(function (n) {
        return ruleTitleOf(sc, n);
      })) + (lv[0] ? (names.length === 1 ? ", needed for " : names.length === 2 ? ", both needed for " : ", all needed for ") + levelName(lv[0]) : "")
    : andList(names.map(function (n, i) {
        return ruleTitleOf(sc, n) + (lv[i] ? " (" + levelName(lv[i]) + ")" : "");
      }));
  return plural(names.length, "check") + " " + (names.length === 1 ? "was" : "were") + " not run in this build: " + list + ".";
}

export function levelPill(level, unassessed, unrun, quiet) {
  var pill = badge(level);
  if (!unassessed && !(unrun && unrun.length)) return pill;
  pill.classList.add("vacuous", unassessed ? "unjudged" : "unrun");
  pill.append(unassessed ? el("span", { className: "vhatch", ariaHidden: "true" }) : el("span", { className: "vglyph", ariaHidden: "true", textContent: " ◌" }));
  var why = unassessed
    ? "No agent has assessed this entity, so its agent-judged rules have nothing to read: this level is provisional"
    : plural(unrun.length, "check") + " this level needs " + (unrun.length === 1 ? "was" : "were") + " not run in this build (" + unrun.join(", ") + "): the level holds only if " + (unrun.length === 1 ? "it passes" : "they pass");
  pill.setAttribute("aria-label", "level " + level + ", " + (unassessed ? "not assessed" : "provisional"));
  if (quiet) return el("span", { className: "pillwrap", title: why }, pill, el("span", { className: "unassessed-tag quiet" + (unassessed ? "" : " unrun"), ariaHidden: "true" }, unassessed ? " not assessed" : "provisional: not run"));
  return el("span", { className: "pillwrap", title: why }, pill, el("span", { className: "unassessed-tag" + (unassessed ? "" : " unrun"), ariaHidden: "true" }, unassessed ? el("span", { className: "vhatch", ariaHidden: "true" }) : null, unassessed ? " not assessed" : "provisional: not run"));
}

export function medal(level, size) {
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
      stroke: none ? "var(--muted)" : c,
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

export function boxLogo() {
  return svg(
    "svg",
    { width: 34, height: 34, viewBox: "0 0 34 34", "aria-hidden": "true" },
    svg("rect", { x: 3, y: 11, width: 28, height: 19, rx: 4, fill: "var(--accent)" }),
    svg("path", { d: "M12 11V8a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v3", fill: "none", stroke: "var(--accent)", "stroke-width": 2.4 }),
    svg("rect", { x: 3, y: 17, width: 28, height: 3, fill: "var(--card)", opacity: ".35" }),
    svg("rect", { x: 14.5, y: 15.5, width: 5, height: 6, rx: 1.2, fill: "var(--gold)" }),
  );
}
