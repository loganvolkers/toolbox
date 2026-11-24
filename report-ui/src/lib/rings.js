import { lockIcon } from "./basis.js";
import { el, svg } from "./dom.js";
import { LEVEL_DEF, ONLY_LEVEL, fill } from "./levels.js";
import { repoLevel } from "./summary.js";
import { standingColor } from "./tracks.js";

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
  var target = null;
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
        arc.setAttribute("stroke-dashoffset", circ * (1 - Math.max(0, Math.min(100, pct)) / 100));
      });
    });
  }
  return g;
}

// A comparison's standing: one segment per factor, coloured ahead, at parity or behind, so it is
// never read as the progress ring's share.
export function tallyRing(v, n, size, bare) {
  var s = size || 52;
  var c = s / 2;
  var r = c - 5;
  var circ = 2 * Math.PI * r;
  var seg = circ / Math.max(n, 1);
  var gap = n > 1 ? Math.min(4, seg / 3) : 0;
  var segs = [];
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
  var text = v.ahead + "/" + n;
  return svg(
    "svg",
    { class: "ring tally", width: s, height: s, viewBox: "0 0 " + s + " " + s, role: "img", "aria-label": tallyWords(v, n) },
    ...segs,
    bare ? null : svg("text", { x: "50%", y: "50%", "text-anchor": "middle", "dominant-baseline": "central", style: s > 70 ? "font-size:18px" : text.length > 3 ? "font-size:12px" : "" }, text),
  );
}

export function tallyWords(v, n) {
  return ["ahead on " + v.ahead, v.parity ? "at parity on " + v.parity : null, "behind on " + v.behind].filter(Boolean).join(", ") + " of " + n + " factors";
}

export function shortId(id) {
  var s = String(id || "");
  var slash = s.lastIndexOf("/");
  var colon = s.lastIndexOf(":");
  var i = Math.max(slash, colon);
  return i >= 0 && i < s.length - 1 ? s.slice(i + 1) : s;
}

export function isTop(D, name, level) {
  var levels = ((D.scorecards || {})[name] || {}).levels || [];
  return !!level && level === levels[levels.length - 1];
}

// A repository standard whose only top level is the repository's level holds the repository there:
// that is a ceiling, not an achievement.
export function capsRepo(D, name, level) {
  var R = repoLevel(D);
  return !!(R && R.next && isTop(D, name, level) && level === R.floor && (D.repoRollup || []).indexOf(name) >= 0);
}

export function topNote(D, name, level) {
  if (capsRepo(D, name, level)) return el("span", { className: "capnote", title: "Its top level is " + level + ", so the repository cannot rise above " + level + " until this scorecard gains a higher level" }, " ", lockIcon(), " at its top level, " + level + ", so it caps the repository");
  var top = isTop(D, name, level) && !ONLY_LEVEL.has(level) ? el("span", { className: "topmark", textContent: " ✓ top level" }) : "";
  return LEVEL_DEF[level] ? el("span", {}, top, el("span", { className: "sub leveldef", textContent: " · " + LEVEL_DEF[level] })) : top;
}

export function title(D, name) {
  var sc = (D.scorecards || {})[name] || {};
  return sc.title || name;
}
