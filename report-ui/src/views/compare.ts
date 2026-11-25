import type { ToolboxReport } from "../element";
import type { Comparison, ReportData, RuleDef } from "../types";
// factorCards, headToHead, compare, compareDetail: methods of <toolbox-report>, installed on its prototype by element.js.
import { grow } from "../lib/bars";
import { basisBadge, basisCounts, basisKey, basisOf, refs, whyEl } from "../lib/basis";
import { TIMED_UNITS, cmpWord, humanize, magnitude, measureLabel, num, ruleValue, setupBrief, valueLine } from "../lib/compare";
import { el, enc, link } from "../lib/dom";
import { UNSCORED } from "../lib/levels";
import { bothCounts, countWords, productName, shortId, title } from "../lib/rings";
import { trackPath } from "../lib/routes";
import { ruleTitle } from "../lib/rules";
import { rich } from "../lib/text";
import { STANDING_MARK, byMajority, factorChip, standingBy, standingChip, standingColor, standingPhrase } from "../lib/tracks";
import { andList, firstSentence, lowerFirst, plural, sentences } from "../lib/words";

// The number a factor leads with: rules behind, else ahead, else at parity; both when it is ahead on some and behind on
// others; or, before any record, that none has values.
function tallyPhrase(n, total, key) {
  if (n.unknown === total) return "Not measured: none of " + plural(total, "rule") + " has values";
  if (n.behind && n.ahead) return n.ahead + " ahead, " + n.behind + " behind, of " + plural(total, "rule");
  return (n[key] || 0) + " of " + plural(total, "rule") + " " + (key === "parity" ? "at parity" : key);
}

export const compareViews = {
  factorCards(D: ReportData, names, current?) {
    var all = D.comparisons || {};
    var order = { behind: 0, unknown: 1, parity: 2, ahead: 3 };
    var verdictOf = function (n) {
      var c = all[n];
      return (c.verdict || {})[(c.peers || [])[0]] || "unknown";
    };
    return el(
      "div",
      { className: "factors" },
      ...names
        .slice()
        .sort(function (a, b) {
          return order[verdictOf(a)] - order[verdictOf(b)] || title(D, a).localeCompare(title(D, b));
        })
        .map(function (n) {
          var c = all[n];
          var def = (D.scorecards || {})[n] || {};
          var peer = (c.peers || [])[0];
          var v = verdictOf(n);
          var counts = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
          Object.keys(c.rules || {}).forEach(function (r) {
            var x = (c.rules[r] || {})[peer] || "unknown";
            counts[x] = (counts[x] || 0) + 1;
          });
          var segs = ["ahead", "parity", "behind", "unknown"].filter(function (k) {
            return counts[k];
          });
          var spans = segs.map(function (k) {
            return el("span", { style: "flex-grow:0;flex-basis:0;background:" + (k === "unknown" ? UNSCORED : standingColor(k)) });
          });
          grow(
            spans,
            segs.map(function (k) {
              return counts[k];
            }),
          );
          var bar = el("div", { className: "bar xs" }, ...spans);
          bar.setAttribute("role", "img");
          var tallyText = segs
            .map(function (k) {
              return counts[k] + " " + k;
            })
            .join(" · ");
          bar.setAttribute("aria-label", "Rules: " + tallyText);
          // Every card leads with the same number, rules behind (else ahead, else at parity); its widest measured lead follows.
          var best = null as { ratio: number; r: string; word: string } | null;
          Object.keys(c.rules || {}).forEach(function (r) {
            var rd = (def.rules || {})[r] || {};
            if ((c.rules[r] || {})[peer] !== "ahead" || !rd.better) return;
            var x = ruleValue(c, r, c.subject);
            var y = ruleValue(c, r, peer);
            if (typeof x !== "number" || typeof y !== "number" || x <= 0 || y <= 0) return;
            var ratio = Math.max(x, y) / Math.min(x, y);
            if (!best || ratio > best.ratio) best = { ratio: ratio, r: r, word: cmpWord(rd, rd.better === "lower") };
          });
          var total = Object.keys(c.rules || {}).length;
          var fxKey = counts.behind ? "behind" : counts.ahead ? "ahead" : "parity";
          var fx = total
            ? el(
                "div",
                { className: "fx" },
                el("b", { className: counts.unknown === total ? "unmeasured" : counts.behind && counts.ahead ? "mixed" : "", textContent: tallyPhrase(counts, total, fxKey) }),
                best ? el("span", { textContent: "widest lead: " + lowerFirst(measureLabel(c, best.r)) + ", " + (best.ratio >= 10 ? Math.round(best.ratio).toLocaleString("en-US") : best.ratio.toFixed(1)) + "× " + best.word }) : null,
              )
            : null;
          var a = link("compare/" + enc(n), "factor", [
            el("div", { className: "fh" }, el("span", { textContent: def.title || n }), standingChip(v, factorChip(v, counts, total))),
            fx,
            def.rationale || def.description ? el("p", {}, ...rich(firstSentence(def.rationale || def.description))) : null,
            el("div", { className: "fbar" }, bar, el("span", { className: "fc", textContent: tallyText })),
          ]);
          a.style.setProperty("--st", standingColor(v));
          if (n === current) a.setAttribute("aria-current", "true");
          return a;
        }),
    );
  },

  // One row per factor: the share of its rules behind grows left of the axis, ahead grows right; parity is the gap.
  headToHead(D: ReportData, all, names, peer, current?) {
    var rows = names
      .map(function (n) {
        var c = { ahead: 0, parity: 0, behind: 0, total: 0 };
        Object.keys(all[n].rules || {}).forEach(function (r) {
          var v = ((all[n].rules[r] || {})[peer]) || "unknown";
          if (v in c) c[v]++;
          c.total++;
        });
        return { n: n, c: c };
      })
      .sort(function (a, b) {
        return b.c.behind / Math.max(1, b.c.total) - a.c.behind / Math.max(1, a.c.total) || a.c.ahead / Math.max(1, a.c.total) - b.c.ahead / Math.max(1, b.c.total);
      });
    var pct = function (k, c) {
      return Math.round((100 * c[k]) / Math.max(1, c.total));
    };
    var box = el(
      "div",
      { className: "h2h" },
      el("div", { className: "h2h-head", ariaHidden: "true" }, el("span", {}), el("span", { className: "behind", textContent: STANDING_MARK.behind + " behind" }), el("span", { className: "ahead", textContent: "ahead " + STANDING_MARK.ahead })),
      ...rows.map(function (r) {
        var a = link("compare/" + enc(r.n), "h2h-row", [
          el("span", { className: "h2h-name", textContent: title(D, r.n) }),
          el("span", { className: "h2h-l" }, el("span", { style: "width:" + pct("behind", r.c) + "%" })),
          el("span", { className: "h2h-r" }, el("span", { style: "width:" + pct("ahead", r.c) + "%" })),
        ]);
        a.setAttribute("aria-label", title(D, r.n) + ": " + r.c.behind + " behind, " + r.c.parity + " at parity, " + r.c.ahead + " ahead, of " + r.c.total + " rules");
        a.title = r.c.behind + " behind · " + r.c.parity + " at parity · " + r.c.ahead + " ahead, of " + r.c.total + " rules";
        if (r.n === current) a.setAttribute("aria-current", "true");
        return a;
      }),
      el("div", { className: "h2h-foot", textContent: "Share of each factor's rules; the gap is parity" }),
    );
    return box;
  },

  compare(D: ReportData, name) {
    var all = D.comparisons || {};
    var names = Object.keys(all).sort();
    if (!names.length) return el("section", {}, el("div", { className: "empty", textContent: "No comparative scorecards in this portal." }));
    var first = all[names[0]];
    var subject = first.subject;
    var peer = (first.peers || [])[0];
    var who = function (id) {
      for (var i = 0; i < names.length; i++) {
        var t = (all[names[i]].titles || {})[id];
        if (t && t !== id) return t;
      }
      return productName(D, id);
    };
    var verdictOf = function (n) {
      return ((all[n].verdict || {})[peer]) || "unknown";
    };
    var counts = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
    names.forEach(function (n) {
      counts[verdictOf(n)]++;
    });
    if (name) return this.compareDetail(D, name, names, who);
    var behindNames = names.filter(function (n) {
      return verdictOf(n) === "behind";
    });
    var ruleCounts = { ahead: 0, parity: 0, behind: 0 };
    var ruleTotal = 0;
    var behindRules = {};
    names.forEach(function (n) {
      var rs = all[n].rules || {};
      behindRules[n] = 0;
      Object.keys(rs).forEach(function (r) {
        var v = (rs[r] || {})[peer];
        ruleTotal++;
        if (v in ruleCounts) ruleCounts[v]++;
        if (v === "behind") behindRules[n]++;
      });
    });
    // A factor count alone overstates a lead built on parity, so the headline pairs it with rules.
    var unmeasured = counts.unknown >= names.length;
    var majority = byMajority(D, names);
    var both = { strict: { ahead: 0, parity: 0, behind: 0, unknown: 0 }, majority: { ahead: 0, parity: 0, behind: 0, unknown: 0 } };
    names.forEach(function (n) {
      var rs = all[n].rules || {};
      var k = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
      Object.keys(rs).forEach(function (r) {
        var v = (rs[r] || {})[peer];
        k[v in k ? v : "unknown"]++;
      });
      both.strict[standingBy(k, false)]++;
      both.majority[standingBy(k, true)]++;
    });
    var tallyOf = function (c) {
      return "ahead on " + c.ahead + " of " + plural(names.length, "factor") + (c.behind ? ", behind on " + c.behind : "") + (c.parity ? ", at parity on " + c.parity : "") + (c.unknown ? ", " + c.unknown + " not measured" : "");
    };
    var strictWords = tallyOf(both.strict);
    var majorityWords = tallyOf(both.majority);
    // Every portal leads with the same sentence shape, strict count first, whichever rule it declares.
    var headline = unmeasured
      ? "Not measured yet: none of the " + plural(names.length, "factor") + " has values for both products."
      : strictWords === majorityWords
        ? "Ahead on " + strictWords.slice("ahead on ".length) + ", by either count."
        : "Strict count: " + countWords(both.strict, "strict") + " of " + plural(names.length, "factor") + (majority ? "." : " (this comparison's verdict).") + " Majority count: " + majorityWords + (majority ? " (this comparison's verdict)." : ".");
    var lean = counts.behind > counts.ahead ? "behind" : counts.ahead > counts.behind ? "ahead" : "parity";
    var hero = el(
      "section",
      { className: "cmp-hero" },
      el(
        "div",
        { className: "cmp-main" },
        el("div", { className: "kicker", textContent: "Head to head" }),
        el("h2", { className: "big", textContent: who(subject) + " vs " + who(peer) }),
        el("p", { className: "verdict", style: "margin:0;font-size:18px" }, el("b", { textContent: headline }), behindNames.length ? el("span", {}, " Behind on ", ...behindNames.map(function (n, i) {
          return el("span", {}, i ? ", " : "", link("compare/" + enc(n), "", title(D, n)), " (" + behindRules[n] + " of " + Object.keys(all[n].rules || {}).length + " rules)");
        }), behindNames.length === 1 ? ": " + andList(Object.keys(all[behindNames[0]].rules || {}).filter(function (r) {
          return (all[behindNames[0]].rules[r] || {})[peer] === "behind";
        }).sort().map(function (r) {
          return ruleTitle(behindNames[0], r);
        })) + ". If your teams rely on those, start with " + title(D, behindNames[0]) + "." : ": if your teams rely on what those factors measure, read them first.") : unmeasured ? " Run the comparison's harness and commit its records to fill this in." : counts.unknown ? " Not behind on any measured factor." : " Never behind."),
        unmeasured
          ? el("p", { className: "rule-tally", textContent: "Rule by rule: none of " + plural(ruleTotal, "rule") + " has values for both products yet." })
          : el(
          "p",
          { className: "rule-tally" },
          "Rule by rule: ",
          el("b", { className: "ahead", textContent: ruleCounts.ahead + " ahead" }),
          " · ",
          el("b", { className: "parity", textContent: ruleCounts.parity + " at parity" }),
          " · ",
          el("b", { className: "behind", textContent: ruleCounts.behind + " behind" }),
          ". Factors are not weighted, so the factor count is a tally, not an overall score.",
        ),
        el(
          "details",
          { className: "sub cmp-def cmp-brief", style: "margin:6px 0 0" },
          el("summary", { textContent: "How a factor and a rule are counted" }),
          "A factor is a scorecard that scores both products on the same rules. On a rule, a product is ahead when its measured value is better (within the rule's tolerance is parity) or it passes a bar the other misses. Factors are counted two ways on every portal. By the strict count, " + who(subject) + " is behind on a factor if it is behind on any one of its rules, and ahead if it is ahead on some and behind on none. By the majority count, it is ahead when it is ahead on more of the factor's rules than it is behind on, behind when it is behind on more, and at parity when they are even. Each comparison declares which count is its verdict; this one declares the " + (majority ? "majority" : "strict") + " count. A factor with any unmeasured rule is not measured. “Mixed” means ahead on some rules and behind on others; “narrowly ahead” means ahead on fewer than half of that factor's rules, at parity on the rest.",
        ),
      ),
      el(
        "div",
        { className: "cmp-side" },
        el("div", { className: "kicker", textContent: "Factors" }),
        bothCounts(Object.assign({}, counts, { both: both, majority: majority }), names.length),
        el(
          "div",
          { className: "tally3" },
          ...["ahead", "parity", "behind"].map(function (k) {
            var mixed = unmeasured
              ? 0
              : names.filter(function (n) {
                  var vs = Object.keys(all[n].rules || {}).map(function (r) {
                    return (all[n].rules[r] || {})[peer];
                  });
                  return verdictOf(n) === k && vs.indexOf("ahead") >= 0 && vs.indexOf("behind") >= 0;
                }).length;
            return el("div", {}, el("strong", { textContent: unmeasured ? "—" : String(counts[k]), style: "color:" + (unmeasured ? "var(--muted)" : standingColor(k)) }), el("span", { textContent: STANDING_MARK[k] + " " + k }), mixed ? el("small", { className: "mixed", textContent: mixed + " mixed" }) : null);
          }),
        ),
        unmeasured ? null : el("div", { className: "kicker", style: "margin-top:10px", textContent: "Rules" }),
        unmeasured
          ? null
          : el(
              "div",
              { className: "tally3 rules" },
              ...["ahead", "parity", "behind"].map(function (k) {
                return el("div", {}, el("strong", { textContent: String(ruleCounts[k]), style: "color:" + standingColor(k) }), el("span", { textContent: STANDING_MARK[k] + " " + k }));
              }),
            ),
        this.headToHead(D, all, names, peer),
      ),
    );
    hero.style.setProperty("--st", standingColor(lean));
    hero.dataset.focus = "";

    // The setup: facts no rule reads (version, corpus, harness, runs) describe how it was measured.
    var read = {};
    var fields = {};
    names.forEach(function (n) {
      Object.values(all[n].reads || {}).forEach(function (fs) {
        (fs || []).forEach(function (f) {
          read[f] = true;
        });
      });
      Object.assign(fields, all[n].fields || {});
    });
    var facts = first.facts || {};
    var ids = [subject].concat(first.peers || []);
    var context = {};
    ids.forEach(function (id) {
      Object.keys(facts[id] || {}).forEach(function (f) {
        if (!read[f] && f !== "title") context[f] = true;
      });
    });
    var show = function (v) {
      return typeof v === "number" ? num(v) : typeof v === "boolean" ? (v ? "yes" : "no") : String(v);
    };
    var setup = el(
      "div",
      { className: "setup" },
      ...Object.keys(context)
        .sort()
        .map(function (f) {
          var vals = ids.map(function (id) {
            return (facts[id] || {})[f];
          });
          var same = vals.every(function (v) {
            return JSON.stringify(v) === JSON.stringify(vals[0]);
          });
          return el(
            "div",
            {},
            el("b", { textContent: humanize(f) }),
            same
              ? el("span", { textContent: show(vals[0]) })
              : el(
                  "span",
                  {},
                  ...ids.map(function (id, i) {
                    return vals[i] == null ? null : el("span", { style: "display:block" }, el("span", { className: "sub", textContent: who(id) + ": " }), show(vals[i]));
                  }),
                ),
            fields[f] ? el("small", { textContent: firstSentence(fields[f]), title: fields[f] }) : null,
            vals.some(function (v) {
              return /\+uncommitted\b/.test(String(v));
            })
              ? el("small", { className: "warn-note", textContent: "“+uncommitted”: the harness had local changes when it ran, so this exact code is not in the repository and a rerun from the commit may differ." })
              : null,
          );
        }),
    );
    var trackName = ((D.scorecards || {})[names[0]] || {}).track;
    var track = trackName ? (D.tracks || {})[trackName] : undefined;
    var bases = { measured: 0, judged: 0 };
    names.forEach(function (n) {
      var b = basisCounts(((D.scorecards || {})[n] || {}).rules);
      bases.measured += b.measured;
      bases.judged += b.judged;
    });
    // The hero carries the description's first sentence; the method section continues from there.
    var desc = (track && track.description) || "";
    var lead = desc ? firstSentence(desc) : "";
    var rest = lead ? desc.trim().slice(lead.length).trim() : "";
    // A "Not covered:" sentence is the scope a reader needs before trusting the verdict, so it moves up beside the method.
    var restSentences = rest ? sentences(rest) : [];
    var scope = restSentences.filter(function (x) {
      return /^Not (covered|measured)\s*:/i.test(x);
    })[0];
    restSentences = restSentences.filter(function (x) {
      return x !== scope;
    });
    var method = el(
      "section",
      {},
      el("div", { className: "head" }, el("h2", { textContent: "How it was measured" }), track ? link(trackPath(D, trackName), "", "Track: " + (track.title || trackName) + " →") : null),
      restSentences.length ? el("ul", { className: "method-lede" }, ...restSentences.map(function (x) {
        return el("li", {}, ...rich(x));
      })) : null,
      Object.keys(context).length ? setup : null,
      unmeasured ? el("p", { className: "sub", style: "margin:8px 0 0", textContent: "Nothing is measured in this build. Once the harness runs and its records are committed, the rules are decided like this:" }) : null,
      basisKey(bases),
    );
    var brief = setupBrief(context, ids, facts, who);
    var sources = track ? refs(track.links) : null;
    if (brief.length || sources)
      (hero.querySelector(".rule-tally") as HTMLElement).after(
        el(
          "div",
          { className: "sub cmp-prov" },
          sources ? el("span", { className: "cmp-src" }, el("b", { textContent: "Evidence: " }), sources, unmeasured ? " (no results are recorded in this build yet)" : null) : null,
        ),
      );
    (hero.querySelector(".rule-tally") as HTMLElement).after(
      el(
        "p",
        { className: "cmp-method", style: "margin:8px 0 0" },
        el("b", { textContent: lead ? "Method: " : "How it was measured: " }),
        ...(lead ? rich(lead + " ") : [(brief.length ? brief.join(" · ") + ". " : "") + "Every rule is " + (bases.judged ? "measured or agent-judged" : "measured by script") + ". "]),
        el("span", { style: "display:block;margin-top:4px" }, el("b", { textContent: "Not covered: " }), ...(scope ? rich(scope.replace(/^Not (covered|measured)\s*:\s*/i, "") + " ") : [el("span", { className: "sub", textContent: "not stated: the track's description has no “Not covered:” sentence. " })])),
        el("button", {
          className: "link",
          textContent: track && track.description ? "The full method, and who checked it ↓" : "Versions, setup and sources ↓",
          onclick: function () {
            method.scrollIntoView({ behavior: "smooth", block: "start" });
          },
        }),
      ),
    );

    var behindRows: Node[] = [];
    var behindIn: string[] = [];
    var behindBases = {};
    names.forEach(function (n) {
      var def = (D.scorecards || {})[n] || {};
      Object.keys(all[n].rules || {}).forEach(function (r) {
        if ((all[n].rules[r] || {})[peer] === "behind") behindBases[basisOf((def.rules || {})[r] || {})] = true;
      });
    });
    var oneBasis = Object.keys(behindBases).length === 1 ? Object.keys(behindBases)[0] : null;
    names.forEach(function (n) {
      var c = all[n];
      var def = (D.scorecards || {})[n] || {};
      Object.keys(c.rules || {}).forEach(function (r) {
        if ((c.rules[r] || {})[peer] !== "behind") return;
        var rd = (def.rules || {})[r] || {};
        behindRows.push(
          el(
            "div",
            { className: "behind-row" },
            el(
              "div",
              {},
              el("div", { style: "font-weight:650", textContent: measureLabel(c, r) }),
              el("div", { className: "sub" }, link("compare/" + enc(n), "", def.title || n), " · ", el("code", { textContent: r }), oneBasis ? null : " ", oneBasis ? null : basisBadge(basisOf(rd), true)),
              el("div", { className: "bh-val" }, valueLine(c, r, rd, ids, who)),
            ),
            el(
              "div",
              {},
              rd.rationale ? el("p", { className: "why" }, el("b", { textContent: "Why it matters: " }), ...rich(rd.rationale), (rd.links || []).length ? " " : null, refs(rd.links)) : null,
              rd.fix ? el("p", { className: "why bh-fix" }, el("b", { textContent: "To close it: " }), ...rich(rd.fix)) : null,
            ),
          ),
        );
        if (behindIn.indexOf(n) < 0) behindIn.push(n);
      });
    });
    var behind = el(
      "section",
      {},
      el("div", { className: "head" }, el("h2", { textContent: unmeasured ? "Where each product leads (not measured yet)" : "Where " + who(subject) + " is behind" }), behindIn.length
        ? el(
            "span",
            { className: "sub" },
            plural(behindRows.length, "rule") + (oneBasis ? ", all " : ""),
            oneBasis ? basisBadge(oneBasis, true) : null,
            "; in full: ",
            ...behindIn.flatMap(function (n, i) {
              return [i ? ", " : "", link("compare/" + enc(n), "", title(D, n) + " →")];
            }),
          )
        : null),
      behindRows.length ? el("div", {}, ...behindRows) : unmeasured ? el("p", { className: "empty", textContent: "Nothing to compare yet: no rule has values for both products." }) : el("p", { className: "empty good", textContent: "✓ Not behind on any rule." }),
    );
    // Measured gaps as ratios, widest first: where the subject leads, and where it trails.
    var ratioRows = function (side) {
      var rows: { n: string; c: Comparison; r: string; rd: Partial<RuleDef>; a: number; b: number; ratio: number }[] = [];
      names.forEach(function (n) {
        var c = all[n];
        var def = (D.scorecards || {})[n] || {};
        Object.keys(c.rules || {}).forEach(function (r) {
          var rd = (def.rules || {})[r] || {};
          if ((c.rules[r] || {})[peer] !== side || !rd.better) return;
          var a = ruleValue(c, r, subject);
          var b = ruleValue(c, r, peer);
          if (typeof a !== "number" || typeof b !== "number" || a <= 0 || b <= 0) return;
          rows.push({ n: n, c: c, r: r, rd: rd, a: a, b: b, ratio: Math.max(a, b) / Math.min(a, b) });
        });
      });
      return rows.sort(function (x, y) {
        return y.ratio - x.ratio;
      });
    };
    var leadRows = ratioRows("ahead");
    var gapRows = ratioRows("behind");
    var unitOf = function (rd, v) {
      return num(v) + (rd.unit ? " " + rd.unit : "");
    };
    var times = function (l) {
      return l.ratio >= 10 ? Math.round(l.ratio).toLocaleString("en-US") : l.ratio.toFixed(1);
    };
    var widest = function (label, l, gap?) {
      return el("p", { className: "cmp-lead" + (gap ? " gaps" : "") }, el("b", { textContent: label }), link("compare/" + enc(l.n), "", [lowerFirst(measureLabel(l.c, l.r)) + ", ", el("span", { className: "lead-x", textContent: (gap ? STANDING_MARK.behind + " " : "") + times(l) + "× " + cmpWord(l.rd, l.a < l.b) })]), " (" + who(subject) + " " + unitOf(l.rd, l.a) + ", " + who(peer) + " " + unitOf(l.rd, l.b) + ").");
    };
    var tiles = function (rows, gap?) {
      return el(
        "div",
        { className: gap ? "leads gaps" : "leads" },
        ...rows.slice(0, 3).map(function (l) {
          var vs = who(subject) + " " + unitOf(l.rd, l.a) + " vs " + who(peer) + " " + unitOf(l.rd, l.b);
          return link("compare/" + enc(l.n), "lead-tile", [
            el("span", { className: "lead-x", textContent: (gap ? STANDING_MARK.behind + " " : "") + times(l) + "×" }),
            " ",
            el("span", { className: "lead-dir", textContent: cmpWord(l.rd, l.a < l.b) }),
            el("span", { className: "sr", textContent: ":" }),
            " ",
            el("span", { className: "lead-what", textContent: measureLabel(l.c, l.r) }),
            el("span", { className: "sr", textContent: "." }),
            " ",
            el("span", { className: "sub", textContent: vs + " · " + title(D, l.n) }),
          ]);
        }),
      );
    };
    var verdictEl = hero.querySelector(".verdict") as HTMLElement;
    if (gapRows[0]) verdictEl.after(widest("Widest gap: ", gapRows[0], true));
    if (leadRows[0]) verdictEl.after(widest("Widest lead: ", leadRows[0]));
    if (gapRows.length) (behind.querySelector(".head") as HTMLElement).after(el("p", { className: "sub", style: "margin:0 0 6px", textContent: "Measured values only, largest ratio first; every rule behind is listed below." }), tiles(gapRows, true));
    var leads = leadRows.length
      ? el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Where " + who(subject) + " leads by the most" }), el("span", { className: "sub", textContent: "Measured values only, largest ratio first" })),
          tiles(leadRows),
        )
      : null;
    var factors = el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Factors" }), el("span", { className: "sub", textContent: unmeasured ? "By name" : "Behind first" })), this.factorCards(D, names));
    var jumps = (
      [
        ["Factors", factors],
        ["Behind", behind],
        ["Leads", leads],
        ["Method", method],
      ] as [string, HTMLElement | null][]
    ).filter(function (j) {
      return j[1];
    });
    var jump = el(
      "nav",
      { className: "jump", ariaLabel: "On this page" },
      el("span", { className: "sub", textContent: "On this page: " }),
      ...jumps.map(function (j, i) {
        return el("span", {}, i ? " · " : "", el("button", { className: "link", textContent: j[0], onclick: function () {
          (j[1] as HTMLElement).scrollIntoView({ behavior: "smooth", block: "start" });
        } }));
      }),
    );
    return el("div", {}, hero, jump, factors, behind, leads, method);
  },

  compareDetail(D: ReportData, name, names, who) {
    var c = (D.comparisons || {})[name];
    var def = (D.scorecards || {})[name] || {};
    var peer = (c.peers || [])[0];
    var ids = [c.subject].concat(c.peers || []);
    var v = (c.verdict || {})[peer] || "unknown";
    var order = { behind: 0, unknown: 1, ahead: 2, parity: 3 };
    var rules = Object.keys(def.rules || {}).sort(function (a, b) {
      var sa = (c.rules[a] || {})[peer] || "unknown";
      var sb = (c.rules[b] || {})[peer] || "unknown";
      return order[sa] - order[sb] || a.localeCompare(b);
    });
    var rows = rules.map(function (r) {
      var rd = def.rules[r];
      var st = (c.rules[r] || {})[peer] || "unknown";
      return el(
        "div",
        { className: "measure" },
        el(
          "div",
          { className: "mt" },
          measureLabel(c, r),
          el("small", {}, el("code", { textContent: r }), basisBadge(basisOf(rd), true), rd.better ? el("span", { textContent: rd.better + " is better" }) : null),
        ),
        valueLine(c, r, rd, ids, who),
        el("div", { className: "ms" }, standingChip(st, st === "parity" ? "parity" : st === "unknown" ? "not measured" : who(c.subject) + " " + st), magnitude(c, r, rd, who)),
        el(
          "div",
          { className: "full" },
          whyEl(rd),
          el(
            "details",
            {},
            el("summary", { textContent: "How it is scored" }),
            el(
              "p",
              { className: "sub", style: "margin:6px 0 0" },
              ((c.reads || {})[r] || []).length
                ? "The benchmark harness recorded " +
                    ((c.reads || {})[r] || [])
                      .map(function (f) {
                        return "“" + ((c.fields || {})[f] || humanize(f)).replace(/\.$/, "") + "”";
                      })
                      .join(" and ") +
                    " for each product, on the versions and machine under "
                : "Scored on the versions and machine under ",
              link("compare", "", "How it was measured"),
              ". " + (rd.better ? "The " + rd.better + " value is ahead." : "Passing beats failing."),
            ),
            el("div", { className: "cel", ariaLabel: "Expression", textContent: rd.check + (rd.value ? "\nvalue: " + rd.value : "") }),
          ),
          rd.fix && st === "behind" ? el("p", { className: "sub", style: "margin:4px 0 0" }, el("b", { textContent: "What would close it: " }), ...rich(rd.fix)) : null,
        ),
      );
    });
    // Parity sorts last; when it is most of a long factor, it folds so the rules that differ lead.
    var atParity = rules.filter(function (r) {
      return ((c.rules[r] || {})[peer] || "unknown") === "parity";
    }).length;
    var fold = atParity >= 3 && atParity < rules.length;
    var read = {};
    names.forEach(function (n) {
      Object.values(((D.comparisons || {})[n] || {}).reads || {}).forEach(function (fs) {
        (fs || []).forEach(function (f) {
          read[f] = true;
        });
      });
    });
    var context = {};
    ids.forEach(function (id) {
      Object.keys((c.facts || {})[id] || {}).forEach(function (f) {
        if (!read[f] && f !== "title") context[f] = true;
      });
    });
    // Numeric setup facts (a corpus size, a run count) parameterise timed runs; a factor with no timed
    // rule was probed once, so only the builds it probed apply.
    var timed = Object.values(def.rules || {}).some(function (rd) {
      return TIMED_UNITS.indexOf(String(rd.unit || "").toLowerCase()) >= 0;
    });
    if (!timed)
      Object.keys(context).forEach(function (f) {
        if (ids.some(function (id) { return typeof ((c.facts || {})[id] || {})[f] === "number"; })) delete context[f];
      });
    var brief = setupBrief(context, ids, c.facts || {}, who);
    var tallyBy = new Map();
    rules.forEach(function (r) {
      var s = (c.rules[r] || {})[peer] || "unknown";
      tallyBy.set(s, (tallyBy.get(s) || 0) + 1);
    });
    var tallyKey = tallyBy.get("behind") ? "behind" : tallyBy.get("ahead") ? "ahead" : "parity";
    var tallyNote = rules.length ? " · " + tallyPhrase({ unknown: tallyBy.get("unknown") || 0, behind: tallyBy.get("behind"), ahead: tallyBy.get("ahead"), parity: tallyBy.get("parity") }, rules.length, tallyKey) : "";
    var head = el(
      "section",
      { className: basisCounts(def.rules).judged || !basisCounts(def.rules).measured ? "" : "allmeasured" },
      el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("compare", "", "Compare"), " / " + (def.title || name)),
      el("div", { className: "head" }, el("h2", { textContent: def.title || name, tabIndex: -1 }), standingChip(v, rules.length && (tallyBy.get("unknown") || 0) === rules.length ? tallyNote.slice(3) : standingPhrase(who(c.subject), v, who(peer)) + tallyNote)),
      brief.length ? el("p", { className: "sub provenance" }, el("b", { textContent: timed ? "Timed on: " : "Probed once on: " }), brief.join(" · ") + ". ", /\+uncommitted\b/.test(brief.join(" ")) ? "“+uncommitted” means the harness had local changes when it ran, so a rerun from that commit may differ. " : "", link("compare", "", "How it was measured, and who checked it →")) : null,
      def.description ? el("p", { className: "lede" }, ...rich(def.description)) : null,
      def.rationale || (def.links || []).length ? el("div", { className: "rationale" }, el("b", { textContent: "Why this factor matters" }), def.rationale ? el("div", {}, ...rich(def.rationale)) : null, refs(def.links)) : null,
      el("p", { className: "sub", textContent: "Each rule scores both products. Where both have a measured value, the better value decides; otherwise passing beats failing. By the strict count, " + who(c.subject) + " is behind on the factor if it is behind on any rule; by the majority count, it follows whichever side more of the rules are on. This comparison's verdict is the " + (c.verdictRule === "majority" ? "majority" : "strict") + " count." }),
      basisKey(basisCounts(def.rules)),
      el("div", {}, ...(fold ? rows.slice(0, rows.length - atParity) : rows)),
      fold
        ? el(
            "details",
            {
              className: "parity-fold",
              ontoggle: function (this: HTMLDetailsElement) {
                (this.firstChild as Node).textContent = (this.open ? "Hide the " : "Show the ") + atParity + " rules at parity (neither product leads)";
              },
            },
            el("summary", { textContent: "Show the " + atParity + " rules at parity (neither product leads)" }),
            ...rows.slice(rows.length - atParity),
          )
        : null,
    );
    head.dataset.focus = "";
    return el("div", {}, head, el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Every factor" }), link("compare", "", "← Back to the summary")), el("div", { className: "h2h-wide" }, this.headToHead(D, D.comparisons, names, peer, name))));
  },
} satisfies ThisType<ToolboxReport>;
