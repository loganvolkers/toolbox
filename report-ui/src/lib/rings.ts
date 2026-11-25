import type { ReportData } from "../types";
import { lockIcon } from "./basis";
import { el, svg } from "./dom";
import { LEVEL_DEF, ONLY_LEVEL, fill } from "./levels";
import { repoLevel } from "./summary";
import { standingColor } from "./tracks";

export function ring(pct, opts) {
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
  var target: SVGElement | null = null;
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
    o.bare ? null : svg("text", { x: "50%", y: "50%", "text-anchor": "middle", "dominant-baseline": "central", style: s > 70 ? "font-size:18px" : label.length > 3 ? "font-size:12px" : "" }, label),
  );
  if (pct != null) {
    requestAnimationFrame(function () {
      requestAnimationFrame(function () {
        arc.setAttribute("stroke-dashoffset", String(circ * (1 - Math.max(0, Math.min(100, pct)) / 100)));
      });
    });
  }
  return g;
}

// A comparison's standing: one segment per factor, coloured ahead, at parity or behind, so it is
// never read as the progress ring's share.
export function tallyRing(v, n, size, bare?) {
  var s = size || 52;
  var c = s / 2;
  var r = c - 5;
  var circ = 2 * Math.PI * r;
  var seg = circ / Math.max(n, 1);
  var gap = n > 1 ? Math.min(4, seg / 3) : 0;
  var segs: SVGElement[] = [];
  ["ahead", "parity", "behind", "unknown"].forEach(function (k) {
    for (var j = 0; j < (v[k] || 0); j++) {
      segs.push(
        svg("circle", {
          cx: c,
          cy: c,
          r: r,
          fill: "none",
          stroke: standingColor(k),
          "stroke-width": 6,
          "stroke-dasharray": seg - gap + " " + (circ - seg + gap),
          "stroke-dashoffset": -segs.length * seg,
          transform: "rotate(-90 " + c + " " + c + ")",
        }),
      );
    }
  });
  var text = n && v.unknown >= n ? "—" : v.ahead + "/" + n;
  return svg(
    "svg",
    { class: "ring tally", width: s, height: s, viewBox: "0 0 " + s + " " + s, role: "img", "aria-label": tallyWords(v, n) },
    ...segs,
    bare ? null : svg("text", { x: "50%", y: "50%", "text-anchor": "middle", "dominant-baseline": "central", style: s > 70 ? "font-size:18px" : text.length > 3 ? "font-size:12px" : "" }, text),
  );
}

// The strict count says what it measures, so a factor three rules ahead and one behind is not called "behind".
export function countWords(c, key?: "strict" | "majority") {
  if (key === "strict") return ["ahead with no rule behind on " + c.ahead, c.parity ? "at parity on " + c.parity : null, "with a rule behind on " + c.behind, c.unknown ? "not measured on " + c.unknown : null].filter(Boolean).join(", ");
  return ["ahead on " + c.ahead, c.parity ? "at parity on " + c.parity : null, "behind on " + c.behind, c.unknown ? "not measured on " + c.unknown : null].filter(Boolean).join(", ");
}

function sameCount(a, b) {
  return a.ahead === b.ahead && a.parity === b.parity && a.behind === b.behind && a.unknown === b.unknown;
}

// Both counting rules, always strict first and majority second, so two portals that declare
// different rules still show the same two numbers in the same place (D01M87WF2QZVPTD8CGDZKY133CD).
export var COUNTS = [
  { key: "strict", label: "Strict count", def: "a factor is behind if any one of its rules is behind" },
  { key: "majority", label: "Majority count", def: "a factor follows whichever side more of its rules are on" },
] as const;

export function tallyWords(v, n) {
  if (n && v.unknown >= n) return "not measured yet: no factor has values for both products";
  var b = v.both;
  if (!b || v.majority == null) return countWords(v) + " of " + n + " factors";
  var mineKey: "strict" | "majority" = v.majority ? "majority" : "strict";
  var otherKey: "strict" | "majority" = v.majority ? "strict" : "majority";
  if (sameCount(b.strict, b.majority)) return countWords(v) + " of " + n + " factors, by the " + mineKey + " count, this comparison's verdict. The " + otherKey + " count agrees.";
  return countWords(b[mineKey], mineKey) + " of " + n + " factors by the " + mineKey + " count, this comparison's verdict; by the " + otherKey + " count, " + countWords(b[otherKey], otherKey);
}

// The factor count in a few words, for strips and cards.
export function tallyShort(v, n) {
  if (n && v.unknown >= n) return "not measured yet";
  var b = v.both;
  if (!b || v.majority == null) return "ahead on " + v.ahead + " of " + n + " factors";
  if (sameCount(b.strict, b.majority)) return "ahead on " + v.ahead + " of " + n + " factors, by the " + (v.majority ? "majority" : "strict") + " count";
  var mine = v.majority ? b.majority : b.strict;
  return "ahead on " + mine.ahead + " of " + n + " factors, behind on " + mine.behind + " (" + (v.majority ? "majority" : "strict") + " count, the verdict)";
}

// The two counts as a small table, the comparison's own rule marked.
export function bothCounts(v, n) {
  var b = v.both;
  if (!b || v.majority == null || (n && v.unknown >= n)) return null;
  return el(
    "dl",
    { className: "both-counts" },
    ...COUNTS.map(function (c) {
      var mine = (c.key === "majority") === !!v.majority;
      return el("div", { className: mine ? "mine" : "", title: "The " + c.label.toLowerCase() + ": " + c.def + "." }, el("dt", {}, c.label, mine ? el("small", { textContent: " · this comparison's verdict" }) : null), el("dd", { textContent: countWords(b[c.key], c.key) }));
    }),
  );
}

var UNIQUE = new Map<string, string>();

function baseName(s: string) {
  var i = Math.max(s.lastIndexOf("/"), s.lastIndexOf(":"));
  return i >= 0 && i < s.length - 1 ? s.slice(i + 1) : s;
}

// Entities whose basenames collide (three check.go files) are named by the shortest path suffix
// that tells them apart, wherever shortId names them.
export function registerIds(ids: string[]) {
  UNIQUE = new Map();
  var byBase = new Map<string, string[]>();
  // The same file can be an entity of several kinds; only names within one kind need to differ.
  ids.forEach(function (id) {
    var b = id.slice(0, Math.max(id.indexOf(":"), 0)) + "|" + baseName(id);
    (byBase.get(b) || byBase.set(b, []).get(b)!).push(id);
  });
  byBase.forEach(function (group) {
    if (group.length < 2) return;
    var paths = group.map(function (id) {
      var p = id.slice(id.indexOf(":") + 1);
      return p.split("/");
    });
    group.forEach(function (id, k) {
      var parts = paths[k];
      for (var n = 2; n <= parts.length; n++) {
        var tail = parts.slice(-n).join("/");
        var clash = paths.some(function (q, j) {
          return j !== k && q.slice(-n).join("/") === tail;
        });
        if (!clash) return UNIQUE.set(id, tail);
      }
    });
  });
}

export function shortId(id) {
  var s = String(id || "");
  return UNIQUE.get(s) || baseName(s);
}

// A product with no entity yet (its records are not loaded) takes the spelling the portal's own
// track prose uses for it, so "openfga" still reads "OpenFGA".
export function productName(D: ReportData, id, titles?: Record<string, string | null>) {
  var t = titles && titles[id];
  if (t && t !== id) return t;
  var s = shortId(id);
  var re = new RegExp("(^|[^\\w])(" + s.replace(/[.*+?^${}()|[\]\\]/g, "\\$&") + ")(?![\\w])", "i");
  var tracks = Object.values(D.tracks || {}) as { title?: string; description?: string }[];
  for (var i = 0; i < tracks.length; i++) {
    var m = ((tracks[i].title || "") + " " + (tracks[i].description || "")).match(re);
    if (m && m[2] !== m[2].toLowerCase()) return m[2];
  }
  return s;
}

export function isTop(D: ReportData, name, level) {
  var levels = ((D.scorecards || {})[name] || {}).levels || [];
  return !!level && level === levels[levels.length - 1];
}

// A repository standard whose only top level is the repository's level holds the repository there:
// that is a ceiling, not an achievement.
export function capsRepo(D: ReportData, name, level) {
  var R = repoLevel(D);
  return !!(R && R.next && isTop(D, name, level) && level === R.floor && (D.repoRollup || []).indexOf(name) >= 0);
}

export function topNote(D: ReportData, name, level) {
  if (capsRepo(D, name, level)) return el("span", { className: "capnote", title: "Its top level is " + level + ", so the repository cannot rise above " + level + " until this scorecard gains a higher level" }, " ", lockIcon(), " at its top level, " + level + ", so it caps the repository");
  var top = isTop(D, name, level) && !ONLY_LEVEL.has(level) ? el("span", { className: "topmark", textContent: " ✓ top level" }) : "";
  return LEVEL_DEF[level] ? el("span", {}, top, el("span", { className: "sub leveldef", textContent: " · " + LEVEL_DEF[level] })) : top;
}

export function title(D: ReportData, name) {
  var sc = (D.scorecards || {})[name] || {};
  return sc.title || name;
}
