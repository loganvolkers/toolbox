// rule: methods of <toolbox-report>, installed on its prototype by element.js.
import { badge, grow } from "../lib/bars.js";
import { BASIS, basisBadge, basisOf, refs } from "../lib/basis.js";
import { el, enc, link } from "../lib/dom.js";
import { UNRUN } from "../lib/levels.js";
import { shortId, title } from "../lib/rings.js";
import { ruleTitle } from "../lib/rules.js";
import { isPassing } from "../lib/status.js";
import { rich } from "../lib/text.js";
import { fmt, plural } from "../lib/words.js";

export const ruleViews = {
  rule(D, S, scName, ruleName) {
    var sc = D.scorecards[scName];
    var def = sc.rules[ruleName];
    var b = basisOf(def);
    var scores = (D.scores || {})[scName] || {};
    var tally = { pass: 0, waived: 0, fail: 0, unknown: 0 };
    var failing = [];
    Object.keys(scores).forEach(function (id) {
      var row = scores[id];
      var o = (row.rules || {})[ruleName];
      if (!o || o === "skipped" || row.level === "skipped") return;
      tally[o] = (tally[o] || 0) + 1;
      if (!isPassing(o)) failing.push(id);
    });
    failing.sort(function (a, c) {
      return (scores[a].rules[ruleName] === "fail" ? 0 : 1) - (scores[c].rules[ruleName] === "fail" ? 0 : 1) || a.localeCompare(c);
    });
    var all = tally.pass + tally.waived + tally.fail + tally.unknown;
    var parts = [
      { k: "pass", n: tally.pass + tally.waived, c: "var(--pass)", t: "pass" },
      { k: "fail", n: tally.fail, c: "var(--fail)", t: "fail" },
      { k: "unknown", n: tally.unknown, c: UNRUN, t: "not run in this build" },
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
            el("div", { className: "name" }, shortId(id), el("span", { className: "chev", ariaHidden: "true", textContent: "›" }), el("small", { textContent: items.length ? (typeof items[0] === "string" ? items[0] : JSON.stringify(items[0])) + (items.length > 1 ? " (+" + (items.length - 1) + ")" : "") : id })),
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
        ? el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Not passing" }), el("span", { className: "sub", textContent: plural(failing.length, "entity", "entities") + ", failures first" })), list)
        : all
          ? el("section", {}, el("p", { className: "empty good", textContent: "✓ Every entity in scope passes this rule." }))
          : null,
    );
  },
};
