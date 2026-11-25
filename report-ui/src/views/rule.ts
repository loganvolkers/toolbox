import type { ToolboxReport } from "../element";
import type { Summary } from "../lib/summary";
import type { ReportData } from "../types";
// rule: methods of <toolbox-report>, installed on its prototype by element.js.
import { badge, grow } from "../lib/bars";
import { BASIS, basisBadge, basisOf, refs } from "../lib/basis";
import { el, enc, link } from "../lib/dom";
import { fill, UNRUN } from "../lib/levels";
import { shortId, title } from "../lib/rings";
import { ruleTitle } from "../lib/rules";
import { isPassing } from "../lib/status";
import { rich } from "../lib/text";
import { fmt, plural } from "../lib/words";

export const ruleViews = {
  rule(D: ReportData, S: Summary, scName, ruleName) {
    var sc = D.scorecards[scName];
    var def = sc.rules[ruleName];
    var b = basisOf(def);
    var scores = (D.scores || {})[scName] || {};
    var tally = { pass: 0, waived: 0, fail: 0, unknown: 0 };
    var failing: string[] = [];
    Object.keys(scores).forEach(function (id) {
      var row = scores[id];
      var o = (row.rules || {})[ruleName];
      if (!o || o === "skipped" || row.level === "skipped") return;
      tally[o] = (tally[o] || 0) + 1;
      if (!isPassing(o)) failing.push(id);
    });
    // A rule that measures how far off each entity is (lines over a limit, findings) lists the furthest first.
    var measureOf = function (id) {
      var m = (scores[id].measures || {})[ruleName];
      return typeof m === "number" ? m : null;
    };
    var measured = failing.some(function (id) {
      return measureOf(id) != null;
    });
    failing.sort(function (a, c) {
      return (scores[a].rules[ruleName] === "fail" ? 0 : 1) - (scores[c].rules[ruleName] === "fail" ? 0 : 1) || (measureOf(c) || 0) - (measureOf(a) || 0) || a.localeCompare(c);
    });
    var allFail = failing.every(function (id) {
      return scores[id].rules[ruleName] === "fail";
    });
    var all = tally.pass + tally.waived + tally.fail + tally.unknown;
    // Worst on the left, as on every level bar.
    var parts = [
      { k: "fail", n: tally.fail, c: "var(--fail)", t: "fail" },
      { k: "unknown", n: tally.unknown, c: UNRUN, t: "not run in this build" },
      { k: "pass", n: tally.pass + tally.waived, c: def.level ? fill(def.level) : "var(--pass)", t: def.level ? "pass, toward " + def.level : "pass" },
    ].filter(function (p) {
      return p.n;
    });
    var spans = parts.map(function (p) {
      return el("span", { style: "flex-grow:0;flex-basis:0;background:" + p.c, title: p.n + " " + p.t });
    });
    grow(
      spans,
      parts.map(function (p) {
        return p.n;
      }),
    );
    var bar = el("div", { className: "bar lg" }, ...spans);
    bar.setAttribute("role", "img");
    bar.setAttribute(
      "aria-label",
      parts
        .map(function (p) {
          return p.n + " " + p.t;
        })
        .join(", "),
    );
    var list = el("div");
    var limit = 40;
    var drawList = function () {
      list.replaceChildren(
        ...failing.slice(0, limit).map(function (id) {
          var o = scores[id].rules[ruleName];
          var ev = (scores[id].evidence || {})[ruleName];
          var items = ev == null ? [] : Array.isArray(ev) ? ev : [ev];
          return link("entity/" + enc(id), "erow", [
            el("div", { className: "name" }, id.indexOf(":") > 0 ? id.slice(id.indexOf(":") + 1) : shortId(id), el("span", { className: "chev", ariaHidden: "true", textContent: "›" }), el("small", { textContent: items.length ? (typeof items[0] === "string" ? items[0] : JSON.stringify(items[0])) + (items.length > 1 ? " (+" + (items.length - 1) + ")" : "") : id })),
            el("span", { className: "outcome " + o, textContent: o === "unknown" ? "not run in this build" : o }),
          ]);
        }),
      );
      if (failing.length > limit)
        list.append(
          el(
            "div",
            { className: "more" },
            el("button", {
              className: "link",
              textContent: "Show all " + failing.length,
              onclick: function () {
                limit = Infinity;
                drawList();
              },
            }),
          ),
        );
    };
    drawList();
    var section = el(
      "section",
      {},
      el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("scorecards", "", "Scorecards"), " / ", link("scorecard/" + enc(scName), "", title(D, scName)), " / " + ruleTitle(scName, ruleName)),
      el(
        "div",
        { className: "ehead" },
        el("div", {}, el("h2", { textContent: ruleTitle(scName, ruleName), tabIndex: -1 }), el("div", { className: "sub" }, el("code", { className: "rid", textContent: ruleName }), " · a rule of ", link("scorecard/" + enc(scName), "", title(D, scName)), def.level ? el("span", {}, " · needed for ", badge(def.level)) : null)),
        basisBadge(b),
      ),
      def.rationale || (def.links || []).length
        ? el(
            "div",
            { className: "rationale" },
            el("b", { textContent: "Why it exists" }),
            def.rationale ? el("div", {}, ...rich(def.rationale)) : null,
            refs(def.links),
            !(def.links || []).length && (sc.links || []).length ? el("div", { className: "sub", style: "margin-top:6px" }, "Read more, from the ", link("scorecard/" + enc(scName), "", title(D, scName)), " scorecard:") : null,
            !(def.links || []).length ? refs(sc.links) : null,
          )
        : el("p", { className: "sub", textContent: "This rule has no rationale yet." }),
      el("h3", { textContent: "How it is decided" }),
      el("p", { style: "margin:0" }, basisBadge(b), " ", BASIS[b].desc),
      el("div", { className: "cel", textContent: def.check }),
      def.fix ? el("h3", { textContent: "How to fix a failure" }) : null,
      def.fix ? el("p", { style: "margin:0" }, ...rich(def.fix)) : null,
      el("h3", { textContent: "Outcomes" }),
      all && def.level ? el("p", { className: "sub", style: "margin:0 0 6px", textContent: "A pass is drawn in the colour of " + def.level + ", the level this rule belongs to, as on the scorecard's bars." }) : null,
      all ? bar : el("p", { className: "empty", textContent: "No entity is in scope." }),
      all
        ? el(
            "div",
            { className: "legend" },
            ...parts.map(function (p) {
              return el("span", {}, el("span", { className: "dot", style: "background:" + p.c }), el("b", { textContent: fmt(p.n) }), " " + p.t);
            }),
          )
        : null,
    );
    section.dataset.focus = "";
    return el(
      "div",
      {},
      section,
      failing.length
        ? el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Not passing" }), el("span", { className: "sub", textContent: plural(failing.length, "entity", "entities") + (measured ? ", furthest off first" : allFail ? "" : ", failures first") })), list)
        : all
          ? el("section", {}, el("p", { className: "empty good", textContent: "✓ Every entity in scope passes this rule." }))
          : null,
    );
  },
} satisfies ThisType<ToolboxReport>;
