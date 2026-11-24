import { el } from "./dom.js";
import { rich } from "./text.js";

// The first whole sentence of a description, for cards: a card shows a complete thought, never a cut one.
// A reason's first sentence, with the rest in a span a clamped list hides, so nothing is cut mid-sentence.
export function whyFirst(text) {
  var first = firstSentence(text);
  var rest = String(text).trim().slice(first.length).trim();
  return el("span", { className: "why" }, ...rich(first), rest ? el("span", { className: "rest" }, " ", ...rich(rest)) : null);
}

export function scPurpose(sc) {
  var strip = function (t) {
    return firstSentence(t).replace(/\s*\([^)]*\)/g, "");
  };
  return sc.rationale ? "Why: " + strip(sc.rationale) : strip(sc.description || "");
}

// The sentence of a kind's description that says why some of it is not scored, when it says so.
export function unscoredWhy(text) {
  var rest = String(text || "").trim();
  var first = firstSentence(rest);
  while (first && rest) {
    if (/not scored/.test(first) && first !== firstSentence(text)) return el("small", { className: "kwhy" }, ...rich(first));
    rest = rest.slice(first.length).trim();
    first = firstSentence(rest);
  }
  return null;
}

export function sentences(text) {
  var out = [];
  var t = String(text || "").trim();
  while (t) {
    var f = firstSentence(t);
    out.push(f);
    t = t.slice(f.length).trim();
  }
  return out;
}

export function firstSentence(text) {
  var t = String(text || "").trim();
  var depth = 0;
  var code = false;
  for (var i = 0; i < t.length - 2; i++) {
    var ch = t.charAt(i);
    if (ch === "`") code = !code;
    else if (!code && (ch === "[" || ch === "(")) depth++;
    else if (!code && (ch === "]" || ch === ")")) depth = Math.max(0, depth - 1);
    else if (!code && !depth && (ch === "." || ch === "!" || ch === "?") && /\s/.test(t.charAt(i + 1)) && /[A-Z]/.test(t.charAt(i + 2))) return t.slice(0, i + 1);
  }
  return t;
}

// "Bytes of…" reads as "bytes of…" mid-sentence; "MCP…" keeps its capitals.
export function lowerFirst(t) {
  // A capital later in the first word marks a name ("GitHub", "OpenSSF"), which keeps its case.
  return /^[A-Z][a-z]/.test(t || "") && !/^[A-Z][a-z]*[A-Z]/.test(t) ? t.charAt(0).toLowerCase() + t.slice(1) : t;
}

export function andList(items) {
  return items.length > 1 ? items.slice(0, -1).join(", ") + " and " + items[items.length - 1] : items.join("");
}

export function plural(n, word, many) {
  return fmt(n) + " " + (n === 1 ? word : many || word + "s");
}

export function fmt(n) {
  return Number(n).toLocaleString("en-US");
}
