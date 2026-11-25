import type { ReportData } from "../types";
import type { Row } from "./summary";
import { levelOrder, medal, provisionalOf, unassessedOf } from "./bars";
import { basisBadge, basisOf } from "./basis";
import { el, enc, link } from "./dom";
import { kindCount, scorecardSplit } from "./kinds";
import { known } from "./levels";
import { isTop, title } from "./rings";
import { LADDER, ruleTitle } from "./rules";
import { blockers } from "./status";
import { unassessedIn } from "./tracks";
import { fmt, plural } from "./words";

// The first scorecard on a track that can still rise, with the rule most of its lowest entities need.
// A scorecard's own level name ("ten", "clean") is quoted so it reads as a name, not a stray word.
export function levelName(l) {
  return l === "none" || l === "not scored" || LADDER.indexOf(l) >= 0 ? l : "“" + l + "”";
}

// One summary of a scorecard row wherever a list shows it: scope, share at its own top level, and the
// levels that may not hold, so a list never reads better than the scorecard's page.
export function scCaption(D: ReportData, r, split?) {
  var head = (r.sc.kind ? kindCount(D, r.sc.kind, r.total) + (split ? scorecardSplit(D, r.sc.kind) : "") : plural(r.total, "entity", "entities")) + (r.total > r.scored ? ", " + fmt(r.total - r.scored) + " not in scope" : "");
  var prov = provisionalOf(D, r.name);
  var unj = unassessedOf(D, r.name);
  return [
    head + (r.scored ? " · " + r.topShare + "% at the top level (" + levelName(r.top || "top") + ")" : " · not scored yet"),
    prov ? el("span", { className: "provisional", title: "Their levels hold only if the checks not run in this build pass; drawn dotted in the bar", textContent: " · " + fmt(prov) + " provisional (checks not run)" }) : null,
    unj ? el("span", { className: "unjudged-n", title: "No agent has assessed them yet, so their agent-judged rules pass or fail for want of anything to read; drawn hatched in the bar", textContent: " · " + fmt(unj) + " not assessed by an agent (hatched)" }) : null,
  ];
}

export function nextMedal(D: ReportData, cards) {
  for (var i = 0; i < cards.length; i++) {
    var c = cards[i];
    var sc = (D.scorecards || {})[c.row.name] || {};
    var up = c.floor && nextLevel(sc, c.floor);
    if (!up) continue;
    var held: Record<string, number> = {};
    var low = 0;
    Object.values((D.scores || {})[c.row.name] || {}).forEach(function (s) {
      if (s.level !== c.floor) return;
      low++;
      blockers(sc, s, up).forEach(function (n) {
        held[n] = (held[n] || 0) + 1;
      });
    });
    var best = Object.entries(held).sort(function (a, b) {
      return b[1] - a[1] || a[0].localeCompare(b[0]);
    })[0];
    return { name: c.row.name, floor: c.floor, up: up, low: low, rule: best && best[0], n: best ? best[1] : 0 };
  }
  return null;
}

export function nextMedalEl(D: ReportData, cards, label?, why?) {
  var nm = nextMedal(D, cards);
  if (!nm) return null;
  var top = (((D.scorecards || {})[nm.name] || {}).levels || []).slice(-1)[0];
  var card = link("scorecard/" + enc(nm.name), "", title(D, nm.name));
  var rule = nm.rule ? [link("rule/" + enc(nm.name) + "/" + enc(nm.rule), "", ruleTitle(nm.name, nm.rule)), " ", basisBadge(basisOf((((D.scorecards || {})[nm.name] || {}).rules || {})[nm.rule]), "tip")] : [];
  var entities = plural(nm.low, "entity", "entities") + " at " + nm.floor;
  var body = nm.rule && nm.n === nm.low
    ? [entities + (nm.low === 1 ? " needs " : " need "), ...rule, " to reach " + nm.up + " on ", card, "."]
    : ["lift " + entities + " to " + nm.up + " on ", card, ...(nm.rule ? ["; " + nm.n + (nm.n === 1 ? " needs " : " need "), ...rule, " first."] : ["."])];
  return el(
    "div",
    { className: "nextmedal" },
    el("b", { textContent: label || "Next medal: " }),
    ...body,
    el("span", { className: "sub" }, isTop(D, nm.name, nm.up) ? " That is its top level." : " Its top level is " + top + (Object.keys(D.tracks || {}).length ? ": the track's percentage moves when they reach it." : ".") + (why || "")),
    isTop(D, nm.name, nm.up) || !Object.keys(D.tracks || {}).length ? null : ringMoverEl(D, cards),
  );
}

// The step that raises a track's ring most: the scorecard whose entities one level below the top
// are the largest share of it, since the ring averages each scorecard's share at its top level.
function ringMoverEl(D: ReportData, cards) {
  var rows: Row[] = cards
    .map(function (c) {
      return c.row;
    })
    .filter(function (r) {
      return !r.compare && r.scored;
    });
  var best = null as { r: Row; n: number; below: string; top: string; gain: number } | null;
  rows.forEach(function (r) {
    var lv = (r.sc.levels || []).filter(function (l) {
      return l !== "none";
    });
    if (lv.length < 1 || r.topShare >= 100) return;
    var below = lv.length > 1 ? lv[lv.length - 2] : "none";
    var p = (r.parts || []).find(function (x) {
      return x.level === below;
    });
    var n = p ? p.n : 0;
    var gain = n ? (n / r.scored) * 100 / rows.length : 0;
    if (gain > (best ? best.gain : 0)) best = { r: r, n: n, below: below, top: lv[lv.length - 1], gain: gain };
  });
  if (!best || best.gain < 1) return null;
  var b = best;
  var sc = (D.scorecards || {})[b.r.name] || {};
  var held: Record<string, number> = {};
  var unrun: Record<string, number> = {};
  var order = sc.levels || [];
  Object.values((D.scores || {})[b.r.name] || {}).forEach(function (s) {
    if (s.level !== b.below) return;
    blockers(sc, s, b.top).forEach(function (n) {
      held[n] = (held[n] || 0) + 1;
    });
    Object.keys(s.rules || {}).forEach(function (n) {
      if (s.rules[n] === "unknown" && order.indexOf(((sc.rules || {})[n] || {}).level) <= order.indexOf(b.top)) unrun[n] = (unrun[n] || 0) + 1;
    });
  });
  var waits = !Object.keys(held).length && Object.entries(unrun).sort(function (x, y) {
    return y[1] - x[1] || x[0].localeCompare(y[0]);
  })[0];
  var rule = Object.entries(held).sort(function (x, y) {
    return y[1] - x[1] || x[0].localeCompare(y[0]);
  })[0];
  var entities = plural(b.n, "entity", "entities") + " at " + levelName(b.below);
  var card = link("scorecard/" + enc(b.r.name), "", title(D, b.r.name));
  var ruleEl = rule ? [link("rule/" + enc(b.r.name) + "/" + enc(rule[0]), "", ruleTitle(b.r.name, rule[0])), " ", basisBadge(basisOf((sc.rules || {})[rule[0]]), "tip")] : [];
  return el(
    "div",
    { className: "sub mover" },
    el("b", { textContent: waits ? "Unverified, run a check first: " : "Or, for the biggest gain in the track's percentage: " }),
    ...(waits
      ? [entities + " on ", card, " may already meet " + levelName(b.top) + ", but its check ", link("rule/" + enc(b.r.name) + "/" + enc(waits[0]), "", ruleTitle(b.r.name, waits[0])), " was not run in this build, so nothing is known yet"]
      : rule && rule[1] === b.n
      ? [entities + (b.n === 1 ? " needs " : " need "), ...ruleEl, " to reach " + levelName(b.top) + " on ", card]
      : ["lift " + entities + " to " + levelName(b.top) + " on ", card, ...(rule ? ["; " + rule[1] + (rule[1] === 1 ? " needs " : " need "), ...ruleEl, " first"] : [])]),
    (waits ? "; if it passes, the track gains about " : ", about ") + Math.round(b.gain) + (Math.round(b.gain) === 1 ? " point." : " points."),
  );
}

export function ownerName(o) {
  return String(o || "").replace(/^(group|user):/, "");
}

// A standard capped at its top level is unblocked by whoever owns it adding the next level.
export function stuckNote(D: ReportData, name, next) {
  var owner = ownerName(((D.scorecards || {})[name] || {}).owner);
  return el("span", { className: "sub" }, " needs a " + next + " level defined" + (owner ? " by its owner, " + owner : "") + ": ", link("scorecard/" + enc(name), "", "see its levels →"));
}

// Campaigns measure progress to a target, so they get a bar with a target tick; rings are kept for
// a track's share at the top level.
export function goalWords(each) {
  return (
    each.filter(function (p) {
      return p.met;
    }).length +
    " of " +
    plural(each.length, "scorecard") +
    " at target"
  );
}

export function goalBar(state, pct, target, big, each?) {
  var empty = state === "empty";
  var parts = (each || []).length;
  var met = (each || []).filter(function (p) {
    return p.met;
  }).length;
  var g = el(
    "div",
    { className: "cprog" + (big ? " big" : "") + (state === "on" || (empty && parts && met === parts) ? " on" : "") },
    el("b", { textContent: empty ? (parts ? met + " of " + parts : "—") : pct + "%" }),
    empty ? null : el("div", { className: "gb" }, el("span", { style: "width:" + Math.max(0, Math.min(100, pct)) + "%" }), el("i", { style: "left:calc(" + target + "% - 1px)" })),
    el("small", { textContent: empty ? (parts ? (big ? "scorecards at target " + target + "%" : "at target") : "no single score") : "target " + target + "%" }),
  );
  g.setAttribute("role", "img");
  g.setAttribute("aria-label", empty ? (parts ? met + " of " + plural(parts, "scorecard") + " at the target of " + target + "%, each scored on its own" : "no single score") : pct + "% of entities meet the goal; target " + target + "%");
  return g;
}

// hollowPct is the part of pct that passes only because no agent has assessed it; it is hatched.
export function kpiBar(pct, label, short?, hollowPct?) {
  var clamp = function (v) {
    return Math.max(0, Math.min(100, v || 0));
  };
  var g = el(
    "div",
    { className: "kpi-bar" + (short ? " short" : "") },
    el("span", { style: "width:" + clamp(pct - (hollowPct || 0)) + "%" }),
    hollowPct ? el("span", { className: "hollow", style: "width:" + clamp(hollowPct) + "%" }) : null,
  );
  g.setAttribute("role", "img");
  g.setAttribute("aria-label", label);
  return g;
}

export function kpiPips(on, total, label) {
  var g = el(
    "div",
    { className: "kpi-pips" },
    ...Array.from({ length: Math.min(total, 12) }, function (_, i) {
      return el("span", { className: i < on ? "on" : "" });
    }),
  );
  g.setAttribute("role", "img");
  g.setAttribute("aria-label", label);
  return g;
}

export function nextLevel(sc, level) {
  var order = sc.levels || [];
  if (level === "none") return order[0] || null;
  var i = order.indexOf(level);
  return i >= 0 && i < order.length - 1 ? order[i + 1] : null;
}

export function campaignState(c) {
  if (!c.inScope && c.measure == null) return "empty";
  return c.onTarget ? "on" : "off";
}

// A campaign whose scorecards rate different kinds has no shared scope, so each included scorecard is
// measured against the campaign's target on its own.
export function includeProgress(D: ReportData, c) {
  var target = c.targetPercent == null ? 100 : c.targetPercent;
  return (c.includes || []).map(function (i) {
    var sc = (D.scorecards || {})[i.scorecard];
    var order = levelOrder(sc && sc.levels);
    var goal = i.atLeast ? order.indexOf(i.atLeast) : order.length - 2;
    var ratedIds = Object.keys((D.scores || {})[i.scorecard] || {}).filter(function (id) {
      return D.scores[i.scorecard][id].level !== "skipped";
    });
    var rated = ratedIds.map(function (id) {
      return D.scores[i.scorecard][id];
    });
    var below = rated.filter(function (x) {
      return order.indexOf(x.level) < goal;
    }).length;
    var hollow = ratedIds.filter(function (id) {
      return order.indexOf(D.scores[i.scorecard][id].level) >= goal && unassessedIn(D, i.scorecard, id);
    }).length;
    var pct = rated.length ? Math.floor((100 * (rated.length - below)) / rated.length) : 0;
    return { name: i.scorecard, title: title(D, i.scorecard), known: !!sc, goal: order[goal], rated: rated.length, below: below, hollow: hollow, pct: pct, met: !!sc && pct >= target };
  });
}

// "on", "off" or "empty" for every campaign, judging one scored per scorecard by its scorecards.
export function campaignStanding(D: ReportData, c) {
  var state = campaignState(c);
  if (state !== "empty") return state;
  var each = includeProgress(D, c).filter(function (p) {
    return p.known;
  });
  if (!each.length) return "empty";
  return each.every(function (p) {
    return p.met;
  })
    ? "on"
    : "off";
}
