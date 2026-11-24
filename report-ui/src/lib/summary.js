import { levelOrder, tallyLevels, unrunIn } from "./bars.js";
import { basisOf } from "./basis.js";
import { campaignStanding } from "./campaigns.js";
import { known } from "./levels.js";
import { floorOf, isComparative } from "./rules.js";
import { isPassing } from "./status.js";
import { furthestTrack, trackTitle } from "./tracks.js";
import { plural } from "./words.js";

export function scorecardRows(D) {
  return Object.keys(D.scorecards || {})
    .map(function (name) {
      var sc = D.scorecards[name];
      var ents = Object.entries((D.scores || {})[name] || {});
      var parts = tallyLevels(ents, levelOrder(sc.levels));
      var count = function (l) {
        var p = parts.find(function (x) {
          return x.level === l;
        });
        return p ? p.n : 0;
      };
      var top = (sc.levels || [])[(sc.levels || []).length - 1];
      var scored = ents.filter(function (p) {
        return p[1].level !== "skipped";
      }).length;
      return {
        name: name,
        sc: sc,
        compare: isComparative(D, name),
        parts: parts,
        none: count("none"),
        bronze: top === "bronze" ? 0 : count("bronze"),
        top: top,
        topShare: scored ? Math.round((count(top) / scored) * 100) : 0,
        scored: scored,
        total: ents.length,
        weight: isComparative(D, name) ? -1 : count("none") * 1000 + (top === "bronze" ? 0 : count("bronze")),
      };
    })
    .sort(function (a, b) {
      return b.weight - a.weight || a.name.localeCompare(b.name);
    });
}

export function campaignNamesSorted(D) {
  var campaigns = D.campaigns || {};
  var rank = { off: 0, empty: 1, on: 2 };
  return Object.keys(campaigns)
    .filter(function (n) {
      var st = campaigns[n].status;
      return st === "active" || st === "scheduled" || st === "ended";
    })
    .sort(function (a, b) {
      var ca = campaigns[a];
      var cb = campaigns[b];
      return rank[campaignStanding(D, ca)] - rank[campaignStanding(D, cb)] || (ca.percent || 0) - (cb.percent || 0) || a.localeCompare(b);
    });
}

export function scoredKinds(D) {
  var set = {};
  Object.keys(D.scorecards || {}).forEach(function (name) {
    var k = D.scorecards[name].kind;
    if (k && !isComparative(D, name)) set[k] = true;
  });
  return set;
}

export function catalogKindsSorted(D) {
  var catalog = D.catalog || {};
  var scored = scoredKinds(D);
  return Object.keys(catalog).sort(function (a, b) {
    var as = scored[a] ? 0 : 1;
    var bs = scored[b] ? 0 : 1;
    if (as !== bs) return as - bs;
    return catalog[b].length - catalog[a].length || a.localeCompare(b);
  });
}

// Facts every view shares, computed once per load.
export function summarize(D) {
  var rows = scorecardRows(D);
  var byEntity = {};
  var marks = new Map();
  var checks = { pass: 0, fail: 0, unknown: 0, vacuous: 0 };
  rows.forEach(function (r) {
    if (r.compare) return;
    var order = levelOrder(r.sc.levels);
    var unassessed = new Set((D.unassessed || {})[r.name] || []);
    Object.entries((D.scores || {})[r.name] || {}).forEach(function (pair) {
      var id = pair[0];
      var s = pair[1];
      if (s.level === "skipped") return;
      var cur = byEntity[id];
      var rank = order.indexOf(s.level);
      var norm = ["none", "bronze", "silver", "gold"].indexOf(s.level) >= 0 ? s.level : rank <= 0 ? "none" : "gold";
      if (!cur || ["none", "bronze", "silver", "gold"].indexOf(norm) < ["none", "bronze", "silver", "gold"].indexOf(cur)) byEntity[id] = norm;
      var st = unassessed.has(id) ? "hollow" : unrunIn(r.sc, s).length ? "unrun" : "";
      if (st) (marks.get(id) || marks.set(id, []).get(id)).push([norm, st]);
      Object.keys(s.rules || {}).forEach(function (rule) {
        var o = s.rules[rule];
        if (isPassing(o) && unassessed.has(id) && basisOf((r.sc.rules || {})[rule]) === "judged") checks.vacuous++;
        else if (isPassing(o)) checks.pass++;
        else if (o === "fail") checks.fail++;
        else if (o === "unknown") checks.unknown++;
      });
    });
  });
  var catalog = D.catalog || {};
  var entityCount = Object.keys(catalog).reduce(function (n, k) {
    return n + catalog[k].length;
  }, 0);
  var kindOf = {};
  Object.keys(catalog).forEach(function (k) {
    catalog[k].forEach(function (id) {
      kindOf[id] = k;
    });
  });
  // Whether an entity's weakest level rests on a scorecard no agent has assessed it on, or on checks not run.
  var worstState = new Map();
  marks.forEach(function (list, id) {
    var at = list.filter(function (m) {
      return m[0] === byEntity[id];
    });
    var st = at.some(function (m) {
      return m[1] === "hollow";
    })
      ? "hollow"
      : at.length
        ? "unrun"
        : "";
    if (st) worstState.set(id, st);
  });
  return { rows: rows, worst: byEntity, worstState: worstState, checks: checks, entityCount: entityCount, kindOf: kindOf, repo: repoLevel(D) };
}

export function repoLevel(D) {
  var rollup = Array.isArray(D.repoRollup) ? D.repoRollup : [];
  if (!rollup.length) return null;
  var ladder = ["none", "bronze", "silver", "gold"];
  var constituents = rollup
    .filter(function (name) {
      return (D.scorecards || {})[name];
    })
    .map(function (name) {
      return { name: name, level: floorOf(D, name) || "none" };
    });
  if (!constituents.length) return null;
  constituents.forEach(function (c) {
    var lv = (D.scorecards[c.name].levels || []).slice(-1)[0];
    c.top = known(lv) || lv;
  });
  var ceiling = constituents.reduce(function (m, c) {
    return ladder.indexOf(c.top) >= 0 && ladder.indexOf(c.top) < ladder.indexOf(m) ? c.top : m;
  }, "gold");
  var floor = constituents.reduce(function (m, c) {
    return ladder.indexOf(c.level) < ladder.indexOf(m) ? c.level : m;
  }, "gold");
  var next = ladder[ladder.indexOf(floor) + 1] || null;
  return {
    ladder: ladder,
    floor: floor,
    ceiling: ceiling,
    next: next,
    constituents: constituents,
    holding: constituents.filter(function (c) {
      return c.level === floor;
    }),
  };
}

export function healthVerdict(D, S, campaignNames) {
  var campaigns = D.campaigns || {};
  var off = campaignNames.filter(function (n) {
    return campaignStanding(D, campaigns[n]) === "off";
  });
  var debt = S.rows.filter(function (r) {
    return r.none > 0 && !r.compare;
  });
  var blind = campaignNames.filter(function (n) {
    return campaignStanding(D, campaigns[n]) === "empty";
  });
  if (!campaignNames.length && !S.rows.length) return "No campaigns or scorecards in this snapshot.";
  if (!off.length && !debt.length && !blind.length) return "On track: every active campaign meets its target, and no scorecard has entities at none.";
  var bits = [];
  var far = furthestTrack(D, S);
  if (far) bits.push("the " + trackTitle(D, far) + " track is furthest from its goal");
  var scored = S.rows.filter(function (r) {
    return !r.compare;
  }).length;
  if (debt.length) bits.push(debt.length + " of " + plural(scored, "scorecard") + " " + (debt.length === 1 ? "has" : "have") + " entities at none");
  if (off.length) bits.push(plural(off.length, "campaign") + " " + (off.length === 1 ? "is" : "are") + " behind target");
  if (blind.length) bits.push(plural(blind.length, "campaign") + " " + (blind.length === 1 ? "has" : "have") + " no single score");
  return "Needs attention across the whole box: " + (bits.length > 1 ? bits.slice(0, -1).join(", ") + " and " + bits[bits.length - 1] : bits[0]) + ".";
}
