import { medal } from "./bars.js";
import { basisBadge, basisOf } from "./basis.js";
import { levelName } from "./campaigns.js";
import { el } from "./dom.js";
import { capsRepo, ring, title } from "./rings.js";
import { LADDER, floorOf, isComparative, rankLevel, ruleLink } from "./rules.js";
import { fmt } from "./words.js";

export function trackTitle(D, name) {
  var t = (D.tracks || {})[name];
  return (t && t.title) || name;
}

// A track's standing: for level scorecards, the share of entities at the top level averaged
// over its scorecards; for comparisons, the factors at parity or ahead.
export function trackStats(D, S, name) {
  var rows = S.rows.filter(function (r) {
    return (r.sc.track || null) === (name || null);
  });
  var compare =
    rows.length > 0 &&
    rows.every(function (r) {
      return isComparative(D, r.name);
    });
  var none = 0;
  var shares = [];
  var scoredRows = 0;
  var capped = [];
  // A scorecard that caps the repository is at its own top level only because its ladder stops short,
  // so it is left out of the average rather than counted as done.
  var cards = rows.map(function (r) {
    if (!r.compare) {
      none += r.none;
      if (r.scored) {
        scoredRows++;
        if (capsRepo(D, r.name, floorOf(D, r.name))) capped.push(r.name);
        else shares.push(r.topShare);
      }
    }
    return { row: r, floor: r.compare ? null : floorOf(D, r.name) };
  });
  cards.sort(function (a, b) {
    return rankLevel(a.floor) - rankLevel(b.floor) || (a.row.topShare || 0) - (b.row.topShare || 0) || b.row.none - a.row.none || title(D, a.row.name).localeCompare(title(D, b.row.name));
  });
  var verdicts = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
  var behind = [];
  rows.forEach(function (r) {
    if (!r.compare) return;
    var c = D.comparisons[r.name];
    var v = (c.verdict || {})[(c.peers || [])[0]] || "unknown";
    verdicts[v] = (verdicts[v] || 0) + 1;
    if (v === "behind") behind.push(title(D, r.name));
    var rules = verdicts.rules || (verdicts.rules = { ahead: 0, parity: 0, behind: 0, total: 0 });
    Object.keys(c.rules || {}).forEach(function (rule) {
      var x = (c.rules[rule] || {})[(c.peers || [])[0]];
      if (x in rules) rules[x]++;
      rules.total++;
    });
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
    return D.campaigns[n].track === name;
  });
  return { name: name, def: (D.tracks || {})[name] || {}, rows: rows, cards: cards, compare: compare, none: none, pct: pct, averaged: shares.length, scoredRows: scoredRows, capped: capped, verdicts: verdicts, behind: behind, campaigns: campaigns };
}

export function tracksSorted(D, S) {
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

export function unassessedNote(D, card, scored) {
  var ids = (D.unassessed || {})[card] || [];
  var inScope = ids.filter(function (id) {
    return ((D.scores[card] || {})[id] || {}).level !== "skipped";
  }).length;
  if (!inScope) return null;
  var rules = ((D.scorecards || {})[card] || {}).rules || {};
  var passing = [];
  var failing = [];
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
    var out = [];
    names(rs).forEach(function (a, i) {
      if (i) out.push(i === rs.length - 1 ? " and " : ", ");
      out.push(a);
    });
    return out;
  };
  var why = [];
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

export function unassessedIn(D, card, id) {
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
    tracks.some(function (t) {
      return t.capped && t.capped.length;
    })
      ? el("span", {}, el("span", { className: "dot", style: "background:var(--unknown)" }), "a scorecard that caps the repository, left out of the average")
      : null,
    compare ? el("b", { textContent: "Segmented ring, one per factor:" }) : null,
    compare ? sw(standingColor("ahead"), "ahead") : null,
    compare ? sw(standingColor("parity"), "at parity") : null,
    compare ? sw(standingColor("behind"), "behind") : null,
    el("span", { className: "medal-key" }, el("b", { textContent: "Medal:" }), medal("bronze", 16), "its weakest entity's level ·", medal("none", 16), "none"),
    quoted ? el("span", {}, "A quoted level such as " + levelName(quoted) + " is its scorecard's own name for a level.") : null,
  );
  return el("details", { className: "keyd" }, el("summary", { textContent: "How to read: one bar per scorecard, and the ring is their average" }), key);
}

export function furthestTrack(D, S) {
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
  return el("span", { className: "standing " + v, textContent: STANDING_MARK[v] + " " + (text || v) });
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
