import type { Summary } from "./summary";
import type { ReportData } from "../types";
import type { Kid } from "./dom";

export type Tally = { ahead: number; parity: number; behind: number; unknown: number };
import { medal } from "./bars";
import { basisBadge, basisOf } from "./basis";
import { levelName } from "./campaigns";
import { el } from "./dom";
import { capsRepo, ring, title } from "./rings";
import { LADDER, floorOf, isComparative, rankLevel, ruleLink } from "./rules";
import { fmt } from "./words";

export function trackTitle(D: ReportData, name) {
  var t = (D.tracks || {})[name];
  return (t && t.title) || name;
}

// A track's standing: for level scorecards, the share of entities at the top level averaged
// over its scorecards; for comparisons, the factors at parity or ahead.
export function trackStats(D: ReportData, S: Summary, name) {
  var rows = S.rows.filter(function (r) {
    return (r.sc.track || null) === (name || null);
  });
  var compare =
    rows.length > 0 &&
    rows.every(function (r) {
      return isComparative(D, r.name);
    });
  var noneIds: Record<string, boolean> = {};
  var noneRows = 0;
  var shares: number[] = [];
  var scoredRows = 0;
  var capped: string[] = [];
  var short: string[] = [];
  // Every scored scorecard counts at its own top level, in every portal; one whose ladder stops
  // below gold is named, and one that also caps the repository level gets a lock note.
  var cards = rows.map(function (r) {
    if (!r.compare) {
      var scoredHere = (D.scores || {})[r.name] || {};
      Object.keys(scoredHere).forEach(function (id) {
        if (scoredHere[id].level === "none") noneIds[id] = true;
      });
      // The scorecard cards list a row each. Their sum is larger when one entity is at none on more than one.
      noneRows += r.none || 0;
      if (r.scored) {
        scoredRows++;
        shares.push(r.topShare);
        if (capsRepo(D, r.name, floorOf(D, r.name))) capped.push(r.name);
        var lv = ((D.scorecards || {})[r.name] || {}).levels || [];
        var top = lv[lv.length - 1];
        if (top && LADDER.indexOf(top) > 0 && top !== LADDER[LADDER.length - 1]) short.push(r.name);
      }
    }
    return { row: r, floor: r.compare ? null : floorOf(D, r.name) };
  });
  cards.sort(function (a, b) {
    return rankLevel(a.floor) - rankLevel(b.floor) || (a.row.topShare || 0) - (b.row.topShare || 0) || b.row.none - a.row.none || title(D, a.row.name).localeCompare(title(D, b.row.name));
  });
  var verdicts: Tally & { rules?: { ahead: number; parity: number; behind: number; total: number }; both?: Both; majority?: boolean } = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
  var behind: string[] = [];
  rows.forEach(function (r) {
    if (!r.compare) return;
    var c = (D.comparisons || {})[r.name];
    var v = (c.verdict || {})[(c.peers || [])[0]] || "unknown";
    verdicts[v] = (verdicts[v] || 0) + 1;
    if (v === "behind") behind.push(title(D, r.name));
    var rules = verdicts.rules || (verdicts.rules = { ahead: 0, parity: 0, behind: 0, total: 0 });
    var k = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
    Object.keys(c.rules || {}).forEach(function (rule) {
      var x = (c.rules[rule] || {})[(c.peers || [])[0]];
      if (x in rules) rules[x]++;
      rules.total++;
      k[x in k ? x : "unknown"]++;
    });
    var both = verdicts.both || (verdicts.both = { strict: { ahead: 0, parity: 0, behind: 0, unknown: 0 }, majority: { ahead: 0, parity: 0, behind: 0, unknown: 0 } });
    both.strict[standingBy(k, false)]++;
    both.majority[standingBy(k, true)]++;
  });
  var pct = compare
    ? Math.round(((verdicts.ahead + verdicts.parity) / rows.length) * 100)
    : shares.length
      ? Math.round(
          shares.reduce(function (a, b) {
            return a + b;
          }, 0) / shares.length,
        )
      : null;
  var campaigns = Object.keys(D.campaigns || {}).filter(function (n) {
    return (D.campaigns || {})[n].track === name;
  });
  var majority = compare && byMajority(D, rows.map(function (r) {
    return r.name;
  }));
  if (compare) verdicts.majority = majority;
  return { name: name, def: (D.tracks || {})[name] || {}, rows: rows, cards: cards, compare: compare, majority: majority, none: Object.keys(noneIds).length, noneRows: noneRows, pct: pct, averaged: shares.length, scoredRows: scoredRows, capped: capped, short: short, verdicts: verdicts, behind: behind, campaigns: campaigns };
}

export function tracksSorted(D: ReportData, S: Summary) {
  return Object.keys(D.tracks || {})
    .map(function (n) {
      return trackStats(D, S, n);
    })
    .filter(function (t) {
      return t.rows.length || t.campaigns.length;
    })
    .sort(function (a, b) {
      return (a.compare ? 1 : 0) - (b.compare ? 1 : 0) || (a.pct == null ? 101 : a.pct) - (b.pct == null ? 101 : b.pct) || b.none - a.none || a.name.localeCompare(b.name);
    });
}

export function unassessedNote(D: ReportData, card, scored) {
  var ids = (D.unassessed || {})[card] || [];
  var inScope = ids.filter(function (id) {
    return ((D.scores[card] || {})[id] || {}).level !== "skipped";
  }).length;
  if (!inScope) return null;
  var rules = ((D.scorecards || {})[card] || {}).rules || {};
  var passing: string[] = [];
  var failing: string[] = [];
  Object.keys(rules)
    .filter(function (r) {
      return basisOf(rules[r]) === "judged";
    })
    .forEach(function (r) {
      var fails = ids.some(function (id) {
        var o = (((D.scores[card] || {})[id] || {}).rules || {})[r];
        return o && o !== "pass";
      });
      (fails ? failing : passing).push(r);
    });
  var names = function (rs) {
    return rs.map(function (r) {
      return ruleLink(card, r);
    });
  };
  var list = function (rs) {
    var out: Kid[] = [];
    names(rs).forEach(function (a, i) {
      if (i) out.push(i === rs.length - 1 ? " and " : ", ");
      out.push(a);
    });
    return out;
  };
  var why: Kid[] = [];
  if (passing.length) why.push(...list(passing), passing.length === 1 ? " passes" : " pass", " only because there is nothing to fail on");
  if (passing.length && failing.length) why.push("; ");
  if (failing.length) why.push(...list(failing), failing.length === 1 ? " fails" : " fail", " until an assessment exists");
  why.push(passing.length ? ". A level they reach means nothing has been found, not that a review found nothing." : ".");
  return el(
    "div",
    { className: "callout judged-note" },
    el("span", {}, el("b", { textContent: (inScope === scored ? "No entity here has an" : fmt(inScope) + " of " + fmt(scored) + " entities have no") + " agent's assessment yet. " }), "On those, of the ", basisBadge("judged", true), " rules, ", ...why),
  );
}

export function unassessedIn(D: ReportData, card, id) {
  return ((D.unassessed || {})[card] || []).indexOf(id) >= 0;
}

// What a track card's ring colours and a comparison's segments mean, in place of a paragraph.
export function trackKey(tracks) {
  var sw = function (c, t) {
    return el("span", {}, el("span", { className: "dot", style: "background:" + c }), t);
  };
  var compare = tracks.some(function (t) {
    return t.compare;
  });
  var quoted = null;
  tracks.forEach(function (t) {
    (t.cards || []).forEach(function (c) {
      if (!quoted && c.floor && c.floor !== "none" && LADDER.indexOf(c.floor) < 0) quoted = c.floor;
    });
  });
  var key = el(
    "div",
    { className: "legend trackkey" },
    el("b", { textContent: "Ring, share at the top level:" }),
    sw(RAMP, "the green arc's length is the share; grey is not scored"),
    el(
      "span",
      { className: "slot-key" },
      el("b", { textContent: "Bars:" }),
      el(
        "span",
        { className: "slots mini", ariaHidden: "true", style: "--avg:55%" },
        ...[80, 30, 55].map(function (p) {
          return el("span", { className: "slot", style: "--h:" + p + "%;--c:" + RAMP });
        }),
        el("span", { className: "avg" }),
      ),
      "one per scorecard, its share at the top level (point at, tap or arrow through a bar to name it under the strip); dashed: their average, the ring",
    ),
    compare ? el("b", { textContent: "Segmented ring, one per factor:" }) : null,
    compare ? sw(standingColor("ahead"), "ahead") : null,
    compare ? sw(standingColor("parity"), "at parity") : null,
    compare ? sw(standingColor("behind"), "behind") : null,
    el("span", { className: "medal-key" }, el("b", { textContent: "Medal:" }), medal("bronze", 16), "its weakest entity's level ·", medal("none", 16), "none"),
    quoted ? el("span", {}, "A quoted level such as " + levelName(quoted) + " is its scorecard's own name for a level.") : null,
  );
  return el("details", { className: "keyd" }, el("summary", { textContent: "How to read: one bar per scorecard, and the ring is their average" }), key);
}

export function furthestTrack(D: ReportData, S: Summary) {
  var t = tracksSorted(D, S).find(function (t) {
    return !t.compare && t.pct != null && t.pct < 100;
  });
  return t ? t.name : null;
}

// One hue for a share at the top level, so arc length alone carries it. Grey stays "not scored" and
// parity, red the none level and "behind", and bronze, silver and gold belong to medals.
export var RAMP = "var(--pass)";

export function trackColor(t) {
  if (t.compare) return "var(--accent)";
  if (t.pct == null) return "var(--faint)";
  return RAMP;
}

export var STANDING_MARK = { ahead: "▲", parity: "=", behind: "▼", unknown: "?" };

export function standingChip(s, text) {
  var v = s || "unknown";
  return el("span", { className: "standing " + v, textContent: STANDING_MARK[v] + " " + (text || (v === "unknown" ? "not measured" : v)) });
}

// Whether every named comparison tallies its factors by the majority of rules rather than behind-on-any
// (D01M87WF2QZVPTD8CGDZKY133CD).
export function byMajority(D: ReportData, names) {
  return (
    names.length > 0 &&
    names.every(function (n) {
      return ((D.comparisons || {})[n] || {}).verdictRule === "majority";
    })
  );
}

export type Both = { strict: Tally; majority: Tally };

// A factor's standing from its rules' standings under either counting rule (D01M87WF2QZVPTD8CGDZKY133CD):
// strict is behind if any rule is behind, majority follows whichever side has more rules.
export function standingBy(k: Tally, majority: boolean): keyof Tally {
  if (majority) return k.unknown ? "unknown" : k.ahead > k.behind ? "ahead" : k.behind > k.ahead ? "behind" : "parity";
  return k.behind ? "behind" : k.unknown ? "unknown" : k.ahead ? "ahead" : "parity";
}

// The chip for a factor whose rules split both ways, or that leads on under half of them.
export function factorChip(v, counts, total) {
  if (counts.ahead && counts.behind) return v + ", mixed";
  return v === "ahead" && counts.ahead * 2 < total ? "narrowly ahead" : null;
}

// "Toolbox is behind Backstage", in words that read as a sentence.
export function standingPhrase(a, s, b) {
  if (s === "ahead") return a + " ahead of " + b;
  if (s === "behind") return a + " behind " + b;
  if (s === "parity") return a + " at parity with " + b;
  return a + " vs " + b + ": not measured";
}

export function standingColor(s) {
  return s === "ahead" ? "var(--pass)" : s === "behind" ? "var(--fail)" : s === "parity" ? "var(--muted)" : "var(--unknown)";
}
