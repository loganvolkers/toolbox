import type { ToolboxReport } from "../element";
import type { Summary } from "../lib/summary";
import type { ReportData, RuleDef } from "../types";
// campaigns: methods of <toolbox-report>, installed on its prototype by element.js.
import { badge, grow } from "../lib/bars";
import { basisBadge, basisOf } from "../lib/basis";
import { campaignState, goalBar, goalWords, includeProgress, kpiBar } from "../lib/campaigns";
import { gapText } from "../lib/compare";
import { el, enc, link } from "../lib/dom";
import { kindCount } from "../lib/kinds";
import { shortId, title } from "../lib/rings";
import { trackPath } from "../lib/routes";
import { ruleLink } from "../lib/rules";
import { blockers, daysLeft } from "../lib/status";
import { template } from "../lib/styles";
import { campaignNamesSorted } from "../lib/summary";
import { rich } from "../lib/text";
import { fmt, plural } from "../lib/words";

// How far the campaign has come since the start the baseline saved; without one, the bar is only where it stands.
function sinceLine(c) {
  var s = c.sinceStart;
  if (!s) return el("p", { className: "sub", style: "margin:0 0 6px", textContent: "This report was built without the campaign's baseline, so the bar shows where the campaign stands today, not how far it has come since its starting point." });
  var parts = [["entitiesShort", "entities short"], ["measure", "total shortfall"]]
    .filter(function (k) {
      return s[k[0]];
    })
    .map(function (k) {
      var x = s[k[0]];
      return k[1] + " " + fmt(x.from) + " → " + fmt(x.now) + (x.percentDone != null ? " (" + x.percentDone + "% of the way)" : "");
    });
  return el("p", { className: "sub", style: "margin:0 0 6px" }, el("b", { textContent: "Since the start" + (s.recorded ? ", " + s.recorded : "") + ": " }), parts.join("; ") || "nothing to compare yet", ".");
}

export const campaignsViews = {
  campaigns(D: ReportData, S: Summary, name) {
    var all = D.campaigns || {};
    var names = campaignNamesSorted(D);
    if (!names.length) names = Object.keys(all).sort();
    if (!names.length) return el("section", {}, el("div", { className: "empty", textContent: "No campaigns in this portal." }));
    var self = this;
    var selected = name && all[name] ? name : names[0];
    var c = all[selected];
    var state = campaignState(c);
    var pct = c.percent == null ? 0 : c.percent;
    var target = c.targetPercent == null ? 100 : c.targetPercent;
    var days = daysLeft(c.end);
    var includes = el("ul", { style: "margin:4px 0 0;padding-left:18px" });
    (c.includes || []).forEach(function (i) {
      includes.append(
        el(
          "li",
          {},
          link("scorecard/" + enc(i.scorecard), "", title(D, i.scorecard)),
          i.atLeast ? el("span", {}, " at ", badge(i.atLeast), " or above") : "",
          i.rules && i.rules.length
            ? el("span", {}, i.rules.length === 1 ? ": passes " : ": passes each of ", ...i.rules.map(function (r, k) {
                return el("span", {}, k ? ", " : "", ruleLink(i.scorecard, r));
              }))
            : "",
        ),
      );
    });
    var bar = el("div", { className: "bar lg" });
    var fillSpan = el("span", { style: "flex-grow:0;flex-basis:0;background:" + (state === "on" ? "var(--pass)" : "var(--muted)") });
    bar.append(fillSpan, el("span", { style: "flex-grow:" + Math.max(0, 100 - pct) + ";background:transparent" }));
    grow([fillSpan], [pct]);
    var marks = el("div", { className: "milestones" });
    (c.milestones || []).forEach(function (m) {
      marks.append(el("span", { className: m.met ? "met" : "", style: "left:" + m.targetPercent + "%", title: "Milestone “" + m.name + "”: " + m.targetPercent + "% of entities meeting the campaign" + (m.met ? ", reached" : ", not reached yet"), textContent: (m.met ? "✓ " : "") + m.name + " " + m.targetPercent + "%" + (m.met ? " reached" : "") }));
    });
    if (target < 100) marks.append(el("span", { style: "left:" + target + "%", textContent: "target " + target + "%" }));
    var size: Record<string, number> = {};
    var gaps = Object.entries(c.entities || {})
      .filter(function (pair) {
        return !pair[1].meeting;
      })
      .sort(function (a, b) {
        return a[0].localeCompare(b[0]);
      });
    var gapList = el("div");
    if (state === "empty") {
      var kinds = (c.includes || []).map(function (i) {
        return ((D.scorecards || {})[i.scorecard] || {}).kind;
      });
      var openRows = includeProgress(D, c).map(function (p) {
        var kind = ((D.scorecards || {})[p.name] || {}).kind;
        var evPct = p.rated ? Math.floor((100 * (p.rated - p.below - p.hollow)) / p.rated) : 0;
        return el(
          "div",
          { className: "gap" },
          link("scorecard/" + enc(p.name), "", p.title),
          el(
            "div",
            { className: "why incl" },
            p.known ? kpiBar(p.pct, p.pct + "% at " + p.goal + " or above" + (p.hollow ? ", " + evPct + "% on an agent's assessment" : "") + "; target " + target + "%", !p.met, p.pct - evPct) : null,
            el("span", { textContent: p.known ? (p.hollow ? evPct + "% at " + p.goal + " or above on an agent's assessment; " + p.pct + "% counting those not assessed yet" : p.pct + "% at " + p.goal + " or above") + " · " + (p.met ? "✓ at target" : fmt(p.below) + " of " + (kind ? kindCount(D, kind, p.rated) : fmt(p.rated)) + " to lift") : "not in this report" }),
            p.hollow ? el("span", { className: "unjudged-n", style: "flex-basis:100%;font-size:13px" }, el("span", { className: "vhatch", ariaHidden: "true", style: "margin:0 5px 0 0" }), fmt(p.hollow) + " pass only because no agent has assessed them yet (hatched)") : null,
          ),
        );
      });
      gapList.append(
        el("p", { className: "next", style: "margin:0 0 10px" }, el("b", { textContent: "Progress, per scorecard. " }), "The scorecards in this campaign rate different things, so each is measured against the target of " + target + "% on its own" + (c.end ? ", by " + c.end + (days != null && days >= 0 ? " (" + plural(days, "day") + " left)" : "") : "") + "."),
        ...openRows,
        el(
          "details",
          { className: "sub", style: "margin-top:8px" },
          el("summary", { style: "padding:3px 0", textContent: "Why each scorecard is measured on its own" }),
          el("p", { textContent: "A campaign's scope is the entities rated by every scorecard it includes, and no entity is rated by all of these" + (kinds.length > 1 && kinds.some(function (k) { return k !== kinds[0]; }) ? " (they rate " + kinds.filter(Boolean).join(" and ") + ")" : "") + ". Include scorecards that rate the same kind to give it one." }),
        ),
      );
    } else if (!gaps.length) {
      gapList.append(el("div", { className: "empty good", textContent: "✓ Every in-scope entity meets the goal." }));
    } else {
      var needCounts: Record<string, number> = {};
      var needFix: Record<string, { sc: string; def: Partial<RuleDef> }> = {};
      gaps.forEach(function (pair) {
        (pair[1].gaps || []).forEach(function (g) {
          const m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
          const gsc = m && (D.scorecards || {})[m[1]];
          const score = m && ((D.scores || {})[m[1]] || {})[pair[0]];
          if (gsc && score)
            blockers(gsc, score, m[2]).forEach(function (r) {
              needCounts[r] = (needCounts[r] || 0) + 1;
              needFix[r] = { sc: m[1], def: (gsc.rules || {})[r] || {} };
            });
        });
      });
      var common = Object.entries(needCounts).sort(function (a, b) {
        return b[1] - a[1] || a[0].localeCompare(b[0]);
      })[0];
      if (common)
        gapList.append(
          el(
            "div",
            { className: "next", style: "margin:0 0 10px" },
            el("b", { textContent: "Biggest lever: " }),
            ruleLink(needFix[common[0]].sc, common[0]),
            " ",
            basisBadge(basisOf(needFix[common[0]].def)),
            " blocks " + common[1] + " of " + gaps.length + (gaps.length === 1 ? " entity" : " entities") + ". ",
            ...rich(needFix[common[0]].def.fix || ""),
            needFix[common[0]].def.rationale ? el("span", { className: "why", style: "display:block;margin-top:4px" }, el("b", { textContent: "Why: " }), ...rich(needFix[common[0]].def.rationale)) : null,
          ),
        );
      // How far off each entity is, from its blocking rules' measures and evidence, so the furthest lead.
      var said: Record<string, string> = {};
      gaps.forEach(function (pair) {
        (pair[1].gaps || []).forEach(function (g) {
          const m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
          const one = !m && /^(.+?):rule (.+) \((\S+)\)$/.exec(g);
          const on = m || one;
          const gsc = on && (D.scorecards || {})[on[1]];
          const score = on && ((D.scores || {})[on[1]] || {})[pair[0]];
          if (gsc && score)
            (m ? blockers(gsc, score, m[2]) : [(one as RegExpExecArray)[2]]).forEach(function (r) {
              var v = (score.measures || {})[r];
              if (typeof v === "number") size[pair[0]] = (size[pair[0]] || 0) + v;
              var ev = (score.evidence || {})[r];
              if (!said[pair[0]] && ev) said[pair[0]] = String(Array.isArray(ev) ? ev[0] : ev).split("\n")[0];
            });
        });
      });
      if (Object.keys(size).length)
        gaps.sort(function (a, b) {
          return (size[b[0]] || 0) - (size[a[0]] || 0) || a[0].localeCompare(b[0]);
        });
      gaps.forEach(function (pair) {
        var bases = {};
        (pair[1].gaps || []).forEach(function (g) {
          const m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
          const gsc = m && (D.scorecards || {})[m[1]];
          const score = m && ((D.scores || {})[m[1]] || {})[pair[0]];
          if (gsc && score)
            blockers(gsc, score, m[2]).forEach(function (r) {
              bases[basisOf((gsc.rules || {})[r] || {})] = true;
            });
        });
        gapList.append(
          el(
            "div",
            { className: "gap" },
            el("div", {}, link("entity/" + enc(pair[0]), "", shortId(pair[0])), el("div", { className: "sub", style: "font-size:12px", textContent: pair[0] })),
            el(
              "div",
              { className: "why" },
              ...Object.keys(bases)
                .sort()
                .map(function (b) {
                  return basisBadge(b, true);
                }),
              Object.keys(bases).length ? " " : "",
              (pair[1].gaps || []).map(function (g) { return gapText(D, g, pair[0]); }).join("; "),
              said[pair[0]] ? el("div", { className: "sub", style: "font-size:12px", textContent: "Found: " + said[pair[0]] }) : null,
            ),
          ),
        );
      });
    }
    var track = c.track && D.tracks && D.tracks[c.track] ? D.tracks[c.track].title || c.track : c.track;
    var picker = el(
      "select",
      {
        ariaLabel: "Campaign",
        onchange: function () {
          location.hash = "campaign/" + enc(picker.value);
        },
      },
      ...names.map(function (n) {
        var cn = all[n];
        return el("option", { value: n, selected: n === selected, textContent: (cn.title || n) + " (" + (campaignState(cn) === "empty" ? goalWords(includeProgress(D, cn)) : (cn.percent || 0) + "%") + ")" });
      }),
    );
    return el(
      "div",
      { className: "split" },
      el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "Campaigns" })),
        el("p", { className: "sub", style: "margin-top:0", textContent: "Behind target first" }),
        el(
          "div",
          { className: "cards", style: "grid-template-columns:1fr" },
          ...names.map(function (n) {
            return self.campaignCard(D, n, n === selected);
          }),
        ),
      ),
      el(
        "section",
        {},
        el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Campaign" }), picker),
        el(
          "div",
          { className: "ehead" },
          el("div", {}, el("h2", { textContent: c.title || selected }), el("div", { className: "sub" }, c.status || "", track ? el("span", {}, " · track ", link(trackPath(D, c.track), "", track)) : null, c.owner ? " · owned by " + c.owner.replace(/^group:/, "") : null)),
          goalBar(state, pct, target, true, includeProgress(D, c)),
        ),
        c.description ? el("p", { className: "lede" }, ...rich(c.description)) : null,
        state === "empty"
          ? null
          : el(
              "p",
              { style: "margin:8px 0 6px" },
              el("b", { textContent: (c.meeting || 0) + " of " + (c.inScope || 0) }),
              " entities meet the goal · target " + target + "%" + (c.end ? " by " + c.end : "") + (days != null && days >= 0 ? " (" + plural(days, "day") + " left)" : "") + ". The bar turns green once it passes the target tick.",
            ),
        state === "empty" ? null : sinceLine(c),
        state === "empty" ? null : bar,
        state === "empty" ? null : marks,
        el("h3", { textContent: "Goal" }),
        includes,
        el("h3", { textContent: state === "empty" ? "Progress" : "Not meeting yet" + (gaps.length ? " (" + gaps.length + ")" : "") + (Object.keys(size).length ? ", furthest first" : "") }),
        gapList,
      ),
    );
  },
} satisfies ThisType<ToolboxReport>;
