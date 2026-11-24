import { badge } from "./bars.js";
import { el, link, svg } from "./dom.js";
import { fill } from "./levels.js";
import { title } from "./rings.js";
import { rich } from "./text.js";
import { plural } from "./words.js";

export var BASIS = {
  measured: {
    label: "Tool-checked",
    desc: "A tool, connector or script decides it from the facts it reads. The same input gives the same result.",
  },
  judged: {
    label: "Agent-judged",
    desc: "Passes on an agent's written assessment against a rubric, reviewed by a second agent; whether that assessment exists and holds counts too. It changes when a verdict does.",
  },
};

export function basisIcon(b) {
  if (b === "judged")
    return svg("svg", { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": "true" }, svg("path", { d: "M6 .6l1.5 3.8 3.9 1.6-3.9 1.6L6 11.4 4.5 7.6.6 6l3.9-1.6z", fill: "currentColor" }));
  return svg(
    "svg",
    { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": "true" },
    svg("path", { d: "M1.4 9.2a4.6 4.6 0 0 1 9.2 0", fill: "none", stroke: "currentColor", "stroke-width": 1.6, "stroke-linecap": "round" }),
    svg("path", { d: "M6 9.2l2.3-3.3", stroke: "currentColor", "stroke-width": 1.6, "stroke-linecap": "round" }),
    svg("circle", { cx: 6, cy: 9.2, r: 1.2, fill: "currentColor" }),
  );
}

export function lockIcon() {
  return svg(
    "svg",
    { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": "true" },
    svg("rect", { x: 2, y: 5.2, width: 8, height: 6, rx: 1.2, fill: "currentColor" }),
    svg("path", { d: "M3.8 5.2V3.8a2.2 2.2 0 0 1 4.4 0v1.4", fill: "none", stroke: "currentColor", "stroke-width": 1.4 }),
  );
}

export function basisOf(rule) {
  return (rule || {}).basis === "judged" ? "judged" : "measured";
}

export function basisBadge(b, mini, inControl) {
  // Mini badges sit inside links and dense rows: the same word, smaller, joins the row's accessible name
  // rather than adding a focus stop of its own, and the page's key explains it in full.
  // A mini badge outside any link ("tip") is its own focus stop, so its meaning opens on focus or tap.
  if (mini === "tip") {
    var tip = el("span", { className: "basis tip " + b + " mini", tabIndex: 0 }, basisIcon(b), el("span", { className: "bl", textContent: BASIS[b].label }), el("span", { className: "sr", textContent: ": " + BASIS[b].desc }));
    tip.dataset.tip = BASIS[b].label + ": " + BASIS[b].desc;
    return tip;
  }
  var quiet = mini
    ? el("span", { className: "basis " + b + " mini" }, basisIcon(b), el("span", {}, el("span", { className: "sr", textContent: " (" }), el("span", { className: "bl", textContent: BASIS[b].label }), el("span", { className: "sr", textContent: ")" })))
    : inControl
      ? el("span", { className: "basis ctl " + b }, basisIcon(b), el("span", { className: "sr", textContent: " (" }), el("span", { className: "bl", textContent: BASIS[b].label }), el("span", { className: "sr", textContent: ")" }))
      : null;
  if (quiet) {
    quiet.dataset.tip = BASIS[b].label + ": " + BASIS[b].desc;
    return quiet;
  }
  // A full badge stands on its own, so keyboard and touch readers can open what it means.
  var full = el("span", { className: "basis tip " + b, tabIndex: 0 }, basisIcon(b), BASIS[b].label, el("span", { className: "sr", textContent: ": " + BASIS[b].desc }));
  full.dataset.tip = BASIS[b].desc;
  return full;
}

// One basis for every rule: the badge and its count, in the line that says what the scorecard rates.
export function basisInline(counts, noun) {
  var only = counts && (counts.measured || 0) + (counts.judged || 0) > 0 && !(counts.measured && counts.judged) ? (counts.measured ? "measured" : "judged") : null;
  if (!only) return null;
  return el(
    "span",
    { className: "basis-inline" },
    " · ",
    basisBadge(only),
    " " + (counts[only] === 1 ? "its one " + (noun || "rule") : "all " + plural(counts[only], noun || "rule")),
    only === "judged" ? ": an agent's assessment is the evidence, so whether one exists and still holds counts too" : null,
  );
}

// What the two badges mean, with how many rules (or checks) each covers when counts are given.
export function basisKey(counts, noun) {
  var row = function (b) {
    var n = counts ? counts[b] || 0 : null;
    return el("div", {}, basisBadge(b), el("span", {}, n != null ? el("b", { textContent: plural(n, noun || "rule") + ". " }) : null, BASIS[b].desc));
  };
  var only = counts && (counts.measured || 0) + (counts.judged || 0) > 0 && !(counts.measured && counts.judged) ? (counts.measured ? "measured" : "judged") : null;
  // One basis only: say so in a line, and name the other so its absence is visible.
  var k = only
    ? el("div", { className: "basis-key one" }, el("div", {}, basisBadge(only), el("span", {}, el("b", { textContent: (counts[only] === 1 ? "" : "All ") + plural(counts[only], noun || "rule") + ". " }), BASIS[only].desc + " None here is " + BASIS[only === "measured" ? "judged" : "measured"].label.toLowerCase() + ".")))
    : el("div", { className: "basis-key" }, row("measured"), row("judged"));
  k.setAttribute("role", "note");
  k.setAttribute("aria-label", "How a rule is decided");
  return k;
}

export function basisCounts(rules) {
  var n = { measured: 0, judged: 0 };
  Object.keys(rules || {}).forEach(function (r) {
    n[basisOf(rules[r])]++;
  });
  return n;
}

export function refs(links) {
  var safe = (links || []).filter(function (l) {
    return l && /^https?:\/\//.test(l.url || "");
  });
  if (!safe.length) return null;
  return el(
    "span",
    { className: "refs" },
    ...safe.map(function (l) {
      // A title that leads with a decision id reads as its claim; the id stays in the hover text.
      var m = /^(D\d+|D0[0-9A-Z]{20,}) (.+)$/.exec(l.title || "");
      var a = el("a", { href: l.url, target: "_blank", rel: "noopener" }, el("span", {}, m ? "Decision: " + m[2] : l.title || l.url, el("span", { className: "ext", ariaHidden: "true", textContent: "\u00a0\u2197" })));
      if (m) a.title = m[1] + ": " + m[2];
      return a;
    }),
  );
}

export function evidenceEl(items, seen, rule) {
  if (!items.length) return "";
  var text = function (i) {
    return typeof i === "string" ? i : JSON.stringify(i);
  };
  var key = items.map(text).join("\n");
  if (seen && seen[key]) return el("p", { className: "sub", textContent: "Evidence: the same as for " + seen[key] + ", above." });
  if (seen) seen[key] = rule;
  var li = function (i) {
    var off = typeof i === "string" && /^no data from connector (\S+)$/.exec(i);
    if (off) return el("li", { className: "sub" }, "Not measured: the ", el("code", { textContent: off[1] }), " connector did not run in this build, so the facts this check reads are missing.");
    return el("li", {}, el("code", { textContent: text(i) }));
  };
  var SHOW = 3;
  if (items.length <= SHOW + 1) return el("ul", { className: "evidence" }, ...items.map(li));
  return el(
    "div",
    { className: "evidence" },
    el("ul", {}, ...items.slice(0, SHOW).map(li)),
    el("details", {}, el("summary", { textContent: "Show " + (items.length - SHOW) + " more" }), el("ul", {}, ...items.slice(SHOW, 50).map(li), items.length > 50 ? el("li", { textContent: "… and " + (items.length - 50) + " more" }) : null)),
  );
}

export function whyEl(def, label) {
  if (!def || (!def.rationale && !(def.links || []).length)) return null;
  return el("div", { className: "why" }, def.rationale ? el("b", { textContent: label || "Why: " }) : null, ...(def.rationale ? rich(def.rationale) : []), refs(def.links));
}
