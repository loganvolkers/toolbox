import type { ReportData } from "../types";
import type { Row } from "./summary";
import { levelOrder, medal, provisionalOf, unassessedOf } from "./bars";
import { el, enc, link } from "./dom";
import { kindCount, scorecardSplit } from "./kinds";
import { known } from "./levels";
import { isTop, shortId, title } from "./rings";
import { LADDER, floorOf, isComparative, rankLevel, ruleTitle } from "./rules";
import { blockers } from "./status";
import { unassessedIn } from "./tracks";
import { fmt, plural } from "./words";

// The first scorecard on a track that can still rise, with the rule most of its lowest entities need.
// A scorecard's own level name ("ten", "clean") is quoted so it reads as a name, not a stray word.
// "Ten" is an OpenSSF check id. In a next-step sentence it reads as a stray word, and as a link it is under 24 px wide.
function ruleWords(name, rule) {
  var t = ruleTitle(name, rule);
  return t === "Ten" ? "a score of 10" : t;
}

export function levelName(l) {
  return l === "none" || l === "not scored" || LADDER.indexOf(l) >= 0 ? l : "“" + l + "”";
}

// One summary of a scorecard row wherever a list shows it: scope, share at its own top level, and the
// levels that may not hold, so a list never reads better than the scorecard's page.
export function scCaption(D: ReportData, r, split?) {
  var head = (r.sc.kind ? kindCount(D, r.sc.kind, r.total) + (split ? scorecardSplit(D, r.sc.kind) : "") : plural(r.total, "entity", "entities")) + (r.total > r.scored ? ", " + fmt(r.total - r.scored) + " not in scope" : "");
  var prov = provisionalOf(D, r.name);
  var unj = unassessedOf(D, r.name);
  var idle = idleLowestMark(D, r.name);
  return [
    head + (r.scored ? " · " + r.topShare + "% at the top level (" + levelName(r.top || "top") + ")" : " · not scored yet"),
    prov ? el("span", { className: "provisional", title: "Their levels hold only if the checks not run in this build pass; drawn dotted in the bar", textContent: " · " + fmt(prov) + " provisional (checks not run)" }) : null,
    unj ? el("span", { className: "unjudged-n", title: "No agent has assessed them yet, so their agent-judged rules pass or fail for want of anything to read; drawn hatched in the bar", textContent: " · " + fmt(unj) + " not assessed by an agent (hatched)" }) : null,
    idle ? el("span", { textContent: " · " + idle }) : null,
  ];
}

// The weakest scorecard on a track, when it sits more than one level below its top.
// Lifting it does not move the track's percentage, so the list says so beside the row the Next box names.
export function idleLowestMark(D: ReportData, name) {
  var sc = (D.scorecards || {})[name];
  if (!sc || !sc.track || isComparative(D, name)) return null;
  var peers = Object.keys(D.scorecards || {}).filter(function (n) {
    return ((D.scorecards || {})[n] || {}).track === sc.track && !isComparative(D, n);
  });
  var ranked = peers
    .map(function (n) {
      return { n: n, floor: floorOf(D, n) };
    })
    .filter(function (x) {
      return x.floor && !isTop(D, x.n, x.floor);
    });
  ranked.sort(function (a, b) {
    return rankLevel(a.floor) - rankLevel(b.floor) || title(D, a.n).localeCompare(title(D, b.n));
  });
  if (!ranked.length || ranked[0].n !== name) return null;
  var order = ["none"].concat((sc.levels || []).filter(function (l) { return l !== "none"; }));
  var idx = order.indexOf(ranked[0].floor as string);
  if (idx < 0 || idx >= order.length - 2) return null;
  return "Lowest on this track; lifting it adds no points until those entities reach the top level.";
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

// A decision file's basename is an id. The next step leads with what it is; the filename follows in small type.
function entityLead(id) {
  var file = shortId(id);
  if (/^D0[0-9A-Z]{8,}\.md$/.test(file)) return { label: "Decision record", file: file };
  return { label: file, file: null };
}

// The entities sitting at `level` on a scorecard, named when a reader can hold the list (three or fewer).
function namedAt(D: ReportData, name, level) {
  var ids = Object.keys((D.scores || {})[name] || {})
    .filter(function (id) {
      return D.scores[name][id].level === level;
    })
    .sort();
  if (!ids.length || ids.length > 3) return null;
  var bits: (HTMLElement | string)[] = [];
  ids.forEach(function (id, i) {
    if (i) bits.push(i === ids.length - 1 ? " and " : ", ");
    var lead = entityLead(id);
    bits.push(link("entity/" + enc(id), "", lead.label));
    if (lead.file) bits.push(el("small", { className: "sub", textContent: " (" + lead.file + ")" }));
  });
  return bits;
}

// A track's next step is the one that moves its percentage most; the next medal on its weakest
// scorecard follows, with its gain stated, because a step below a scorecard's top level adds nothing.
// The basis badge stays off this sentence: it belongs on the rule, and in the sentence it splits the
// rule from the verb.
export function nextMedalEl(D: ReportData, cards, label?, why?) {
  var nm = nextMedal(D, cards);
  var tracked = Object.keys(D.tracks || {}).length > 0;
  var movers = tracked ? ringMovers(D, cards) : [];
  var verified = movers.find(function (m) {
    return !m.waits;
  });
  var unverified = movers.find(function (m) {
    return m.waits && (!verified || m.gain > verified.gain);
  });
  if (!nm && !verified) return null;
  var medalBody = function () {
    if (!nm) return [];
    var card = link("scorecard/" + enc(nm.name), "", title(D, nm.name));
    var rule = nm.rule ? [link("rule/" + enc(nm.name) + "/" + enc(nm.rule), "", ruleWords(nm.name, nm.rule))] : [];
    var named = nm.rule && nm.n === nm.low ? namedAt(D, nm.name, nm.floor) : null;
    var entities = plural(nm.low, "entity", "entities") + " at " + nm.floor;
    if (named) return [...named, " on ", card, nm.low === 1 ? " needs " : " need ", ...rule, "."];
    return nm.rule && nm.n === nm.low
      ? [entities + " on ", card, nm.low === 1 ? " needs " : " need ", ...rule, "."]
      : ["lift " + entities + " to " + nm.up + " on ", card, ...(nm.rule ? [". " + nm.n + (nm.n === 1 ? " needs " : " need "), ...rule, " first."] : ["."])];
  };
  var top = nm ? (((D.scorecards || {})[nm.name] || {}).levels || []).slice(-1)[0] : "";
  var medalAtTop = nm ? isTop(D, nm.name, nm.up) : false;
  var medalNote = !nm ? "" : medalAtTop ? " That is its top level." : " Its top level is " + top + (tracked ? ", so this adds no points to the track's percentage until they reach it." : ".");
  var leadVerified = !!(verified && !(medalAtTop && verified.r.name === nm?.name));
  var extras = [
    leadVerified && nm ? el("div", { className: "sub mover" }, el("b", { textContent: "Next medal: " }), ...medalBody(), medalNote) : null,
    unverified ? moverEl(D, unverified) : null,
  ].filter(Boolean);
  var more = extras.length ? el("details", { className: "sub", style: "margin-top:6px" }, el("summary", { textContent: "Other steps" }), ...extras) : null;
  if (leadVerified && verified) {
    return el(
      "div",
      { className: "nextmedal" },
      el("b", { textContent: label || "Next: " }),
      ...moverBody(D, verified),
      why ? el("span", { className: "sub" }, why) : null,
      more,
    );
  }
  if (!nm) return null;
  return el(
    "div",
    { className: "nextmedal" },
    el("b", { textContent: label || "Next medal: " }),
    ...medalBody(),
    el("span", { className: "sub" }, medalNote + (why || "")),
    more,
  );
}

type Mover = { r: Row; n: number; below: string; top: string; gain: number; rule: [string, number] | undefined; waits: [string, number] | false | undefined };

// Steps that raise a track's ring, largest first: lifting a scorecard's entities one level below its
// top to the top, since the ring averages each scorecard's share at its top level.
function ringMovers(D: ReportData, cards): Mover[] {
  var rows: Row[] = cards
    .map(function (c) {
      return c.row;
    })
    .filter(function (r) {
      return !r.compare && r.scored;
    });
  var out: Mover[] = [];
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
    if (gain < 1) return;
    var top = lv[lv.length - 1];
    var sc = (D.scorecards || {})[r.name] || {};
    var held: Record<string, number> = {};
    var unrun: Record<string, number> = {};
    var order = sc.levels || [];
    Object.values((D.scores || {})[r.name] || {}).forEach(function (s) {
      if (s.level !== below) return;
      blockers(sc, s, top).forEach(function (k) {
        held[k] = (held[k] || 0) + 1;
      });
      Object.keys(s.rules || {}).forEach(function (k) {
        if (s.rules[k] === "unknown" && order.indexOf(((sc.rules || {})[k] || {}).level) <= order.indexOf(top)) unrun[k] = (unrun[k] || 0) + 1;
      });
    });
    var byCount = function (x, y) {
      return y[1] - x[1] || x[0].localeCompare(y[0]);
    };
    var waits = !Object.keys(held).length && (Object.entries(unrun).sort(byCount)[0] as [string, number] | undefined);
    var rule = Object.entries(held).sort(byCount)[0] as [string, number] | undefined;
    out.push({ r: r, n: n, below: below, top: top, gain: gain, rule: rule, waits: waits });
  });
  return out.sort(function (a, b) {
    return b.gain - a.gain || a.r.name.localeCompare(b.r.name);
  });
}

function points(g: number) {
  var p = Math.round(g);
  return p + (p === 1 ? " point" : " points");
}

// The verified step that moves a track's percentage most. The same step leads the overview and the track card.
export function verifiedMover(D: ReportData, cards): Mover | null {
  if (!Object.keys(D.tracks || {}).length) return null;
  return (
    ringMovers(D, cards).find(function (m) {
      return !m.waits;
    }) || null
  );
}

export function moverBits(D: ReportData, b: Mover) {
  return moverBody(D, b);
}

function moverBody(D: ReportData, b: Mover) {
  var rule = b.rule;
  var entities = plural(b.n, "entity", "entities") + " at " + levelName(b.below);
  var card = link("scorecard/" + enc(b.r.name), "", title(D, b.r.name));
  var ruleEl = rule ? [link("rule/" + enc(b.r.name) + "/" + enc(rule[0]), "", ruleWords(b.r.name, rule[0]))] : [];
  var named = rule && rule[1] === b.n ? namedAt(D, b.r.name, b.below) : null;
  var gain = " The track gains about " + points(b.gain) + ".";
  // Naming the entities dropped the count, so a reader had to open the campaign to learn how many.
  var kind = ((D.scorecards || {})[b.r.name] || {}).kind;
  var count = kind ? kindCount(D, kind, b.n) : plural(b.n, "entity", "entities");
  if (named) return [count + ", ", ...named, ", on ", card, b.n === 1 ? " needs " : " need ", ...ruleEl, ".", gain];
  return [
    ...(rule && rule[1] === b.n
      ? [entities + " on ", card, b.n === 1 ? " needs " : " need ", ...ruleEl, "."]
      : ["Lift " + entities + " to " + levelName(b.top) + " on ", card, ...(rule ? [". " + rule[1] + (rule[1] === 1 ? " needs " : " need "), ...ruleEl, " first."] : ["."])]),
    gain,
  ];
}

function moverEl(D: ReportData, b: Mover) {
  var waits = b.waits as [string, number];
  var card = link("scorecard/" + enc(b.r.name), "", title(D, b.r.name));
  return el(
    "div",
    { className: "sub mover" },
    el("b", { textContent: "Unverified, run a check first: " }),
    plural(b.n, "entity", "entities") + " at " + levelName(b.below) + " on ",
    card,
    " may already meet " + levelName(b.top) + ", but its check ",
    link("rule/" + enc(b.r.name) + "/" + enc(waits[0]), "", ruleWords(b.r.name, waits[0])),
    " was not run in this build, so the gain is unknown until it runs.",
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
// Pass, fail, not run and not assessed, each a segment, so the four counts are the whole bar.
export function checkBar(parts: { cls: string; n: number }[], label) {
  var total = parts.reduce(function (s, p) {
    return s + p.n;
  }, 0) || 1;
  var g = el(
    "div",
    { className: "kpi-bar parts" },
    ...parts.filter(function (p) { return p.n > 0; }).map(function (p) {
      return el("span", { className: p.cls, style: "width:" + (100 * p.n) / total + "%" });
    }),
  );
  g.setAttribute("role", "img");
  g.setAttribute("aria-label", label);
  return g;
}

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
