import { levelOrder, medal, provisionalOf, unassessedOf } from "./bars.js";
import { basisBadge, basisOf } from "./basis.js";
import { el, enc, link } from "./dom.js";
import { kindCount, scorecardSplit } from "./kinds.js";
import { known } from "./levels.js";
import { isTop, title } from "./rings.js";
import { LADDER, ruleTitle } from "./rules.js";
import { blockers } from "./status.js";
import { unassessedIn } from "./tracks.js";
import { fmt, plural } from "./words.js";

// The first scorecard on a track that can still rise, with the rule most of its lowest entities need.
// A scorecard's own level name ("ten", "clean") is quoted so it reads as a name, not a stray word.
export function levelName(l) {
  return l === "none" || l === "not scored" || LADDER.indexOf(l) >= 0 ? l : "“" + l + "”";
}

// One summary of a scorecard row wherever a list shows it: scope, share at its own top level, and the
// levels that may not hold, so a list never reads better than the scorecard's page.
export function scCaption(D, r, split) {
  var head = (r.sc.kind ? kindCount(D, r.sc.kind, r.total) + (split ? scorecardSplit(D, r.sc.kind) : "") : plural(r.total, "entity", "entities")) + (r.total > r.scored ? ", " + fmt(r.total - r.scored) + " not in scope" : "");
  var prov = provisionalOf(D, r.name);
  var unj = unassessedOf(D, r.name);
  return [
    head + (r.scored ? " · " + r.topShare + "% at the top level (" + levelName(r.top || "top") + ")" : " · not scored yet"),
    prov ? el("span", { className: "provisional", title: "Their levels hold only if the checks not run in this build pass; drawn dotted in the bar", textContent: " · " + fmt(prov) + " provisional (checks not run)" }) : null,
    unj ? el("span", { className: "unjudged-n", title: "No agent has assessed them yet, so their agent-judged rules pass or fail for want of anything to read; drawn hatched in the bar", textContent: " · " + fmt(unj) + " not assessed by an agent (hatched)" }) : null,
  ];
}

export function nextMedal(D, cards) {
  for (var i = 0; i < cards.length; i++) {
    var c = cards[i];
    var sc = (D.scorecards || {})[c.row.name] || {};
    var up = c.floor && nextLevel(sc, c.floor);
    if (!up) continue;
    var held = {};
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

export function nextMedalEl(D, cards, label) {
  var nm = nextMedal(D, cards);
  if (!nm) return null;
  return el(
    "div",
    { className: "nextmedal" },
    el("b", { textContent: label || "Next medal: " }),
    link("scorecard/" + enc(nm.name), "", title(D, nm.name)),
    " to " + nm.up + (isTop(D, nm.name, nm.up) ? ", its top level" : " (its top level is " + (((D.scorecards || {})[nm.name] || {}).levels || []).slice(-1)[0] + ")") + ". " + plural(nm.low, "entity", "entities") + " at " + nm.floor + " to lift" + (nm.rule ? "; " + nm.n + (nm.n === 1 ? " needs " : " need ") : "."),
    nm.rule ? link("rule/" + enc(nm.name) + "/" + enc(nm.rule), "", ruleTitle(nm.name, nm.rule)) : null,
    nm.rule ? " " : null,
    nm.rule ? basisBadge(basisOf((((D.scorecards || {})[nm.name] || {}).rules || {})[nm.rule]), "tip") : null,
    nm.rule ? "." : null,
  );
}

export function ownerName(o) {
  return String(o || "").replace(/^(group|user):/, "");
}

// A standard capped at its top level is unblocked by whoever owns it adding the next level.
export function stuckNote(D, name, next) {
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

export function goalBar(state, pct, target, big, each) {
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
export function kpiBar(pct, label, short, hollowPct) {
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
export function includeProgress(D, c) {
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
export function campaignStanding(D, c) {
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
