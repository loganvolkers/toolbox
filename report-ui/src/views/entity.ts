import type { ToolboxReport } from "../element";
import type { Summary } from "../lib/summary";
import type { ReportData } from "../types";
// rulesFor, nextStep, entity: methods of <toolbox-report>, installed on its prototype by element.js.
import { badge, levelPill, medal, unrunIn, unrunWords } from "../lib/bars";
import { basisBadge, basisIcon, basisKey, basisOf, evidenceEl, whyEl } from "../lib/basis";
import { levelName, nextLevel } from "../lib/campaigns";
import { keepWithLast } from "../lib/compare";
import { el, enc, KNOWN, link } from "../lib/dom";
import { kindTitle } from "../lib/kinds";
import { ink, ownLevel } from "../lib/levels";
import { productName, shortId, title } from "../lib/rings";
import { isComparative, ruleLink, ruleTitle, ruleTitleOf } from "../lib/rules";
import { blockers, isPassing } from "../lib/status";
import { rich } from "../lib/text";
import { standingChip, unassessedIn } from "../lib/tracks";
import { firstSentence, plural } from "../lib/words";

export const entityViews = {
  rulesFor(sc, score, scName, unjudged, id) {
    var wrap = el("div");
    var kind = String(id || "").split(":")[0];
    var fixFor = function (text) {
      return id && kind ? String(text).split(kind + ":<name>").join(id) : text;
    };
    var rules = Object.entries(score.rules || {});
    var rank = function (o) {
      return o === "fail" ? 0 : o === "unknown" ? 1 : o === "waived" ? 2 : o === "skipped" ? 3 : 4;
    };
    var order = sc.levels || [];
    rules.sort(function (a, b) {
      var la = order.indexOf(((sc.rules || {})[a[0]] || {}).level);
      var lb = order.indexOf(((sc.rules || {})[b[0]] || {}).level);
      return rank(a[1]) - rank(b[1]) || la - lb || a[0].localeCompare(b[0]);
    });
    var passed = {};
    var npass = 0;
    var seen = {};
    var nextIdx = order.indexOf(score.level) + 1;
    var later: Node[] = [];
    var laterLevels = new Set();
    rules.forEach(function (ro) {
      var n = ro[0];
      var o = ro[1];
      var rule = (sc.rules && sc.rules[n]) || {};
      if (o === "pass" && unjudged && basisOf(rule) === "judged") {
        wrap.append(
          el(
            "div",
            { className: "rule unassessed" },
            el(
              "div",
              { style: "display:flex;flex-wrap:wrap;gap:4px 8px;align-items:center" },
              el("h4", { className: "rn" }, scName ? ruleLink(scName, n, null, true) : n),
              scName ? el("code", { className: "rid", textContent: n }) : null,
              el("span", { className: "outcome unknown", style: "margin:0", textContent: "not assessed yet" }),
              basisBadge("judged"),
              rule.level ? el("span", { className: "sub", textContent: "counts for " + rule.level }) : "",
            ),
            el("div", { className: "fix" }, "No agent has assessed this entity, so this rule has not been reviewed. It is not a pass."),
            whyEl(rule),
          ),
        );
        return;
      }
      if (o === "pass") {
        var lv = rule.level || "";
        (passed[lv] = passed[lv] || []).push(el("span", { className: basisOf(rule) === "judged" ? "judged" : "", title: scName ? "" : rule.rationale || rule.fix || rule.check || "" }, scName ? ruleLink(scName, n) : n, " ", basisBadge(basisOf(rule), "tip")));
        npass++;
        return;
      }
      var ev = (score.evidence || {})[n];
      var items = ev == null ? [] : Array.isArray(ev) ? ev : [ev];
      if (rule.level && order.indexOf(rule.level) > nextIdx) {
        laterLevels.add(rule.level);
        later.push(
          el(
            "details",
            { className: "rule rule-c " + o },
            el(
              "summary",
              {},
              el("span", { className: "rn", textContent: scName ? ruleTitle(scName, n) : n }),
              el("span", { className: "outcome " + o, style: "margin:0", textContent: o === "unknown" ? "not run in this build" : o }),
              basisBadge(basisOf(rule), false, true),
              el("span", { className: "sub", textContent: "needed for " + rule.level }),
              rule.rationale ? el("span", { className: "rwhy", textContent: "Why: " + firstSentence(rule.rationale).replace(/`/g, "") }) : null,
            ),
            scName ? el("div", { className: "sub" }, "Rule page: ", ruleLink(scName, n, null, true), " ", el("code", { className: "rid", textContent: n })) : null,
            rule.fix && o !== "unknown" ? el("div", { className: "fix" }, el("b", { textContent: "Fix: " }), ...rich(fixFor(rule.fix))) : "",
            whyEl(rule),
            evidenceEl(items, seen, scName ? "“" + ruleTitle(scName, n) + "”" : n),
          ),
        );
        return;
      }
      wrap.append(
        el(
          "div",
          { className: "rule " + o },
          el(
            "div",
            { style: "display:flex;flex-wrap:wrap;gap:4px 8px;align-items:center" },
            el("h4", { className: "rn" }, scName ? ruleLink(scName, n, null, true) : n),
              scName ? el("code", { className: "rid", textContent: n }) : null,
            el("span", { className: "outcome " + o, style: "margin:0", textContent: o === "unknown" ? "not run in this build" : o }),
            basisBadge(basisOf(rule)),
            rule.level ? el("span", { className: "sub", textContent: "needed for " + rule.level }) : "",
          ),
          rule.fix && o !== "unknown" ? el("div", { className: "fix" }, el("b", { textContent: "Fix: " }), ...rich(fixFor(rule.fix))) : "",
          whyEl(rule),
          evidenceEl(items, seen, scName ? "“" + ruleTitle(scName, n) + "”" : n),
        ),
      );
    });
    if (later.length) {
      var further = order.filter(function (lv) {
        return laterLevels.has(lv);
      });
      wrap.append(el("div", { className: "later-h", textContent: "Further up: " + plural(later.length, "more rule") + " needed for " + further.join(" and ") + ". Open one for its fix and evidence." }), ...later);
    }
    if (npass) {
      wrap.append(el("div", { className: "sub", style: "margin-top:10px", textContent: plural(npass, "check passes", "checks pass") }));
      order.concat([""]).forEach(function (lv) {
        if (!passed[lv]) return;
        var all = Object.keys(sc.rules || {}).filter(function (n) {
          return (sc.rules[n].level || "") === lv && score.rules[n] !== "skipped";
        }).length;
        wrap.append(
          el(
            "div",
            { className: "passes" },
            lv ? badge(lv) : null,
            el("em", { textContent: passed[lv].length === all ? "all " + all + " met" : passed[lv].length + " of " + all }),
            ...passed[lv],
          ),
        );
      });
    }
    return wrap;
  },

  nextStep(sc, score) {
    var order = sc.levels || [];
    var here = order.indexOf(score.level);
    var unsure = Object.keys(score.rules || {})
      .filter(function (n) {
        return score.rules[n] === "unknown" && order.indexOf(((sc.rules || {})[n] || {}).level) <= here;
      })
      .sort();
    var lowUnrun = unsure
      .map(function (n) {
        return ((sc.rules || {})[n] || {}).level;
      })
      .sort(function (a, b) {
        return order.indexOf(a) - order.indexOf(b);
      })[0];
    var caveat = unsure.length
      ? el(
          "div",
          { className: "callout", style: "background:color-mix(in srgb,var(--unknown) 12%,var(--card))" },
          el("span", {}, el("b", { className: "provisional", textContent: "Provisional " + score.level + ". " }), unrunWords(sc, unsure) + " " + levelName(score.level).charAt(0).toUpperCase() + levelName(score.level).slice(1) + " holds only if " + (unsure.length === 1 ? "it passes" : "they pass") + (lowUnrun && lowUnrun !== score.level ? ", and so does " + levelName(lowUnrun) : "") + "."),
        )
      : null;
    var next = nextLevel(sc, score.level);
    if (!next) return el("div", {}, caveat, el("div", { className: "callout done" }, "✓ Top level reached: every rule passes."));
    var need = blockers(sc, score, next).filter(function (n) {
      return unsure.indexOf(n) < 0;
    });
    if (!need.length) return caveat;
    if (need.length === 1)
      return el("div", {}, caveat, el("div", { className: "callout near" }, el("span", {}, el("b", { textContent: "One rule from " }), badge(next), " · pass “" + ruleTitleOf(sc, need[0]) + "” ", basisBadge(basisOf((sc.rules || {})[need[0]]), "tip"), " and this entity moves up a level.")));
    return el("div", {}, caveat, el("div", { className: "callout" }, el("span", {}, el("b", { textContent: "Next: " }), badge(next), " needs: ", ...need.map(function (n, i) {
      return el("span", { className: "nr" }, ...keepWithLast(ruleTitleOf(sc, n), basisBadge(basisOf((sc.rules || {})[n]), "tip")), i < need.length - 1 ? ", " : "");
    }), ".")));
  },

  entity(D: ReportData, S: Summary, id) {
    var kind = S.kindOf[id];
    var self = this;
    var cards = Object.keys(D.scorecards || {}).filter(function (name) {
      return D.scores[name] && D.scores[name][id];
    });
    var METALS = ["none", "bronze", "silver", "gold"];
    var metalOf = function (name) {
      var lv = D.scores[name][id].level;
      if (METALS.indexOf(lv) >= 0) return lv;
      return ((D.scorecards[name] || {}).levels || []).indexOf(lv) <= 0 ? "none" : "gold";
    };
    // Lowest level first, so the scorecard that holds the entity's level is the first one on the page.
    var placeOf = function (name) {
      if (isComparative(D, name)) return 9;
      if (D.scores[name][id].level === "skipped") return 8;
      return METALS.indexOf(metalOf(name));
    };
    cards = cards
      .map(function (n, i) {
        return { n: n, i: i };
      })
      .sort(function (a, b) {
        return placeOf(a.n) - placeOf(b.n) || a.i - b.i;
      })
      .map(function (x) {
        return x.n;
      });
    var holding = cards.filter(function (n) {
      return placeOf(n) < 8 && metalOf(n) === S.worst[id];
    });
    var total = { pass: 0, hollow: 0, all: 0 };
    var bases = { measured: 0, judged: 0 };
    cards.forEach(function (n) {
      if (isComparative(D, n)) return;
      var rules = D.scores[n][id].rules || {};
      var unread = unassessedIn(D, n, id);
      Object.keys(rules).forEach(function (r) {
        var o = rules[r];
        if (o === "skipped") return;
        total.all++;
        bases[basisOf((D.scorecards[n].rules || {})[r])]++;
        if (isPassing(o) && unread && basisOf((D.scorecards[n].rules || {})[r]) === "judged") total.hollow++;
        else if (isPassing(o)) total.pass++;
      });
    });
    var judgedAt: { name: string; rules: string[]; block: HTMLElement }[] = [];
    var blocks = cards.map(function (name) {
      var sc = D.scorecards[name];
      var score = D.scores[name][id];
      if (isComparative(D, name)) {
        var c = (D.comparisons || {})[name];
        var peer = (c.peers || [])[0];
        var v = (c.verdict || {})[peer];
        var whoSubject = productName(D, c.subject, c.titles);
        var whoPeer = productName(D, peer, c.titles);
        return el(
          "div",
          { className: "sc-block" },
          el("div", { className: "head" }, el("div", {}, el("h3", { className: "sct" }, link("compare/" + enc(name), "", title(D, name))), " ", standingChip(v, whoSubject + " " + (v || "unknown"))), el("span", { className: "sub", textContent: "comparison · " + whoSubject + " vs " + whoPeer })),
        );
      }
      var here = { measured: 0, judged: 0 };
      var judgedHere: string[] = [];
      Object.keys(score.rules || {}).forEach(function (r) {
        var b = basisOf((sc.rules || {})[r]);
        here[b]++;
        if (b === "judged") judgedHere.push(r);
      });
      var block = el(
        "div",
        { className: "sc-block" },
        el(
          "div",
          { className: "head" },
          el("div", {}, el("h3", { className: "sct" }, link("scorecard/" + enc(name), "", title(D, name))), " ", score.level === "skipped" ? badge(score.level) : levelPill(score.level, unassessedIn(D, name, id), unrunIn(sc, score))),
          el("span", { className: "sub" }, name + " · ", here.measured ? el("span", {}, basisIcon("measured"), " " + here.measured + " measured") : null, here.measured && here.judged ? " · " : "", here.judged ? el("span", { className: "judged-n" }, basisIcon("judged"), " " + here.judged + " agent-judged") : null),
        ),
        score.level === "skipped" ? el("div", { className: "sub", textContent: "Not in this scorecard's scope." }) : self.nextStep(sc, score),
        self.rulesFor(sc, score, name, unassessedIn(D, name, id), id),
      );
      if (judgedHere.length && score.level !== "skipped") judgedAt.push({ name: name, rules: judgedHere, block: block });
      return block;
    });
    var judgedList = judgedAt.length
      ? el(
          "p",
          { className: "sub judged-list" },
          el("b", { textContent: "Agent-judged here: " }),
          ...judgedAt.map(function (j, i) {
            return el(
              "span",
              {},
              i ? "; " : "",
              el("button", {
                className: "link",
                textContent: title(D, j.name),
                onclick: function () {
                  j.block.scrollIntoView({ behavior: "smooth", block: "start" });
                  var h = j.block.querySelector<HTMLElement>("h3 a");
                  if (h) h.focus({ preventScroll: true });
                },
              }),
              " (" + j.rules.map(function (r) { return ruleTitle(j.name, r); }).join(", ") + ")",
            );
          }),
        )
      : null;
    var jumpTo = function (name) {
      var block = blocks[cards.indexOf(name)];
      return el("button", {
        className: "link",
        textContent: title(D, name),
        onclick: function () {
          block.scrollIntoView({ behavior: "smooth", block: "start" });
          var h = block.querySelector<HTMLElement>("h3 a");
          if (h) h.focus({ preventScroll: true });
        },
      });
    };
    var heldEl =
      cards.length > 1 && holding.length && S.worst[id]
        ? el(
            "span",
            { className: "held" },
            "Held at " + ownLevel(D, kind, S.worst[id]) + " by ",
            ...(holding.length > 2
              ? [plural(holding.length, "scorecard") + " (the first sections below)"]
              : holding.map(function (h, i) {
                  var sc = D.scorecards[h];
                  var score = D.scores[h][id];
                  var next = nextLevel(sc, score.level);
                  var need = next ? blockers(sc, score, next) : [];
                  var why =
                    need.length === 1
                      ? ": “" + ruleTitleOf(sc, need[0]) + "” " + (score.rules[need[0]] === "unknown" ? "was not run" : "fails")
                      : need.length
                        ? ": " + plural(need.length, "rule") + " short of " + levelName(next)
                        : "";
                  return el("span", {}, i ? "; and by " : "", jumpTo(h), why);
                })),
            ".",
          )
        : null;
    var short = shortId(id);
    var notRun = 0;
    cards.forEach(function (n) {
      var sc = D.scorecards[n];
      var score = D.scores[n][id];
      // An unrun check on any scorecard at or below the entity's level could still pull it down.
      if (isComparative(D, n) || score.level === "skipped") return;
      var here = (sc.levels || []).indexOf(S.worst[id]);
      if (here < 0) return;
      notRun += Object.keys(score.rules || {}).filter(function (r) {
        return score.rules[r] === "unknown" && (sc.levels || []).indexOf(((sc.rules || {})[r] || {}).level) <= here;
      }).length;
    });
    var unassessedCards = cards.filter(function (n) {
      return !isComparative(D, n) && D.scores[n][id].level !== "skipped" && unassessedIn(D, n, id);
    }).length;
    var atTop = cards.every(function (n) {
      var lv = (D.scorecards[n] || {}).levels || [];
      var at = D.scores[n][id].level;
      return isComparative(D, n) || at === "skipped" || at === lv[lv.length - 1];
    });
    var provisional = [notRun ? plural(notRun, "check") + " not run" : null, unassessedCards ? plural(unassessedCards, "agent-judged scorecard") + " not assessed" : null].filter(Boolean);
    return el(
      "section",
      {},
      el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("catalog", "", "Catalog"), kind ? el("span", {}, " / ", link("catalog/" + enc(kind), "", kindTitle(D, kind))) : "", " / " + short),
      el(
        "div",
        { className: "ehead" },
        el("div", {}, el("h2", { textContent: short }), short !== id ? el("code", { textContent: id }) : null, el("div", { className: "sub", textContent: (kind ? kindTitle(D, kind) : "unknown kind") + " · " + plural(cards.length, "scorecard") })),
        S.worst[id]
          ? el(
              "div",
              { className: "elevel" },
              notRun
                ? el("span", { title: "Checks this level needs were not run, so it is not established." }, medal("not scored", 52))
                : S.worstState.get(id) === "hollow"
                ? el("span", { className: "medal-hollow", title: "No agent has assessed this entity yet, so this level may not hold." }, medal(S.worst[id], 52))
                : medal(S.worst[id], 52),
              el(
                "div",
                {},
                el("strong", { style: "color:" + (notRun ? "var(--muted)" : ink(S.worst[id])) + (notRun || KNOWN.indexOf(ownLevel(D, kind, S.worst[id])) < 0 ? ";text-transform:none" : "") }, notRun ? "Not established" : ownLevel(D, kind, S.worst[id]), !notRun && atTop ? el("span", { className: "topmark", textContent: " ✓ top" + (KNOWN.indexOf(S.worst[id]) < 0 ? " level" : "") }) : null, !notRun && provisional.length ? el("span", { className: "pv-head", textContent: " (may not hold)" }) : null),
                provisional.length ? el("span", { className: "provisional", style: "display:block;font-size:13px", title: "The level holds only if the checks not run in this build pass and an agent's assessment finds nothing; the scorecards below name them.", textContent: provisional.join(", ") }) : null,
                heldEl,
                el("span", { className: "sub", textContent: total.pass + " of " + plural(total.all, "check passes", "checks pass") + " on evidence" + (total.hollow ? " · " + plural(total.hollow, "more") + " not assessed yet, not counted as " + (total.hollow === 1 ? "a pass" : "passes") : "") }),
              ),
            )
          : null,
      ),
      total.all ? basisKey(bases, "check") : null,
      judgedList,
      cards.length ? el("div", {}, ...blocks) : el("div", { className: "empty", textContent: "Not scored by any scorecard in this snapshot." }),
    );
  },
} satisfies ThisType<ToolboxReport>;
