import type { ReportData } from "../types";
import { el, enc, link } from "./dom";
import { title } from "./rings";

// Manifest prose (a rule's fix) marks code with backticks.
// Decision ids belong in a standard's links, not in the sentence a reader is reading.
// A parenthetical made only of citations a reader can't resolve here: decision, question and
// lesson ids, doc paths and plan sections. The standard's links carry the sources.
export var CITATION = "(?:D\\d+|D0[0-9A-Z]{20,}|Q\\d+|lessons? L\\d+(?: and L\\d+)?|plan §[\\d.]+|[\\w./-]+\\.md(?:#[\\w-]+| §[\\d.]+)?(?: for [a-z ]+)?|step \\d+)";
export var DECISION_REFS = new RegExp("\\s*\\(" + CITATION + "(?:[,;] ?" + CITATION + ")*\\)", "g");
export function prose(s) {
  return String(s || "").replace(DECISION_REFS, "");
}

// Words a standard's own prose uses that a reader outside the project would have to look up.
export var GLOSSARY = {
  ratchet: "The CI gate that compares each build with the committed results and fails when any score falls.",
  cel: "Common Expression Language: the small expression language a rule's check is written in.",
  fixture: "A sample input in the scorecard's tests, built to pass or to fail a rule, so the rule is shown to decide both ways.",
  harness: "The program that runs something: an agent harness (such as Claude Code, Cursor or Codex) runs an agent, and a benchmark harness runs and times the products it compares.",
};
export var GLOSS_RE = /\b(ratchet|CEL|fixtures?|harness)\b/i;

export function glossed(text) {
  var m = GLOSS_RE.exec(text);
  if (!m) return [text];
  var word = m[1].toLowerCase();
  var def = GLOSSARY[word] || GLOSSARY[word.replace(/s$/, "")];
  if (!def || (word === "cel" && m[1] !== "CEL")) return [text];
  // Focusable, with the definition in the accessible name, so keyboard and touch readers reach it too.
  var term = el("abbr", { className: "gloss", tabIndex: 0 }, m[1], el("span", { className: "sr", textContent: " (" + def + ")" }));
  term.dataset.tip = def;
  return [text.slice(0, m.index), term, text.slice(m.index + m[1].length)];
}

export function rich(s) {
  var first = true;
  return prose(s)
    .split("`")
    .flatMap(function (part, i) {
      if (i % 2) return [el("code", { textContent: part })];
      if (!first) return [part];
      var out = glossed(part);
      if (out.length > 1) first = false;
      return out;
    });
}

// Prose that names another scorecard by id links to it under its title, so the reader needn't decode ids.
export function richCards(D: ReportData, s, self) {
  return rich(s).flatMap(function (part) {
    if (typeof part !== "string") return [part];
    return part.split(/([a-z0-9]+(?:-[a-z0-9]+)+)/).map(function (bit, i) {
      return i % 2 && bit !== self && (D.scorecards || {})[bit] ? link("scorecard/" + enc(bit), "", title(D, bit)) : bit;
    });
  });
}
