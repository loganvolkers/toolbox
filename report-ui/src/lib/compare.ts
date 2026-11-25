import type { ReportData } from "../types";
import { BASIS, basisBadge, basisCounts, basisIcon } from "./basis";
import { el } from "./dom";
import { title } from "./rings";
import { ruleTitle } from "./rules";
import { blockers } from "./status";

// The setup facts no rule reads, as one line: "corpus 10,000 · runs 5 · version Toolbox … vs Backstage …".
export var TIMED_UNITS = ["ms", "s", "rps", "seconds", "minutes", "req/s"];

export function setupBrief(context, ids, facts, who) {
  var show = function (v) {
    return typeof v === "number" ? num(v) : typeof v === "boolean" ? (v ? "yes" : "no") : String(v);
  };
  return Object.keys(context)
    .sort()
    .map(function (f) {
      var vals = ids.map(function (id) {
        return (facts[id] || {})[f];
      });
      var same = vals.every(function (v) {
        return JSON.stringify(v) === JSON.stringify(vals[0]);
      });
      if (same) return humanize(f).toLowerCase() + " " + show(vals[0]);
      return (
        humanize(f).toLowerCase() +
        " " +
        ids
          .map(function (id, i) {
            return vals[i] == null ? null : who(id) + " " + show(vals[i]);
          })
          .filter(Boolean)
          .join(" vs ")
      );
    })
    .filter(Boolean);
}

export function humanize(s) {
  var t = String(s || "")
    .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
    .replace(/[-_]+/g, " ")
    .toLowerCase();
  return t.charAt(0).toUpperCase() + t.slice(1);
}

export function num(v) {
  return Number(v).toLocaleString("en-US", { maximumFractionDigits: 2 });
}

// A comparison rule in the reader's words: the description of the one fact it reads.
export function measureLabel(c, r) {
  var fs = (c.reads || {})[r] || [];
  var d = fs.length === 1 ? (c.fields || {})[fs[0]] : null;
  return d ? d.replace(/\.$/, "") : humanize(r);
}

export function ruleValue(c, r, id) {
  var s = (c.scores || {})[id];
  var v = s && s.values ? s.values[r] : undefined;
  if (v != null) return v;
  var fs = (c.reads || {})[r] || [];
  return fs.length === 1 ? ((c.facts || {})[id] || {})[fs[0]] : undefined;
}

export function valueLine(c, r, rd, ids, who) {
  var vals = ids.map(function (id) {
    return ruleValue(c, r, id);
  });
  var numeric = vals.every(function (v) {
    return v == null || typeof v === "number";
  });
  var peer = (c.peers || [])[0];
  var st = (c.rules[r] || {})[peer];
  var leader = st === "ahead" ? c.subject : st === "behind" ? peer : null;
  var max = Math.max.apply(
    null,
    vals
      .filter(function (v) {
        return typeof v === "number";
      })
      .concat([0]),
  );
  var yesNo = vals.every(function (v) {
    return typeof v === "boolean";
  });
  return el(
    "div",
    { className: "vals" + (yesNo ? " yn" : "") },
    ...ids.map(function (id, i) {
      var outcome = (((c.scores || {})[id] || {}).rules || {})[r] || "unknown";
      var v = vals[i];
      var color = id === leader ? "var(--pass)" : st === "parity" ? "var(--muted)" : "var(--faint)";
      var text =
        v == null
          ? outcome === "pass"
            ? "✓ meets it"
            : outcome === "fail"
              ? "✗ misses it"
              : "not measured"
          : typeof v === "boolean"
            ? v
              ? "✓ yes"
              : "✗ no"
            : typeof v === "number"
              ? num(v) + (rd.unit ? " " + rd.unit : "")
              : String(v);
      var fillEl: HTMLElement | null = null;
      if (numeric && typeof v === "number" && max > 0) {
        var fill = el("span", { style: "background:" + color });
        fillEl = fill;
        var w = Math.max(2, (v / max) * 100);
        requestAnimationFrame(function () {
          requestAnimationFrame(function () {
            fill.style.width = w + "%";
          });
        });
      }
      return el(
        "div",
        { className: "val" },
        el("span", { className: "who", textContent: who(id), title: who(id) }),
        fillEl ? el("span", { className: "vb" }, fillEl) : el("span"),
        // Colour marks the leader only; the rule's absolute bar, when it has one, is in the hover text.
        el("span", { className: "v " + (id === leader ? "pass" : ""), textContent: text, title: outcome === "pass" ? "Meets the rule's bar" : outcome === "fail" ? "Misses the rule's bar" : "" }),
      );
    }),
  );
}

export var COUNT_UNITS = ["files", "lines", "packages", "steps", "vulnerabilities", "contributors", "bytes"];
export var SPAN_UNITS = ["ms", "s", "seconds", "minutes"];
export var SIZE_UNITS = ["MB", "KB", "GB"];
// The comparative a reader would say for a unit: fewer files, faster seconds, smaller megabytes.
export function cmpWord(rd, lower) {
  var u = (rd && rd.unit) || "";
  if (COUNT_UNITS.indexOf(u) >= 0) return lower ? "fewer" : "more";
  if (SPAN_UNITS.indexOf(u) >= 0) return lower ? "faster" : "slower";
  if (SIZE_UNITS.indexOf(u) >= 0) return lower ? "smaller" : "larger";
  return lower ? "lower" : "higher";
}

// How far apart two measured values are, in the reader's terms ("8.6× faster").
export function magnitude(c, r, rd, who) {
  var peer = (c.peers || [])[0];
  var a = ruleValue(c, r, c.subject);
  var b = ruleValue(c, r, peer);
  if (typeof a !== "number" || typeof b !== "number") return null;
  if (a === b) return el("span", { className: "mag", textContent: "same value" });
  if (a <= 0 || b <= 0) return null;
  var ratio = Math.max(a, b) / Math.min(a, b);
  var x = ratio >= 10 ? Math.round(ratio).toLocaleString("en-US") : ratio.toFixed(1);
  var lead = rd.better === "lower" ? (a < b ? c.subject : peer) : rd.better === "higher" ? (a > b ? c.subject : peer) : c.subject;
  var la = lead === c.subject ? a : b;
  var lb = lead === c.subject ? b : a;
  return el("span", { className: "mag", textContent: who(lead) + ": " + x + "× " + cmpWord(rd, la < lb) });
}

export function gapText(D: ReportData, g, id) {
  var m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
  if (m) {
    var sc = (D.scorecards || {})[m[1]];
    var score = ((D.scores || {})[m[1]] || {})[id];
    var need = sc && score ? blockers(sc, score, m[2]) : [];
    var card = m[1];
    return (need.length ? "Needs " + need.map(function (r) { return "“" + ruleTitle(card, r) + "”"; }).join(", ") + " for " + m[2] : m[2].charAt(0).toUpperCase() + m[2].slice(1) + " is the goal") + " on " + title(D, m[1]) + "; " + m[3] + " now.";
  }
  m = /^(.+?):rule (.+) \((\S+)\)$/.exec(g);
  if (m) return title(D, m[1]) + ": " + ({ fail: "fails", pass: "passes", unknown: "has not run" }[m[3]] || "is " + m[3]) + " “" + ruleTitle(m[1], m[2]) + "”.";
  m = /^(.+?): unknown scorecard$/.exec(g);
  if (m) return "Scorecard " + m[1] + " is unknown.";
  return g;
}

// Beside a scorecard's name in any list: whether an agent's judgement decides some of its rules.
export function judgedMark(D: ReportData, name, iconOnly?) {
  var n = basisCounts(((D.scorecards || {})[name] || {}).rules);
  if (!n.judged && !n.measured) return null;
  var b = n.judged ? "judged" : "measured";
  var text = n.judged ? (n.measured ? n.judged + " of " + (n.judged + n.measured) + " rules are" : "Every rule is") + " agent-judged: " + BASIS.judged.desc : "Every rule is measured: " + BASIS.measured.desc;
  if (iconOnly) {
    var i = el("span", { className: "bicon " + b, title: text, role: "img" }, basisIcon(b));
    i.setAttribute("aria-label", n.judged ? "has agent-judged rules" : "every rule measured");
    return i;
  }
  var m = basisBadge(b, true);
  m.title = text;
  return m;
}

// A trailing glyph never wraps onto a line of its own: it stays with the last word.
export function keepWithLast(text, node) {
  if (!node) return [text];
  var words = text.split(" ");
  var last = words.pop();
  return [words.length ? words.join(" ") + " " : "", el("span", { className: "nowrap" }, last + "\u00a0", node)];
}

export function cardName(D: ReportData, name) {
  var t = title(D, name);
  var mark = judgedMark(D, name);
  var head = el("span", { className: "t" }, t, mark ? " " : null, mark);
  return t === name ? [head] : [head, el("code", { className: "c", textContent: name })];
}
