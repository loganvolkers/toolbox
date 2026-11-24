import { badge, unrunIn } from "./bars.js";
import { el, enc, link } from "./dom.js";
import { known } from "./levels.js";
import { shortId, title } from "./rings.js";
import { firstSentence, fmt } from "./words.js";

export var RULE_TITLE = new Map();
export var RULE_WORDS = { api: "API", ci: "CI", id: "ID", ids: "IDs", mcp: "MCP", sdlc: "SDLC", sha: "SHA", otel: "OTel", otlp: "OTLP", cel: "CEL", url: "URL", ui: "UI", kubernetes: "Kubernetes" };
export var RULE_WHY = new Map();
export var SC_NAME = new WeakMap();
export function ruleTitles(D) {
  RULE_TITLE.clear();
  RULE_WHY.clear();
  Object.entries(D.scorecards || {}).forEach(function (e) {
    if (e[1] && typeof e[1] === "object") SC_NAME.set(e[1], e[0]);
    Object.entries(e[1].rules || {}).forEach(function (r) {
      if (r[1] && r[1].title) RULE_TITLE.set(e[0] + "\u0000" + r[0], r[1].title);
      if (r[1] && r[1].rationale)
        RULE_WHY.set(
          e[0] + "\u0000" + r[0],
          firstSentence(r[1].rationale)
            .replace(/\[([^\]]+)\]\([^)]*\)/g, "$1")
            .replace(/`/g, ""),
        );
    });
  });
}
// A rule's id (public-api-documented) in the reader's words (Public API documented), unless it names a title.
export function ruleTitle(scName, rule) {
  var t = RULE_TITLE.get(scName + "\u0000" + rule);
  if (t) return t;
  return String(rule)
    .split("-")
    .map(function (w, i) {
      if (Object.prototype.hasOwnProperty.call(RULE_WORDS, w)) return RULE_WORDS[w];
      return i ? w : w.charAt(0).toUpperCase() + w.slice(1);
    })
    .join(" ");
}

// The same, from the scorecard's definition rather than its name.
export function ruleTitleOf(sc, rule) {
  return ruleTitle(SC_NAME.get(sc) || "", rule);
}

// `shown`: the rule's why is already printed beside the link, so it gets no tooltip.
export function ruleLink(scName, rule, text, shown) {
  var a = link("rule/" + enc(scName) + "/" + enc(rule), "rl", text || ruleTitle(scName, rule));
  var why = !shown && RULE_WHY.get(scName + "\u0000" + rule);
  if (why) {
    a.dataset.tip = "Why: " + why + (text ? "" : " (" + rule + ")");
    a.setAttribute("aria-description", "Why: " + why);
  } else if (!text) a.title = rule;
  return a;
}

// The members of a folded group, each a link, so a reader can open one without unfolding it. It sits
// after the details element: links inside a summary would nest interactive controls.
export function memberLinks(ids) {
  var cut = ids.length > 8 ? 6 : ids.length;
  return el(
    "div",
    { className: "members" },
    "In it: ",
    ...ids.slice(0, cut).flatMap(function (id, i) {
      return [i ? ", " : "", link("entity/" + enc(id), "", shortId(id))];
    }),
    ids.length > cut ? " and " + (ids.length - cut) + " more" : null,
  );
}

export function isComparative(D, name) {
  return !!(D.comparisons || {})[name];
}

export var LADDER = ["none", "bronze", "silver", "gold"];

export function rankLevel(l) {
  if (l == null) return 9;
  var k = known(l);
  return k ? LADDER.indexOf(k) : 0;
}

// A scorecard's lowest level among the entities it really scores: demo fixtures, out-of-scope
// entities and entities whose every rule is unknown don't count.
// A scorecard's weakest level as a pill, drawn hollow with the hatched or dotted glyph when at least
// half the entities holding it hold it only for want of an assessment or of checks not run.
export function floorPill(D, name) {
  var floor = floorOf(D, name);
  var pill = badge(floor || "none");
  if (!floor || floor === "none") return pill;
  var sc = (D.scorecards || {})[name];
  var rows = (D.scores || {})[name] || {};
  var un = new Set((D.unassessed || {})[name] || []);
  var at = 0;
  var unj = 0;
  var unr = 0;
  Object.keys(rows).forEach(function (id) {
    if (id.indexOf("/demo/") >= 0 || rows[id].level !== floor) return;
    at++;
    if (un.has(id)) unj++;
    else if (unrunIn(sc, rows[id]).length) unr++;
  });
  if (!at || (unj + unr) * 2 < at) return pill;
  var judged = unj >= unr;
  pill.classList.add("vacuous", judged ? "unjudged" : "unrun");
  pill.append(judged ? el("span", { className: "vhatch", ariaHidden: "true" }) : el("span", { className: "vglyph", ariaHidden: "true", textContent: " ◌" }));
  pill.setAttribute("aria-label", "level " + floor + ", may not hold: " + fmt(unj + unr) + " of " + fmt(at) + (judged ? " not assessed" : " provisional"));
  pill.title = fmt(unj + unr) + " of " + fmt(at) + " entities at " + floor + (judged ? " are not assessed by an agent yet" : " rest on checks not run in this build") + ", so this level may not hold";
  return pill;
}

export function floorOf(D, name) {
  var sc = (D.scorecards || {})[name];
  if (!sc) return null;
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
  return best ? best.level : null;
}
