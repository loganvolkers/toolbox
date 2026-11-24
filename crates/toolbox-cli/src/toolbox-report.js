// <toolbox-report>: Overview, Catalog, Campaigns, Scorecards, and Compare as a custom element.
//
// Data is a child <script type="application/json">, or the src attribute (a URL of that JSON).
// A host page projects its own chrome into the slots header, nav, and footer. Those children
// stay in light DOM; this file does not read or concatenate them. Mode chrome and scorecard
// navigation live inside the shadow tree and are not slots.
// Generated from report-ui/src by `npm run build`; edit the sources, not this file.
(() => {
  // src/lib/dom.js
  var SVGNS = "http://www.w3.org/2000/svg";
  var KNOWN = ["none", "bronze", "silver", "gold", "skipped"];
  var TAG = /^[a-z][a-z0-9]*$/;
  function el(t, a) {
    if (!TAG.test(t)) throw new Error("bad tag name");
    var e = document.createElement(t);
    var kids = Array.prototype.slice.call(arguments, 2);
    Object.assign(e, a || {});
    kids.forEach(function(k) {
      if (k === "" || k == null || k === false) return;
      e.append(k);
    });
    return e;
  }
  function svg(t, attrs) {
    var e = document.createElementNS(SVGNS, t);
    Object.keys(attrs || {}).forEach(function(k) {
      e.setAttribute(k, attrs[k]);
    });
    Array.prototype.slice.call(arguments, 2).forEach(function(k) {
      if (k) e.append(k);
    });
    return e;
  }
  function link(path, cls, kids) {
    var a = el("a", { href: "#" + path, className: cls || "" });
    (Array.isArray(kids) ? kids : [kids]).forEach(function(k) {
      if (k != null && k !== "") a.append(k);
    });
    return a;
  }
  function enc(s) {
    return encodeURIComponent(s);
  }

  // src/lib/levels.js
  var ALIAS = {};
  var LEVEL_NOTE = {};
  var LEVEL_DEF = {};
  var ONLY_LEVEL = /* @__PURE__ */ new Set();
  function aliasLevels(D) {
    Object.entries(D.scorecards || {}).forEach(function(e) {
      var sc = e[1];
      var levels = sc.levels || [];
      levels.forEach(function(l, i) {
        if (KNOWN.indexOf(l) < 0 && !ALIAS[l]) {
          ALIAS[l] = ["gold", "silver", "bronze"][Math.min(2, levels.length - 1 - i)];
          var said = (sc.description || "").split(/(?<=\.)\s+/).filter(function(s) {
            return s.toLowerCase().split(/[^a-z0-9-]+/).indexOf(String(l).toLowerCase()) >= 0;
          })[0];
          if (said) LEVEL_DEF[l] = said.replace(/`/g, "");
          LEVEL_NOTE[l] = "“" + l + "” is a level " + (sc.title || e[0]) + " names itself" + (levels.length === 1 ? ": its only level, so reaching it is the top." : ", ranked like " + ALIAS[l] + ".") + (said ? " " + LEVEL_DEF[l] : "");
          if (levels.length === 1) ONLY_LEVEL.add(l);
        }
      });
    });
  }
  function ownLevel(D, kind, l) {
    var cards = Object.keys(D.scorecards || {}).filter(function(n) {
      return D.scorecards[n].kind === kind && !(D.comparisons || {})[n];
    }).map(function(n) {
      return D.scorecards[n];
    });
    if (cards.length !== 1 || l === "none" || l === "unscored") return l;
    var own = (cards[0].levels || []).find(function(x) {
      return x === l || known(x) === l;
    });
    return own || l;
  }
  function known(l) {
    return KNOWN.indexOf(l) >= 0 ? l : ALIAS[l] || null;
  }
  var UNSCORED = "color-mix(in srgb,var(--faint) 45%,var(--bg))";
  var UNRUN = "radial-gradient(circle,var(--faint) 1.3px,transparent 1.7px) 0 0/5px 5px";
  function fill(l) {
    if (l === "unscored") return UNSCORED;
    return "var(--" + (known(l) || "accent") + ")";
  }
  function ink(l) {
    return known(l) ? "var(--" + known(l) + "-ink)" : "var(--accent)";
  }

  // src/lib/text.js
  var CITATION = "(?:D\\d+|D0[0-9A-Z]{20,}|Q\\d+|lessons? L\\d+(?: and L\\d+)?|plan §[\\d.]+|[\\w./-]+\\.md(?:#[\\w-]+| §[\\d.]+)?(?: for [a-z ]+)?|step \\d+)";
  var DECISION_REFS = new RegExp("\\s*\\(" + CITATION + "(?:[,;] ?" + CITATION + ")*\\)", "g");
  function prose(s) {
    return String(s || "").replace(DECISION_REFS, "");
  }
  var GLOSSARY = {
    ratchet: "The CI gate that compares each build with the committed results and fails when any score falls.",
    cel: "Common Expression Language: the small expression language a rule's check is written in.",
    fixture: "A sample input in the scorecard's tests, built to pass or to fail a rule, so the rule is shown to decide both ways.",
    harness: "The program that runs something: an agent harness (such as Claude Code, Cursor or Codex) runs an agent, and a benchmark harness runs and times the products it compares."
  };
  var GLOSS_RE = /\b(ratchet|CEL|fixtures?|harness)\b/i;
  function glossed(text) {
    var m = GLOSS_RE.exec(text);
    if (!m) return [text];
    var word = m[1].toLowerCase();
    var def = GLOSSARY[word] || GLOSSARY[word.replace(/s$/, "")];
    if (!def || word === "cel" && m[1] !== "CEL") return [text];
    var term = el("abbr", { className: "gloss", tabIndex: 0 }, m[1], el("span", { className: "sr", textContent: " (" + def + ")" }));
    term.dataset.tip = def;
    return [text.slice(0, m.index), term, text.slice(m.index + m[1].length)];
  }
  function rich(s) {
    var first = true;
    return prose(s).split("`").flatMap(function(part, i) {
      if (i % 2) return [el("code", { textContent: part })];
      if (!first) return [part];
      var out = glossed(part);
      if (out.length > 1) first = false;
      return out;
    });
  }
  function richCards(D, s, self) {
    return rich(s).flatMap(function(part) {
      if (typeof part !== "string") return [part];
      return part.split(/([a-z0-9]+(?:-[a-z0-9]+)+)/).map(function(bit, i) {
        return i % 2 && bit !== self && (D.scorecards || {})[bit] ? link("scorecard/" + enc(bit), "", title(D, bit)) : bit;
      });
    });
  }

  // src/lib/words.js
  function whyFirst(text) {
    var first = firstSentence(text);
    var rest = String(text).trim().slice(first.length).trim();
    return el("span", { className: "why" }, ...rich(first), rest ? el("span", { className: "rest" }, " ", ...rich(rest)) : null);
  }
  function scPurpose(sc) {
    var strip = function(t) {
      return firstSentence(t).replace(/\s*\([^)]*\)/g, "");
    };
    return sc.rationale ? "Why: " + strip(sc.rationale) : strip(sc.description || "");
  }
  function unscoredWhy(text) {
    var rest = String(text || "").trim();
    var first = firstSentence(rest);
    while (first && rest) {
      if (/not scored/.test(first) && first !== firstSentence(text)) return el("small", { className: "kwhy" }, ...rich(first));
      rest = rest.slice(first.length).trim();
      first = firstSentence(rest);
    }
    return null;
  }
  function sentences(text) {
    var out = [];
    var t = String(text || "").trim();
    while (t) {
      var f = firstSentence(t);
      out.push(f);
      t = t.slice(f.length).trim();
    }
    return out;
  }
  function firstSentence(text) {
    var t = String(text || "").trim();
    var depth = 0;
    var code = false;
    for (var i = 0; i < t.length - 2; i++) {
      var ch = t.charAt(i);
      if (ch === "`") code = !code;
      else if (!code && (ch === "[" || ch === "(")) depth++;
      else if (!code && (ch === "]" || ch === ")")) depth = Math.max(0, depth - 1);
      else if (!code && !depth && (ch === "." || ch === "!" || ch === "?") && /\s/.test(t.charAt(i + 1)) && /[A-Z]/.test(t.charAt(i + 2))) return t.slice(0, i + 1);
    }
    return t;
  }
  function lowerFirst(t) {
    return /^[A-Z][a-z]/.test(t || "") && !/^[A-Z][a-z]*[A-Z]/.test(t) ? t.charAt(0).toLowerCase() + t.slice(1) : t;
  }
  function andList(items) {
    return items.length > 1 ? items.slice(0, -1).join(", ") + " and " + items[items.length - 1] : items.join("");
  }
  function plural(n, word, many) {
    return fmt(n) + " " + (n === 1 ? word : many || word + "s");
  }
  function fmt(n) {
    return Number(n).toLocaleString("en-US");
  }

  // src/lib/rules.js
  var RULE_TITLE = /* @__PURE__ */ new Map();
  var RULE_WORDS = { api: "API", ci: "CI", id: "ID", ids: "IDs", mcp: "MCP", sdlc: "SDLC", sha: "SHA", otel: "OTel", otlp: "OTLP", cel: "CEL", url: "URL", ui: "UI", kubernetes: "Kubernetes" };
  var RULE_WHY = /* @__PURE__ */ new Map();
  var SC_NAME = /* @__PURE__ */ new WeakMap();
  function ruleTitles(D) {
    RULE_TITLE.clear();
    RULE_WHY.clear();
    Object.entries(D.scorecards || {}).forEach(function(e) {
      if (e[1] && typeof e[1] === "object") SC_NAME.set(e[1], e[0]);
      Object.entries(e[1].rules || {}).forEach(function(r) {
        if (r[1] && r[1].title) RULE_TITLE.set(e[0] + "\0" + r[0], r[1].title);
        if (r[1] && r[1].rationale)
          RULE_WHY.set(
            e[0] + "\0" + r[0],
            firstSentence(r[1].rationale).replace(/\[([^\]]+)\]\([^)]*\)/g, "$1").replace(/`/g, "")
          );
      });
    });
  }
  function ruleTitle(scName, rule) {
    var t = RULE_TITLE.get(scName + "\0" + rule);
    if (t) return t;
    return String(rule).split("-").map(function(w, i) {
      if (Object.prototype.hasOwnProperty.call(RULE_WORDS, w)) return RULE_WORDS[w];
      return i ? w : w.charAt(0).toUpperCase() + w.slice(1);
    }).join(" ");
  }
  function ruleTitleOf(sc, rule) {
    return ruleTitle(SC_NAME.get(sc) || "", rule);
  }
  function ruleLink(scName, rule, text, shown) {
    var a = link("rule/" + enc(scName) + "/" + enc(rule), "rl", text || ruleTitle(scName, rule));
    var why = !shown && RULE_WHY.get(scName + "\0" + rule);
    if (why) {
      a.dataset.tip = "Why: " + why + (text ? "" : " (" + rule + ")");
      a.setAttribute("aria-description", "Why: " + why);
    } else if (!text) a.title = rule;
    return a;
  }
  function memberLinks(ids) {
    var cut = ids.length > 8 ? 6 : ids.length;
    return el(
      "div",
      { className: "members" },
      "In it: ",
      ...ids.slice(0, cut).flatMap(function(id, i) {
        var name = shortId(id);
        var twin = ids.some(function(o) {
          return o !== id && shortId(o) === name;
        });
        return [i ? " " : "", el("span", { style: "white-space:nowrap" }, link("entity/" + enc(id), "", twin ? id.replace(/^[^:]*:/, "") : name), i < cut - 1 ? "," : "")];
      }),
      ids.length > cut ? " and " + (ids.length - cut) + " more" : null
    );
  }
  function isComparative(D, name) {
    return !!(D.comparisons || {})[name];
  }
  var LADDER = ["none", "bronze", "silver", "gold"];
  function rankLevel(l) {
    if (l == null) return 9;
    var k = known(l);
    return k ? LADDER.indexOf(k) : 0;
  }
  function floorPill(D, name) {
    var floor = floorOf(D, name);
    var pill = badge(floor || "none");
    if (!floor || floor === "none") return pill;
    var sc = (D.scorecards || {})[name];
    var rows = (D.scores || {})[name] || {};
    var un = new Set((D.unassessed || {})[name] || []);
    var at = 0;
    var unj = 0;
    var unr = 0;
    Object.keys(rows).forEach(function(id) {
      if (id.indexOf("/demo/") >= 0 || rows[id].level !== floor) return;
      at++;
      if (un.has(id)) unj++;
      else if (unrunIn(sc, rows[id]).length) unr++;
    });
    if (!at || (unj + unr) * 2 < at) return pill;
    var judged = unj >= unr;
    pill.classList.add("vacuous", judged ? "unjudged" : "unrun");
    pill.append(judged ? el("span", { className: "vhatch", ariaHidden: "true" }) : el("span", { className: "vglyph", ariaHidden: "true", textContent: " ◌" }));
    pill.setAttribute("aria-label", "level " + floor + ", may not hold: " + fmt(unj + unr) + " of " + fmt(at) + (judged ? " not assessed" : " provisional"));
    pill.title = fmt(unj + unr) + " of " + fmt(at) + " entities at " + floor + (judged ? " are not assessed by an agent yet" : " rest on checks not run in this build") + ", so this level may not hold";
    return pill;
  }
  function floorOf(D, name) {
    var sc = (D.scorecards || {})[name];
    if (!sc) return null;
    var order = ["none"].concat(sc.levels || []);
    var best = null;
    Object.keys((D.scores || {})[name] || {}).forEach(function(id) {
      if (id.indexOf("/demo/") >= 0) return;
      var row = D.scores[name][id];
      if (!row || row.level === "skipped") return;
      var rules = row.rules || {};
      var keys = Object.keys(rules);
      if (keys.length && keys.every(function(k) {
        return rules[k] === "unknown";
      }))
        return;
      var r = order.indexOf(row.level);
      if (best == null || r < best.rank) best = { level: row.level, rank: r };
    });
    return best ? best.level : null;
  }

  // src/lib/tracks.js
  function trackTitle(D, name) {
    var t = (D.tracks || {})[name];
    return t && t.title || name;
  }
  function trackStats(D, S, name) {
    var rows = S.rows.filter(function(r) {
      return (r.sc.track || null) === (name || null);
    });
    var compare = rows.length > 0 && rows.every(function(r) {
      return isComparative(D, r.name);
    });
    var none = 0;
    var shares = [];
    var scoredRows = 0;
    var capped = [];
    var short = [];
    var cards = rows.map(function(r) {
      if (!r.compare) {
        none += r.none;
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
    cards.sort(function(a, b) {
      return rankLevel(a.floor) - rankLevel(b.floor) || (a.row.topShare || 0) - (b.row.topShare || 0) || b.row.none - a.row.none || title(D, a.row.name).localeCompare(title(D, b.row.name));
    });
    var verdicts = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
    var behind = [];
    rows.forEach(function(r) {
      if (!r.compare) return;
      var c = D.comparisons[r.name];
      var v = (c.verdict || {})[(c.peers || [])[0]] || "unknown";
      verdicts[v] = (verdicts[v] || 0) + 1;
      if (v === "behind") behind.push(title(D, r.name));
      var rules = verdicts.rules || (verdicts.rules = { ahead: 0, parity: 0, behind: 0, total: 0 });
      var k = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
      Object.keys(c.rules || {}).forEach(function(rule) {
        var x = (c.rules[rule] || {})[(c.peers || [])[0]];
        if (x in rules) rules[x]++;
        rules.total++;
        k[x in k ? x : "unknown"]++;
      });
      var other = verdicts.other || (verdicts.other = { ahead: 0, parity: 0, behind: 0, unknown: 0 });
      other[c.verdictRule === "majority" ? k.behind ? "behind" : k.unknown ? "unknown" : k.ahead ? "ahead" : "parity" : k.unknown ? "unknown" : k.ahead > k.behind ? "ahead" : k.behind > k.ahead ? "behind" : "parity"]++;
    });
    var pct = compare ? Math.round((verdicts.ahead + verdicts.parity) / rows.length * 100) : shares.length ? Math.round(
      shares.reduce(function(a, b) {
        return a + b;
      }, 0) / shares.length
    ) : null;
    var campaigns = Object.keys(D.campaigns || {}).filter(function(n) {
      return D.campaigns[n].track === name;
    });
    var majority = compare && byMajority(D, rows.map(function(r) {
      return r.name;
    }));
    if (compare) verdicts.majority = majority;
    return { name, def: (D.tracks || {})[name] || {}, rows, cards, compare, majority, none, pct, averaged: shares.length, scoredRows, capped, short, verdicts, behind, campaigns };
  }
  function tracksSorted(D, S) {
    return Object.keys(D.tracks || {}).map(function(n) {
      return trackStats(D, S, n);
    }).filter(function(t) {
      return t.rows.length || t.campaigns.length;
    }).sort(function(a, b) {
      return (a.compare ? 1 : 0) - (b.compare ? 1 : 0) || (a.pct == null ? 101 : a.pct) - (b.pct == null ? 101 : b.pct) || b.none - a.none || a.name.localeCompare(b.name);
    });
  }
  function unassessedNote(D, card, scored) {
    var ids = (D.unassessed || {})[card] || [];
    var inScope = ids.filter(function(id) {
      return ((D.scores[card] || {})[id] || {}).level !== "skipped";
    }).length;
    if (!inScope) return null;
    var rules = ((D.scorecards || {})[card] || {}).rules || {};
    var passing = [];
    var failing = [];
    Object.keys(rules).filter(function(r) {
      return basisOf(rules[r]) === "judged";
    }).forEach(function(r) {
      var fails = ids.some(function(id) {
        var o = (((D.scores[card] || {})[id] || {}).rules || {})[r];
        return o && o !== "pass";
      });
      (fails ? failing : passing).push(r);
    });
    var names = function(rs) {
      return rs.map(function(r) {
        return ruleLink(card, r);
      });
    };
    var list = function(rs) {
      var out = [];
      names(rs).forEach(function(a, i) {
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
      el("span", {}, el("b", { textContent: (inScope === scored ? "No entity here has an" : fmt(inScope) + " of " + fmt(scored) + " entities have no") + " agent's assessment yet. " }), "On those, of the ", basisBadge("judged", true), " rules, ", ...why)
    );
  }
  function unassessedIn(D, card, id) {
    return ((D.unassessed || {})[card] || []).indexOf(id) >= 0;
  }
  function trackKey(tracks) {
    var sw = function(c, t) {
      return el("span", {}, el("span", { className: "dot", style: "background:" + c }), t);
    };
    var compare = tracks.some(function(t) {
      return t.compare;
    });
    var quoted = null;
    tracks.forEach(function(t) {
      (t.cards || []).forEach(function(c) {
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
          ...[80, 30, 55].map(function(p) {
            return el("span", { className: "slot", style: "--h:" + p + "%;--c:" + RAMP });
          }),
          el("span", { className: "avg" })
        ),
        "one per scorecard, its share at the top level (point at, tap or arrow through a bar to name it under the strip); dashed: their average, the ring"
      ),
      compare ? el("b", { textContent: "Segmented ring, one per factor:" }) : null,
      compare ? sw(standingColor("ahead"), "ahead") : null,
      compare ? sw(standingColor("parity"), "at parity") : null,
      compare ? sw(standingColor("behind"), "behind") : null,
      el("span", { className: "medal-key" }, el("b", { textContent: "Medal:" }), medal("bronze", 16), "its weakest entity's level ·", medal("none", 16), "none"),
      quoted ? el("span", {}, "A quoted level such as " + levelName(quoted) + " is its scorecard's own name for a level.") : null
    );
    return el("details", { className: "keyd" }, el("summary", { textContent: "How to read: one bar per scorecard, and the ring is their average" }), key);
  }
  function furthestTrack(D, S) {
    var t = tracksSorted(D, S).find(function(t2) {
      return !t2.compare && t2.pct != null && t2.pct < 100;
    });
    return t ? t.name : null;
  }
  var RAMP = "var(--pass)";
  function trackColor(t) {
    if (t.compare) return "var(--accent)";
    if (t.pct == null) return "var(--faint)";
    return RAMP;
  }
  var STANDING_MARK = { ahead: "▲", parity: "=", behind: "▼", unknown: "?" };
  function standingChip(s, text) {
    var v = s || "unknown";
    return el("span", { className: "standing " + v, textContent: STANDING_MARK[v] + " " + (text || (v === "unknown" ? "not measured" : v)) });
  }
  function byMajority(D, names) {
    return names.length > 0 && names.every(function(n) {
      return ((D.comparisons || {})[n] || {}).verdictRule === "majority";
    });
  }
  function factorRule(majority) {
    return majority ? "a factor is ahead when more of its rules are ahead than behind, behind when more are behind, at parity when they are even" : "a factor is behind if any one of its rules is behind";
  }
  function ruleTag(majority) {
    return el("span", { className: "rule-tag", title: "By this comparison's rule, " + factorRule(majority) + "." }, majority ? "counted by the majority of rules" : "behind if any rule is behind");
  }
  function factorChip(v, counts, total) {
    if (counts.ahead && counts.behind) return v === "parity" ? "even split" : v + ", mixed";
    return v === "ahead" && counts.ahead * 2 < total ? "narrowly ahead" : null;
  }
  function standingPhrase(a, s, b) {
    if (s === "ahead") return a + " ahead of " + b;
    if (s === "behind") return a + " behind " + b;
    if (s === "parity") return a + " at parity with " + b;
    return a + " vs " + b + ": not measured";
  }
  function standingColor(s) {
    return s === "ahead" ? "var(--pass)" : s === "behind" ? "var(--fail)" : s === "parity" ? "var(--muted)" : "var(--unknown)";
  }

  // src/lib/status.js
  function daysLeft(end) {
    if (!end) return null;
    var ms = Date.parse(end + "T23:59:59Z") - Date.now();
    return isNaN(ms) ? null : Math.ceil(ms / 864e5);
  }
  function isPassing(o) {
    return o === "pass" || o === "waived";
  }
  function isFailing(o) {
    return o != null && !isPassing(o) && o !== "skipped" && o !== "unknown";
  }
  function blockers(sc, score, level) {
    var order = sc.levels || [];
    var cap = order.indexOf(level);
    return Object.keys(score.rules || {}).filter(function(n) {
      var rl = ((sc.rules || {})[n] || {}).level;
      return isFailing(score.rules[n]) && order.indexOf(rl) <= cap;
    }).sort();
  }
  function ringWords(t) {
    return "average share at the top level, across its " + fmt(t.averaged) + " scorecards";
  }
  function shortWhy(D, t) {
    if (!(t.short || []).length) return "";
    return andList(
      t.short.map(function(n) {
        var lv = ((D.scorecards || {})[n] || {}).levels || [];
        return title(D, n) + " (top level " + lv[lv.length - 1] + ")";
      })
    ) + (t.short.length === 1 ? " stops below gold, so the ring counts its share at that top level" : " stop below gold, so the ring counts each one's share at its own top level");
  }
  function ringWhy(D, t) {
    if (!t.capped.length) return "";
    var lv = levelName(floorOf(D, t.capped[0]));
    return andList(
      t.capped.map(function(n) {
        return title(D, n);
      })
    ) + (t.capped.length === 1 ? " holds the repository level at " + lv + " until it gains a higher level" : " hold the repository level until they gain a higher level");
  }
  function trackSlots(D, t) {
    var cards = t.cards.filter(function(c) {
      return !c.row.compare && c.row.scored;
    });
    if (!cards.length || t.pct == null) return null;
    var said = cards.map(function(c) {
      return title(D, c.row.name) + ": " + c.row.topShare + "% at the top level (" + levelName(c.row.top || "top") + ")";
    });
    var named = function(c) {
      var top = c.row.top || "top";
      return title(D, c.row.name) + ": " + c.row.topShare + "%" + (top === "gold" ? "" : " at " + levelName(top));
    };
    var low = cards.reduce(function(a, c, i) {
      return a >= 0 && cards[a].row.topShare <= c.row.topShare ? a : i;
    }, -1);
    var high = cards.reduce(function(a, c, i) {
      return a >= 0 && cards[a].row.topShare >= c.row.topShare ? a : i;
    }, -1);
    var rest = (low >= 0 ? "Lowest bar, " + named(cards[low]) : "Each bar is one scorecard") + (high >= 0 && high !== low && cards[high].row.topShare > cards[low].row.topShare ? " · highest, " + named(cards[high]) : "") + " · dashed line: their average, " + t.pct + "%";
    var cap = el("div", { className: "slotcap", ariaHidden: "true", textContent: rest });
    var bars = cards.map(function(c, i) {
      return el("span", {
        className: "slot",
        style: "--h:" + c.row.topShare + "%;--c:" + RAMP,
        onpointerenter: function() {
          show(i);
        },
        onclick: function() {
          if (on === i) location.hash = "scorecard/" + enc(c.row.name);
          else show(i);
        }
      });
    });
    var on = -1;
    var show = function(i) {
      if (on >= 0) bars[on].classList.remove("on");
      on = i;
      if (i < 0) {
        cap.textContent = rest;
        return;
      }
      bars[i].classList.add("on");
      cap.textContent = named(cards[i]) + " · bar " + (i + 1) + " of " + cards.length + " · click or Enter to open it";
    };
    var wrap = el(
      "div",
      { className: "mix" },
      el(
        "div",
        {
          className: "slots",
          role: "img",
          tabIndex: 0,
          ariaLabel: "Share at the top level, per scorecard: " + said.join("; ") + ". Average " + t.pct + "%.",
          style: "--avg:" + t.pct + "%",
          onpointerleave: function() {
            show(-1);
          },
          onfocus: function() {
            if (on < 0) show(low >= 0 ? low : 0);
          },
          onblur: function() {
            show(-1);
          },
          onkeydown: function(e) {
            if (e.key === "Enter" && on >= 0) {
              location.hash = "scorecard/" + enc(cards[on].row.name);
              return;
            }
            if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
            e.preventDefault();
            show(on < 0 ? e.key === "ArrowRight" ? 0 : bars.length - 1 : (on + (e.key === "ArrowRight" ? 1 : bars.length - 1)) % bars.length);
          }
        },
        ...bars,
        el("span", { className: "avg", ariaHidden: "true" })
      ),
      cap
    );
    return wrap;
  }

  // src/lib/summary.js
  function share(n, of) {
    if (!of) return 0;
    var p = Math.round(100 * n / of);
    return n === of ? 100 : n === 0 ? 0 : Math.min(99, Math.max(1, p));
  }
  function scorecardRows(D) {
    return Object.keys(D.scorecards || {}).map(function(name) {
      var sc = D.scorecards[name];
      var ents = Object.entries((D.scores || {})[name] || {});
      var parts = tallyLevels(ents, levelOrder(sc.levels));
      var count = function(l) {
        var p = parts.find(function(x) {
          return x.level === l;
        });
        return p ? p.n : 0;
      };
      var top = (sc.levels || [])[(sc.levels || []).length - 1];
      var scored = ents.filter(function(p) {
        return p[1].level !== "skipped";
      }).length;
      return {
        name,
        sc,
        compare: isComparative(D, name),
        parts,
        none: count("none"),
        bronze: top === "bronze" ? 0 : count("bronze"),
        top,
        topShare: share(count(top), scored),
        scored,
        total: ents.length,
        weight: isComparative(D, name) ? -1 : count("none") * 1e3 + (top === "bronze" ? 0 : count("bronze"))
      };
    }).sort(function(a, b) {
      return b.weight - a.weight || a.name.localeCompare(b.name);
    });
  }
  function campaignNamesSorted(D) {
    var campaigns = D.campaigns || {};
    var rank = { off: 0, empty: 1, on: 2 };
    return Object.keys(campaigns).filter(function(n) {
      var st = campaigns[n].status;
      return st === "active" || st === "scheduled" || st === "ended";
    }).sort(function(a, b) {
      var ca = campaigns[a];
      var cb = campaigns[b];
      return rank[campaignStanding(D, ca)] - rank[campaignStanding(D, cb)] || (ca.percent || 0) - (cb.percent || 0) || a.localeCompare(b);
    });
  }
  function scoredKinds(D) {
    var set = {};
    Object.keys(D.scorecards || {}).forEach(function(name) {
      var k = D.scorecards[name].kind;
      if (k && !isComparative(D, name)) set[k] = true;
    });
    return set;
  }
  function catalogKindsSorted(D) {
    var catalog = D.catalog || {};
    var scored = scoredKinds(D);
    return Object.keys(catalog).sort(function(a, b) {
      var as = scored[a] ? 0 : 1;
      var bs = scored[b] ? 0 : 1;
      if (as !== bs) return as - bs;
      return catalog[b].length - catalog[a].length || a.localeCompare(b);
    });
  }
  function summarize(D) {
    var rows = scorecardRows(D);
    var byEntity = {};
    var shortOfTop = /* @__PURE__ */ new Set();
    var marks = /* @__PURE__ */ new Map();
    var checks = { pass: 0, fail: 0, unknown: 0, vacuous: 0 };
    rows.forEach(function(r) {
      if (r.compare) return;
      var order = levelOrder(r.sc.levels);
      var unassessed = new Set((D.unassessed || {})[r.name] || []);
      Object.entries((D.scores || {})[r.name] || {}).forEach(function(pair) {
        var id = pair[0];
        var s = pair[1];
        if (s.level === "skipped") return;
        var cur = byEntity[id];
        var rank = order.indexOf(s.level);
        var norm = ["none", "bronze", "silver", "gold"].indexOf(s.level) >= 0 ? s.level : rank <= 0 ? "none" : "gold";
        if (!cur || ["none", "bronze", "silver", "gold"].indexOf(norm) < ["none", "bronze", "silver", "gold"].indexOf(cur)) byEntity[id] = norm;
        if (s.level !== (r.sc.levels || []).slice(-1)[0]) shortOfTop.add(id);
        var st = unassessed.has(id) ? "hollow" : unrunIn(r.sc, s).length ? "unrun" : "";
        if (st) (marks.get(id) || marks.set(id, []).get(id)).push([norm, st]);
        Object.keys(s.rules || {}).forEach(function(rule) {
          var o = s.rules[rule];
          if (isPassing(o) && unassessed.has(id) && basisOf((r.sc.rules || {})[rule]) === "judged") checks.vacuous++;
          else if (isPassing(o)) checks.pass++;
          else if (o === "fail") checks.fail++;
          else if (o === "unknown") checks.unknown++;
        });
      });
    });
    var catalog = D.catalog || {};
    var entityCount = Object.keys(catalog).reduce(function(n, k) {
      return n + catalog[k].length;
    }, 0);
    var kindOf = {};
    Object.keys(catalog).forEach(function(k) {
      catalog[k].forEach(function(id) {
        kindOf[id] = k;
      });
    });
    var worstState = /* @__PURE__ */ new Map();
    marks.forEach(function(list, id) {
      var at = list.filter(function(m) {
        return m[0] === byEntity[id];
      });
      var st = at.some(function(m) {
        return m[1] === "hollow";
      }) ? "hollow" : at.length ? "unrun" : "";
      if (st) worstState.set(id, st);
    });
    var atTop = Object.keys(byEntity).filter(function(id) {
      return !shortOfTop.has(id);
    });
    return { rows, worst: byEntity, atTop, worstState, checks, entityCount, kindOf, repo: repoLevel(D) };
  }
  function repoLevel(D) {
    var rollup = Array.isArray(D.repoRollup) ? D.repoRollup : [];
    if (!rollup.length) return null;
    var ladder = ["none", "bronze", "silver", "gold"];
    var constituents = rollup.filter(function(name) {
      return (D.scorecards || {})[name];
    }).map(function(name) {
      return { name, level: floorOf(D, name) || "none" };
    });
    if (!constituents.length) return null;
    constituents.forEach(function(c) {
      var lv = (D.scorecards[c.name].levels || []).slice(-1)[0];
      c.top = known(lv) || lv;
    });
    var ceiling = constituents.reduce(function(m, c) {
      return ladder.indexOf(c.top) >= 0 && ladder.indexOf(c.top) < ladder.indexOf(m) ? c.top : m;
    }, "gold");
    var floor = constituents.reduce(function(m, c) {
      return ladder.indexOf(c.level) < ladder.indexOf(m) ? c.level : m;
    }, "gold");
    var next = ladder[ladder.indexOf(floor) + 1] || null;
    return {
      ladder,
      floor,
      ceiling,
      next,
      constituents,
      holding: constituents.filter(function(c) {
        return c.level === floor;
      })
    };
  }
  var NO_FACTS = "Not scored yet: no connector has delivered facts, so no entity has a level.";
  function noFacts(S) {
    return S.rows.length > 0 && !Object.keys(S.worst).length;
  }
  function healthVerdict(D, S, campaignNames) {
    var campaigns = D.campaigns || {};
    var off = campaignNames.filter(function(n) {
      return campaignStanding(D, campaigns[n]) === "off";
    });
    var debt = S.rows.filter(function(r) {
      return r.none > 0 && !r.compare;
    });
    var blind = campaignNames.filter(function(n) {
      return campaignStanding(D, campaigns[n]) === "empty";
    });
    if (!campaignNames.length && !S.rows.length) return "No campaigns or scorecards in this snapshot.";
    if (noFacts(S)) return NO_FACTS;
    if (!off.length && !debt.length && !blind.length) return "On track: every active campaign meets its target, and no scorecard has entities at none.";
    var bits = [];
    var far = furthestTrack(D, S);
    if (far) bits.push("the " + trackTitle(D, far) + " track is furthest from its goal");
    var scored = S.rows.filter(function(r) {
      return !r.compare;
    }).length;
    if (debt.length) bits.push(debt.length + " of " + plural(scored, "scorecard") + " " + (debt.length === 1 ? "has" : "have") + " entities at none");
    if (off.length) bits.push(plural(off.length, "campaign") + " " + (off.length === 1 ? "is" : "are") + " behind target");
    if (blind.length) bits.push(plural(blind.length, "campaign") + " " + (blind.length === 1 ? "has" : "have") + " no single score");
    var atNone2 = debt.reduce(function(n, r) {
      return n + r.none;
    }, 0);
    if (!off.length && !blind.length && atNone2 <= 2)
      return "Almost there: " + plural(atNone2, "entity is", "entities are") + " at none, in " + andList(debt.map(function(r) {
        return title(D, r.name);
      })) + "; every other scored entity holds a medal.";
    var lead = "Needs attention: ";
    return lead + (bits.length > 1 ? bits.slice(0, -1).join(", ") + " and " + bits[bits.length - 1] : bits[0]) + ".";
  }

  // src/lib/rings.js
  function ring(pct, opts) {
    var o = opts || {};
    var s = o.size || 52;
    var r = s / 2 - 5;
    var circ = 2 * Math.PI * r;
    var color = o.color || "var(--accent)";
    var arc = svg("circle", {
      cx: s / 2,
      cy: s / 2,
      r,
      fill: "none",
      stroke: color,
      "stroke-width": 6,
      "stroke-linecap": "round",
      "stroke-dasharray": circ,
      "stroke-dashoffset": circ,
      transform: "rotate(-90 " + s / 2 + " " + s / 2 + ")"
    });
    var label = String(o.text != null ? o.text : pct == null ? "—" : pct + "%");
    var target = null;
    if (o.target != null && o.target < 100) {
      var a = o.target / 100 * 2 * Math.PI - Math.PI / 2;
      target = svg("line", {
        x1: s / 2 + (r - 5) * Math.cos(a),
        y1: s / 2 + (r - 5) * Math.sin(a),
        x2: s / 2 + (r + 5) * Math.cos(a),
        y2: s / 2 + (r + 5) * Math.sin(a),
        stroke: "var(--fg)",
        "stroke-width": 2
      });
    }
    var g = svg(
      "svg",
      { class: "ring", width: s, height: s, viewBox: "0 0 " + s + " " + s, role: "img", "aria-label": o.label || pct + "%" },
      svg("circle", { cx: s / 2, cy: s / 2, r, fill: "none", stroke: "var(--line)", "stroke-width": 6 }),
      pct == null ? null : arc,
      target,
      o.bare ? null : svg("text", { x: "50%", y: "50%", "text-anchor": "middle", "dominant-baseline": "central", style: s > 70 ? "font-size:18px" : label.length > 3 ? "font-size:12px" : "" }, label)
    );
    if (pct != null) {
      requestAnimationFrame(function() {
        requestAnimationFrame(function() {
          arc.setAttribute("stroke-dashoffset", circ * (1 - Math.max(0, Math.min(100, pct)) / 100));
        });
      });
    }
    return g;
  }
  function tallyRing(v, n, size, bare) {
    var s = size || 52;
    var c = s / 2;
    var r = c - 5;
    var circ = 2 * Math.PI * r;
    var seg = circ / Math.max(n, 1);
    var gap = n > 1 ? Math.min(4, seg / 3) : 0;
    var segs = [];
    ["ahead", "parity", "behind", "unknown"].forEach(function(k) {
      for (var j = 0; j < (v[k] || 0); j++) {
        segs.push(
          svg("circle", {
            cx: c,
            cy: c,
            r,
            fill: "none",
            stroke: standingColor(k),
            "stroke-width": 6,
            "stroke-dasharray": seg - gap + " " + (circ - seg + gap),
            "stroke-dashoffset": -segs.length * seg,
            transform: "rotate(-90 " + c + " " + c + ")"
          })
        );
      }
    });
    var text = n && v.unknown >= n ? "—" : v.ahead + "/" + n;
    return svg(
      "svg",
      { class: "ring tally", width: s, height: s, viewBox: "0 0 " + s + " " + s, role: "img", "aria-label": tallyWords(v, n) },
      ...segs,
      bare ? null : svg("text", { x: "50%", y: "50%", "text-anchor": "middle", "dominant-baseline": "central", style: s > 70 ? "font-size:18px" : text.length > 3 ? "font-size:12px" : "" }, text)
    );
  }
  function otherWay(v) {
    var o = v.other;
    if (v.majority == null || !o || o.ahead === v.ahead && o.parity === v.parity && o.behind === v.behind) return "";
    return " (" + (v.majority ? "behind if any rule is behind" : "by the majority of rules") + ": ahead on " + o.ahead + (o.parity ? ", at parity on " + o.parity : "") + ", behind on " + o.behind + ")";
  }
  function tallyWords(v, n) {
    if (n && v.unknown >= n) return "not measured yet: no factor has values for both products";
    var r = v.rules;
    var byRule = r && r.total ? "; by rule, " + r.ahead + " ahead and " + r.behind + " behind of " + r.total : "";
    return ["ahead on " + v.ahead, v.parity ? "at parity on " + v.parity : null, "behind on " + v.behind, v.unknown ? "not measured on " + v.unknown : null].filter(Boolean).join(", ") + " of " + n + " factors" + (v.majority == null ? "" : v.majority ? ", counted by the majority of rules" : ", behind if any rule is behind") + otherWay(v) + byRule;
  }
  function shortId(id) {
    var s = String(id || "");
    var slash = s.lastIndexOf("/");
    var colon = s.lastIndexOf(":");
    var i = Math.max(slash, colon);
    return i >= 0 && i < s.length - 1 ? s.slice(i + 1) : s;
  }
  function isTop(D, name, level) {
    var levels = ((D.scorecards || {})[name] || {}).levels || [];
    return !!level && level === levels[levels.length - 1];
  }
  function capsRepo(D, name, level) {
    var R = repoLevel(D);
    return !!(R && R.next && isTop(D, name, level) && level === R.floor && (D.repoRollup || []).indexOf(name) >= 0);
  }
  function topNote(D, name, level) {
    if (capsRepo(D, name, level)) return el("span", { className: "capnote", title: "Its top level is " + level + ", so the repository cannot rise above " + level + " until this scorecard gains a higher level" }, " ", lockIcon(), " at its top level, " + level + ", so it caps the repository");
    var top = isTop(D, name, level) && !ONLY_LEVEL.has(level) ? el("span", { className: "topmark", textContent: " ✓ top level" }) : "";
    return LEVEL_DEF[level] ? el("span", {}, top, el("span", { className: "sub leveldef", textContent: " · " + LEVEL_DEF[level] })) : top;
  }
  function title(D, name) {
    var sc = (D.scorecards || {})[name] || {};
    return sc.title || name;
  }

  // src/lib/basis.js
  var BASIS = {
    measured: {
      label: "Tool-checked",
      desc: "A tool, connector or script decides it from the facts it reads. The same input gives the same result."
    },
    judged: {
      label: "Agent-judged",
      desc: "Passes on an agent's written assessment against a rubric, reviewed by a second agent; whether that assessment exists and holds counts too. It changes when a verdict does."
    }
  };
  function basisIcon(b) {
    if (b === "judged")
      return svg("svg", { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": "true" }, svg("path", { d: "M6 .6l1.5 3.8 3.9 1.6-3.9 1.6L6 11.4 4.5 7.6.6 6l3.9-1.6z", fill: "currentColor" }));
    return svg(
      "svg",
      { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": "true" },
      svg("path", { d: "M1.4 9.2a4.6 4.6 0 0 1 9.2 0", fill: "none", stroke: "currentColor", "stroke-width": 1.6, "stroke-linecap": "round" }),
      svg("path", { d: "M6 9.2l2.3-3.3", stroke: "currentColor", "stroke-width": 1.6, "stroke-linecap": "round" }),
      svg("circle", { cx: 6, cy: 9.2, r: 1.2, fill: "currentColor" })
    );
  }
  function lockIcon() {
    return svg(
      "svg",
      { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": "true" },
      svg("rect", { x: 2, y: 5.2, width: 8, height: 6, rx: 1.2, fill: "currentColor" }),
      svg("path", { d: "M3.8 5.2V3.8a2.2 2.2 0 0 1 4.4 0v1.4", fill: "none", stroke: "currentColor", "stroke-width": 1.4 })
    );
  }
  function basisOf(rule) {
    return (rule || {}).basis === "judged" ? "judged" : "measured";
  }
  function basisBadge(b, mini, inControl) {
    if (mini === "tip") {
      var tip = el("span", { className: "basis tip " + b + " mini", tabIndex: 0 }, basisIcon(b), el("span", { className: "bl", textContent: BASIS[b].label }), el("span", { className: "sr", textContent: ": " + BASIS[b].desc }));
      tip.dataset.tip = BASIS[b].label + ": " + BASIS[b].desc;
      return tip;
    }
    var quiet = mini ? el("span", { className: "basis " + b + " mini" }, basisIcon(b), el("span", {}, el("span", { className: "sr", textContent: " (" }), el("span", { className: "bl", textContent: BASIS[b].label }), el("span", { className: "sr", textContent: ")" }))) : inControl ? el("span", { className: "basis ctl " + b }, basisIcon(b), el("span", { className: "sr", textContent: " (" }), el("span", { className: "bl", textContent: BASIS[b].label }), el("span", { className: "sr", textContent: ")" })) : null;
    if (quiet) {
      quiet.dataset.tip = BASIS[b].label + ": " + BASIS[b].desc;
      return quiet;
    }
    var full = el("span", { className: "basis tip " + b, tabIndex: 0 }, basisIcon(b), BASIS[b].label, el("span", { className: "sr", textContent: ": " + BASIS[b].desc }));
    full.dataset.tip = BASIS[b].desc;
    return full;
  }
  function basisInline(counts, noun) {
    var only = counts && (counts.measured || 0) + (counts.judged || 0) > 0 && !(counts.measured && counts.judged) ? counts.measured ? "measured" : "judged" : null;
    if (!only) return null;
    return el(
      "span",
      { className: "basis-inline" },
      " · ",
      basisBadge(only),
      " " + (counts[only] === 1 ? "its one " + (noun || "rule") : "all " + plural(counts[only], noun || "rule")),
      only === "judged" ? ": an agent's assessment is the evidence, so whether one exists and still holds counts too" : null
    );
  }
  function basisKey(counts, noun) {
    var row = function(b) {
      var n = counts ? counts[b] || 0 : null;
      return el("div", {}, basisBadge(b), el("span", {}, n != null ? el("b", { textContent: plural(n, noun || "rule") + ". " }) : null, BASIS[b].desc));
    };
    var only = counts && (counts.measured || 0) + (counts.judged || 0) > 0 && !(counts.measured && counts.judged) ? counts.measured ? "measured" : "judged" : null;
    var k = only ? el("div", { className: "basis-key one" }, el("div", {}, basisBadge(only), el("span", {}, el("b", { textContent: (counts[only] === 1 ? "" : "All ") + plural(counts[only], noun || "rule") + ". " }), BASIS[only].desc + " None here is " + BASIS[only === "measured" ? "judged" : "measured"].label.toLowerCase() + "."))) : el("div", { className: "basis-key" }, row("measured"), row("judged"));
    k.setAttribute("role", "note");
    k.setAttribute("aria-label", "How a rule is decided");
    return k;
  }
  function basisCounts(rules) {
    var n = { measured: 0, judged: 0 };
    Object.keys(rules || {}).forEach(function(r) {
      n[basisOf(rules[r])]++;
    });
    return n;
  }
  function refs(links) {
    var safe = (links || []).filter(function(l) {
      return l && /^https?:\/\//.test(l.url || "");
    });
    if (!safe.length) return null;
    return el(
      "span",
      { className: "refs" },
      ...safe.map(function(l) {
        var m = /^(D\d+|D0[0-9A-Z]{20,}) (.+)$/.exec(l.title || "");
        var a = el("a", { href: l.url, target: "_blank", rel: "noopener" }, el("span", {}, m ? "Decision: " + m[2] : l.title || l.url, el("span", { className: "ext", ariaHidden: "true", textContent: " ↗" })));
        if (m) a.title = m[1] + ": " + m[2];
        return a;
      })
    );
  }
  function evidenceEl(items, seen, rule) {
    if (!items.length) return "";
    var text = function(i) {
      return typeof i === "string" ? i : JSON.stringify(i);
    };
    var key = items.map(text).join("\n");
    if (seen && seen[key]) return el("p", { className: "sub", textContent: "Evidence: the same as for " + seen[key] + ", above." });
    if (seen) seen[key] = rule;
    var li = function(i) {
      var off = typeof i === "string" && /^no data from connector (\S+)$/.exec(i);
      if (off) return el("li", { className: "sub" }, "Not measured: the ", el("code", { textContent: off[1] }), " connector did not run in this build, so the facts this check reads are missing.");
      return el("li", {}, el("code", { textContent: text(i) }));
    };
    var SHOW = 3;
    if (items.length <= SHOW + 1) return el("ul", { className: "evidence" }, ...items.map(li));
    return el(
      "div",
      { className: "evidence" },
      el("ul", {}, ...items.slice(0, SHOW).map(li)),
      el("details", {}, el("summary", { textContent: "Show " + (items.length - SHOW) + " more" }), el("ul", {}, ...items.slice(SHOW, 50).map(li), items.length > 50 ? el("li", { textContent: "… and " + (items.length - 50) + " more" }) : null))
    );
  }
  function whyEl(def, label) {
    if (!def || !def.rationale && !(def.links || []).length) return null;
    return el("div", { className: "why" }, def.rationale ? el("b", { textContent: label || "Why: " }) : null, ...def.rationale ? rich(def.rationale) : [], refs(def.links));
  }

  // src/lib/kinds.js
  function kindGroupsOf(D) {
    var catalog = D.catalog || {};
    var seen = {};
    var groups = (D.kindGroups || []).map(function(g) {
      var kinds = (g.kinds || []).filter(function(k) {
        seen[k] = true;
        return catalog[k];
      });
      return { name: g.name, title: g.title || g.name, description: g.description, kinds };
    }).filter(function(g) {
      return g.kinds.length;
    });
    var rest = catalogKindsSorted(D).filter(function(k) {
      return !seen[k];
    });
    if (rest.length) groups.push({ name: "", title: groups.length ? "Other kinds" : "Kinds", kinds: rest });
    groups.forEach(function(g) {
      g.count = g.kinds.reduce(function(n, k) {
        return n + catalog[k].length;
      }, 0);
    });
    return groups;
  }
  function kindDescription(D, k) {
    return (((D.kinds || {})[k] || {}).description || "").trim();
  }
  var ACRONYMS = { Api: "API", Cli: "CLI", Json: "JSON", Mcp: "MCP", Sdlc: "SDLC", Ui: "UI", Url: "URL" };
  function kindTitle(D, k) {
    var t = ((D.kinds || {})[k] || {}).title;
    if (t) return t;
    var words = String(k || "").match(/OpenSSF|[A-Z]{2,}(?![a-z])|[A-Z][a-z0-9]*|[a-z0-9]+/g) || [k];
    return words.map(function(w, i) {
      if (ACRONYMS[w]) return ACRONYMS[w];
      if (w === "OpenSSF" || /^[A-Z]{2,}$/.test(w)) return w;
      return i ? w.toLowerCase() : w;
    }).join(" ");
  }
  function kindCount(D, k, n) {
    var t = kindTitle(D, k);
    if (!/^[A-Z]{2,}|^OpenSSF/.test(t)) t = t.charAt(0).toLowerCase() + t.slice(1);
    var tail = (t.match(/ \([^)]*\)$/) || [""])[0];
    t = t.slice(0, t.length - tail.length);
    if (n !== 1) t = /[^aeiou]y$/.test(t) ? t.slice(0, -1) + "ies" : /(s|x|ch|sh)$/.test(t) ? t + "es" : t + "s";
    return fmt(n) + " " + t + tail;
  }
  function scorecardSplit(D, k) {
    if (k !== "Scorecard") return "";
    var ids = (D.catalog || {})[k] || [];
    var hits = {};
    var split = ids.map(function(id) {
      var m = /^[^:]+:([^/]+)\/(.+)$/.exec(id);
      if (m && (D.scorecards || {})[m[2]]) hits[m[1]] = (hits[m[1]] || 0) + 1;
      return m;
    });
    var mine = Object.keys(hits).sort(function(a, b) {
      return hits[b] - hits[a];
    })[0];
    var own = 0;
    var cmp = 0;
    split.forEach(function(m) {
      if (!m || m[1] !== mine || !(D.scorecards || {})[m[2]]) return;
      if ((D.comparisons || {})[m[2]]) cmp++;
      else own++;
    });
    var parts = [fmt(own) + " of this portal's own", cmp ? plural(cmp, "comparison factor") : null, ids.length - own - cmp ? fmt(ids.length - own - cmp) + " in example portals" : null].filter(Boolean);
    return parts.length > 1 ? " (" + parts.join(", ") + ")" : "";
  }

  // src/lib/campaigns.js
  function levelName(l) {
    return l === "none" || l === "not scored" || LADDER.indexOf(l) >= 0 ? l : "“" + l + "”";
  }
  function scCaption(D, r, split) {
    var head = (r.sc.kind ? kindCount(D, r.sc.kind, r.total) + (split ? scorecardSplit(D, r.sc.kind) : "") : plural(r.total, "entity", "entities")) + (r.total > r.scored ? ", " + fmt(r.total - r.scored) + " not in scope" : "");
    var prov = provisionalOf(D, r.name);
    var unj = unassessedOf(D, r.name);
    return [
      head + (r.scored ? " · " + r.topShare + "% at the top level (" + levelName(r.top || "top") + ")" : " · not scored yet"),
      prov ? el("span", { className: "provisional", title: "Their levels hold only if the checks not run in this build pass; drawn dotted in the bar", textContent: " · " + fmt(prov) + " provisional (checks not run)" }) : null,
      unj ? el("span", { className: "unjudged-n", title: "No agent has assessed them yet, so their agent-judged rules pass or fail for want of anything to read; drawn hatched in the bar", textContent: " · " + fmt(unj) + " not assessed by an agent (hatched)" }) : null
    ];
  }
  function nextMedal(D, cards) {
    for (var i = 0; i < cards.length; i++) {
      var c = cards[i];
      var sc = (D.scorecards || {})[c.row.name] || {};
      var up = c.floor && nextLevel(sc, c.floor);
      if (!up) continue;
      var held = {};
      var low = 0;
      Object.values((D.scores || {})[c.row.name] || {}).forEach(function(s) {
        if (s.level !== c.floor) return;
        low++;
        blockers(sc, s, up).forEach(function(n) {
          held[n] = (held[n] || 0) + 1;
        });
      });
      var best = Object.entries(held).sort(function(a, b) {
        return b[1] - a[1] || a[0].localeCompare(b[0]);
      })[0];
      return { name: c.row.name, floor: c.floor, up, low, rule: best && best[0], n: best ? best[1] : 0 };
    }
    return null;
  }
  function nextMedalEl(D, cards, label, why) {
    var nm = nextMedal(D, cards);
    if (!nm) return null;
    var top = (((D.scorecards || {})[nm.name] || {}).levels || []).slice(-1)[0];
    var card = link("scorecard/" + enc(nm.name), "", title(D, nm.name));
    var rule = nm.rule ? [link("rule/" + enc(nm.name) + "/" + enc(nm.rule), "", ruleTitle(nm.name, nm.rule)), " ", basisBadge(basisOf((((D.scorecards || {})[nm.name] || {}).rules || {})[nm.rule]), "tip")] : [];
    var entities = plural(nm.low, "entity", "entities") + " at " + nm.floor;
    var body = nm.rule && nm.n === nm.low ? [entities + (nm.low === 1 ? " needs " : " need "), ...rule, " to reach " + nm.up + " on ", card, "."] : ["lift " + entities + " to " + nm.up + " on ", card, ...nm.rule ? ["; " + nm.n + (nm.n === 1 ? " needs " : " need "), ...rule, " first."] : ["."]];
    return el(
      "div",
      { className: "nextmedal" },
      el("b", { textContent: label || "Next medal: " }),
      ...body,
      el("span", { className: "sub" }, isTop(D, nm.name, nm.up) ? " That is its top level." : " Its top level is " + top + (Object.keys(D.tracks || {}).length ? ": the track's percentage moves when they reach it." : ".") + (why || "")),
      isTop(D, nm.name, nm.up) || !Object.keys(D.tracks || {}).length ? null : ringMoverEl(D, cards)
    );
  }
  function ringMoverEl(D, cards) {
    var rows = cards.map(function(c) {
      return c.row;
    }).filter(function(r) {
      return !r.compare && r.scored;
    });
    var best = null;
    rows.forEach(function(r) {
      var lv = (r.sc.levels || []).filter(function(l) {
        return l !== "none";
      });
      if (lv.length < 1 || r.topShare >= 100) return;
      var below = lv.length > 1 ? lv[lv.length - 2] : "none";
      var p = (r.parts || []).find(function(x) {
        return x.level === below;
      });
      var n = p ? p.n : 0;
      var gain = n ? n / r.scored * 100 / rows.length : 0;
      if (gain > (best ? best.gain : 0)) best = { r, n, below, top: lv[lv.length - 1], gain };
    });
    if (!best || best.gain < 1) return null;
    return el(
      "div",
      { className: "sub mover" },
      el("b", { textContent: "Moves the track's percentage most: " }),
      "lift " + plural(best.n, "entity", "entities") + " from " + levelName(best.below) + " to " + levelName(best.top) + " on ",
      link("scorecard/" + enc(best.r.name), "", title(D, best.r.name)),
      ", about " + Math.round(best.gain) + (Math.round(best.gain) === 1 ? " point." : " points.")
    );
  }
  function ownerName(o) {
    return String(o || "").replace(/^(group|user):/, "");
  }
  function stuckNote(D, name, next) {
    var owner = ownerName(((D.scorecards || {})[name] || {}).owner);
    return el("span", { className: "sub" }, " needs a " + next + " level defined" + (owner ? " by its owner, " + owner : "") + ": ", link("scorecard/" + enc(name), "", "see its levels →"));
  }
  function goalWords(each) {
    return each.filter(function(p) {
      return p.met;
    }).length + " of " + plural(each.length, "scorecard") + " at target";
  }
  function goalBar(state, pct, target, big, each) {
    var empty = state === "empty";
    var parts = (each || []).length;
    var met = (each || []).filter(function(p) {
      return p.met;
    }).length;
    var g = el(
      "div",
      { className: "cprog" + (big ? " big" : "") + (state === "on" || empty && parts && met === parts ? " on" : "") },
      el("b", { textContent: empty ? parts ? met + " of " + parts : "—" : pct + "%" }),
      empty ? null : el("div", { className: "gb" }, el("span", { style: "width:" + Math.max(0, Math.min(100, pct)) + "%" }), el("i", { style: "left:calc(" + target + "% - 1px)" })),
      el("small", { textContent: empty ? parts ? big ? "scorecards at target " + target + "%" : "at target" : "no single score" : "target " + target + "%" })
    );
    g.setAttribute("role", "img");
    g.setAttribute("aria-label", empty ? parts ? met + " of " + plural(parts, "scorecard") + " at the target of " + target + "%, each scored on its own" : "no single score" : pct + "% of entities meet the goal; target " + target + "%");
    return g;
  }
  function kpiBar(pct, label, short, hollowPct) {
    var clamp = function(v) {
      return Math.max(0, Math.min(100, v || 0));
    };
    var g = el(
      "div",
      { className: "kpi-bar" + (short ? " short" : "") },
      el("span", { style: "width:" + clamp(pct - (hollowPct || 0)) + "%" }),
      hollowPct ? el("span", { className: "hollow", style: "width:" + clamp(hollowPct) + "%" }) : null
    );
    g.setAttribute("role", "img");
    g.setAttribute("aria-label", label);
    return g;
  }
  function kpiPips(on, total, label) {
    var g = el(
      "div",
      { className: "kpi-pips" },
      ...Array.from({ length: Math.min(total, 12) }, function(_, i) {
        return el("span", { className: i < on ? "on" : "" });
      })
    );
    g.setAttribute("role", "img");
    g.setAttribute("aria-label", label);
    return g;
  }
  function nextLevel(sc, level) {
    var order = sc.levels || [];
    if (level === "none") return order[0] || null;
    var i = order.indexOf(level);
    return i >= 0 && i < order.length - 1 ? order[i + 1] : null;
  }
  function campaignState(c) {
    if (!c.inScope && c.measure == null) return "empty";
    return c.onTarget ? "on" : "off";
  }
  function includeProgress(D, c) {
    var target = c.targetPercent == null ? 100 : c.targetPercent;
    return (c.includes || []).map(function(i) {
      var sc = (D.scorecards || {})[i.scorecard];
      var order = levelOrder(sc && sc.levels);
      var goal = i.atLeast ? order.indexOf(i.atLeast) : order.length - 2;
      var ratedIds = Object.keys((D.scores || {})[i.scorecard] || {}).filter(function(id) {
        return D.scores[i.scorecard][id].level !== "skipped";
      });
      var rated = ratedIds.map(function(id) {
        return D.scores[i.scorecard][id];
      });
      var below = rated.filter(function(x) {
        return order.indexOf(x.level) < goal;
      }).length;
      var hollow = ratedIds.filter(function(id) {
        return order.indexOf(D.scores[i.scorecard][id].level) >= goal && unassessedIn(D, i.scorecard, id);
      }).length;
      var pct = rated.length ? Math.floor(100 * (rated.length - below) / rated.length) : 0;
      return { name: i.scorecard, title: title(D, i.scorecard), known: !!sc, goal: order[goal], rated: rated.length, below, hollow, pct, met: !!sc && pct >= target };
    });
  }
  function campaignStanding(D, c) {
    var state = campaignState(c);
    if (state !== "empty") return state;
    var each = includeProgress(D, c).filter(function(p) {
      return p.known;
    });
    if (!each.length) return "empty";
    return each.every(function(p) {
      return p.met;
    }) ? "on" : "off";
  }

  // src/lib/bars.js
  function levelOrder(levels) {
    return ["none"].concat(levels || [], ["skipped"]);
  }
  function tallyLevels(rows, order) {
    var tally = {};
    rows.forEach(function(pair) {
      tally[pair[1].level] = (tally[pair[1].level] || 0) + 1;
    });
    return order.filter(function(l) {
      return tally[l];
    }).map(function(l) {
      return { level: l, n: tally[l] };
    });
  }
  function grow(nodes, targets) {
    requestAnimationFrame(function() {
      requestAnimationFrame(function() {
        nodes.forEach(function(n, i) {
          n.style.flexGrow = targets[i];
        });
      });
    });
  }
  function barEl(all, cls, label) {
    var parts = all.filter(function(p) {
      return p.level !== "skipped";
    });
    var spans = parts.map(function(p) {
      return p.hollow ? el("span", { className: "hollow", style: "flex-grow:0;flex-basis:0;--seg:" + fill(p.level), title: (p.label || p.level) + ": " + p.n + ", not assessed by an agent yet" }) : p.unrun ? el("span", { className: "unrun", style: "flex-grow:0;flex-basis:0;--seg:" + fill(p.level), title: (p.label || p.level) + ": " + p.n + ", provisional: checks it needs were not run in this build" }) : p.level === "unscored" ? el("span", { className: "unscored", style: "flex-grow:0;flex-basis:0", title: "not scored: " + p.n }) : el("span", { style: "flex-grow:0;flex-basis:0;background:" + fill(p.level), title: (p.label || p.level) + ": " + p.n });
    });
    grow(
      spans,
      parts.map(function(p) {
        return p.n;
      })
    );
    var bar = el("div", { className: "bar" + (cls ? " " + cls : "") }, ...spans);
    bar.setAttribute("role", "img");
    bar.setAttribute(
      "aria-label",
      label || parts.map(function(p) {
        return p.n + " " + (p.label || p.level) + (p.hollow ? " not assessed" : p.unrun ? " provisional" : "");
      }).join(", ") || "no entities scored yet"
    );
    return bar;
  }
  function levelLegend(D, rows) {
    var hollow = (rows || []).some(function(r) {
      return !r.compare && unassessedOf(D, r.name);
    });
    var unrun = (rows || []).some(function(r) {
      return !r.compare && provisionalOf(D, r.name);
    });
    var used = /* @__PURE__ */ new Set();
    (rows || []).forEach(function(r) {
      if (!r.compare)
        ((r.sc || {}).levels || []).forEach(function(l) {
          used.add(known(l) || "gold");
        });
    });
    return el(
      "div",
      { className: "legend", style: "margin-bottom:8px" },
      ...["none", "bronze", "silver", "gold"].filter(function(l) {
        return l === "none" || !used.size || used.has(l);
      }).map(function(l) {
        return el("span", {}, el("span", { className: "dot", style: "background:" + fill(l) }), l);
      }),
      hollow ? el("span", {}, el("span", { className: "dot hatch", style: "--seg:" + fill("bronze") }), "hatched, on a bar or a pill: not assessed by an agent yet") : null,
      unrun ? el("span", {}, el("span", { className: "dot dots", style: "--seg:" + fill("silver") }), "◌ dotted, or on a pill: provisional, checks not run in this build") : null
    );
  }
  function splitUnassessed(D, name, parts) {
    var ids = (D.unassessed || {})[name] || [];
    var sc = (D.scorecards || {})[name];
    var rows = (D.scores || {})[name] || {};
    var unassessed = new Set(ids);
    var by = /* @__PURE__ */ new Map();
    var unrunBy = /* @__PURE__ */ new Map();
    Object.keys(rows).forEach(function(id) {
      var s = rows[id];
      if (!s || s.level === "skipped") return;
      if (unassessed.has(id)) by.set(s.level, (by.get(s.level) || 0) + 1);
      else if (unrunIn(sc, s).length) unrunBy.set(s.level, (unrunBy.get(s.level) || 0) + 1);
    });
    var out = [];
    parts.forEach(function(p) {
      var u = Math.min(by.get(p.level) || 0, p.n);
      var r = Math.min(unrunBy.get(p.level) || 0, p.n - u);
      if (p.n - u - r) out.push({ level: p.level, n: p.n - u - r });
      if (r) out.push({ level: p.level, n: r, unrun: true });
      if (u) out.push({ level: p.level, n: u, hollow: true });
    });
    return out;
  }
  function kindParts(S, ids) {
    var tally = /* @__PURE__ */ new Map();
    ids.forEach(function(id) {
      var key = (S.worst[id] || "unscored") + "|" + (S.worst[id] ? S.worstState.get(id) || "" : "");
      tally.set(key, (tally.get(key) || 0) + 1);
    });
    var out = [];
    ["none", "bronze", "silver", "gold", "unscored"].forEach(function(l) {
      ["", "unrun", "hollow"].forEach(function(st) {
        var n = tally.get(l + "|" + st);
        if (n) out.push({ level: l, n, unrun: st === "unrun", hollow: st === "hollow" });
      });
    });
    return out;
  }
  function unassessedOf(D, name) {
    var rows = (D.scores || {})[name] || {};
    return ((D.unassessed || {})[name] || []).filter(function(id) {
      return rows[id] && rows[id].level !== "skipped";
    }).length;
  }
  function legendEl(parts) {
    return el(
      "div",
      { className: "legend" },
      ...parts.map(function(p) {
        if (p.level === "skipped") return el("span", {}, el("b", { textContent: fmt(p.n) }), " not in scope");
        if (p.level === "unscored") return el("span", {}, el("span", { className: "dot unscored" }), el("b", { textContent: fmt(p.n) }), " not scored");
        return el(
          "span",
          {},
          el("span", { className: "dot" + (p.hollow ? " hatch" : p.unrun ? " dots" : ""), style: p.hollow || p.unrun ? "--seg:" + fill(p.level) : "background:" + fill(p.level) }),
          el("b", { textContent: fmt(p.n) }),
          " " + (p.label || p.level) + (p.hollow ? " not assessed" : p.unrun ? " ◌ provisional" : "")
        );
      })
    );
  }
  function badge(level) {
    var only = ONLY_LEVEL.has(level);
    var span = el("span", { className: "lvl " + level, style: "--fill:" + fill(level) + ";--ink:" + ink(level) }, el("i"), level === "skipped" ? "not in scope" : level, only ? el("span", { className: "lvl-top", ariaHidden: "true", textContent: " ✓ top" }) : null);
    span.setAttribute("aria-label", level === "skipped" ? "not in this scorecard's scope" : "level " + level + (only ? ", its scorecard's only and top level" : ""));
    if (LEVEL_NOTE[level]) span.title = LEVEL_NOTE[level];
    return span;
  }
  function unrunIn(sc, score) {
    var order = sc && sc.levels || [];
    var here = order.indexOf(score.level);
    return Object.keys(score.rules || {}).filter(function(r) {
      return score.rules[r] === "unknown" && order.indexOf(((sc.rules || {})[r] || {}).level) <= here;
    });
  }
  function provisionalOf(D, name) {
    var sc = (D.scorecards || {})[name];
    var rows = (D.scores || {})[name] || {};
    return Object.keys(rows).filter(function(id) {
      return rows[id].level !== "skipped" && unrunIn(sc, rows[id]).length > 0;
    }).length;
  }
  function unrunWords(sc, names) {
    var lv = names.map(function(n) {
      return ((sc.rules || {})[n] || {}).level;
    });
    var same = lv.every(function(l) {
      return l === lv[0];
    });
    var list = same ? andList(names.map(function(n) {
      return ruleTitleOf(sc, n);
    })) + (lv[0] ? (names.length === 1 ? ", needed for " : names.length === 2 ? ", both needed for " : ", all needed for ") + levelName(lv[0]) : "") : andList(names.map(function(n, i) {
      return ruleTitleOf(sc, n) + (lv[i] ? " (" + levelName(lv[i]) + ")" : "");
    }));
    return plural(names.length, "check") + " " + (names.length === 1 ? "was" : "were") + " not run in this build: " + list + ".";
  }
  function levelPill(level, unassessed, unrun, quiet) {
    var pill = badge(level);
    if (!unassessed && !(unrun && unrun.length)) return pill;
    pill.classList.add("vacuous", unassessed ? "unjudged" : "unrun");
    pill.append(unassessed ? el("span", { className: "vhatch", ariaHidden: "true" }) : el("span", { className: "vglyph", ariaHidden: "true", textContent: " ◌" }));
    var why = unassessed ? "No agent has assessed this entity, so its agent-judged rules have nothing to read: this level is provisional" : plural(unrun.length, "check") + " this level needs " + (unrun.length === 1 ? "was" : "were") + " not run in this build (" + unrun.join(", ") + "): the level holds only if " + (unrun.length === 1 ? "it passes" : "they pass");
    pill.setAttribute("aria-label", "level " + level + ", " + (unassessed ? "not assessed" : "provisional"));
    if (quiet) return el("span", { className: "pillwrap", title: why }, pill, el("span", { className: "unassessed-tag quiet" + (unassessed ? "" : " unrun"), ariaHidden: "true" }, unassessed ? " not assessed" : "provisional: not run"));
    return el("span", { className: "pillwrap", title: why }, pill, el("span", { className: "unassessed-tag" + (unassessed ? "" : " unrun"), ariaHidden: "true" }, unassessed ? el("span", { className: "vhatch", ariaHidden: "true" }) : null, unassessed ? " not assessed" : "provisional: not run"));
  }
  function medal(level, size) {
    var s = size || 56;
    var none = level === "none" || !known(level);
    var c = fill(level);
    var g = svg(
      "svg",
      { width: s, height: s, viewBox: "0 0 56 56", "aria-hidden": "true" },
      svg("path", { d: "M18 4h8l-4 22h-8z", fill: none ? "var(--line)" : "var(--accent)", opacity: ".85" }),
      svg("path", { d: "M30 4h8l2 22h-8z", fill: none ? "var(--line)" : "var(--accent)", opacity: ".6" }),
      svg("circle", {
        cx: 28,
        cy: 34,
        r: 17,
        fill: none ? "var(--card)" : c,
        stroke: none ? "var(--muted)" : c,
        "stroke-width": none ? 2.5 : 0,
        "stroke-dasharray": none ? "4 3" : ""
      }),
      none ? null : svg("circle", { cx: 28, cy: 34, r: 12, fill: "none", stroke: "#fff", "stroke-opacity": ".55", "stroke-width": 1.5 }),
      level === "not scored" ? svg("text", { x: 28, y: 41, "text-anchor": "middle", "font-size": 20, "font-weight": 700, fill: "var(--muted)", "font-family": "inherit" }, "?") : none ? null : ONLY_LEVEL.has(level) ? svg("path", { d: "M21 34.5l5 5 9.5-10", fill: "none", stroke: "#fff", "stroke-opacity": ".9", "stroke-width": 3.2, "stroke-linecap": "round", "stroke-linejoin": "round" }) : svg("path", {
        d: "M28 26.5l2.3 4.7 5.2.8-3.8 3.7.9 5.1-4.6-2.4-4.6 2.4.9-5.1-3.8-3.7 5.2-.8z",
        fill: "#fff",
        "fill-opacity": ".8"
      })
    );
    return g;
  }
  function boxLogo() {
    return svg(
      "svg",
      { width: 34, height: 34, viewBox: "0 0 34 34", "aria-hidden": "true" },
      svg("rect", { x: 3, y: 11, width: 28, height: 19, rx: 4, fill: "var(--accent)" }),
      svg("path", { d: "M12 11V8a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v3", fill: "none", stroke: "var(--accent)", "stroke-width": 2.4 }),
      svg("rect", { x: 3, y: 17, width: 28, height: 3, fill: "var(--card)", opacity: ".35" }),
      svg("rect", { x: 14.5, y: 15.5, width: 5, height: 6, rx: 1.2, fill: "var(--gold)" })
    );
  }

  // src/lib/routes.js
  function parseHash() {
    var h = (location.hash || "").replace(/^#/, "");
    var query = {};
    var qi = h.indexOf("?");
    if (qi >= 0) {
      var params = new URLSearchParams(h.slice(qi + 1));
      ["q", "level", "outcome", "rule"].forEach(function(k) {
        if (params.has(k)) query[k] = params.get(k);
      });
      h = h.slice(0, qi);
    }
    var route = parsePath(h);
    route.query = query;
    return route;
  }
  function parsePath(h) {
    if (!h || h === "overview") return { mode: "overview" };
    if (h === "catalog") return { mode: "catalog" };
    if (h.indexOf("catalog/") === 0) return { mode: "catalog", kind: decodeURIComponent(h.slice(8)) };
    if (h.indexOf("entity/") === 0) return { mode: "entity", id: decodeURIComponent(h.slice(7)) };
    if (h === "tracks") return { mode: "tracks" };
    if (h.indexOf("track/") === 0) return { mode: "tracks", name: decodeURIComponent(h.slice(6)) };
    if (h.indexOf("rule/") === 0) {
      var rest = h.slice(5);
      var cut = rest.indexOf("/");
      return { mode: "rule", scorecard: decodeURIComponent(rest.slice(0, cut)), rule: decodeURIComponent(rest.slice(cut + 1)) };
    }
    if (h === "campaigns") return { mode: "campaigns" };
    if (h.indexOf("campaign/") === 0) return { mode: "campaign", name: decodeURIComponent(h.slice(9)) };
    if (h === "compare") return { mode: "compare" };
    if (h.indexOf("compare/") === 0) return { mode: "compare", name: decodeURIComponent(h.slice(8)) };
    if (h.indexOf("scorecard/") === 0) return { mode: "scorecard", name: decodeURIComponent(h.slice(10)) };
    if (h === "scorecards") return { mode: "scorecards" };
    return { mode: "scorecard", name: decodeURIComponent(h), bare: true };
  }
  function slugOf(text) {
    return String(text).toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-+|-+$/g, "");
  }
  function trackPath(D, name) {
    var def = (D.tracks || {})[name] || {};
    var slug = def.title ? slugOf(def.title) : "";
    var clash = Object.keys(D.tracks || {}).some(function(n) {
      return n !== name && (n === slug || slugOf((D.tracks[n] || {}).title || "") === slug);
    });
    return "track/" + enc(slug && !clash ? slug : name);
  }
  function trackByPath(D, key) {
    if ((D.tracks || {})[key]) return key;
    var hit = Object.keys(D.tracks || {}).filter(function(n) {
      return trackPath(D, n) === "track/" + enc(key);
    })[0];
    return hit || key;
  }
  function missingTarget(D, route) {
    if (route.bare && !(D.scorecards || {})[route.name]) return "The link “#" + route.name + "” doesn't match anything in snapshot " + D.snapshot + ".";
    if (route.mode === "scorecard" && route.name && !(D.scorecards || {})[route.name]) return "No scorecard is named “" + route.name + "”.";
    if (route.mode === "tracks" && route.name && !(D.tracks || {})[route.name]) return "No track is named “" + route.name + "”.";
    if (route.mode === "rule") {
      var rsc = (D.scorecards || {})[route.scorecard];
      if (!rsc) return "No scorecard is named “" + route.scorecard + "”.";
      if (!(rsc.rules || {})[route.rule]) return title(D, route.scorecard) + " has no rule named “" + route.rule + "”.";
    }
    if (route.mode === "campaign" && !(D.campaigns || {})[route.name]) return "No campaign is named “" + route.name + "”.";
    if (route.mode === "compare" && route.name && !(D.comparisons || {})[route.name]) return "No comparison is named “" + route.name + "”.";
    if (route.mode === "catalog" && route.kind && !(D.catalog || {})[route.kind]) return "No kind is named “" + route.kind + "”.";
    if (route.mode === "entity") {
      var known2 = Object.values(D.catalog || {}).some(function(ids) {
        return ids.indexOf(route.id) >= 0;
      });
      if (!known2) return "No entity has the id “" + route.id + "” in this snapshot.";
    }
    return null;
  }
  function notFound(message) {
    return el(
      "section",
      {},
      el("h2", { textContent: "Not found" }),
      el("p", { className: "lede", textContent: message + " The link may be from an older report." }),
      el("p", {}, link("overview", "", "Go to the overview"), " · ", link("catalog", "", "Browse the catalog"))
    );
  }

  // src/lib/styles.js
  var CSS = [
    ":host{display:block;color:var(--fg);background:var(--bg);font:15px/1.55 system-ui,-apple-system,'Segoe UI',sans-serif;",
    "--bg:#f7f6f2;--fg:#1d1d1b;--muted:#5f5f5a;--faint:#8a8a84;--line:#e4e2dc;--card:#fff;--tint:#f1efe9;",
    "--accent:#2f5d8a;--accent-bg:#e8eff6;--pass:#21683e;--fail:#b23a2f;--unknown:#7a6420;",
    "--none:#d0493d;--none-ink:#a8352a;--bronze:#bf7b3a;--bronze-ink:#86501b;",
    "--silver:#9aa6b2;--silver-ink:#525d69;--gold:#e1b030;--gold-ink:#7a5a00;--skipped:#c9c8c2;--skipped-ink:#5f5f5a;",
    "--judged:#6a45b8;--judged-bg:#f1ecfb;",
    "--shadow:0 1px 2px rgba(30,30,25,.06),0 4px 14px rgba(30,30,25,.05)}",
    "@media (prefers-color-scheme:dark){:host{--bg:#141413;--fg:#ecebe7;--muted:#b0afa8;--faint:#8b8a84;--line:#2f2f2c;",
    "--card:#1d1d1b;--tint:#252523;--accent:#8fbbe6;--accent-bg:#1e2a36;--pass:#6fc493;--fail:#f08a80;--unknown:#d6c177;",
    "--none:#e0665b;--none-ink:#f39a91;--bronze:#d39a5c;--bronze-ink:#e9b985;--silver:#aab5c1;--silver-ink:#cbd4dd;",
    "--gold:#e6c04c;--gold-ink:#f0d57e;--skipped:#5c5c58;--skipped-ink:#b0afa8;--judged:#c3aef5;--judged-bg:#2a2340;--shadow:none}}",
    "*{box-sizing:border-box}",
    "h2[tabindex='-1']:focus{outline:none}",
    "main{max-width:1120px;margin:0 auto;padding:28px 20px 72px}",
    "a{color:var(--accent)}",
    "a:focus-visible,button:focus-visible,summary:focus-visible,input:focus-visible{outline:2px solid var(--accent);outline-offset:2px;border-radius:6px}",
    ".top{display:flex;align-items:center;gap:14px;margin-bottom:6px}",
    ".top svg{flex:none}",
    "h1{font-size:28px;line-height:1.2;margin:0;letter-spacing:-.01em}",
    "h2{font-size:20px;line-height:1.3;margin:0;letter-spacing:-.005em}",
    "h3{font-size:12px;margin:18px 0 8px;color:var(--muted);font-weight:650;text-transform:uppercase;letter-spacing:.06em}",
    ".meta{color:var(--muted);font-size:13px}",
    ".modes{display:flex;gap:4px;margin:20px 0 22px;border-bottom:1px solid var(--line);overflow-x:auto;scrollbar-width:none}",
    ".modes::-webkit-scrollbar{display:none}",
    ".modes.more{-webkit-mask-image:linear-gradient(90deg,#000 85%,transparent);mask-image:linear-gradient(90deg,#000 85%,transparent)}",
    ".modes.less{-webkit-mask-image:linear-gradient(90deg,transparent,#000 15%);mask-image:linear-gradient(90deg,transparent,#000 15%)}",
    ".modes.more.less{-webkit-mask-image:linear-gradient(90deg,transparent,#000 15%,#000 85%,transparent);mask-image:linear-gradient(90deg,transparent,#000 15%,#000 85%,transparent)}",
    ".modes button{border:0;background:none;color:var(--muted);padding:10px 14px;font:inherit;font-weight:600;cursor:pointer;border-bottom:3px solid transparent;margin-bottom:-1px;white-space:nowrap}",
    ".modes button:hover{color:var(--fg)}",
    ".modes button[aria-selected=true]{color:var(--fg);border-bottom-color:var(--accent)}",
    ".modes .count{display:inline-block;min-width:20px;margin-left:6px;padding:0 6px;border-radius:10px;background:var(--tint);color:var(--muted);font-size:12px;font-weight:600}",
    "section{background:var(--card);border:1px solid var(--line);border-radius:14px;padding:20px;margin-bottom:18px;box-shadow:var(--shadow)}",
    ".head{display:flex;flex-wrap:wrap;gap:6px 16px;align-items:baseline;justify-content:space-between;margin-bottom:10px}",
    ".sub{color:var(--muted);font-size:13px}",
    ".lede{color:var(--muted);margin:4px 0 12px;max-width:72ch}",
    ".hero{display:grid;grid-template-columns:minmax(0,5fr) minmax(0,7fr);gap:24px;align-items:start;padding:24px;",
    "background:linear-gradient(135deg,color-mix(in srgb,var(--lvl) 16%,var(--card)) 0%,var(--card) 55%)}",
    ".hero .badge-big{display:flex;align-items:center;gap:16px}",
    ".hero .kicker{font-size:12px;font-weight:650;text-transform:uppercase;letter-spacing:.08em;color:var(--muted)}",
    ".hero .levelword{font-size:42px;line-height:1.05;font-weight:750;color:var(--lvl-ink);text-transform:capitalize;letter-spacing:-.02em}",
    ".hero.plain,.hero.first-run{grid-template-columns:minmax(0,1fr)}.hero.plain .levelword,.hero.first-run .levelword{text-transform:none;font-size:34px}.hero.plain .verdict,.hero.first-run .verdict{max-width:72ch}",
    ".hero.first-run{--lvl:var(--muted);--lvl-ink:var(--fg)}",
    ".hero .verdict{font-size:18px;line-height:1.5;margin:16px 0 0}.attn{margin:12px 0 0;font-size:14px;line-height:1.5}",
    ".wins{margin:8px 0 0;font-size:14px;color:var(--pass)}.wins b{font-weight:650}",
    ".next{margin-top:14px;padding:12px 14px;border-radius:10px;background:var(--tint);font-size:14px}",
    ".next b{font-weight:650}",
    ".ladder{display:grid;grid-template-columns:repeat(var(--steps),minmax(0,1fr));gap:8px}",
    ".rung{border-radius:10px;padding:10px;background:var(--tint);min-height:120px;border:1px solid transparent}",
    ".rung.here{border-color:var(--lvl);background:color-mix(in srgb,var(--lvl) 12%,var(--card))}",
    ".rung .rh{display:flex;align-items:center;gap:6px;font-size:13px;font-weight:650;text-transform:capitalize;margin-bottom:8px}",
    ".rung .rh .n{margin-left:auto;color:var(--muted);font-weight:600}",
    ".rung a{display:block;font-size:13px;line-height:1.3;padding:5px 7px;margin-top:4px;border-radius:6px;background:var(--card);color:var(--fg);text-decoration:none;border:1px solid var(--line)}",
    ".rung a:hover{border-color:var(--accent)}",
    ".rung a.done{border-color:color-mix(in srgb,var(--pass) 45%,var(--line))}",
    ".rung .topmark{display:block;font-size:12px;font-weight:650;color:var(--pass);margin-top:2px}",
    ".nextmedal .mover{display:block;margin-top:6px}",
    "details.ent>summary .lvl{flex-wrap:wrap;max-width:100%}details.ent>summary .lvl-top{flex-basis:100%;padding-left:14px}",
    ".elevel .topmark{font-size:14px;font-weight:650;color:var(--pass);text-transform:none;letter-spacing:0}",
    ".rung .capmark{display:block;font-size:12px;font-weight:650;color:var(--unknown);margin-top:2px}.rung a.capped{border-color:var(--unknown);border-left-width:3px}.weakest .topmark{color:var(--pass);font-weight:650}.weakest .capnote{color:var(--unknown);font-weight:650}.capnote svg,.capmark svg,.debt svg{vertical-align:-1px}.tcap{display:block;font-size:13px;font-weight:500;line-height:1.4;margin-top:4px}",
    ".rung .empty{font-size:12px}",
    ".tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(190px,1fr));gap:12px;margin-bottom:18px}",
    ".tile{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:14px 16px;display:flex;gap:12px;align-items:center;box-shadow:var(--shadow);text-decoration:none;color:var(--fg)}",
    "a.tile:hover{border-color:var(--accent)}",
    ".tile strong{display:block;font-size:24px;line-height:1.15;font-variant-numeric:tabular-nums}",
    ".tile span{color:var(--muted);font-size:13px;line-height:1.3;display:block}",
    ".tile.bad strong{color:var(--fail)}.tile.good strong{color:var(--pass)}",
    ".bar{display:flex;height:10px;border-radius:5px;overflow:hidden;background:var(--line);gap:2px}",
    ".bar span{display:block;height:100%;transition:flex-grow .6s cubic-bezier(.2,.7,.2,1);box-shadow:inset 0 0 0 1px color-mix(in srgb,var(--fg) 34%,transparent)}",
    ".bar.lg{height:16px;border-radius:8px}",
    ".bar.xs{height:6px;gap:1px}",
    ".legend{display:flex;flex-wrap:wrap;gap:6px 14px;font-size:12px;color:var(--muted);margin-top:6px}",
    ".legend b{color:var(--fg);font-weight:650;font-variant-numeric:tabular-nums}",
    ".dot{display:inline-block;width:9px;height:9px;border-radius:50%;margin-right:5px;vertical-align:-1px}.legend.trackkey{margin:2px 0 18px}.medal-key{display:inline-flex;align-items:center;gap:4px;white-space:nowrap}",
    ".lvl{display:inline-flex;align-items:center;gap:5px;padding:1px 8px 1px 6px;border-radius:999px;font-size:12px;font-weight:650;white-space:nowrap;",
    "color:var(--ink);background:color-mix(in srgb,var(--fill) 18%,transparent);border:1px solid color-mix(in srgb,var(--fill) 50%,transparent)}",
    ".lvl i{width:8px;height:8px;border-radius:50%;background:var(--fill);display:inline-block}",
    ".lvl.none i{background:transparent;border:1.5px solid var(--fill)}",
    ".rows{display:flex;flex-direction:column}",
    "a.row{display:grid;grid-template-columns:minmax(0,2.2fr) minmax(0,2.4fr) 90px;gap:16px;align-items:center;padding:11px 10px;border-top:1px solid var(--line);color:var(--fg);text-decoration:none;border-radius:8px}",
    "a.row:hover{background:var(--tint)}",
    ".keyd>summary{display:inline-flex;align-items:center;gap:6px;min-height:24px;font-size:13px;font-weight:600;color:var(--muted);cursor:pointer;list-style:none}.keyd>summary::-webkit-details-marker{display:none}.keyd>summary::before{content:'\\25B8';transition:transform .15s}.keyd[open]>summary::before{transform:rotate(90deg)}.keyd>summary:hover{color:var(--fg)}.keyd{margin-bottom:14px}",
    ".row .t{font-weight:600;line-height:1.3}a.row{row-gap:2px}.row .rp{grid-column:1/3;font-size:13px;line-height:1.4;color:var(--muted)}.row .rp.in{padding-left:40px}",
    ".row .c{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);margin-top:2px;overflow-wrap:anywhere}",
    ".row .num{text-align:right;font-variant-numeric:tabular-nums}",
    ".row .num strong{display:block;font-size:18px;line-height:1.1}",
    ".row .num.bad strong{color:var(--fail)}.row .num.ok strong{color:var(--pass)}",
    ".row .num span{font-size:12px;color:var(--muted)}",
    ".rows-head{display:grid;grid-template-columns:minmax(0,2.2fr) minmax(0,2.4fr) 90px;gap:16px;padding:0 10px 6px;font-size:12px;color:var(--muted);font-weight:600}",
    ".rows-head span:last-child{text-align:right}",
    ".cards{display:grid;grid-template-columns:repeat(auto-fill,minmax(250px,1fr));gap:12px}",
    "a.card{display:flex;gap:14px;align-items:center;padding:14px;border:1px solid var(--line);border-radius:12px;background:var(--card);color:var(--fg);text-decoration:none;transition:transform .15s,border-color .15s}",
    "a.card:hover{border-color:var(--accent);transform:translateY(-1px)}",
    "a.card[aria-current=true]{border-color:var(--accent);box-shadow:0 0 0 1px var(--accent)}",
    "a.card .t{font-weight:600;line-height:1.3}",
    "a.card .s{font-size:13px;color:var(--muted);margin-top:3px}",
    ".ring{flex:none}",
    ".ring circle{transition:stroke-dashoffset .8s cubic-bezier(.2,.7,.2,1)}",
    ".ring text{font:700 13px system-ui,sans-serif;fill:var(--fg)}",
    ".pill{display:inline-block;padding:1px 8px;border-radius:999px;font-size:12px;font-weight:600;background:var(--tint);color:var(--muted)}",
    ".pill.good{background:color-mix(in srgb,var(--pass) 16%,transparent);color:var(--pass)}",
    ".pill.bad{background:color-mix(in srgb,var(--fail) 14%,transparent);color:var(--fail)}.pill.unknown{background:transparent;border:1px dashed var(--unknown);color:var(--unknown)}.pill.neutral{background:var(--tint);border:1px solid var(--line);color:var(--muted)}",
    "input[type=search]{width:100%;margin:10px 0;padding:9px 12px;border:1px solid var(--line);border-radius:8px;background:var(--bg);color:var(--fg);font:inherit}",
    ".filters{display:flex;flex-wrap:wrap;gap:8px 16px;margin:12px 0 4px;align-items:center}",
    ".seg{display:inline-flex;border:1px solid var(--line);border-radius:8px;overflow:hidden}",
    ".seg button{border:0;border-right:1px solid var(--line);background:var(--card);color:var(--fg);padding:6px 12px;font:inherit;font-size:13px;cursor:pointer;min-height:32px}.seg button::first-letter{text-transform:uppercase}",
    ".seg button:last-child{border-right:0}",
    ".chip-filter{display:inline-flex;align-items:center;gap:4px;min-height:32px;padding:4px 12px;border:1px solid var(--accent);border-radius:999px;background:var(--tint);color:var(--fg);font:inherit;font-size:13px;cursor:pointer}",
    ".chip-filter:hover{background:var(--card)}",
    "a.card,.track,a.kind,a.tile,a.factor,a.erow,a.row,.rung a,.box a,.rank a,.fail-row,.seg button,.modes button,details.ent>summary,.chip-filter{transition:background-color .15s,border-color .15s,color .15s,transform .15s,box-shadow .15s}",
    "@keyframes enter{from{opacity:0;transform:translateY(8px)}to{opacity:1;transform:none}}",
    ".view>*{animation:enter .3s cubic-bezier(.2,.7,.2,1) both}.view>:nth-child(2){animation-delay:.04s}.view>:nth-child(3){animation-delay:.08s}.view>:nth-child(n+4){animation-delay:.12s}",
    ".seg button[aria-pressed=true]{background:var(--accent-bg);color:var(--accent);font-weight:650}",
    "details.ent{border-top:1px solid var(--line)}",
    "details.ent>summary{cursor:pointer;display:grid;grid-template-columns:112px minmax(min(14rem,100%),1fr) auto;gap:12px;align-items:center;padding:10px 6px;list-style:none;border-radius:8px}",
    "details.ent>summary:hover{background:var(--tint)}",
    "details.ent>summary::-webkit-details-marker{display:none}",
    "details.ent>summary .name{font-weight:600;overflow-wrap:anywhere;min-width:0}details.ent>summary .name .tail{white-space:nowrap}details.ent>summary .name .tail.long{white-space:normal}",
    "details.ent>summary .name small{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);font-weight:400;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:100%}",
    "details.ent>summary .tally{font-size:12px;color:var(--muted);text-align:right;min-width:0}",
    "details.ent>summary .tally b{color:var(--fg);font-weight:650}details.ent>summary .tally b.bad{color:var(--fail)}",
    "details.ent[open]>summary{background:var(--tint)}",
    ".ent-body{padding:6px 6px 14px 104px}.ent-members{margin:0 0 10px 24px;border-left:2px solid var(--line)}.ent-members details.ent:first-child{border-top:0}details.ent-group>summary .name small{white-space:normal;overflow-wrap:anywhere}",
    ".chev{display:inline-block;transition:transform .15s;color:var(--muted);margin-left:6px}",
    "details[open]>summary .chev{transform:rotate(90deg)}",
    ".rule{padding:10px 12px;border-radius:8px;border:1px solid var(--line);border-left:4px solid var(--fail);margin-top:8px;background:var(--card)}",
    ".callout.judged-note{background:var(--judged-bg)}",
    ".bar span.unscored,.dot.unscored{background:var(--card);box-shadow:inset 0 0 0 1.5px var(--faint)}.bar span.hollow{background:repeating-linear-gradient(135deg,var(--judged) 0 1.5px,transparent 1.5px 6px),color-mix(in srgb,var(--seg) 30%,var(--card));box-shadow:inset 0 0 0 1.5px var(--judged)}.unjudged-n,.judged-n{color:var(--judged)}.dot.hatch{border-radius:2px;background:repeating-linear-gradient(135deg,var(--judged) 0 1px,transparent 1px 3.5px),color-mix(in srgb,var(--seg) 30%,var(--card));box-shadow:inset 0 0 0 1.5px var(--judged)}.judged-list{margin:8px 0 0}",
    ".sc-status{display:flex;gap:14px;align-items:flex-start;margin:4px 0 14px;padding:12px 14px;border:1px solid var(--line);border-radius:12px;background:var(--tint)}.sc-status>svg{flex:none}.sc-status-b{flex:1;min-width:0}.sc-status-t{display:flex;flex-wrap:wrap;align-items:center;gap:2px 6px;margin-bottom:8px;font-size:14px}.sc-status-t .topmark{color:var(--pass);font-weight:650}.sc-status .legend{margin-top:6px}.sc-status-t .sep{color:var(--muted)}@media (max-width:640px){.sc-status-t .sep{display:none}.sc-status-t .fact{flex-basis:100%}}.why-text.clamped{display:-webkit-box;-webkit-box-orient:vertical;-webkit-line-clamp:3;overflow:hidden}.why-more{margin-top:2px;min-height:24px}",
    ".bar span.unrun{background:radial-gradient(circle,var(--seg) 1.6px,transparent 1.9px) 0 0/6px 6px;box-shadow:inset 0 0 0 1.5px var(--unknown)}.dot.dots{border-radius:2px;background:radial-gradient(circle,var(--seg) 1.2px,transparent 1.5px) 0 0/4px 4px;box-shadow:inset 0 0 0 1.5px var(--unknown)}",
    ".rule.unknown{border-left-color:var(--unknown)}.rule.waived{border-left-color:var(--bronze)}.rule.unassessed{border-left-color:var(--judged)}",
    ".basis-note p{margin:0 0 4px}.h2h-wide{max-width:560px}.h2h-wide .h2h{width:100%;font-size:13px}.h2h-wide .h2h-head,.h2h-wide .h2h-row{grid-template-columns:160px 1fr 1fr}.h2h-row[aria-current]{font-weight:700;background:var(--accent-bg);border-radius:6px}.rank-top:not([hidden]){display:block;margin:8px 0 0;font-size:13px}.cmp-prov{margin:6px 0 0;display:grid;gap:2px}.cmp-src{display:flex;flex-wrap:wrap;align-items:center;gap:2px 10px}.cmp-src .refs{margin:0;line-height:22px}.cmp-brief>summary{cursor:pointer;width:max-content;max-width:100%;padding:3px 0;color:var(--accent)}",
    ".why.incl{display:flex;flex-wrap:wrap;align-items:center;gap:4px 8px}.kpi-bar{display:flex;flex:0 0 46px;height:10px;border-radius:5px;background:var(--line);overflow:hidden}.why.incl .kpi-bar{flex-basis:90px}.kpi-bar span.hollow,.kpi-bar.short span.hollow{background:repeating-linear-gradient(135deg,var(--judged) 0 1.5px,transparent 1.5px 4px);box-shadow:inset 0 0 0 1px var(--judged)}.kpi-bar span{display:block;height:100%;background:var(--pass)}.kpi-bar.short span{background:var(--muted)}.kpi-pips{flex:0 0 46px;display:flex;flex-wrap:wrap;gap:4px}.kpi-pips span{width:10px;height:10px;border-radius:50%;box-shadow:inset 0 0 0 2px var(--faint)}.kpi-pips span.on{background:var(--pass);box-shadow:none}",
    ".cprog{flex:0 0 76px;display:grid;gap:3px}.cprog b{font-size:20px;font-weight:700;font-variant-numeric:tabular-nums;line-height:1.1}.cprog small{font-size:13px;color:var(--muted)}.cprog .gb{position:relative;height:8px;border-radius:4px;background:var(--line)}.cprog .gb>span{position:absolute;left:0;top:0;bottom:0;border-radius:4px;background:var(--muted)}.cprog.on .gb>span{background:var(--pass)}.cprog .gb>i{position:absolute;top:-3px;bottom:-3px;width:2px;border-radius:1px;background:var(--fg)}.cprog.big{flex-basis:120px}.cprog.big b{font-size:28px}",
    ".hollow-key{display:flex;flex-wrap:wrap;align-items:center;gap:4px 16px;padding:6px 10px;font-size:13px;color:var(--muted);background:var(--tint);border-radius:8px;margin:4px 0}.hollow-key>span{display:inline-flex;align-items:center;gap:6px}.hollow-key b{color:var(--fg);font-weight:600}",
    ".rid{font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);overflow-wrap:anywhere}.rule .rn{font-weight:650;font-size:inherit;margin:0;display:inline}.sc-block h3.sct{display:inline;margin:0;font-size:16px}",
    ".rule .fix{margin-top:4px;font-size:14px}",
    ".tile-note{display:block;margin-top:3px;font-size:13px;color:var(--muted);line-height:1.35}",
    ".later-h{margin:14px 0 2px;font-size:13px;color:var(--muted)}.rule-c{padding:4px 12px;margin-top:4px}.rule-c>summary{display:flex;flex-wrap:wrap;gap:4px 8px;align-items:center;min-height:28px;cursor:pointer}.rule-c>summary .rn{font-weight:600}.rule-c>summary::before{content:'\\25B8';color:var(--muted)}.rule-c[open]>summary::before{content:'\\25BE'}.rule-c[open]{padding-bottom:10px}.rule-c[open]>summary{margin-bottom:4px}.rule-c>summary .rwhy{flex-basis:100%;padding-left:18px;font-size:13px;line-height:1.4;color:var(--muted)}.rule-c[open]>summary .rwhy{display:none}",
    ".rule ul{margin:6px 0 0;padding-left:18px;font-size:13px;color:var(--muted)}",
    ".rule li{overflow-wrap:anywhere}",
    ".rule .evidence details{font-size:13px;color:var(--muted);margin:2px 0 0 18px}.rule .evidence details ul{margin-top:2px}.rule .evidence summary{cursor:pointer;min-height:24px;line-height:24px}",
    ".rule code,.evid code{font:13px ui-monospace,SFMono-Regular,Menlo,monospace;overflow-wrap:anywhere}",
    ".outcome{font-size:12px;font-weight:650;margin-left:6px}",
    ".outcome.fail{color:var(--fail)}.outcome.unknown{color:var(--unknown)}.outcome.waived{color:var(--bronze-ink)}.outcome.skipped{color:var(--muted)}.outcome.unassessed{color:var(--judged)}",
    ".passes{display:flex;flex-wrap:wrap;align-items:center;gap:6px;margin-top:8px}",
    ".passes>span:not(.lvl){font-size:12px;padding:2px 8px 2px 6px;border-radius:999px;background:color-mix(in srgb,var(--pass) 12%,transparent);color:var(--pass);font-weight:600}",
    ".passes>span:not(.lvl)::before{content:'\\2713  '}",
    ".passes>span:not(.lvl){display:inline-flex;align-items:center;gap:2px}.passes>span a{color:inherit;text-decoration:none}.passes .basis.measured .bl{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0)}.passes .basis.tip.measured.mini{display:inline-flex;padding:0;min-width:24px;justify-content:center}.passes>span a:hover{text-decoration:underline}",
    ".passes .basis.mini{border:0;background:none;padding:0;min-width:0;color:inherit}.passes>span.judged:not(.lvl){background:var(--judged-bg);color:var(--judged)}",
    ".passes em{font-style:normal;font-size:12px;color:var(--muted);margin-right:4px}",
    ".callout{display:flex;gap:10px;align-items:flex-start;padding:10px 12px;border-radius:10px;background:var(--accent-bg);font-size:14px;margin:8px 0}",
    ".callout.done{background:color-mix(in srgb,var(--pass) 12%,var(--card))}",
    ".callout.near{background:color-mix(in srgb,var(--pass) 7%,var(--card));border-left:3px solid var(--pass)}",
    ".close-note{color:var(--fg)}.close-note b{color:var(--pass)}",
    ".fails{display:flex;flex-direction:column;gap:2px}",
    ".fail-row{display:grid;grid-template-columns:minmax(200px,2fr) minmax(0,3fr) 70px;gap:12px;align-items:center;padding:7px 8px;border:0;border-radius:8px;background:none;color:var(--fg);font:inherit;font-size:13px;text-align:left;cursor:pointer}",
    ".fail-row:hover{background:var(--tint)}",
    ".fail-row .rn{display:flex;flex-wrap:wrap;gap:4px 8px;align-items:center;font-weight:600;overflow-wrap:break-word}",
    ".fail-row .fc{text-align:right;color:var(--muted);font-variant-numeric:tabular-nums}",
    ".scope-note details{display:inline}.scope-note summary{display:inline-flex;align-items:center;min-height:24px;cursor:pointer;color:var(--accent)}.scope-note details[open] div{margin-top:4px}",
    ".basis.tip{position:relative;cursor:help;line-height:22px}.basis.tip:hover::after,.basis.tip:focus::after{content:attr(data-tip);position:absolute;left:0;top:calc(100% + 4px);z-index:5;width:max-content;max-width:min(280px,70vw);padding:6px 9px;border-radius:8px;background:var(--fg);color:var(--bg);font-size:13px;font-weight:450;line-height:1.4;letter-spacing:0;text-transform:none;white-space:normal;box-shadow:0 4px 14px rgba(0,0,0,.18)}",
    "abbr.gloss{position:relative;display:inline-block;min-height:24px;line-height:24px;vertical-align:bottom;text-decoration:underline dotted;text-underline-offset:3px;cursor:help}abbr.gloss:hover::after,abbr.gloss:focus::after{content:attr(data-tip);position:absolute;left:0;top:calc(100% + 4px);z-index:5;width:max-content;max-width:min(280px,70vw);padding:6px 9px;border-radius:8px;background:var(--fg);color:var(--bg);font-size:13px;font-weight:450;line-height:1.4;white-space:normal;text-decoration:none;box-shadow:0 4px 14px rgba(0,0,0,.18)}",
    ".jumpbar{display:flex;flex-wrap:wrap;align-items:center;gap:4px 14px;margin:14px 0 0;padding:6px 10px;border-radius:8px;background:var(--tint);font-size:13px}.jumpbar button.link{min-height:24px}",
    ".levels-need{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:10px;margin:8px 0 4px;align-items:start}",
    ".need{padding:10px 12px;border-radius:10px;background:var(--tint);font-size:13px}",
    ".need .lvl{margin-bottom:6px}.lvl-top{text-transform:none;font-weight:500}.bicon{display:inline-flex;vertical-align:-1px;color:var(--muted)}.bicon.judged{color:var(--judged)}",
    ".need ul{margin:0;padding-left:16px}",
    ".need li{margin:6px 0}",
    ".need li .why{display:block;color:var(--muted);font-size:13px;line-height:1.4}.levels-need.clamped li .why .rest{display:none}.need-head{display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:4px 8px;margin-bottom:6px}.need-head .lvl{margin-bottom:0}.need-basis{font-size:12px;color:var(--muted);display:inline-flex;align-items:center;gap:4px}.need-toggle{display:inline-flex;align-items:center;min-height:24px;font-size:13px;margin:0 0 4px}",
    "code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.92em}",
    ".picker{display:none}",
    ".picker select{width:100%;min-width:0;max-width:100%;padding:9px 10px;border:1px solid var(--line);border-radius:8px;background:var(--bg);color:var(--fg);font:inherit;min-height:40px}",
    ".kinds{display:grid;grid-template-columns:repeat(auto-fill,minmax(170px,1fr));gap:10px}",
    "a.kind{display:block;padding:12px;border:1px solid var(--line);border-radius:10px;background:var(--card);color:var(--fg);text-decoration:none}",
    "a.kind:hover{border-color:var(--accent)}",
    "a.kind[aria-current=true]{border-color:var(--accent);box-shadow:0 0 0 1px var(--accent)}",
    "a.kind strong{display:block;font-size:14px}",
    "a.kind span{display:block;color:var(--muted);font-size:12px;margin-bottom:6px}",
    ".box{display:flex;flex-wrap:wrap;gap:6px}",
    ".box a{font-size:13px;padding:4px 10px;border-radius:999px;background:var(--tint);color:var(--fg);text-decoration:none;border:1px solid transparent}",
    ".box a:hover{border-color:var(--accent)}",
    ".box a b{font-variant-numeric:tabular-nums;margin-left:4px;color:var(--muted);font-weight:600}",
    "a.erow{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:12px;align-items:center;padding:10px 8px;border-top:1px solid var(--line);color:var(--fg);text-decoration:none;border-radius:8px}",
    "a.erow:hover{background:var(--tint)}.egroup>summary{display:grid;gap:12px;align-items:center;padding:10px 8px;border-top:1px solid var(--line);border-radius:8px;cursor:pointer;list-style:none}.egroup>summary::-webkit-details-marker{display:none}.egroup>summary:hover{background:var(--tint)}.egroup>summary .name{font-weight:600;min-width:0}.egroup>summary .name small{display:block;font-size:12px;color:var(--muted);font-weight:400;white-space:nowrap;overflow:hidden;text-overflow:ellipsis}.egroup>summary .chev{display:inline-block;transition:transform .15s}.egroup[open]>summary .chev{transform:rotate(90deg)}.members{font-size:12px;color:var(--muted);padding:0 10px 9px;margin-top:-2px;line-height:1.6}.members a{color:inherit}details[open]+.members{display:none}.egroup-members{margin-left:10px;padding-left:10px;border-left:2px solid var(--line)}",
    "a.erow .name{font-weight:600;overflow-wrap:anywhere;min-width:0}",
    "a.erow .name small{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);font-weight:400;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:100%}",
    "a.erow .badges{display:flex;flex-wrap:wrap;gap:4px;justify-content:flex-end}",
    "code.kid{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;font-weight:400;color:var(--muted)}",
    ".kchips{display:grid;grid-template-columns:minmax(9rem,12rem) 1fr;gap:4px 12px;align-items:baseline;padding:6px 0;border-top:1px solid var(--line)}.kchips-g{font-size:12px;font-weight:650;color:var(--muted);text-transform:uppercase;letter-spacing:.05em}.kchips .box a{display:inline-flex;align-items:center;min-height:28px}.kchips .box a[aria-current=true]{border-color:var(--accent);background:var(--card);font-weight:650}",
    ".ehdr{display:grid;gap:12px;padding:6px 8px;font-size:12px;font-weight:650;color:var(--muted)}.ehdr span{overflow-wrap:break-word}",
    ".weakest{display:block;margin-top:3px;font-size:13px;color:var(--muted)}.weakest .lvl{font-size:12px}",
    ".vhatch{display:inline-block;width:9px;height:9px;margin-left:5px;border-radius:2px;vertical-align:-1px;background:repeating-linear-gradient(135deg,var(--judged) 0 1.5px,transparent 1.5px 3.5px);box-shadow:inset 0 0 0 1px var(--judged)}.unassessed-tag .vhatch{margin:0 4px 0 0}.ecell{font-size:13px}.unassessed-tag{display:block;margin-top:2px;font-size:13px;color:var(--judged)}.unassessed-tag.unrun{color:var(--unknown)}.lvl.vacuous{background:transparent;border-style:dashed}.lvl.vacuous.unjudged{border-color:var(--judged)}.lvl.vacuous.unrun{border-style:dotted;border-color:var(--unknown)}.unassessed-tag svg{vertical-align:-1px}.lvl.vacuous.unjudged .vglyph{color:var(--judged)}.lvl.vacuous.unrun .vglyph{color:var(--unknown)}.pillwrap{display:inline-flex;flex-direction:column;align-items:flex-start;vertical-align:middle}",
    "@media (max-width:820px){.unassessed-tag{display:inline;margin:0}}",
    ".pillwrap .unassessed-tag.quiet{display:none}",
    ".crumb{font-size:13px;color:var(--muted);margin-bottom:12px}",
    ".sc-group{margin-top:20px}.sc-group h3{margin:0 0 4px}.sc-group h3 .sub{font-weight:400;text-transform:none;letter-spacing:0}.sc-index input[type=search]{max-width:420px}",
    ".sc-rows{border-top:1px solid var(--line)}.sc-row{display:grid;grid-template-columns:26px minmax(0,1.25fr) minmax(0,1.6fr) 104px;gap:12px;align-items:center;padding:7px 8px;border-bottom:1px solid var(--line);color:var(--fg);text-decoration:none}.sc-row:hover{background:var(--tint)}.sc-row .sc-name{display:flex;flex-wrap:wrap;align-items:baseline;gap:2px 8px;min-width:0}.sc-row .sc-name .t{font-weight:600}.sc-row .sc-name .c{font-size:12px;color:var(--muted)}.sc-row{row-gap:2px}.sc-row>svg{grid-row:1/span 2}.sc-row .sc-why{grid-column:2/-1;grid-row:2;min-width:0;font-size:14px;line-height:1.4;color:var(--muted)}.sc-row .sc-dist small{display:block;font-size:13px;color:var(--muted);margin-top:3px}.sc-head{display:grid;grid-template-columns:26px minmax(0,1.25fr) minmax(0,1.6fr) 104px;gap:12px;padding:4px 8px;font-size:12px;color:var(--muted);border-bottom:1px solid var(--line)}.sc-head span:last-child{text-align:right}.sc-row .sc-weak{display:flex;flex-direction:column;align-items:flex-end;gap:1px}.sc-row .sc-weak small{font-size:13px}.sc-row .sc-weak .bad{color:var(--fail);font-weight:600}.sc-row .sc-weak .topmark{color:var(--pass);font-weight:600}",
    "@media (max-width:640px){.sc-head{display:none}.sc-row .sc-name code.c{display:none}.sc-row{grid-template-columns:26px minmax(0,1fr) auto;row-gap:4px}.sc-row .sc-weak{grid-column:3;grid-row:1}.sc-row .sc-dist{grid-column:2/4;grid-row:2}.sc-row .sc-why{grid-column:2/4;grid-row:3}.sc-row>svg{grid-row:1}}",
    ".crumb a{color:var(--muted);display:inline-block;padding:4px 0;margin:-4px 0}",
    "a.rl{display:inline-flex;align-items:center;min-height:24px;min-width:24px;vertical-align:middle}.refs a{display:inline-flex;align-items:center;min-height:26px}",
    ".passes>span a{display:inline-block;padding:4px 0;margin:-4px 0}",
    ".ehead{display:flex;flex-wrap:wrap;gap:16px;align-items:center;justify-content:space-between}",
    ".ehead h2{font-size:24px}",
    ".ehead code{font:13px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);overflow-wrap:anywhere}",
    ".sc-block{border-top:1px solid var(--line);padding-top:14px;margin-top:14px}",
    ".sc-block .head a{font-weight:650;font-size:16px;text-decoration:none}",
    ".sc-block .head a:hover{text-decoration:underline}",
    ".rank{position:relative;display:flex;flex-direction:column;max-height:min(62vh,560px);overflow:auto;margin-top:4px;scrollbar-width:thin;scrollbar-color:var(--faint) transparent;border-top:1px solid var(--line);border-bottom:1px solid var(--line)}",
    ".rank.more{mask-image:linear-gradient(to bottom,#000 calc(100% - 14px),transparent)}.rank.less{mask-image:linear-gradient(to bottom,transparent,#000 14px)}.rank.less.more{mask-image:linear-gradient(to bottom,transparent,#000 14px,#000 calc(100% - 14px),transparent)}",
    ".rank a{display:flex;justify-content:space-between;gap:12px;align-items:baseline;padding:8px 10px;border-radius:8px;color:var(--fg);text-decoration:none}",
    ".rank a:hover{background:var(--tint)}",
    ".rank-head{font-size:12px;font-weight:650;letter-spacing:.04em;text-transform:uppercase;color:var(--muted);padding:10px 10px 4px}",
    ".rank a[aria-current=true]{background:var(--accent-bg);font-weight:650}",
    ".rank .debt{font-size:12px;color:var(--muted);white-space:nowrap}",
    ".rank .debt.bad{color:var(--fail);font-weight:650}.rank .debt.ok{color:var(--pass)}",
    ".split{display:grid;grid-template-columns:300px minmax(0,1fr);gap:18px;align-items:start}",
    ".split>section:first-child{position:sticky;top:12px}",
    ".gap{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,2fr);gap:12px;padding:9px 6px;border-top:1px solid var(--line);font-size:14px}",
    ".gap a{font-weight:600}",
    ".gap .why{color:var(--muted);font-size:13px;overflow-wrap:anywhere}",
    ".empty{color:var(--muted);font-size:14px;padding:10px 0}",
    ".empty.good{color:var(--pass);font-weight:600}",
    "button.link{background:none;border:0;padding:4px 0;margin:-4px 0;color:var(--accent);font:inherit;cursor:pointer;text-decoration:underline;text-underline-offset:2px}",
    ':where(section a:not([class]),section a[class=""]){display:inline-block;padding:4px 0;margin:-4px 0}',
    ".more{margin-top:10px}",
    ".milestones{position:relative;height:18px;margin-top:2px}",
    ".milestones span{position:absolute;top:0;transform:translateX(-50%);font-size:12px;color:var(--muted);white-space:nowrap}",
    ".milestones span::before{content:'';display:block;width:2px;height:6px;background:var(--faint);margin:0 auto 1px}",
    ".milestones span.met{color:var(--pass);font-weight:650}",
    ".tablewrap{overflow-x:auto}",
    "table.cmp{width:100%;border-collapse:collapse;font-size:14px}",
    "table.cmp th{text-align:left;font-size:12px;font-weight:650;color:var(--muted);padding:6px 10px 6px 0;border-bottom:1px solid var(--line)}",
    "table.cmp td{padding:10px 10px 10px 0;border-top:1px solid var(--line);vertical-align:top}",
    "table.cmp td.n{font-variant-numeric:tabular-nums;color:var(--muted)}",
    "table.cmp a{font-weight:600}",
    ".standing{display:inline-block;padding:1px 8px;border-radius:999px;font-size:12px;font-weight:650;background:var(--tint);color:var(--muted)}",
    ".standing.ahead{background:color-mix(in srgb,var(--pass) 16%,transparent);color:var(--pass)}",
    ".standing.behind{background:color-mix(in srgb,var(--fail) 14%,transparent);color:var(--fail)}",
    ".standing.parity{background:var(--tint);color:var(--fg);box-shadow:inset 0 0 0 1px var(--line)}",
    ".value{display:block;font-size:13px;color:var(--muted);margin-top:2px}",
    ".basis{display:inline-flex;align-items:center;gap:4px;padding:0 7px 0 5px;border-radius:6px;font-size:12px;font-weight:650;white-space:nowrap;line-height:20px;border:1px solid var(--line);color:var(--muted);background:var(--card);vertical-align:1px;text-transform:none;letter-spacing:0}",
    ".basis.judged{color:var(--judged);border-color:color-mix(in srgb,var(--judged) 45%,transparent);background:var(--judged-bg)}",
    ".basis.mini{padding:0 5px 0 3px;gap:3px;font-size:12px;font-weight:600;line-height:18px}",
    ".basis.tip.mini{min-height:24px;align-items:center;vertical-align:middle}",
    ".basis svg{display:block;flex:none}",
    ".basis-key{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:8px 18px;padding:10px 12px;border-radius:10px;border:1px dashed var(--line);font-size:13px;color:var(--muted);margin:10px 0}",
    ".basis-key>div{display:flex;gap:8px;align-items:flex-start}.basis-key.one{display:block;padding:6px 12px}",
    ".basis-key b{color:var(--fg);font-weight:650}",
    ".why{margin-top:6px;font-size:13px;line-height:1.5}",
    ".why>b{font-weight:650}",
    ".refs{display:flex;flex-wrap:wrap;align-items:center;gap:4px 14px;margin:4px 0 2px;font-size:13px;line-height:26px}",
    ".refs a{text-underline-offset:2px}",
    ".refs a .ext{font-size:12px}",
    ".rationale{margin:6px 0 12px;padding:10px 14px;border-left:3px solid var(--accent);background:var(--tint);border-radius:0 10px 10px 0;max-width:80ch;font-size:14px}",
    ".rationale>b{display:block;font-size:12px;color:var(--muted);text-transform:uppercase;letter-spacing:.06em;margin-bottom:2px}",
    ".cel{margin:6px 0;padding:10px 12px;border-radius:8px;background:var(--tint);overflow-x:auto;font:13px/1.5 ui-monospace,SFMono-Regular,Menlo,monospace;white-space:pre-wrap;overflow-wrap:anywhere}",
    ".tracks{display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:14px;margin-top:6px;align-items:start}",
    ".track{position:relative;display:flex;flex-direction:column;gap:10px;padding:16px;border:1px solid var(--line);border-radius:14px;color:var(--fg);text-decoration:none;transition:transform .15s,border-color .15s;",
    "background:linear-gradient(160deg,color-mix(in srgb,var(--tc) 13%,var(--card)) 0%,var(--card) 62%)}",
    ".track:hover{border-color:var(--accent);transform:translateY(-2px)}.track:focus-within{border-color:var(--accent)}",
    'a.tlink{display:inline-block;min-height:24px;color:inherit;text-decoration:none}a.tlink::after{content:"";position:absolute;inset:0;border-radius:14px}a.tlink:focus-visible{outline:none}.track:has(a.tlink:focus-visible){outline:2px solid var(--accent);outline-offset:2px}',
    ".shelf{display:flex;flex-wrap:wrap;align-items:center;gap:4px 12px;font-size:13px;color:var(--muted)}.shelf .sh{display:inline-flex;align-items:center;gap:3px;white-space:nowrap}.shelf .sh.own{white-space:normal;max-width:100%}.shelf .sh b{color:var(--fg);font-size:14px}.tnext{font-size:13px;color:var(--muted)}.tnext b{color:var(--fg)}.tnext.good{color:var(--pass)}",
    ".track .medals a,.track .nextmedal a,.track .tnext a{position:relative;z-index:1}.track .medals>li{border-radius:10px}.medals a{color:inherit;text-decoration:none}.medals a:hover span:first-of-type{text-decoration:underline}.medals .lw{color:var(--muted)}",
    ".basis.tip:hover::after,.basis.tip:focus::after,abbr.gloss:hover::after,abbr.gloss:focus::after{content:none}",
    ".ftip{position:fixed;z-index:50;width:max-content;max-width:min(280px,calc(100vw - 24px));padding:6px 9px;border-radius:8px;background:var(--fg);color:var(--bg);font-size:13px;font-weight:450;line-height:1.4;box-shadow:0 4px 14px rgba(0,0,0,.18)}.ftip[hidden]{display:none}",
    ".allmeasured .basis.measured:is(.mini,.ctl){border-color:transparent;background:none;font-weight:500}",
    ".medal-col{display:flex;gap:10px;align-items:center}.medal-col>svg{flex:0 0 30px}.mix{display:grid;gap:4px}.slots{position:relative;display:flex;gap:3px;align-items:flex-end;height:30px;padding-bottom:1px;border-bottom:1px solid var(--line)}.slot{position:relative;flex:1 1 0;max-width:30px;height:100%;border-radius:3px 3px 0 0;background:var(--tint);overflow:hidden}.slot::after{content:'';position:absolute;left:0;right:0;bottom:0;height:max(var(--h),3px);background:var(--c)}.slots .slot{cursor:pointer}.slots .slot.on{outline:2px solid var(--fg);outline-offset:1px}.slotcap{font-size:12px;color:var(--muted);margin-top:4px;min-height:18px;line-height:1.5}.slots[data-tip]:hover::after,.slots[data-tip]:focus::after{content:attr(data-tip);position:absolute;left:0;bottom:calc(100% + 6px);z-index:5;width:max-content;max-width:min(320px,calc(100vw - 48px));padding:6px 9px;border-radius:8px;background:var(--fg);color:var(--bg);font-size:12px;font-weight:500;line-height:1.5;white-space:pre-line;box-shadow:var(--shadow)}.slots .avg{position:absolute;left:0;right:0;bottom:var(--avg);border-top:1.5px dashed var(--fg);opacity:.7}.slot-key{display:inline-flex;align-items:center;gap:5px}.sc-done>summary{display:flex;flex-wrap:wrap;align-items:baseline;gap:2px 10px;padding:10px 8px 10px 28px;list-style:none;min-height:24px;cursor:pointer;border-bottom:1px solid var(--line);font-size:14px}.sc-done>summary:hover{background:var(--tint)}.sc-done>summary small{color:var(--muted);font-size:13px}.sc-done>summary::-webkit-details-marker{display:none}.sc-done>summary::before{content:'\\25B8';color:var(--muted);width:12px}.sc-done[open]>summary::before{content:'\\25BE'}.sc-done .topmark{color:var(--pass);font-weight:650}.slots.mini{display:inline-flex;width:30px;height:14px;gap:2px}.slots.mini .slot{border-radius:1px 1px 0 0}.mix small{font-size:12px;color:var(--muted)}.nextmedal{font-size:13px;padding:8px 10px;border-radius:8px;background:var(--tint)}.nextmedal b{color:var(--fg)}",
    ".track .th{display:flex;gap:14px;align-items:center}",
    ".track .tt{font-weight:700;font-size:16px;line-height:1.25}",
    ".track .td{font-size:13px;color:var(--muted)}",
    ".trackstrip{display:flex;flex-wrap:wrap;align-items:center;gap:8px 10px;margin-top:28px;padding-top:16px;border-top:1px solid var(--line)}",
    ".trackstrip a{display:inline-flex;align-items:center;gap:8px;min-height:36px;padding:4px 12px 4px 4px;border:1px solid var(--line);border-radius:999px;color:var(--fg);text-decoration:none;font-size:13px}",
    ".trackstrip a{transition:border-color .15s,transform .15s,box-shadow .15s}.trackstrip a:hover,.trackstrip a.far:hover{border-color:var(--accent);transform:translateY(-2px);box-shadow:0 4px 12px color-mix(in srgb,var(--fg) 10%,transparent)}",
    ".leads{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px}a.lead-tile{display:flex;flex-direction:column;gap:4px;padding:14px 16px;border:1px solid var(--line);border-left:4px solid var(--pass);border-radius:12px;color:var(--fg);text-decoration:none;background:var(--card);transition:transform .15s,border-color .15s}a.lead-tile:hover{transform:translateY(-2px);border-color:var(--accent);border-left-color:var(--pass)}.lead-x{font-size:28px;font-weight:750;line-height:1;color:var(--pass)}.lead-dir{font-size:12px;font-weight:650;text-transform:uppercase;letter-spacing:.06em;color:var(--muted)}.lead-what{font-weight:650;line-height:1.35}a.lead-tile{display:grid;grid-template-columns:auto 1fr;align-items:baseline;gap:4px 8px}.lead-tile>.lead-what,.lead-tile>.sub{grid-column:1/-1}",
    ".climb{margin-top:12px;padding:10px 12px;border:1px solid var(--line);border-radius:10px;background:var(--card)}.climb-head{display:flex;flex-wrap:wrap;justify-content:space-between;align-items:center;gap:4px 10px;font-weight:650}.climb-bar{height:8px;margin:8px 0;border-radius:4px;background:var(--bg);box-shadow:inset 0 0 0 1px var(--line);overflow:hidden}.climb-bar span{display:block;height:100%;border-radius:3px}.climb ul{list-style:none;margin:0;padding:0;display:grid;gap:4px;font-size:14px}.climb li{display:grid;grid-template-columns:auto 1fr;align-items:baseline;gap:4px}.climb .mark{display:inline-block;width:16px;flex:0 0 16px;text-align:center;color:var(--muted);font-weight:700}.climb li.stuck .mark{color:var(--unknown)}",
    ".climb-track{display:flex;align-items:center;gap:8px}.climb-track .climb-bar{flex:1}.climb-lock{display:inline-flex;align-items:center;gap:3px;font-size:12px;font-weight:650;color:var(--muted);white-space:nowrap}",
    ".trackstrip.lead{margin-top:4px;padding-top:0;border-top:0}",
    ".trackstrip.lead a{padding:6px 14px 6px 6px;border-radius:14px;background:var(--card)}",
    ".trackstrip small{display:block;font-size:13px;color:var(--muted)}",
    ".trackstrip a.far{border-color:var(--fail)}.trackstrip a.far small{color:var(--fail);font-weight:650}",
    ".track .foot{display:flex;flex-wrap:wrap;justify-content:space-between;gap:4px 10px;font-size:12px;color:var(--muted);margin-top:auto}",
    ".medals{display:flex;flex-wrap:wrap;gap:4px;margin:0;padding:0;list-style:none}",
    ".medals>li{display:inline-flex;align-items:center;gap:4px;padding:2px 8px 2px 3px;border-radius:999px;background:var(--tint);font-size:12px;line-height:1.3}",
    ".sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}",
    ".medals>li.more{padding:2px 8px;color:var(--muted)}",
    ".flag{position:absolute;top:-10px;right:14px;font-size:12px;font-weight:700;padding:1px 9px;border-radius:999px;background:var(--fail);color:var(--card)}",
    ".chips{display:flex;flex-wrap:wrap;gap:4px}",
    ".chips .standing{font-size:12px}",
    ".rule-tag{display:inline-block;font-size:12px;line-height:1.5;border:1px dashed var(--line);border-radius:999px;padding:0 8px;color:var(--muted);margin:2px 0}",
    ".kgroup{margin-top:16px}",
    ".kgroup:first-of-type{margin-top:4px}",
    ".kgh{display:flex;flex-wrap:wrap;gap:4px 12px;align-items:baseline}",
    ".kgh h3{margin:0;color:var(--fg);font-size:13px}",
    ".kgd{margin:2px 0 8px;max-width:80ch}",
    "a.kind small{display:block;color:var(--muted);font-size:13px;line-height:1.35;margin:-2px 0 6px}a.kind small.kmix{margin:5px 0 0}a.kind small.kwhy{font-style:italic}",
    ".boxrow{display:grid;grid-template-columns:200px minmax(0,1fr);gap:12px;align-items:start;padding:10px 0;border-top:1px solid var(--line)}",
    ".boxrow:first-of-type{border-top:0}",
    ".boxrow .bg{font-weight:650;font-size:14px;line-height:1.3}",
    ".boxrow .bg small{display:block;font-weight:400;color:var(--muted);font-size:13px}",
    ".cmp-hero{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:20px;align-items:center;background:linear-gradient(135deg,color-mix(in srgb,var(--st) 14%,var(--card)) 0%,var(--card) 60%)}",
    ".cmp-hero .big{font-size:28px;line-height:1.2;font-weight:750;letter-spacing:-.01em;margin:4px 0 8px}",
    ".cmp-hero .kicker{font-size:12px;font-weight:650;text-transform:uppercase;letter-spacing:.08em;color:var(--muted)}",
    ".cmp-lead{margin:8px 0 0;font-size:15px}.cmp-lead .lead-x{font-size:inherit;line-height:inherit}.gaps .lead-x{color:var(--fail)}.gaps a.lead-tile,.gaps a.lead-tile:hover{border-left-color:var(--fail)}",
    ".nowrap{white-space:nowrap}.parity-fold{margin-top:8px;border-top:1px solid var(--line);padding-top:8px}.parity-fold>summary{cursor:pointer;color:var(--accent);font-size:14px;min-height:24px;display:flex;align-items:center;text-decoration:underline;text-underline-offset:2px}",
    ".method-lede{max-width:80ch;margin:0 0 12px;padding-left:20px;font-size:15px;line-height:1.55}.method-lede li+li{margin-top:4px}",
    ".cmp-side{display:grid;gap:12px;justify-items:center}.h2h{display:grid;gap:2px;width:300px;font-size:12px}.h2h-head,.h2h-row{display:grid;grid-template-columns:110px 1fr 1fr;align-items:center;gap:0}.h2h-head{color:var(--muted);font-weight:650}.h2h-head .behind{text-align:right;padding-right:6px;color:var(--fail)}.h2h-head .ahead{padding-left:6px;color:var(--pass)}.h2h-row{min-height:24px;color:var(--fg);text-decoration:none;border-radius:6px;padding:0 2px}.h2h-row:hover,.h2h-row:focus-visible{background:color-mix(in srgb,var(--accent) 10%,transparent)}.h2h-name{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;padding-right:6px}.h2h-l,.h2h-r{display:flex;height:10px}.h2h-l{justify-content:flex-end;border-right:1.5px solid var(--muted)}.h2h-l span{background:var(--fail);border-radius:3px 0 0 3px}.h2h-r span{background:var(--pass);border-radius:0 3px 3px 0}.h2h-foot{color:var(--muted);font-size:12px;text-align:center;margin-top:2px}",
    ".tally3{display:flex;gap:18px;text-align:center}.rule-tally{margin:6px 0 0;font-size:14px}.rule-tally .ahead{color:var(--pass)}.rule-tally .behind{color:var(--fail)}.rule-tally .parity{color:var(--muted)}",
    ".tally3 strong{display:block;font-size:28px;line-height:1.1;font-variant-numeric:tabular-nums}",
    ".tally3 span{font-size:12px;color:var(--muted);font-weight:650;text-transform:uppercase;letter-spacing:.05em}.tally3 .mixed{display:block;font-size:12px;color:var(--muted)}.tally3>div{display:flex;flex-direction:column}",
    ".setup{display:grid;grid-template-columns:repeat(auto-fit,minmax(210px,1fr));gap:10px;margin:10px 0;align-items:start}",
    ".setup>div{padding:10px 12px;border-radius:10px;background:var(--tint);font-size:14px}",
    ".setup b{display:block;font-size:12px;color:var(--muted);font-weight:650;text-transform:uppercase;letter-spacing:.05em;margin-bottom:2px}",
    ".setup small{display:block;color:var(--muted);font-size:13px;margin-top:4px;line-height:1.4}",
    ".factors{display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));gap:12px}",
    "a.factor{display:flex;flex-direction:column;gap:8px;padding:14px;border:1px solid var(--line);border-left:5px solid var(--st);border-radius:12px;background:var(--card);color:var(--fg);text-decoration:none;transition:transform .15s,border-color .15s}",
    "a.factor:hover{border-color:var(--accent);border-left-color:var(--st);transform:translateY(-1px)}",
    "a.factor[aria-current=true]{box-shadow:0 0 0 1px var(--accent)}",
    ".factor .fh{display:flex;justify-content:space-between;gap:8px;align-items:center;font-weight:700}",
    ".factor p{margin:0;font-size:13px;color:var(--muted)}.fx{display:flex;flex-direction:column;gap:1px;line-height:1.2}.fx b{font-size:24px;font-weight:750;color:var(--st);font-variant-numeric:tabular-nums}.fx span{font-size:13px;color:var(--muted)}.fx b.unmeasured{font-size:15px;font-weight:600;color:var(--muted)}.fx b.mixed{font-size:18px;color:var(--fg)}",
    ".factor .fc{font-size:13px;color:var(--muted)}.fbar{display:flex;align-items:center;gap:10px}.fbar>.bar{flex:1 1 auto;min-width:0}.fbar>.fc{flex:none}.cmp-method{font-size:14px;line-height:1.5}.jump{margin:-6px 0 14px;font-size:14px}.bh-fix{margin-top:4px}",
    ".measure{display:grid;grid-template-columns:minmax(0,5fr) minmax(0,6fr) minmax(130px,auto);gap:10px 18px;align-items:center;padding:14px 4px;border-top:1px solid var(--line)}",
    ".measure .mt{font-weight:650;line-height:1.35}",
    ".measure .mt small{display:flex;flex-wrap:wrap;gap:4px 8px;align-items:center;font-weight:400;color:var(--muted);font-size:12px;margin-top:4px}",
    ".measure .ms{text-align:right}",
    ".measure .full{grid-column:1/-1;margin-top:-4px}",
    ".measure details summary{cursor:pointer;font-size:13px;color:var(--muted);padding:4px 0;min-height:24px}",
    ".vals{display:flex;flex-direction:column;gap:6px}.vals.yn{flex-direction:row;flex-wrap:wrap;gap:4px 28px}.vals.yn .val{display:inline-flex;gap:8px}",
    ".val{display:grid;grid-template-columns:92px minmax(0,1fr) auto;gap:8px;align-items:center;font-size:13px}",
    ".val .who{color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}",
    ".val .vb{height:12px;border-radius:6px;background:var(--tint);overflow:hidden}",
    ".val .vb span{display:block;height:100%;border-radius:6px;width:0;transition:width .7s cubic-bezier(.2,.7,.2,1)}",
    ".val .v{font-variant-numeric:tabular-nums;font-weight:650;white-space:nowrap}",
    ".val .v.pass{color:var(--pass)}.val .v.fail{color:var(--fail)}",
    ".mag{display:block;font-size:13px;color:var(--muted);margin-top:4px}",
    ".behind-row{display:grid;grid-template-columns:minmax(0,2fr) minmax(0,3fr);gap:6px 18px;padding:12px 4px;border-top:1px solid var(--line);font-size:14px}",
    ".behind-row .why{margin:0 0 6px;color:var(--muted)}.behind-row .bh-val{margin-top:6px}.behind-row .why .refs{display:inline;margin:0;line-height:inherit}.behind-row .why .refs a{min-height:0;padding:3px 0}.behind-row .why b{color:var(--fg);font-weight:600}",
    ".provisional{color:var(--unknown);font-weight:650}",
    "::slotted(*){display:block;max-width:1120px;margin:0 auto;padding:12px 20px 0;box-sizing:border-box}",
    "@media (max-width:820px){.hero{grid-template-columns:1fr}.split{grid-template-columns:minmax(0,1fr)}.split>section:first-child{display:none}.picker{display:flex;flex-direction:column;gap:4px;margin:0 0 16px}.kchips-wrap{display:none}}",
    ".ering{display:flex;flex-direction:column;align-items:center;gap:4px;flex:none}",
    ".elevel{display:flex;align-items:center;gap:12px;flex:0 1 auto;min-width:0;max-width:100%}",
    ".elevel>div{min-width:0;overflow-wrap:anywhere}",
    ".elevel strong{display:block;font-size:28px;line-height:1.1;font-weight:750;text-transform:capitalize;letter-spacing:-.01em}",
    ".elevel .sub{display:block}.elevel strong .pv-head{font-size:18px;font-weight:600;text-transform:none;letter-spacing:0;color:var(--unknown)}",
    "@media (max-width:600px){main{padding:18px 12px 56px}.hero .levelword{font-size:28px}",
    ".modes{display:grid;grid-template-columns:repeat(var(--tabcols,3),minmax(0,1fr));gap:0;overflow:visible}.modes.more,.modes.less{-webkit-mask-image:none;mask-image:none}.modes button{padding:9px 4px;text-align:center}.modes .count{min-width:0;margin-left:4px;padding:0 5px;font-size:12px}",
    ".kinds{grid-template-columns:repeat(2,minmax(0,1fr));gap:8px}a.kind{padding:9px}.fail-row{grid-template-columns:minmax(0,1fr) 64px}.fail-row .bar{display:none}",
    "a.row,.rows-head{grid-template-columns:minmax(0,1fr) 64px}.row .dist,.rows-head .dist{display:none}",
    ".ladder{grid-template-columns:1fr 1fr}.rung{min-height:0}.rung.vacant{grid-column:1/-1;display:flex;align-items:center;gap:12px;padding:6px 10px}.rung.vacant .rh{margin:0}.rung.vacant .empty{padding:0;margin:0}",
    "details.ent>summary{grid-template-columns:76px minmax(0,1fr)}details.ent>summary .tally{grid-column:2;text-align:left}",
    ".ent-body{padding-left:6px}.gap{grid-template-columns:minmax(0,1fr)}.gap .sub{overflow-wrap:anywhere}.hide-sm{display:none}",
    ".kchips{grid-template-columns:1fr}.boxrow{grid-template-columns:1fr;gap:6px}.measure{grid-template-columns:1fr}.measure .ms{text-align:left}.cmp-hero{grid-template-columns:1fr;gap:10px}.cmp-main{display:contents}.cmp-side{order:1}.cmp-hero .cmp-lead,.cmp-hero .rule-tally,.cmp-hero .cmp-method{order:2}.cmp-hero .cmp-prov{order:3}.cmp-hero .cmp-def{order:4}.h2h{display:none}",
    '.ehdr{display:none}a.erow.cols,.egroup>summary.erow.cols{grid-template-columns:1fr!important;gap:4px}.ecell{display:flex;gap:6px;align-items:center}.ecell::before{content:attr(data-sc);color:var(--muted)}.ecell[data-judged]::before{content:attr(data-sc) " \\2726"}',
    ".behind-row{grid-template-columns:1fr}.val{grid-template-columns:76px minmax(0,1fr) auto}.pillwrap .unassessed-tag.quiet{display:inline}}",
    ".skip{position:absolute;left:8px;top:-48px;z-index:10;padding:8px 12px;border:0;border-radius:8px;background:var(--fg);color:var(--bg);font:inherit;font-weight:650;cursor:pointer}.skip:focus{top:8px}",
    "@media (min-width:601px){.modes{position:sticky;top:0;z-index:6;background:var(--bg);margin-top:8px;padding-top:12px}.split>section:first-child{top:72px}h2,h3,section,details,.filters,[id]{scroll-margin-top:76px}}",
    "@media (prefers-reduced-motion:reduce){*{transition:none!important;animation:none!important}}"
  ].join("");
  var template = document.createElement("template");
  template.innerHTML = "<style>" + CSS + '</style><slot name="header"></slot><slot name="nav"></slot><main><button type="button" class="skip" id="skip">Skip to content</button><div class="top" id="top"></div><div class="meta" id="meta"></div><div class="modes" id="modes" role="tablist" aria-label="Report views"></div><div id="body"></div></main><slot name="footer"></slot>';

  // src/lib/compare.js
  var TIMED_UNITS = ["ms", "s", "rps", "seconds", "minutes", "req/s"];
  function setupBrief(context, ids, facts, who) {
    var show = function(v) {
      return typeof v === "number" ? num(v) : typeof v === "boolean" ? v ? "yes" : "no" : String(v);
    };
    return Object.keys(context).sort().map(function(f) {
      var vals = ids.map(function(id) {
        return (facts[id] || {})[f];
      });
      var same = vals.every(function(v) {
        return JSON.stringify(v) === JSON.stringify(vals[0]);
      });
      if (same) return humanize(f).toLowerCase() + " " + show(vals[0]);
      return humanize(f).toLowerCase() + " " + ids.map(function(id, i) {
        return vals[i] == null ? null : who(id) + " " + show(vals[i]);
      }).filter(Boolean).join(" vs ");
    }).filter(Boolean);
  }
  function humanize(s) {
    var t = String(s || "").replace(/([a-z0-9])([A-Z])/g, "$1 $2").replace(/[-_]+/g, " ").toLowerCase();
    return t.charAt(0).toUpperCase() + t.slice(1);
  }
  function num(v) {
    return Number(v).toLocaleString("en-US", { maximumFractionDigits: 2 });
  }
  function measureLabel(c, r) {
    var fs = (c.reads || {})[r] || [];
    var d = fs.length === 1 ? (c.fields || {})[fs[0]] : null;
    return d ? d.replace(/\.$/, "") : humanize(r);
  }
  function ruleValue(c, r, id) {
    var s = (c.scores || {})[id];
    var v = s && s.values ? s.values[r] : void 0;
    if (v != null) return v;
    var fs = (c.reads || {})[r] || [];
    return fs.length === 1 ? ((c.facts || {})[id] || {})[fs[0]] : void 0;
  }
  function valueLine(c, r, rd, ids, who) {
    var vals = ids.map(function(id) {
      return ruleValue(c, r, id);
    });
    var numeric = vals.every(function(v) {
      return v == null || typeof v === "number";
    });
    var peer = (c.peers || [])[0];
    var st = (c.rules[r] || {})[peer];
    var leader = st === "ahead" ? c.subject : st === "behind" ? peer : null;
    var max = Math.max.apply(
      null,
      vals.filter(function(v) {
        return typeof v === "number";
      }).concat([0])
    );
    var yesNo = vals.every(function(v) {
      return typeof v === "boolean";
    });
    return el(
      "div",
      { className: "vals" + (yesNo ? " yn" : "") },
      ...ids.map(function(id, i) {
        var outcome = (((c.scores || {})[id] || {}).rules || {})[r] || "unknown";
        var v = vals[i];
        var color = id === leader ? "var(--pass)" : st === "parity" ? "var(--muted)" : "var(--faint)";
        var text = v == null ? outcome === "pass" ? "✓ meets it" : outcome === "fail" ? "✗ misses it" : "not measured" : typeof v === "boolean" ? v ? "✓ yes" : "✗ no" : typeof v === "number" ? num(v) + (rd.unit ? " " + rd.unit : "") : String(v);
        var fillEl = null;
        if (numeric && typeof v === "number" && max > 0) {
          fillEl = el("span", { style: "background:" + color });
          var w = Math.max(2, v / max * 100);
          requestAnimationFrame(function() {
            requestAnimationFrame(function() {
              fillEl.style.width = w + "%";
            });
          });
        }
        return el(
          "div",
          { className: "val" },
          el("span", { className: "who", textContent: who(id), title: who(id) }),
          fillEl ? el("span", { className: "vb" }, fillEl) : el("span"),
          // Colour marks the leader only; the rule's absolute bar, when it has one, is in the hover text.
          el("span", { className: "v " + (id === leader ? "pass" : ""), textContent: text, title: outcome === "pass" ? "Meets the rule's bar" : outcome === "fail" ? "Misses the rule's bar" : "" })
        );
      })
    );
  }
  var COUNT_UNITS = ["files", "lines", "packages", "steps", "vulnerabilities", "contributors", "bytes"];
  var SPAN_UNITS = ["ms", "s", "seconds", "minutes"];
  var SIZE_UNITS = ["MB", "KB", "GB"];
  function cmpWord(rd, lower) {
    var u = rd && rd.unit || "";
    if (COUNT_UNITS.indexOf(u) >= 0) return lower ? "fewer" : "more";
    if (SPAN_UNITS.indexOf(u) >= 0) return lower ? "faster" : "slower";
    if (SIZE_UNITS.indexOf(u) >= 0) return lower ? "smaller" : "larger";
    return lower ? "lower" : "higher";
  }
  function magnitude(c, r, rd, who) {
    var peer = (c.peers || [])[0];
    var a = ruleValue(c, r, c.subject);
    var b = ruleValue(c, r, peer);
    if (typeof a !== "number" || typeof b !== "number") return null;
    if (a === b) return el("span", { className: "mag", textContent: "same value" });
    if (a <= 0 || b <= 0) return null;
    var ratio = Math.max(a, b) / Math.min(a, b);
    var x = ratio >= 10 ? Math.round(ratio).toLocaleString("en-US") : ratio.toFixed(1);
    var lead = rd.better === "lower" ? a < b ? c.subject : peer : rd.better === "higher" ? a > b ? c.subject : peer : c.subject;
    var la = lead === c.subject ? a : b;
    var lb = lead === c.subject ? b : a;
    return el("span", { className: "mag", textContent: who(lead) + ": " + x + "× " + cmpWord(rd, la < lb) });
  }
  function gapText(D, g, id) {
    var m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
    if (m) {
      var sc = (D.scorecards || {})[m[1]];
      var score = ((D.scores || {})[m[1]] || {})[id];
      var need = sc && score ? blockers(sc, score, m[2]) : [];
      return (need.length ? "Needs " + need.map(function(r) {
        return "“" + ruleTitle(m[1], r) + "”";
      }).join(", ") + " for " + m[2] : m[2].charAt(0).toUpperCase() + m[2].slice(1) + " is the goal") + " on " + title(D, m[1]) + "; " + m[3] + " now.";
    }
    m = /^(.+?):rule (.+) \((\S+)\)$/.exec(g);
    if (m) return title(D, m[1]) + ": " + ({ fail: "fails", pass: "passes", unknown: "has not run" }[m[3]] || "is " + m[3]) + " “" + ruleTitle(m[1], m[2]) + "”.";
    m = /^(.+?): unknown scorecard$/.exec(g);
    if (m) return "Scorecard " + m[1] + " is unknown.";
    return g;
  }
  function judgedMark(D, name, iconOnly) {
    var n = basisCounts(((D.scorecards || {})[name] || {}).rules);
    if (!n.judged && !n.measured) return null;
    var b = n.judged ? "judged" : "measured";
    var text = n.judged ? (n.measured ? n.judged + " of " + (n.judged + n.measured) + " rules are" : "Every rule is") + " agent-judged: " + BASIS.judged.desc : "Every rule is measured: " + BASIS.measured.desc;
    if (iconOnly) {
      var i = el("span", { className: "bicon " + b, title: text, role: "img" }, basisIcon(b));
      i.setAttribute("aria-label", n.judged ? "has agent-judged rules" : "every rule measured");
      return i;
    }
    var m = basisBadge(b, true);
    m.title = text;
    return m;
  }
  function keepWithLast(text, node) {
    if (!node) return [text];
    var words = text.split(" ");
    var last = words.pop();
    return [words.length ? words.join(" ") + " " : "", el("span", { className: "nowrap" }, last + " ", node)];
  }
  function cardName(D, name) {
    var t = title(D, name);
    var mark = judgedMark(D, name);
    var head = el("span", { className: "t" }, t, mark ? " " : null, mark);
    return t === name ? [head] : [head, el("code", { className: "c", textContent: name })];
  }

  // src/views/overview.js
  var overviewViews = {
    hero(D, S, campaignNames) {
      var R = S.repo;
      var verdict = healthVerdict(D, S, campaignNames);
      if (noFacts(S)) {
        var start = el(
          "section",
          { className: "hero first-run" },
          el(
            "div",
            {},
            el("div", { className: "kicker", textContent: "First run" }),
            el("h2", { className: "levelword", textContent: "Not scored yet" }),
            el("p", { className: "verdict", textContent: "No connector has delivered facts, so no entity has a level." }),
            el(
              "div",
              { className: "next" },
              el("b", { textContent: "Next: " }),
              "run ",
              el("code", { textContent: "toolbox report" }),
              " in the repository. It runs the connectors listed under ",
              el("code", { textContent: "connectors:" }),
              " in the portal's ",
              el("code", { textContent: "toolbox.yaml" }),
              ", and ",
              el("code", { textContent: "toolbox doctor" }),
              " lists any connector or tool that is missing. The " + plural(S.rows.length, "scorecard") + " below will fill in."
            )
          )
        );
        start.setAttribute("aria-label", "Status");
        return start;
      }
      if (!R) {
        var at = verdict.indexOf(": ");
        var needs = at > 0 && verdict.indexOf("Needs") === 0;
        var almost = at > 0 && verdict.indexOf("Almost") === 0;
        var tone = needs ? "none" : almost ? "bronze" : "gold";
        var far = furthestTrack(D, S);
        var next = needs || almost ? nextMedalEl(D, trackStats(D, S, far || void 0).cards, "Next: ", far ? " It comes first because " + trackTitle(D, far) + " is the track furthest from its goal." : "") : null;
        var plain = el("section", { className: "hero plain", style: "--lvl:" + fill(tone) + ";--lvl-ink:" + ink(tone) }, el("div", {}, el("div", { className: "kicker", textContent: "Status" }), el("h2", { className: "levelword", textContent: at > 0 ? verdict.slice(0, at) : verdict }), at > 0 ? el("p", { className: "verdict", textContent: verdict.slice(at + 2).replace(/^./, function(ch) {
          return ch.toUpperCase();
        }) }) : null, next, S.rows.length ? el("p", { className: "sub", style: "margin:8px 0 0", textContent: "No overall level: this portal does not combine its scorecards into one level for the repository, so each scorecard stands on its own." }) : null));
        plain.setAttribute("aria-label", "Status");
        return plain;
      }
      var level = R.floor;
      var section = el("section", { className: "hero", style: "--lvl:" + fill(level) + ";--lvl-ink:" + ink(level) });
      section.setAttribute("aria-label", "Status");
      var holdingNames = R.holding.map(function(c) {
        return title(D, c.name);
      });
      var rated = " The repository is rated by the weakest of its " + plural(R.constituents.length, "standard") + ".";
      var sentence = R.next ? "Held at " + level + " by " + plural(R.holding.length, "standard") + ", listed below." + rated : "Every one of its " + plural(R.constituents.length, "standard") + " is at the top level.";
      var wins = R.ladder.slice().reverse().filter(function(l) {
        return l !== "none" && l !== level;
      }).map(function(l) {
        var n = R.constituents.filter(function(c) {
          return c.level === l;
        }).length;
        return n ? plural(n, "standard") + " at " + l : null;
      }).filter(Boolean);
      var capped = R.holding.filter(function(c) {
        return c.level === c.top;
      });
      var liftNames = R.holding.filter(function(c) {
        return c.level !== c.top;
      }).map(function(c) {
        return title(D, c.name);
      });
      if (R.next && capped.length)
        sentence = "Held at " + level + " by " + capped.map(function(c) {
          return title(D, c.name);
        }).join(", ") + (capped.length === 1 ? ", which has" : ", which have") + " no " + R.next + " level yet." + (liftNames.length ? " After that, " + plural(liftNames.length, "more standard needs", "more standards need") + " " + R.next + "." : "") + rated;
      var nextBox = !R.next ? el("div", { className: "next" }, el("b", { textContent: "Top of the ladder. " }), "Every repository standard is at its highest level.") : capped.length ? null : el("div", { className: "next" }, el("b", { textContent: "Next: " }), "reach ", badge(R.next), " by lifting " + plural(holdingNames.length, "standard") + " to " + R.next + ", listed below.");
      var climb = null;
      if (R.next) {
        var rank = function(l) {
          return R.ladder.indexOf(l);
        };
        var there = R.constituents.filter(function(c) {
          return rank(c.level) >= rank(R.next);
        }).length;
        var share2 = share(there, R.constituents.length);
        var bar = el("div", { className: "climb-bar" }, el("span", { style: "width:" + share2 + "%;background:" + ink(R.next) }));
        bar.setAttribute("role", "img");
        var blockedBy = capped.length ? capped.map(function(c) {
          return title(D, c.name);
        }).join(", ") : "";
        bar.setAttribute("aria-label", there + " of " + R.constituents.length + " repository standards at " + R.next + " or above" + (blockedBy ? "; blocked until " + blockedBy + " gains a " + R.next + " level" : ""));
        climb = el(
          "div",
          { className: "climb" + (blockedBy ? " blocked" : "") },
          el("div", { className: "climb-head" }, el("span", {}, "Toward ", badge(R.next)), el("span", { className: "sub", textContent: there + " of " + R.constituents.length + " standards there (" + share2 + "%)" })),
          blockedBy ? el("div", { className: "climb-track" }, bar, el("span", { className: "climb-lock", title: "No " + R.next + " level exists yet on " + blockedBy }, lockIcon(), " blocked by a missing level")) : bar,
          el(
            "ul",
            {},
            ...R.holding.map(function(c) {
              var stuck = c.level === c.top;
              return el("li", { className: stuck ? "stuck" : "" }, el("span", { className: "mark", ariaHidden: "true", textContent: stuck ? "⊘" : "↑" }), el("span", {}, link("scorecard/" + enc(c.name), "", title(D, c.name)), stuck ? stuckNote(D, c.name, R.next) : el("span", { className: "sub", textContent: " lift from " + c.level + " to " + R.next })));
            })
          )
        );
      }
      var left = el(
        "div",
        {},
        el(
          "div",
          { className: "badge-big" },
          medal(level, 72),
          el("div", {}, el("div", { className: "kicker", textContent: "Repository level" }), el("h2", { className: "levelword", textContent: level }))
        ),
        el("p", { className: "verdict", textContent: sentence }),
        verdict.indexOf(": ") > 0 ? el(
          "p",
          { className: "attn" },
          el("b", { textContent: verdict.slice(0, verdict.indexOf(": ") + 1) + " " }),
          verdict.slice(verdict.indexOf(": ") + 2) + " ",
          el("button", {
            className: "link",
            textContent: "See them ↓",
            onclick: function() {
              var h = this.getRootNode().querySelector("#need-att");
              if (h) h.scrollIntoView({ behavior: "smooth", block: "start" });
            }
          })
        ) : el("p", { className: "attn", textContent: verdict }),
        wins.length ? el("p", { className: "wins" }, el("b", { textContent: "✓ Already: " }), wins.join(" · ")) : null,
        nextBox,
        climb
      );
      var ladder = el("div", { className: "ladder", style: "--steps:" + R.ladder.length });
      ladder.setAttribute("aria-label", "Repository standards by lowest level");
      R.ladder.forEach(function(l) {
        var here = R.constituents.filter(function(c) {
          return c.level === l;
        });
        var rung = el(
          "div",
          { className: "rung" + (l === level ? " here" : "") + (here.length ? "" : " vacant"), style: "--lvl:" + fill(l) },
          el("div", { className: "rh" }, medal(l, 22), l, el("span", { className: "n", textContent: String(here.length) }))
        );
        if (!here.length) rung.append(l === "none" ? el("div", { className: "empty good", textContent: "✓ No repository standard at none" }) : el("div", { className: "empty", textContent: "—" }));
        here.forEach(function(c) {
          var done = c.level === c.top;
          var caps = done && R.next && c.level === level;
          var a = link("scorecard/" + enc(c.name), caps ? "capped" : done ? "done" : "", [title(D, c.name), caps ? el("span", { className: "capmark" }, lockIcon(), " at its top level, so it caps the repository") : done ? el("span", { className: "topmark", textContent: "✓ top level" }) : null]);
          a.title = caps ? c.name + ": at its only top level, " + c.level + ", which holds the repository there" : done ? c.name + ": already at its top level, " + c.level : c.name;
          rung.append(a);
        });
        ladder.append(rung);
      });
      section.append(
        left,
        el(
          "div",
          {},
          el("h3", { style: "margin-top:0", textContent: "Where each repository standard stands" }),
          ladder,
          el(
            "p",
            { className: "sub", style: "margin:10px 0 0" },
            "Only these standards set the repository level. The other scorecards rate single entities, such as one rule or one skill, and are listed below." + (R.ceiling !== R.ladder[R.ladder.length - 1] && R.ceiling !== level ? " " + plural(
              R.constituents.filter(function(c) {
                return c.top === R.ceiling;
              }).length,
              "standard tops",
              "standards top"
            ) + " out at " + R.ceiling + ", so " + R.ceiling + " is the highest level the repository can reach." : "")
          )
        )
      );
      return section;
    },
    tiles(D, S, campaignNames) {
      var campaigns = D.campaigns || {};
      var c = S.checks;
      var total = c.pass + c.vacuous + c.fail + c.unknown;
      var pct = share(c.pass, total);
      var checkNote = [c.vacuous ? fmt(c.vacuous) + " more pass by default, because no agent has reviewed them yet" : null, c.unknown ? fmt(c.unknown) + " not run" : null].filter(Boolean);
      var on = campaignNames.filter(function(n) {
        return campaignStanding(D, campaigns[n]) === "on";
      }).length;
      var debt = S.rows.filter(function(r) {
        return r.none > 0 && !r.compare;
      }).length;
      var scoredEntities = Object.keys(S.worst).length;
      var tops = new Set(
        S.rows.filter(function(r) {
          return !r.compare;
        }).map(function(r) {
          return known((r.sc.levels || []).slice(-1)[0]) || "gold";
        })
      );
      var topList = LADDER.filter(function(l) {
        return tops.has(l);
      });
      var topMedal = topList[topList.length - 1] || "gold";
      var goldIds = S.atTop;
      var golden = goldIds.length;
      var unjudged = /* @__PURE__ */ new Set();
      Object.keys(D.unassessed || {}).forEach(function(card) {
        var rows = (D.scores || {})[card] || {};
        D.unassessed[card].forEach(function(id) {
          if (rows[id] && rows[id].level !== "skipped") unjudged.add(id);
        });
      });
      var goldUnjudged = goldIds.filter(function(id) {
        return unjudged.has(id);
      }).length;
      var tile = function(path, cls, lead, big, text) {
        return link(path, "tile " + (cls || ""), [lead, el("div", {}, el("strong", { textContent: big }), el("span", {}, ...[].concat(text)))]);
      };
      var shown = [
        total ? tile("scorecards", "good", kpiBar(pct, pct + "% of checks pass"), pct + "%", [fmt(c.pass) + " of " + fmt(total) + " checks pass on evidence", checkNote.length ? el("small", { className: "tile-note", textContent: checkNote.join("; ") }) : null]) : null,
        scoredEntities ? tile("catalog", "", medal(topMedal, 40), fmt(golden), ["of " + fmt(scoredEntities) + " scored entities are at the top level of every scorecard that rates them", topList.length > 1 ? el("small", { className: "tile-note", textContent: "Top levels differ by scorecard: " + andList(topList) + "." }) : null, goldUnjudged ? el("small", { className: "tile-note", textContent: fmt(goldUnjudged) + " of those still await an agent's assessment" }) : null]) : null,
        scoredEntities ? tile("scorecards", debt ? "bad" : "good", medal(debt ? "none" : "gold", 40), String(debt), debt === 1 ? "scorecard has entities at none" : debt ? "scorecards have entities at none" : "scorecards with entities at none") : null,
        campaignNames.length ? tile("campaigns", on === campaignNames.length ? "good" : "", kpiPips(on, campaignNames.length, on + " of " + campaignNames.length + " campaigns on target"), on + " of " + campaignNames.length, "campaigns on target") : null
      ].filter(Boolean);
      return shown.length ? el("div", { className: "tiles" }, ...shown) : null;
    },
    campaignCard(D, name, current) {
      var c = (D.campaigns || {})[name];
      var state = campaignState(c);
      var pct = c.percent == null ? 0 : c.percent;
      var target = c.targetPercent == null ? 100 : c.targetPercent;
      var days = daysLeft(c.end);
      var each = includeProgress(D, c);
      var toGo = each.reduce(function(n, p) {
        return n + (p.met ? 0 : p.below);
      }, 0);
      var status = state === "empty" ? toGo ? el("span", { className: "pill bad", textContent: plural(toGo, "entity", "entities") + " to go" }) : el("span", { className: "pill good", textContent: "on target" }) : state === "on" ? el("span", { className: "pill good", textContent: "on target" }) : el("span", { className: "pill bad", textContent: plural((c.inScope || 0) - (c.meeting || 0), "entity", "entities") + " to go" });
      var a = link("campaign/" + enc(name), "card", [
        goalBar(state, pct, target, false, each),
        el(
          "div",
          {},
          el("div", { className: "t", textContent: c.title || name }),
          el("div", { className: "s" }, status, days != null && days >= 0 && c.status !== "ended" ? " · " + plural(days, "day") + " left" : c.status === "ended" ? " · ended" : "")
        )
      ]);
      if (current) a.setAttribute("aria-current", "true");
      return a;
    },
    scorecardTable(D, rows, floors) {
      var why = function(r, indent) {
        var p = scPurpose(r.sc);
        return p ? el("span", { className: "rp" + (indent ? " in" : ""), textContent: p }) : null;
      };
      var wrap = el("div", { className: "rows" });
      wrap.append(el("div", { className: "rows-head" }, el("span", { textContent: "Scorecard" }), el("span", { className: "dist", textContent: "Entities by level" }), el("span", { textContent: "At none" })));
      rows.forEach(function(r) {
        wrap.append(
          link("scorecard/" + enc(r.name), "row", [
            floors ? el("div", { className: "medal-col" }, medal(floorOf(D, r.name) || "not scored", 30), el("div", {}, ...cardName(D, r.name), el("span", { className: "weakest" }, "weakest ", floorOf(D, r.name) ? floorPill(D, r.name) : "not scored", topNote(D, r.name, floorOf(D, r.name))))) : el("div", {}, ...cardName(D, r.name)),
            el("div", { className: "dist" }, barEl(splitUnassessed(D, r.name, r.parts)), el("div", { className: "sub", style: "font-size:12px;margin-top:4px" }, ...scCaption(D, r, true))),
            el("div", { className: "num" + (r.none ? " bad" : "") }, el("strong", { textContent: r.scored ? String(r.none) : "—" }), el("span", { textContent: r.scored ? "at none" : "not scored" })),
            why(r, floors)
          ])
        );
      });
      return wrap;
    },
    overview(D, S) {
      var catalog = D.catalog || {};
      var campaignNames = campaignNamesSorted(D);
      var self = this;
      var attention = S.rows.filter(function(r) {
        return r.none > 0 && !r.compare;
      });
      var clear = S.rows.filter(function(r) {
        return r.none === 0 && !r.compare && r.scored;
      });
      var unscored = S.rows.filter(function(r) {
        return !r.compare && !r.scored;
      });
      var clearHollow = clear.filter(function(r) {
        return splitUnassessed(D, r.name, r.parts).some(function(p) {
          return p.hollow;
        });
      });
      var clearBelow = clear.filter(function(r) {
        return clearHollow.indexOf(r) < 0 && r.topShare < 100;
      });
      var clearLine = el(
        "span",
        {},
        "✓ " + plural(clear.length - clearHollow.length, "scorecard has", "scorecards have") + " no entity at none",
        clearHollow.length ? el("span", { className: "unjudged-n" }, "; " + fmt(clearHollow.length) + " more only because no agent has assessed some of their entities yet (", ...clearHollow.flatMap(function(r, i) {
          return [i ? ", " : "", link("scorecard/" + enc(r.name), "", title(D, r.name))];
        }), ")") : null,
        clearBelow.length ? el("span", {}, "; " + (clear.length - clearHollow.length === 1 ? "it still has entities below its top level" : clearBelow.length === 1 ? "one of them still has entities below its top level" : fmt(clearBelow.length) + " of them still have entities below their top level") + " (", ...clearBelow.flatMap(function(r, i) {
          return [i ? ", " : "", link("scorecard/" + enc(r.name), "", title(D, r.name))];
        }), ")") : null,
        ". "
      );
      var showAll = false;
      var clearWrap = el("div");
      var drawClear = function() {
        clearWrap.replaceChildren(
          showAll ? el(
            "div",
            {},
            el(
              "p",
              { className: "empty good" },
              clearLine,
              el("button", {
                className: "link",
                textContent: "Hide them",
                ariaExpanded: "true",
                onclick: function() {
                  showAll = false;
                  drawClear();
                  var b = clearWrap.querySelector("button");
                  if (b) b.focus();
                }
              })
            ),
            self.scorecardTable(D, clear)
          ) : el(
            "p",
            { className: "empty good" },
            clearLine,
            el("button", {
              className: "link",
              textContent: "Show them",
              ariaExpanded: "false",
              onclick: function() {
                showAll = true;
                drawClear();
                var b = clearWrap.querySelector("button");
                if (b) b.focus();
              }
            })
          )
        );
      };
      drawClear();
      var attFold = attention.length > 4 ? 3 : attention.length;
      var attTable = attention.length ? this.scorecardTable(D, attention.slice(0, attFold)) : null;
      var attMore = attention.length > attFold ? el(
        "div",
        { className: "more" },
        el("button", {
          className: "link",
          ariaExpanded: "false",
          textContent: "Show the other " + fmt(attention.length - attFold) + " that need attention",
          onclick: function() {
            var open = this.getAttribute("aria-expanded") !== "true";
            var shown = open ? self.scorecardTable(D, attention) : self.scorecardTable(D, attention.slice(0, attFold));
            attTable.replaceWith(shown);
            attTable = shown;
            this.setAttribute("aria-expanded", String(open));
            this.textContent = open ? "Show only the first " + attFold : "Show the other " + fmt(attention.length - attFold) + " that need attention";
            var next = open && shown.querySelectorAll("a.row")[attFold];
            if (next) next.focus();
          }
        })
      ) : null;
      return el(
        "div",
        {},
        this.hero(D, S, campaignNames),
        tracksSorted(D, S).length ? el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Tracks" }), link("tracks", "", "All tracks →")),
          el("p", { className: "lede", textContent: (furthestTrack(D, S) ? plural(tracksSorted(D, S).length, "long-running line") + " of work; " + trackTitle(D, furthestTrack(D, S)) + " is furthest from its goal. " : "") + "Each ring is its scorecards' average share of entities at the top level." + (tracksSorted(D, S).some(function(t) {
            return t.compare;
          }) ? " The comparison track's ring counts factors ahead of a rival instead." : "") + " Open a track for its scorecards and campaigns." }),
          this.trackStrip(D, S, tracksSorted(D, S), "Tracks", true)
        ) : null,
        this.tiles(D, S, campaignNames),
        campaignNames.length ? el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Campaigns" }), el("span", { className: "sub", textContent: "Goals with a target, behind target first; a bar turns green once it passes its target mark" })),
          el(
            "div",
            { className: "cards" },
            ...campaignNames.map(function(n) {
              return self.campaignCard(D, n);
            })
          )
        ) : null,
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { id: "need-att", textContent: attention.length ? "Scorecards that need attention" : "Scorecards" }),
            attention.length ? el("span", { className: "sub", textContent: "Most entities at none first" }) : null
          ),
          el("p", { className: "lede", textContent: "Each scorecard rates every entity of one kind against levels of rules. An entity is at none until it passes every rule of the first level." }),
          unscored.length === S.rows.length ? null : levelLegend(D, S.rows),
          attTable,
          attMore,
          clear.length ? clearWrap : null,
          unscored.length ? el("div", {}, el("p", { className: "empty", textContent: plural(unscored.length, "scorecard has", "scorecards have") + " no scored entity yet:" }), this.scorecardTable(D, unscored)) : null
        ),
        !S.entityCount ? null : el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "What's in the box" }), link("catalog", "", "Browse the catalog →")),
          ...kindGroupsOf(D).map(function(g) {
            return el(
              "div",
              { className: "boxrow" },
              el("div", { className: "bg" }, g.title, el("small", { textContent: plural(g.count, "entity", "entities") })),
              el(
                "div",
                { className: "box" },
                ...g.kinds.map(function(k) {
                  var a = link("catalog/" + enc(k), "", [kindTitle(D, k), el("b", { textContent: fmt(catalog[k].length) })]);
                  if (kindDescription(D, k)) a.title = kindDescription(D, k).replace(/`/g, "");
                  return a;
                })
              )
            );
          })
        )
      );
    }
  };

  // src/views/scorecards.js
  var scorecardsViews = {
    scorecardIndex(D, S, query) {
      var self = this;
      var standards = S.rows.filter(function(r) {
        return !r.compare;
      });
      var factors = S.rows.length - standards.length;
      var groups = tracksSorted(D, S).filter(function(t) {
        return !t.compare && t.cards.length;
      }).map(function(t) {
        return { name: t.name, title: t.def.title || t.name, rows: t.cards.map(function(c) {
          return c.row;
        }) };
      });
      var placed = {};
      groups.forEach(function(g) {
        g.rows.forEach(function(r) {
          placed[r.name] = true;
        });
      });
      var loose = standards.filter(function(r) {
        return !placed[r.name];
      });
      if (loose.length) groups.push({ name: null, title: "Not in a track", rows: loose });
      var search = el("input", { type: "search", placeholder: "Find a scorecard by name, id, kind, or how it's decided…", ariaLabel: "Find a scorecard", value: (query || {}).q || "" });
      var only = function(b) {
        return el(
          "button",
          {
            className: "link",
            onclick: function() {
              search.value = BASIS[b].label.toLowerCase();
              search.oninput();
            }
          },
          basisIcon(b),
          " " + BASIS[b].label
        );
      };
      var mixedBases = standards.some(function(r) {
        return basisCounts(r.sc.rules).judged;
      }) && standards.some(function(r) {
        return basisCounts(r.sc.rules).measured;
      });
      var anyScored = standards.some(function(r) {
        return r.scored > 0;
      });
      var basisPick = el("p", { className: "sub", style: "margin:6px 0 0" }, "Show only: ", only("judged"), " · ", only("measured"));
      var list = el("div");
      var row = function(r) {
        var floor = floorOf(D, r.name);
        var purpose = scPurpose(r.sc);
        return link("scorecard/" + enc(r.name), "sc-row", [
          medal(floor || "not scored", 26),
          el("div", { className: "sc-name" }, ...cardName(D, r.name)),
          el("div", { className: "sc-dist" }, barEl(splitUnassessed(D, r.name, r.parts)), el("small", {}, ...scCaption(D, r))),
          el("div", { className: "sc-weak" }, floor ? floorPill(D, r.name) : el("span", { className: "sub", textContent: "not scored" }), r.none ? el("small", { className: "bad", textContent: fmt(r.none) + " at none" }) : floor && capsRepo(D, r.name, floor) ? el("small", { className: "capnote" }, lockIcon(), " at its top level, so it caps the repository") : floor && isTop(D, r.name, floor) && !ONLY_LEVEL.has(floor) ? el("small", { className: "topmark", textContent: "✓ top level" }) : null),
          purpose ? el("span", { className: "sc-why", textContent: purpose }) : null
        ]);
      };
      var draw = function() {
        var q = search.value.toLowerCase();
        var hit = function(r) {
          var basisWords = basisCounts(r.sc.rules).judged ? "agent-judged judged by an agent assessed" : "measured deterministic mechanical";
          return !q || r.name.toLowerCase().indexOf(q) >= 0 || title(D, r.name).toLowerCase().indexOf(q) >= 0 || kindTitle(D, r.sc.kind || "").toLowerCase().indexOf(q) >= 0 || basisWords.indexOf(q) >= 0;
        };
        var shown = groups.map(function(g) {
          return { g, rows: g.rows.filter(hit) };
        }).filter(function(x) {
          return x.rows.length;
        });
        var want = "#scorecards" + (search.value ? "?q=" + encodeURIComponent(search.value) : "");
        if (location.hash !== want && history.replaceState) history.replaceState(history.state, "", want);
        if (!shown.length) {
          list.replaceChildren(
            el("div", { className: "empty" }, "No scorecards match “" + search.value + "”. ", el("button", {
              className: "link",
              textContent: "Clear search",
              onclick: function() {
                search.value = "";
                draw();
                search.focus();
              }
            }))
          );
          return;
        }
        var doneOf = function(r) {
          return !q && r.scored > 0 && r.topShare === 100 && !capsRepo(D, r.name, floorOf(D, r.name));
        };
        list.replaceChildren(
          ...shown.map(function(x) {
            var done = x.rows.filter(doneOf);
            if (done.length < 2) done = [];
            var open = x.rows.filter(function(r) {
              return done.indexOf(r) < 0;
            });
            var SHOWN = 4;
            var rest = !q && open.length > SHOWN + 1 ? open.slice(SHOWN) : [];
            if (rest.length) open = open.slice(0, SHOWN);
            var restRows = el("div", { className: "sc-rest", hidden: true });
            var restBtn = rest.length ? el("button", {
              className: "link sc-more",
              ariaExpanded: "false",
              textContent: "Show the other " + plural(rest.length, "scorecard") + " in " + x.g.title,
              onclick: function() {
                var opening = this.getAttribute("aria-expanded") !== "true";
                if (opening && !restRows.childElementCount) restRows.append(...rest.map(row));
                restRows.hidden = !opening;
                this.setAttribute("aria-expanded", String(opening));
                this.textContent = opening ? "Show only the weakest " + SHOWN : "Show the other " + plural(rest.length, "scorecard") + " in " + x.g.title;
                var first = opening && restRows.querySelector("a");
                if (first) first.focus();
              }
            }) : null;
            var byLevel = {};
            var topAt = {};
            var levelOrderSeen = [];
            x.rows.forEach(function(r) {
              var f = floorOf(D, r.name);
              if (!f) return;
              if (!byLevel[f]) levelOrderSeen.push(f);
              byLevel[f] = (byLevel[f] || 0) + 1;
              if (isTop(D, r.name, f) && !ONLY_LEVEL.has(f) && LADDER.indexOf(f) < LADDER.length - 1) topAt[f] = (topAt[f] || 0) + 1;
            });
            var mix = q ? "" : levelOrderSeen.sort(function(a, b) {
              var ra = LADDER.indexOf(a);
              var rb = LADDER.indexOf(b);
              return (ra < 0 ? 9 : ra) - (rb < 0 ? 9 : rb);
            }).map(function(l) {
              return byLevel[l] + " at " + levelName(l) + (topAt[l] ? " (" + topAt[l] + " at its top level)" : "");
            }).join(" · ");
            var fold = null;
            if (done.length) {
              var body = el("div", { className: "sc-done-rows" });
              fold = el(
                "details",
                {
                  className: "sc-done",
                  ontoggle: function() {
                    if (fold.open && !body.childElementCount) body.append(...done.map(row));
                  }
                },
                el("summary", {}, el("span", { className: "topmark", textContent: "✓ " + plural(done.length, "scorecard") + " with every entity at the top level" }), el("small", { textContent: done.map(function(r) {
                  return title(D, r.name);
                }).join(", ") })),
                body
              );
            }
            return el(
              "div",
              { className: "sc-group" },
              groups.length === 1 && !x.g.name ? null : el("h3", {}, x.g.name ? link(trackPath(D, x.g.name), "", x.g.title) : x.g.title, el("span", { className: "sub", textContent: " · " + plural(x.rows.length, "scorecard") + (x.g.name ? ", weakest first" : "") + (mix ? ": " + mix : "") })),
              el("div", { className: "sc-rows" }, el("div", { className: "sc-head", ariaHidden: "true" }, el("span"), el("span", { textContent: "Scorecard" }), el("span", { textContent: "Entities by level" }), el("span", { textContent: "Weakest entity" })), ...open.map(row), restRows, restBtn ? el("div", { className: "more" }, restBtn) : null, fold)
            );
          })
        );
      };
      search.oninput = draw;
      draw();
      return el(
        "section",
        { className: "sc-index" },
        el("div", { className: "head" }, el("h2", { textContent: "Scorecards" }), el("span", { className: "sub", textContent: plural(standards.length, "scorecard") + (groups.length > 1 ? " in " + plural(groups.length, "group") : "") })),
        el("p", { className: "lede" }, "Each scorecard rates every entity of one kind against levels of rules; the medal is its weakest entity's level, and the bar shows where the rest stand. " + (Object.keys(D.tracks || {}).length ? "They are grouped by track, the line of work each one serves. " : "") + "Open one for its failures, entities and what each level needs.", factors ? el("span", {}, " The " + plural(factors, "comparison factor") + " against a rival are on ", link("compare", "", "the Compare tab"), ".") : null),
        anyScored ? el("details", { className: "keyd", open: !matchMedia("(max-width:640px)").matches }, el("summary", { textContent: "How to read the bars" }), levelLegend(D, standards)) : null,
        search,
        mixedBases ? basisPick : null,
        list
      );
    },
    scorecard(D, S, name, query) {
      query = query || {};
      var ranked = S.rows;
      var active = name && D.scorecards[name] ? name : ranked[0] && ranked[0].name;
      if (!active) return el("section", {}, el("div", { className: "empty", textContent: "No scorecards in this report." }));
      var self = this;
      var sc = D.scorecards[active];
      var rows = Object.entries(D.scores[active] || {});
      var order = levelOrder(sc.levels);
      var offBy = function(r) {
        return Object.keys(r.rules || {}).reduce(function(n, rule) {
          var v = (r.measures || {})[rule];
          return n + (isFailing(r.rules[rule]) && typeof v === "number" ? v : 0);
        }, 0);
      };
      var parts = tallyLevels(rows, order);
      var cardSearch = el("input", { type: "search", placeholder: "Find a scorecard by name or id…", ariaLabel: "Find a scorecard" });
      var cardNav = el("nav", { className: "rank", ariaLabel: "Scorecards, most debt first" });
      var drawCards = function() {
        var q = cardSearch.value.toLowerCase();
        var shown = ranked.filter(function(r) {
          return !q || r.name.toLowerCase().indexOf(q) >= 0 || title(D, r.name).toLowerCase().indexOf(q) >= 0 || (r.sc.kind || "").toLowerCase().indexOf(q) >= 0;
        });
        if (!shown.length) {
          cardNav.replaceChildren(
            el("div", { className: "empty", textContent: "No scorecards match." }),
            el("button", {
              className: "link",
              textContent: "Clear search",
              onclick: function() {
                cardSearch.value = "";
                drawCards();
              }
            })
          );
          return;
        }
        var nStd = shown.filter(function(r) {
          return !r.compare;
        }).length;
        var items = [];
        shown.forEach(function(r, i) {
          if (i === 0 && !r.compare) items.push(el("div", { className: "rank-head", textContent: plural(nStd, "scorecard") }));
          if (r.compare && (i === 0 || !shown[i - 1].compare)) items.push(el("div", { className: "rank-head" }, plural(shown.length - nStd, "comparison factor") + " · ", link("compare", "", "Compare →")));
          var c = r.compare ? (D.comparisons || {})[r.name] || {} : null;
          var verdict = c ? (c.verdict || {})[(c.peers || [])[0]] || "unknown" : null;
          var debt = c ? verdict === "unknown" ? "not measured" : verdict : r.none > 0 ? r.none + " at none" : r.bronze > 0 ? r.bronze + " at bronze" : r.scored && r.topShare === 100 ? "✓ top level" : r.scored ? "✓ clear" : "not scored";
          var capped = !c && !r.none && !r.bronze && r.scored && r.topShare === 100 && capsRepo(D, r.name, floorOf(D, r.name));
          var a = link("scorecard/" + enc(r.name), "", [el("span", {}, ...keepWithLast(title(D, r.name), judgedMark(D, r.name, true))), el("span", { className: "debt " + (c ? verdict === "behind" ? "bad" : verdict === "ahead" ? "ok" : "" : r.none > 0 ? "bad" : r.bronze > 0 || !r.scored ? "" : capped ? "capnote" : "ok") }, ...capped ? [lockIcon(), " caps the repository"] : [debt])]);
          a.title = r.name + " · " + r.sc.kind;
          if (r.name === active) a.setAttribute("aria-current", "true");
          items.push(a);
        });
        cardNav.replaceChildren.apply(cardNav, items);
      };
      cardSearch.oninput = function() {
        drawCards();
        edges();
      };
      drawCards();
      var toTop = el("button", {
        className: "link rank-top",
        hidden: true,
        textContent: "↑ Back to the top of the list",
        onclick: function() {
          cardNav.scrollTop = 0;
          var first = cardNav.querySelector("a");
          if (first) first.focus({ preventScroll: true });
        }
      });
      var edges = function() {
        cardNav.classList.toggle("more", cardNav.scrollTop + cardNav.clientHeight < cardNav.scrollHeight - 4);
        cardNav.classList.toggle("less", cardNav.scrollTop > 4);
        toTop.hidden = cardNav.scrollTop <= 4;
      };
      cardNav.onscroll = edges;
      requestAnimationFrame(function() {
        var cur = cardNav.querySelector("[aria-current]");
        if (cur) {
          var at = cur.getBoundingClientRect().top - cardNav.getBoundingClientRect().top + cardNav.scrollTop;
          if (at + cur.offsetHeight > cardNav.clientHeight) {
            var want = at - cardNav.clientHeight / 3;
            var base = cardNav.getBoundingClientRect().top - cardNav.scrollTop;
            var row = [].find.call(cardNav.querySelectorAll("a"), function(a) {
              return a.getBoundingClientRect().top - base >= want;
            });
            cardNav.scrollTop = row ? row.getBoundingClientRect().top - base - 4 : want;
          }
        }
        edges();
      });
      var levelFilter = query.level === "not-in-scope" || query.level === "skipped" ? "skipped" : query.level && order.indexOf(query.level) >= 0 ? query.level : "all";
      var outcomeFilter = query.outcome === "failing" || query.outcome === "passing" ? query.outcome : "all";
      var ruleFilter = query.rule && (sc.rules || {})[query.rule] ? query.rule : null;
      var search = el("input", { type: "search", placeholder: "Filter by entity or rule…", ariaLabel: "Filter entities", value: query.q || "" });
      var keepInUrl = function() {
        var qs = new URLSearchParams();
        if (levelFilter !== "all") qs.set("level", levelFilter === "skipped" ? "not-in-scope" : levelFilter);
        if (outcomeFilter !== "all") qs.set("outcome", outcomeFilter);
        if (ruleFilter) qs.set("rule", ruleFilter);
        if (search.value) qs.set("q", search.value);
        var tail = qs.toString();
        if (!tail && location.hash.indexOf("?") < 0) return;
        var want = "#scorecard/" + enc(active) + (tail ? "?" + tail : "");
        if (location.hash !== want && history.replaceState) history.replaceState(history.state, "", want);
      };
      var filters = el("div", { className: "filters" });
      var list = el("div");
      var limit = 12;
      var seg = function(label, options, current, set) {
        var g = el(
          "div",
          { className: "seg" },
          ...options.map(function(o) {
            return el("button", {
              textContent: o.label,
              ariaPressed: String(current === o.value),
              onclick: function() {
                set(o.value);
                redrawFilters();
                draw();
              }
            });
          })
        );
        g.setAttribute("role", "group");
        g.setAttribute("aria-label", label);
        return g;
      };
      var clearFilters = function() {
        levelFilter = "all";
        outcomeFilter = "all";
        ruleFilter = null;
        search.value = "";
        redrawFilters();
        draw();
      };
      var redrawFilters = function() {
        var present = order.filter(function(l) {
          return parts.some(function(p) {
            return p.level === l;
          });
        });
        filters.replaceChildren(
          seg(
            "Level",
            [{ label: "All levels", value: "all" }].concat(
              present.map(function(l) {
                return { label: l === "skipped" ? "not in scope" : l, value: l };
              })
            ),
            levelFilter,
            function(v) {
              levelFilter = v;
            }
          ),
          seg(
            "Outcome",
            [
              { label: "All outcomes", value: "all" },
              { label: "Failing", value: "failing" },
              { label: "Passing", value: "passing" }
            ],
            outcomeFilter,
            function(v) {
              outcomeFilter = v;
            }
          ),
          ruleFilter ? el(
            "button",
            {
              className: "chip-filter",
              ariaLabel: "Remove filter: not passing " + ruleFilter,
              onclick: function() {
                ruleFilter = null;
                redrawFilters();
                draw();
              }
            },
            "Not passing ",
            el("b", { textContent: ruleFilter }),
            el("span", { ariaHidden: "true", textContent: " ×" })
          ) : ""
        );
      };
      var draw = function() {
        keepInUrl();
        var q = search.value.toLowerCase();
        var shown = rows.filter(function(pair) {
          return pair[1].level !== "skipped" || levelFilter === "skipped";
        }).filter(function(pair) {
          return levelFilter === "all" || pair[1].level === levelFilter;
        }).filter(function(pair) {
          if (!q) return true;
          if (pair[0].toLowerCase().indexOf(q) >= 0) return true;
          return Object.keys(pair[1].rules || {}).some(function(rn) {
            return rn.toLowerCase().indexOf(q) >= 0;
          });
        }).filter(function(pair) {
          if (!ruleFilter) return true;
          var o = (pair[1].rules || {})[ruleFilter];
          return isFailing(o) || o === "unknown";
        }).filter(function(pair) {
          if (outcomeFilter === "all") return true;
          var outs = Object.values(pair[1].rules || {});
          return outcomeFilter === "failing" ? outs.some(isFailing) : outs.every(function(o) {
            return isPassing(o) || o === "skipped";
          });
        }).sort(function(a, b) {
          return order.indexOf(a[1].level) - order.indexOf(b[1].level) || offBy(b[1]) - offBy(a[1]) || a[0].localeCompare(b[0]);
        });
        var sized = shown.some(function(pair) {
          return offBy(pair[1]) > 0;
        });
        if (!rows.length) {
          list.replaceChildren(el("div", { className: "empty" }, "No " + kindTitle(D, sc.kind).toLowerCase() + " has been collected yet. Run ", el("code", { textContent: "toolbox report" }), " in the repository with a connector for it listed under ", el("code", { textContent: "connectors:" }), " in the portal's ", el("code", { textContent: "toolbox.yaml" }), "; ", el("code", { textContent: "toolbox doctor" }), " lists any that is missing."));
          return;
        }
        if (!shown.length) {
          list.replaceChildren(
            el("div", { className: "empty", textContent: "No entities match these filters." }),
            el("button", { className: "link", textContent: "Clear filters", onclick: clearFilters })
          );
          return;
        }
        var outOfScope = rows.filter(function(pair) {
          return pair[1].level === "skipped";
        }).length;
        list.replaceChildren(el("div", { className: "sub", style: "margin:6px 0", textContent: plural(shown.length, "entity", "entities") + (outcomeFilter === "failing" ? " failing a rule" : outcomeFilter === "passing" ? " passing every rule" : "") + (outOfScope && levelFilter === "all" && outcomeFilter === "all" && !ruleFilter && !q ? " in scope, of " + kindCount(D, sc.kind, rows.length) : "") + ", lowest level first" + (sized ? ", then furthest off" : "") }));
        var facts = function(pair) {
          var r = pair[1];
          var outs = Object.entries(r.rules || {});
          var next = nextLevel(sc, r.level);
          var need = next ? blockers(sc, r, next) : [];
          return [r.level, unassessedIn(D, active, pair[0]) ? 1 : 0, unrunIn(sc, r).length, need.join(","), outs.filter(function(ro) {
            return isFailing(ro[1]);
          }).length, outs.filter(function(ro) {
            return isPassing(ro[1]);
          }).length, outs.length].join("|");
        };
        var worstOf = function(g) {
          var r = g[0][1];
          if (!(offBy(r) > 0)) return "";
          var rule = Object.keys(r.rules || {}).find(function(n) {
            return isFailing(r.rules[n]) && typeof (r.measures || {})[n] === "number";
          });
          var ev = rule && (r.evidence || {})[rule];
          return shortId(g[0][0]) + (ev ? ", " + String(Array.isArray(ev) ? ev[0] : ev).split("\n")[0] : "");
        };
        var groups = [];
        var byFacts = /* @__PURE__ */ new Map();
        shown.forEach(function(pair) {
          var key = facts(pair);
          var g = byFacts.get(key);
          if (!g) {
            g = [];
            byFacts.set(key, g);
            groups.push(g);
          }
          g.push(pair);
        });
        var entRow = function(pair) {
          var id = pair[0];
          var r = pair[1];
          var outs = Object.entries(r.rules || {});
          var failing = outs.filter(function(ro) {
            return isFailing(ro[1]);
          });
          var passing = outs.filter(function(ro) {
            return isPassing(ro[1]);
          });
          var unrun = outs.filter(function(ro) {
            return ro[1] === "unknown";
          }).length;
          var short = shortId(id);
          var next = nextLevel(sc, r.level);
          var need = next ? blockers(sc, r, next) : [];
          var later = failing.length - need.length;
          var d = el(
            "details",
            { className: "ent" },
            el(
              "summary",
              {},
              r.level === "skipped" ? badge(r.level) : levelPill(r.level, unassessedIn(D, active, id), unrunIn(sc, r), !unassessedIn(D, active, id)),
              el("span", { className: "name" }, short.replace(/[^\s\-/]+$/, ""), el("span", { className: /[^\s\-/]{25,}$/.test(short) ? "tail long" : "tail" }, (/[^\s\-/]+$/.exec(short) || [""])[0], el("span", { className: "chev", textContent: "›" })), short !== id ? el("small", { textContent: id, title: id }) : null),
              r.level === "skipped" ? el("span", { className: "tally sub", textContent: "not rated: outside this scorecard's scope" }) : el(
                "span",
                { className: "tally" },
                need.length ? el(
                  "b",
                  { className: r.level === "none" ? "bad" : "" },
                  (KNOWN.indexOf(next) >= 0 ? next.charAt(0).toUpperCase() + next.slice(1) : levelName(next)) + " needs: ",
                  ...need.slice(0, 2).map(function(n, i) {
                    return el("span", { className: "nr" }, ...keepWithLast(ruleTitle(active, n), basisBadge(basisOf((sc.rules || {})[n]), true)), i < Math.min(need.length, 2) - 1 ? ", " : "");
                  }),
                  need.length > 2 ? " +" + (need.length - 2) : ""
                ) : failing.length ? "" : "✓ all pass",
                later > 0 ? (need.length ? " · " : "") + later + " more for higher levels" : "",
                " · " + passing.length + " of " + outs.length + " pass",
                unrun ? el("span", { className: "provisional" }, " · " + unrun + " not run") : ""
              )
            )
          );
          var bodyBuilt = false;
          d.addEventListener("toggle", function() {
            if (!d.open || bodyBuilt) return;
            bodyBuilt = true;
            d.append(
              el(
                "div",
                { className: "ent-body" },
                self.nextStep(sc, r),
                self.rulesFor(sc, r, active, unassessedIn(D, active, id), id),
                el("p", { style: "margin:12px 0 0" }, link("entity/" + enc(id), "", "Open " + short + " across every scorecard →"))
              )
            );
          });
          return d;
        };
        var rowsShown = 0;
        var entsShown = 0;
        groups.forEach(function(g) {
          if (rowsShown >= limit) return;
          if (g.length < 3) {
            g.forEach(function(pair) {
              if (rowsShown >= limit) return;
              list.append(entRow(pair));
              rowsShown++;
              entsShown++;
            });
            return;
          }
          rowsShown++;
          entsShown += g.length;
          var first = entRow(g[0]).querySelector("summary");
          var grp = el(
            "details",
            { className: "ent ent-group" },
            el(
              "summary",
              {},
              first.children[0],
              el("span", { className: "name" }, kindCount(D, sc.kind, g.length) + " with the same result", el("span", { className: "tail" }, el("span", { className: "chev", textContent: "›" })), worstOf(g) ? el("small", { textContent: "furthest off: " + worstOf(g) }) : null),
              first.children[2]
            )
          );
          var built = false;
          grp.addEventListener("toggle", function() {
            if (!grp.open || built) return;
            built = true;
            grp.append(el("div", { className: "ent-members" }, ...g.map(entRow)));
          });
          list.append(
            grp,
            memberLinks(g.map(function(pair) {
              return pair[0];
            }))
          );
        });
        if (entsShown < shown.length) {
          list.append(
            el(
              "div",
              { className: "more" },
              el("button", {
                className: "link",
                textContent: "Show all " + fmt(shown.length) + " (" + fmt(shown.length - entsShown) + " more)",
                onclick: function() {
                  limit = Infinity;
                  draw();
                }
              })
            )
          );
        }
      };
      search.oninput = draw;
      redrawFilters();
      draw();
      var scored = rows.filter(function(pair) {
        return pair[1].level !== "skipped";
      });
      var failCounts = {};
      var unrunCounts = {};
      var provisionalN = 0;
      scored.forEach(function(pair) {
        Object.entries(pair[1].rules || {}).forEach(function(ro) {
          if (isFailing(ro[1])) failCounts[ro[0]] = (failCounts[ro[0]] || 0) + 1;
          else if (ro[1] === "unknown") unrunCounts[ro[0]] = (unrunCounts[ro[0]] || 0) + 1;
        });
        if (unrunIn(sc, pair[1]).length) provisionalN++;
      });
      var unrunNames = Object.keys(unrunCounts).sort();
      var unrunNote = unrunNames.length ? el(
        "p",
        { className: "sub unrun-note" },
        el("b", { className: "provisional", textContent: "Not run in this build: " }),
        ...unrunNames.map(function(n, i) {
          return el("span", { className: "nr" }, i ? ", " : "", ruleLink(active, n), " ", basisBadge(basisOf((sc.rules || {})[n]), "tip"), " (" + unrunCounts[n] + " of " + scored.length + ")");
        }),
        ". A check that was not run is neither a pass nor a failure, so it is left out of the bars above and of which rule to fix first. Levels that need one are provisional: they hold only if it passes."
      ) : null;
      var provisionalNote = provisionalN ? el("p", { className: "sub", style: "margin:6px 0 0" }, el("b", { className: "provisional", textContent: "Provisional: " }), plural(provisionalN, "entity's level rests", "entities' levels rest") + " on checks not run in this build (drawn dotted in the bar above and the list below). ", el("span", { textContent: "They hold only if those checks pass." })) : null;
      var lever = null;
      var leverRule = null;
      var low = order.find(function(l) {
        return l !== "skipped" && nextLevel(sc, l) && parts.some(function(p) {
          return p.level === l;
        });
      });
      if (low) {
        var held = {};
        var up = nextLevel(sc, low);
        scored.forEach(function(pair) {
          if (pair[1].level !== low) return;
          blockers(sc, pair[1], up).forEach(function(n) {
            held[n] = (held[n] || 0) + 1;
          });
        });
        var best = Object.entries(held).sort(function(a, b) {
          return b[1] - a[1] || a[0].localeCompare(b[0]);
        })[0];
        if (best) leverRule = best[0];
        var close = scored.filter(function(pair) {
          var nx = pair[1].level !== "skipped" && nextLevel(sc, pair[1].level);
          return nx && blockers(sc, pair[1], nx).length === 1;
        }).length;
        if (best)
          lever = el(
            "div",
            { className: "next", style: "margin:0 0 10px" },
            el("b", { textContent: "Biggest lever" }),
            el("span", { className: "sub", textContent: " (the rule that lifts the most entities a level)" }),
            el("br"),
            ruleLink(active, best[0]),
            " ",
            basisBadge(basisOf((sc.rules || {})[best[0]])),
            " keeps " + plural(best[1], "entity", "entities") + " at " + low + "; they need it for " + up + ". ",
            ...rich(((sc.rules || {})[best[0]] || {}).fix),
            close ? el("span", { className: "close-note" }, el("br"), el("b", { textContent: "Within reach: " }), plural(close, "entity is", "entities are") + " one rule from " + (close === 1 ? "its" : "their") + " next level.") : null
          );
      }
      var topFails = Object.entries(failCounts).sort(function(a, b) {
        return b[1] - a[1] || a[0].localeCompare(b[0]);
      }).slice(0, 5);
      var failChart = topFails.length ? el(
        "div",
        { className: "fails" },
        ...topFails.map(function(f) {
          var rl = ((sc.rules || {})[f[0]] || {}).level;
          var share2 = Math.round(f[1] / scored.length * 100);
          var fillSpan = el("span", { style: "flex-grow:0;background:var(--fail)" });
          grow([fillSpan], [share2]);
          var b = el(
            "button",
            {
              className: "fail-row",
              title: ((sc.rules || {})[f[0]] || {}).fix || "",
              onclick: function() {
                search.value = "";
                levelFilter = "all";
                if (outcomeFilter === "passing") outcomeFilter = "all";
                ruleFilter = f[0];
                limit = 25;
                redrawFilters();
                draw();
                filters.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
                var chip = filters.querySelector(".chip-filter");
                if (chip) chip.focus({ preventScroll: true });
              }
            },
            el("span", { className: "rn" }, el("span", { textContent: ruleTitle(active, f[0]) }), basisBadge(basisOf((sc.rules || {})[f[0]]), false, true), rl ? badge(rl) : null, lever && f[0] === leverRule ? el("span", { className: "pill", title: "Fixing it lifts the most entities a level", textContent: "fix first" }) : null),
            el("span", { className: "bar xs" }, fillSpan, el("span", { style: "flex-grow:" + (100 - share2) + ";background:transparent" })),
            el("span", { className: "fc" }, f[1] + " of " + scored.length, el("span", { className: "sr", textContent: " fail it. Show them." }))
          );
          return b;
        })
      ) : !scored.length ? el("p", { className: "empty", textContent: "No entity is scored yet, so no rule has failed or passed." }) : el("p", { className: "empty good", textContent: unrunNames.length ? "✓ No rule fails on any entity; some checks were not run." : "✓ No rule fails on any entity." });
      var needs = el("div", { className: "levels-need" });
      (sc.levels || []).forEach(function(l) {
        var names = Object.keys(sc.rules || {}).filter(function(n) {
          return sc.rules[n].level === l;
        }).sort();
        var bases = names.map(function(n) {
          return basisOf(sc.rules[n]);
        });
        var shared = bases.length && bases.every(function(b) {
          return b === bases[0];
        }) ? bases[0] : null;
        needs.append(
          el(
            "div",
            { className: "need" },
            el("div", { className: "need-head" }, badge(l), shared ? el("span", { className: "need-basis" }, names.length > 1 ? "All " + names.length + " rules" : "1 rule", basisBadge(shared, "tip")) : null),
            names.length ? el(
              "ul",
              {},
              ...names.map(function(n) {
                var rd = sc.rules[n];
                return el("li", {}, el("span", { style: "display:inline-flex;flex-wrap:wrap;gap:4px 6px;align-items:center" }, el("b", {}, ruleLink(active, n, null, true)), shared ? null : basisBadge(basisOf(rd), "tip")), rd.rationale || rd.fix ? whyFirst(rd.rationale || rd.fix) : null, refs(rd.links));
              })
            ) : el("div", { className: "sub", textContent: "No rules at this level." })
          )
        );
      });
      needs.classList.add("clamped");
      var needsToggle = el("button", {
        className: "link need-toggle",
        textContent: "Show full reasons, with thresholds and sources",
        ariaExpanded: "false",
        onclick: function() {
          var open = needs.classList.toggle("clamped") === false;
          needsToggle.textContent = open ? "Show shorter reasons" : "Show full reasons, with thresholds and sources";
          needsToggle.setAttribute("aria-expanded", String(open));
        }
      });
      var picker = el(
        "select",
        {
          ariaLabel: "Scorecard",
          onchange: function() {
            location.hash = "scorecard/" + enc(picker.value);
          }
        },
        ...[false, true].map(function(cmp) {
          var group = ranked.filter(function(r) {
            return !!r.compare === cmp;
          });
          if (!group.length) return null;
          return el(
            "optgroup",
            { label: cmp ? plural(group.length, "comparison factor") : plural(group.length, "scorecard") },
            ...group.map(function(r) {
              var c = cmp ? (D.comparisons || {})[r.name] || {} : null;
              var debt = c ? (c.verdict || {})[(c.peers || [])[0]] || "not measured" : r.none > 0 ? r.none + " at none" : r.bronze > 0 ? r.bronze + " at bronze" : r.scored && r.topShare === 100 ? "top level" : r.scored ? "clear" : "not scored";
              return el("option", { value: r.name, selected: r.name === active, textContent: title(D, r.name) + " (" + debt + ")" });
            })
          );
        }).filter(Boolean)
      );
      var owner = sc.owner ? sc.owner.replace(/^group:/, "") : "";
      var hStand = el("h3", { textContent: "What holds it where it is", tabIndex: -1 });
      var hFail = el("h3", { textContent: "Most common failures, at any level", tabIndex: -1 });
      var hNeeds = el("h3", { textContent: "What each level needs (each level also needs the ones below)", tabIndex: -1 });
      var hList = el("h3", { textContent: "Entities", tabIndex: -1 });
      var jump = function(label, target) {
        return el("button", {
          className: "link",
          textContent: label,
          onclick: function() {
            target.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
            target.focus({ preventScroll: true });
          }
        });
      };
      var standNotes = [unassessedNote(D, active, scored.length), provisionalNote, lever ? el("div", { style: "margin-top:12px" }, lever) : null].filter(Boolean);
      var jumpBar = el("nav", { className: "jumpbar", ariaLabel: "On this page" }, el("span", { className: "sub", textContent: "On this page:" }), standNotes.length ? jump("What holds it", hStand) : null, jump("Failures", hFail), jump("Entities (" + fmt(scored.length) + ")", hList), jump("What each level needs", hNeeds));
      var floorHere = floorOf(D, active);
      var topHere = order.filter(function(l) {
        return l !== "skipped" && l !== "none";
      }).pop();
      var atTop = scored.filter(function(pair) {
        return pair[1].level === topHere;
      }).length;
      var split = splitUnassessed(D, active, parts);
      var statusLegend = legendEl(parts);
      if (split.some(function(p) {
        return p.hollow;
      })) statusLegend.append(el("span", {}, el("span", { className: "dot hatch", style: "--seg:" + fill("bronze") }), "hatched: not assessed"));
      if (split.some(function(p) {
        return p.unrun;
      })) statusLegend.append(el("span", {}, el("span", { className: "dot dots", style: "--seg:" + fill("silver") }), "◌ dotted: provisional, checks not run"));
      var status = el(
        "div",
        { className: "sc-status" },
        medal(floorHere || (scored.length ? "none" : "not scored"), 44),
        el(
          "div",
          { className: "sc-status-b" },
          el(
            "div",
            { className: "sc-status-t" },
            el("span", { className: "sub", textContent: "Weakest entity " }),
            floorHere ? floorPill(D, active) : el("b", { textContent: "not scored" }),
            floorHere && isTop(D, active, floorHere) && !ONLY_LEVEL.has(floorHere) ? el("span", { className: "topmark", textContent: " ✓ top level" }) : null,
            topHere && scored.length ? el("span", { className: "sep", ariaHidden: "true", textContent: "·" }) : null,
            topHere && scored.length ? el("span", { className: "fact", textContent: fmt(atTop) + " of " + fmt(scored.length) + " at the top level (" + levelName(topHere) + ")" }) : null,
            leverRule ? el("span", { className: "sep", ariaHidden: "true", textContent: "·" }) : null,
            leverRule ? el("span", { className: "fact" }, "fix first: ", el("button", { className: "link", textContent: ruleTitle(active, leverRule), onclick: function() {
              hStand.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
              hStand.focus({ preventScroll: true });
            } })) : null
          ),
          barEl(split, "lg"),
          statusLegend
        )
      );
      var whyEl2 = null;
      if (sc.rationale || (sc.links || []).length) {
        var whyText = sc.rationale ? el("div", { className: "why-text" }, ...richCards(D, sc.rationale, active)) : null;
        var long = String(sc.rationale || "").length > 260;
        if (long) whyText.classList.add("clamped");
        whyEl2 = el(
          "div",
          { className: "rationale" },
          el("b", { textContent: "Why this scorecard" }),
          whyText,
          long ? el("button", {
            className: "link why-more",
            textContent: "Read the full why",
            ariaExpanded: "false",
            onclick: function(e) {
              var open = whyText.classList.toggle("clamped") === false;
              e.currentTarget.textContent = open ? "Show less" : "Read the full why";
              e.currentTarget.setAttribute("aria-expanded", String(open));
            }
          }) : null,
          refs(sc.links)
        );
      }
      var anyJudged = ranked.some(function(r) {
        return basisCounts(((D.scorecards || {})[r.name] || {}).rules).judged;
      });
      return el(
        "div",
        { className: "split" },
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Scorecards" })),
          el("p", { className: "sub", style: "margin:0" }, ranked.some(function(r) {
            return r.scored;
          }) ? "Most entities at none first" : "Nothing is scored yet", anyJudged ? el("span", {}, " · ", el("span", { className: "bicon judged" }, basisIcon("judged")), " has agent-judged rules · ", el("span", { className: "bicon" }, basisIcon("measured")), " every rule measured") : null),
          cardSearch,
          toTop,
          cardNav
        ),
        el(
          "section",
          { className: basisCounts(sc.rules).judged || !basisCounts(sc.rules).measured ? "" : "allmeasured" },
          el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("scorecards", "", "Scorecards"), " / " + title(D, active)),
          el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Scorecard" }), picker),
          el("div", { className: "head" }, el("h2", { textContent: title(D, active) }), el("code", { className: "sub", textContent: active })),
          status,
          el("p", { className: "lede" }, ...richCards(D, sc.description, active)),
          whyEl2,
          el(
            "div",
            { className: "sub" },
            "Rates every " + kindTitle(D, sc.kind).replace(/^[A-Z](?=[a-z]|$)/, function(c) {
              return c.toLowerCase();
            }) + (owner ? " · owned by " + owner : ""),
            sc.track ? el("span", {}, " · track ", link(trackPath(D, sc.track), "", trackTitle(D, sc.track))) : null,
            basisInline(basisCounts(sc.rules))
          ),
          isComparative(D, active) ? el("div", { className: "callout" }, el("span", {}, "This scorecard compares products rather than levelling them. ", link("compare/" + enc(active), "", "See how they stand →"))) : null,
          basisInline(basisCounts(sc.rules)) ? null : basisKey(basisCounts(sc.rules)),
          jumpBar,
          standNotes.length ? hStand : null,
          ...standNotes,
          hFail,
          failChart && rows.length ? el("p", { className: "sub", style: "margin:0 0 6px", textContent: "Each bar is the share of entities failing the rule; the pill beside it names the level the rule is needed for." }) : null,
          failChart,
          unrunNote,
          hList,
          rows.length ? filters : null,
          rows.length ? search : null,
          list,
          hNeeds,
          needsToggle,
          needs
        )
      );
    }
  };

  // src/views/tracks.js
  function atNone(t) {
    var n = (t.cards || []).reduce(function(sum, c) {
      return sum + (c.row.none || 0);
    }, 0);
    return n ? "; " + plural(n, "entity", "entities") + " still at none" : "";
  }
  var tracksViews = {
    trackCard(D, S, t, flag) {
      var def = t.def;
      var color = trackColor(t);
      var kids = [
        flag ? el("span", { className: "flag", textContent: flag }) : null,
        el(
          "div",
          { className: "th" },
          t.compare ? tallyRing(t.verdicts, t.rows.length, 60) : ring(t.pct, { size: 60, color, label: t.pct == null ? "not scored" : t.pct + "% at the top level" }),
          el("div", {}, el("div", { className: "tt", textContent: def.title || t.name }), el("div", { className: "sub", textContent: t.compare ? "factors ahead, a segment per factor" : (t.averaged > 1 ? ringWords(t) : "of entities at the top level") + atNone(t) }))
        ),
        def.description ? el("div", { className: "td", textContent: prose(firstSentence(def.description)) }) : null
      ];
      var slots = !t.compare && trackSlots(D, t);
      if (slots) kids.push(slots);
      if (slots && t.capped.length) kids.push(el("div", { className: "capnote tcap" }, lockIcon(), " " + ringWhy(D, t)));
      if (slots && t.short.length) kids.push(el("div", { className: "capnote tcap" }, lockIcon(), " " + shortWhy(D, t) + "."));
      if (t.compare) {
        kids.push(
          el(
            "div",
            { className: "chips" },
            ...["ahead", "parity", "behind", "unknown"].filter(function(v) {
              return t.verdicts[v];
            }).map(function(v) {
              return standingChip(v, t.verdicts[v] + " " + v);
            }),
            ruleTag(t.majority)
          )
        );
      } else if (t.cards.length) {
        var shelf = {};
        var shelfTop = {};
        var shelfOrder = [];
        t.cards.forEach(function(c) {
          var l = c.floor || "not scored";
          if (!shelf[l]) {
            shelf[l] = 0;
            shelfTop[l] = 0;
            shelfOrder.push(l);
          }
          shelf[l]++;
          if (c.floor && isTop(D, c.row.name, c.floor)) shelfTop[l]++;
        });
        shelfOrder.sort(function(a2, b) {
          return rankLevel(a2 === "not scored" ? null : a2) - rankLevel(b === "not scored" ? null : b);
        });
        kids.push(
          el(
            "div",
            { className: "shelf", role: "img", ariaLabel: "Scorecards by their weakest entity's level: " + shelfOrder.map(function(l) {
              return shelf[l] + " " + l + (shelfTop[l] && shelfTop[l] < shelf[l] ? ", " + shelfTop[l] + " of them at its top level" : shelfTop[l] ? ", at its top level" : "");
            }).join("; ") },
            el("span", { className: "sub", textContent: "Medals:" }),
            ...shelfOrder.map(function(l) {
              var own = ONLY_LEVEL.has(l) && LADDER.indexOf(l) < 0;
              var owner = own ? t.cards.find(function(c) {
                return c.floor === l;
              }) : null;
              var gloss = null;
              if (own) {
                var def2 = "The one level " + (owner ? title(D, owner.row.name) : "its scorecard") + " has, so reaching it is the top." + (LEVEL_DEF[l] ? " " + LEVEL_DEF[l] : "");
                gloss = el("abbr", { className: "gloss", tabIndex: 0 }, levelName(l), el("span", { className: "sr", textContent: " (" + def2 + ")" }));
                gloss.dataset.tip = def2;
              }
              return el("span", { className: "sh" + (own ? " own" : "") }, medal(l, 20), el("b", { textContent: String(shelf[l]) }), " ", gloss || (l === "none" ? "with no medal yet" : levelName(l)), own ? " ✓ top" : shelfTop[l] === shelf[l] || ONLY_LEVEL.has(l) ? " ✓ top" : shelfTop[l] ? " (" + shelfTop[l] + " ✓ top)" : "");
            })
          )
        );
        var nm = nextMedal(D, t.cards);
        var wk = t.cards[0] && t.cards[0].floor && !isTop(D, t.cards[0].row.name, t.cards[0].floor) ? t.cards[0] : null;
        if (wk && (!nm || nm.name !== wk.row.name)) kids.push(el("div", { className: "tnext" }, el("b", { textContent: "Lowest level: " }), link("scorecard/" + enc(wk.row.name), "", title(D, wk.row.name)), " at ", badge(wk.floor)));
        kids.push(
          nm ? el("div", { className: "tnext" }, el("b", { textContent: wk && nm.name === wk.row.name ? "Lowest level, and the next medal: " : "Next medal: " }), link("scorecard/" + enc(nm.name), "", title(D, nm.name)), wk && nm.name === wk.row.name ? el("span", {}, " at ", badge(wk.floor), " → " + nm.up) : " to " + nm.up, " · " + plural(nm.low, "entity", "entities") + " to lift") : el("div", { className: "tnext good", textContent: "✓ Every scorecard is at its top level" })
        );
      }
      var weakest = !t.compare && t.cards[0] && t.cards[0].floor ? title(D, t.cards[0].row.name) + " at " + t.cards[0].floor + (capsRepo(D, t.cards[0].row.name, t.cards[0].floor) ? ", its top level, which caps the repository" : isTop(D, t.cards[0].row.name, t.cards[0].floor) ? ", its top level" : "") : null;
      kids.push(
        el(
          "div",
          { className: "foot" },
          el("span", { textContent: plural(t.rows.length, t.compare ? "factor" : "scorecard") + (t.campaigns.length ? " · " + plural(t.campaigns.length, "campaign") : "") }),
          t.compare ? el("span", { textContent: t.behind.length ? "behind on " + t.behind.join(", ") : t.verdicts.unknown >= t.rows.length ? "not measured yet" : t.verdicts.unknown ? "not behind on any measured factor" : "never behind" }) : null
        )
      );
      var tl = kids[1].lastChild.firstChild;
      var tlink = link(trackPath(D, t.name), "tlink", def.title || t.name);
      tl.replaceChildren(tlink);
      var a = el("div", { className: "track" }, ...kids);
      a.style.setProperty("--tc", color);
      var standing = t.compare ? tallyWords(t.verdicts, t.rows.length) : t.pct == null ? "not scored" : (t.averaged > 1 ? t.pct + "% " + ringWords(t) : t.pct + "% of entities at the top level") + (t.capped.length ? ". " + ringWhy(D, t) : "");
      tlink.setAttribute("aria-label", (def.title || t.name) + ": " + standing + (flag ? ". " + flag : "") + (weakest ? ". Lowest level: " + weakest : ""));
      return a;
    },
    trackGrid(D, S, shown) {
      var self = this;
      var furthest = furthestTrack(D, S);
      return el(
        "div",
        { className: "tracks" },
        ...shown.map(function(t) {
          return self.trackCard(D, S, t, t.name === furthest ? "Furthest from goal" : null);
        })
      );
    },
    tracks(D, S, name) {
      if (name) return this.track(D, S, name);
      var all = tracksSorted(D, S);
      if (!all.length) return el("section", {}, el("div", { className: "empty", textContent: "This portal declares no tracks." }));
      var loose = S.rows.filter(function(r) {
        return !r.sc.track || !(D.tracks || {})[r.sc.track];
      });
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Tracks" }), el("span", { className: "sub", textContent: "Lowest share at the top level first" })),
          el("p", { className: "lede", textContent: "A track is a long-running line of work: the scorecards that measure it and the campaigns that push on it. Its goal is every entity at the top level." }),
          trackKey(all),
          this.trackGrid(
            D,
            S,
            all.filter(function(t) {
              return !t.compare;
            })
          )
        ),
        all.some(function(t) {
          return t.compare;
        }) ? el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Measured against a rival" })),
          el("p", { className: "lede", textContent: "Its ring counts the factors where the product leads its rival, so it stands outside the order above." }),
          this.trackGrid(
            D,
            S,
            all.filter(function(t) {
              return t.compare;
            })
          )
        ) : null,
        loose.length ? el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Not on a track" })),
          levelLegend(D, loose),
          this.scorecardTable(D, loose)
        ) : null
      );
    },
    track(D, S, name) {
      var t = trackStats(D, S, name);
      var def = t.def;
      var self = this;
      var color = trackColor(t);
      var owner = def.owner ? def.owner.replace(/^group:/, "") : "";
      var weakest = t.cards[0];
      var goal = t.compare ? el("p", {}, el("b", { textContent: tallyWords(t.verdicts, t.rows.length).replace(/^a/, "A") + ". " }), link("compare", "", "Open the comparison →")) : el(
        "p",
        {},
        el("b", { textContent: "Goal: every entity at the top level of each scorecard below. " }),
        t.pct == null ? "Nothing is scored yet." : "On average " + t.pct + "% are there" + (t.none ? "; " + plural(t.none, "entity is", "entities are") + " still at none" : "") + "." + (t.capped.length ? " " + ringWhy(D, t) + "." : "") + (t.short.length ? " " + shortWhy(D, t) + "." : ""),
        weakest && weakest.floor ? el("span", {}, " The weakest scorecard is ", link("scorecard/" + enc(weakest.row.name), "", title(D, weakest.row.name)), ", at ", badge(weakest.floor), capsRepo(D, weakest.row.name, weakest.floor) ? ", its only top level, so it caps the repository there until it gains a higher one." : isTop(D, weakest.row.name, weakest.floor) ? ", already its top level." : ".") : null
      );
      var trackNext = t.compare ? null : nextMedalEl(D, t.cards);
      var section = el(
        "section",
        {},
        el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("tracks", "", "Tracks"), " / " + (def.title || name)),
        el(
          "div",
          { className: "ehead" },
          el("div", {}, el("h2", { textContent: def.title || name, tabIndex: -1 }), el("div", { className: "sub", textContent: [owner ? "owned by " + owner : null, plural(t.rows.length, t.compare ? "factor" : "scorecard"), t.campaigns.length ? plural(t.campaigns.length, "campaign") : null].filter(Boolean).join(" · ") })),
          t.compare ? tallyRing(t.verdicts, t.rows.length, 88) : ring(t.pct, { size: 88, color, label: t.pct == null ? "not scored" : t.pct + "% at the top level" })
        ),
        def.description ? el("p", { className: "lede", style: "max-width:80ch" }, ...rich(def.description)) : null,
        refs(def.links),
        goal,
        trackNext ? el("div", { style: "max-width:80ch;margin-top:8px" }, trackNext) : null
      );
      section.dataset.focus = "";
      var parts = [section];
      if (t.compare) {
        parts.push(el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Factors" })), this.factorCards(D, t.rows.map(function(r) {
          return r.name;
        }))));
      } else if (t.rows.length) {
        var rows = t.cards.map(function(c) {
          return c.row;
        });
        parts.push(
          el(
            "section",
            {},
            el("div", { className: "head" }, el("h2", { textContent: "Scorecards" }), el("span", { className: "sub", textContent: "Lowest level first" })),
            levelLegend(D, rows),
            this.scorecardTable(D, rows, true)
          )
        );
      }
      if (t.campaigns.length) {
        parts.push(
          el(
            "section",
            {},
            el("div", { className: "head" }, el("h2", { textContent: "Campaigns on this track" }), el("span", { className: "sub", textContent: "Time-bound pushes toward the goal" })),
            el(
              "div",
              { className: "cards" },
              ...t.campaigns.map(function(n) {
                return self.campaignCard(D, n);
              })
            )
          )
        );
      }
      var others = tracksSorted(D, S).filter(function(o) {
        return o.name !== name;
      });
      if (others.length) parts.push(this.trackStrip(D, S, others, "Other tracks"));
      return el("div", {}, ...parts);
    },
    // Tracks as a row of small rings; `lead` drops the separator and names each standing.
    trackStrip(D, S, shown, label, lead) {
      var furthest = furthestTrack(D, S);
      return el(
        "nav",
        { className: "trackstrip" + (lead ? " lead" : ""), ariaLabel: label },
        lead ? null : el("span", { className: "sub", textContent: label }),
        ...shown.map(function(o) {
          var standing = o.compare ? tallyWords(o.verdicts, o.rows.length) : o.pct == null ? "not scored" : o.pct + "% at the top level";
          var flag = lead && o.name === furthest;
          var a = link(trackPath(D, o.name), flag ? "far" : "", [
            o.compare ? tallyRing(o.verdicts, o.rows.length, 30, true) : ring(o.pct, { size: 30, color: trackColor(o), label: standing, bare: true }),
            el("span", {}, el("span", { textContent: o.def.title || o.name }), lead ? el("small", { textContent: (o.compare ? standing : o.pct == null ? "not scored" : o.pct + "%") + (flag ? " · furthest from goal" : "") }) : el("span", { className: "sr", textContent: ": " + standing }))
          ]);
          a.title = standing + (o.capped && o.capped.length ? ". " + ringWhy(D, o) : "");
          return a;
        })
      );
    }
  };

  // src/views/campaigns.js
  function sinceLine(c) {
    var s = c.sinceStart;
    if (!s) return el("p", { className: "sub", style: "margin:0 0 6px", textContent: "This report was built without the campaign's baseline, so the bar shows where the campaign stands today, not how far it has come since its starting point." });
    var parts = [["entitiesShort", "entities short"], ["measure", "total shortfall"]].filter(function(k) {
      return s[k[0]];
    }).map(function(k) {
      var x = s[k[0]];
      return k[1] + " " + fmt(x.from) + " → " + fmt(x.now) + (x.percentDone != null ? " (" + x.percentDone + "% of the way)" : "");
    });
    return el("p", { className: "sub", style: "margin:0 0 6px" }, el("b", { textContent: "Since the start" + (s.recorded ? ", " + s.recorded : "") + ": " }), parts.join("; ") || "nothing to compare yet", ".");
  }
  var campaignsViews = {
    campaigns(D, S, name) {
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
      (c.includes || []).forEach(function(i) {
        includes.append(
          el(
            "li",
            {},
            link("scorecard/" + enc(i.scorecard), "", title(D, i.scorecard)),
            i.atLeast ? el("span", {}, " at ", badge(i.atLeast), " or above") : "",
            i.rules && i.rules.length ? el("span", {}, i.rules.length === 1 ? ": passes " : ": passes each of ", ...i.rules.map(function(r, k) {
              return el("span", {}, k ? ", " : "", ruleLink(i.scorecard, r));
            })) : ""
          )
        );
      });
      var bar = el("div", { className: "bar lg" });
      var fillSpan = el("span", { style: "flex-grow:0;flex-basis:0;background:" + (state === "on" ? "var(--pass)" : "var(--muted)") });
      bar.append(fillSpan, el("span", { style: "flex-grow:" + Math.max(0, 100 - pct) + ";background:transparent" }));
      grow([fillSpan], [pct]);
      var marks = el("div", { className: "milestones" });
      (c.milestones || []).forEach(function(m) {
        marks.append(el("span", { className: m.met ? "met" : "", style: "left:" + m.targetPercent + "%", title: "Milestone “" + m.name + "”: " + m.targetPercent + "% of entities meeting the campaign" + (m.met ? ", reached" : ", not reached yet"), textContent: (m.met ? "✓ " : "") + m.name + " " + m.targetPercent + "%" + (m.met ? " reached" : "") }));
      });
      if (target < 100) marks.append(el("span", { style: "left:" + target + "%", textContent: "target " + target + "%" }));
      var gaps = Object.entries(c.entities || {}).filter(function(pair) {
        return !pair[1].meeting;
      }).sort(function(a, b) {
        return a[0].localeCompare(b[0]);
      });
      var gapList = el("div");
      if (state === "empty") {
        var kinds = (c.includes || []).map(function(i) {
          return ((D.scorecards || {})[i.scorecard] || {}).kind;
        });
        var openRows = includeProgress(D, c).map(function(p) {
          var kind = ((D.scorecards || {})[p.name] || {}).kind;
          var evPct = p.rated ? Math.floor(100 * (p.rated - p.below - p.hollow) / p.rated) : 0;
          return el(
            "div",
            { className: "gap" },
            link("scorecard/" + enc(p.name), "", p.title),
            el(
              "div",
              { className: "why incl" },
              p.known ? kpiBar(p.pct, p.pct + "% at " + p.goal + " or above" + (p.hollow ? ", " + evPct + "% on an agent's assessment" : "") + "; target " + target + "%", !p.met, p.pct - evPct) : null,
              el("span", { textContent: p.known ? (p.hollow ? evPct + "% at " + p.goal + " or above on an agent's assessment; " + p.pct + "% counting those not assessed yet" : p.pct + "% at " + p.goal + " or above") + " · " + (p.met ? "✓ at target" : fmt(p.below) + " of " + (kind ? kindCount(D, kind, p.rated) : fmt(p.rated)) + " to lift") : "not in this report" }),
              p.hollow ? el("span", { className: "unjudged-n", style: "flex-basis:100%;font-size:13px" }, el("span", { className: "vhatch", ariaHidden: "true", style: "margin:0 5px 0 0" }), fmt(p.hollow) + " pass only because no agent has assessed them yet (hatched)") : null
            )
          );
        });
        gapList.append(
          el("p", { className: "next", style: "margin:0 0 10px" }, el("b", { textContent: "Progress, per scorecard. " }), "The scorecards in this campaign rate different things, so each is measured against the target of " + target + "% on its own" + (c.end ? ", by " + c.end + (days != null && days >= 0 ? " (" + plural(days, "day") + " left)" : "") : "") + "."),
          ...openRows,
          el(
            "details",
            { className: "sub", style: "margin-top:8px" },
            el("summary", { style: "padding:3px 0", textContent: "Why each scorecard is measured on its own" }),
            el("p", { textContent: "A campaign's scope is the entities rated by every scorecard it includes, and no entity is rated by all of these" + (kinds.length > 1 && kinds.some(function(k) {
              return k !== kinds[0];
            }) ? " (they rate " + kinds.filter(Boolean).join(" and ") + ")" : "") + ". Include scorecards that rate the same kind to give it one." })
          )
        );
      } else if (!gaps.length) {
        gapList.append(el("div", { className: "empty good", textContent: "✓ Every in-scope entity meets the goal." }));
      } else {
        var needCounts = {};
        var needFix = {};
        gaps.forEach(function(pair) {
          (pair[1].gaps || []).forEach(function(g) {
            var m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
            var gsc = m && (D.scorecards || {})[m[1]];
            var score = m && ((D.scores || {})[m[1]] || {})[pair[0]];
            if (gsc && score)
              blockers(gsc, score, m[2]).forEach(function(r) {
                needCounts[r] = (needCounts[r] || 0) + 1;
                needFix[r] = { sc: m[1], def: (gsc.rules || {})[r] || {} };
              });
          });
        });
        var common = Object.entries(needCounts).sort(function(a, b) {
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
              needFix[common[0]].def.rationale ? el("span", { className: "why", style: "display:block;margin-top:4px" }, el("b", { textContent: "Why: " }), ...rich(needFix[common[0]].def.rationale)) : null
            )
          );
        var size = {};
        var said = {};
        gaps.forEach(function(pair) {
          (pair[1].gaps || []).forEach(function(g) {
            var m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
            var one = !m && /^(.+?):rule (.+) \((\S+)\)$/.exec(g);
            var on = m || one;
            var gsc = on && (D.scorecards || {})[on[1]];
            var score = on && ((D.scores || {})[on[1]] || {})[pair[0]];
            if (gsc && score)
              (m ? blockers(gsc, score, m[2]) : [one[2]]).forEach(function(r) {
                var v = (score.measures || {})[r];
                if (typeof v === "number") size[pair[0]] = (size[pair[0]] || 0) + v;
                var ev = (score.evidence || {})[r];
                if (!said[pair[0]] && ev) said[pair[0]] = String(Array.isArray(ev) ? ev[0] : ev).split("\n")[0];
              });
          });
        });
        if (Object.keys(size).length)
          gaps.sort(function(a, b) {
            return (size[b[0]] || 0) - (size[a[0]] || 0) || a[0].localeCompare(b[0]);
          });
        gaps.forEach(function(pair) {
          var bases = {};
          (pair[1].gaps || []).forEach(function(g) {
            var m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
            var gsc = m && (D.scorecards || {})[m[1]];
            var score = m && ((D.scores || {})[m[1]] || {})[pair[0]];
            if (gsc && score)
              blockers(gsc, score, m[2]).forEach(function(r) {
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
                ...Object.keys(bases).sort().map(function(b) {
                  return basisBadge(b, true);
                }),
                Object.keys(bases).length ? " " : "",
                (pair[1].gaps || []).map(function(g) {
                  return gapText(D, g, pair[0]);
                }).join("; "),
                said[pair[0]] ? el("div", { className: "sub", style: "font-size:12px", textContent: "Found: " + said[pair[0]] }) : null
              )
            )
          );
        });
      }
      var track = c.track && D.tracks && D.tracks[c.track] ? D.tracks[c.track].title || c.track : c.track;
      var picker = el(
        "select",
        {
          ariaLabel: "Campaign",
          onchange: function() {
            location.hash = "campaign/" + enc(picker.value);
          }
        },
        ...names.map(function(n) {
          var cn = all[n];
          return el("option", { value: n, selected: n === selected, textContent: (cn.title || n) + " (" + (campaignState(cn) === "empty" ? goalWords(includeProgress(D, cn)) : (cn.percent || 0) + "%") + ")" });
        })
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
            ...names.map(function(n) {
              return self.campaignCard(D, n, n === selected);
            })
          )
        ),
        el(
          "section",
          {},
          el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Campaign" }), picker),
          el(
            "div",
            { className: "ehead" },
            el("div", {}, el("h2", { textContent: c.title || selected }), el("div", { className: "sub" }, c.status || "", track ? el("span", {}, " · track ", link(trackPath(D, c.track), "", track)) : null, c.owner ? " · owned by " + c.owner.replace(/^group:/, "") : null)),
            goalBar(state, pct, target, true, includeProgress(D, c))
          ),
          c.description ? el("p", { className: "lede" }, ...rich(c.description)) : null,
          state === "empty" ? null : el(
            "p",
            { style: "margin:8px 0 6px" },
            el("b", { textContent: (c.meeting || 0) + " of " + (c.inScope || 0) }),
            " entities meet the goal · target " + target + "%" + (c.end ? " by " + c.end : "") + (days != null && days >= 0 ? " (" + plural(days, "day") + " left)" : "") + ". The bar turns green once it passes the target tick."
          ),
          state === "empty" ? null : sinceLine(c),
          state === "empty" ? null : bar,
          state === "empty" ? null : marks,
          el("h3", { textContent: "Goal" }),
          includes,
          el("h3", { textContent: state === "empty" ? "Progress" : "Not meeting yet" + (gaps.length ? " (" + gaps.length + ")" : "") + (Object.keys(size || {}).length ? ", furthest first" : "") }),
          gapList
        )
      );
    }
  };

  // src/views/catalog.js
  var catalogViews = {
    catalog(D, S, kind, query) {
      query = query || {};
      var catalog = D.catalog || {};
      var groups = kindGroupsOf(D);
      var kinds = [].concat.apply(
        [],
        groups.map(function(g) {
          return g.kinds;
        })
      );
      if (!kinds.length) return el("section", {}, el("div", { className: "empty", textContent: "Catalog is empty in this report." }));
      var selected = kind && catalog[kind] ? kind : null;
      var scored = scoredKinds(D);
      var compared = {};
      Object.keys(D.comparisons || {}).forEach(function(n) {
        var k = ((D.scorecards || {})[n] || {}).kind;
        if (k) compared[k] = true;
      });
      var groupOf = {};
      groups.forEach(function(g) {
        g.kinds.forEach(function(k) {
          groupOf[k] = g;
        });
      });
      var tile = function(k) {
        var ids = catalog[k];
        var unscored = scored[k] ? ids.filter(function(id) {
          return !S.worst[id];
        }).length : 0;
        var a = link("catalog/" + enc(k), "kind", [
          el("strong", {}, kindTitle(D, k)),
          kindDescription(D, k) ? el("small", {}, ...rich(firstSentence(kindDescription(D, k)))) : null,
          el("span", { textContent: plural(ids.length, "entity", "entities") + scorecardSplit(D, k) + (scored[k] ? unscored ? " · " + fmt(unscored) + " not scored" : "" : compared[k] ? " · compared" : " · not scored") }),
          unscored * 10 > ids.length ? unscoredWhy(kindDescription(D, k)) : null
        ]);
        if (kindDescription(D, k)) a.title = kindDescription(D, k).replace(/`/g, "");
        if (scored[k]) {
          var parts = kindParts(S, ids);
          if (parts.length) a.append(barEl(parts, "xs"));
          var byLevel = {};
          parts.forEach(function(p) {
            if (p.level !== "unscored") byLevel[p.level] = (byLevel[p.level] || 0) + p.n;
          });
          var mix = Object.keys(byLevel).map(function(l) {
            return fmt(byLevel[l]) + " " + ownLevel(D, k, l);
          });
          if (mix.length) a.append(el("small", { className: "kmix", textContent: mix.join(" · "), title: "Each entity counted at its weakest level" }));
        }
        if (k === selected) a.setAttribute("aria-current", "true");
        return a;
      };
      var grid = el(
        "div",
        {},
        ...groups.map(function(g) {
          return el(
            "div",
            { className: "kgroup" },
            el("div", { className: "kgh" }, el("h3", { textContent: g.title }), el("span", { className: "sub", textContent: plural(g.count, "entity", "entities") + (g.kinds.length > 1 ? ": " + g.kinds.map(function(k) {
              return fmt(((D.catalog || {})[k] || []).length) + " " + kindTitle(D, k);
            }).join(" · ") : "") })),
            g.description ? el("p", { className: "sub kgd", textContent: g.description }) : null,
            el("div", { className: "kinds" }, ...g.kinds.map(tile))
          );
        })
      );
      var top = el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "Catalog" }), el("span", { className: "sub", textContent: fmt(S.entityCount) + " entities in " + kinds.length + " kinds and " + plural(groups.length, "group") })),
        el("p", { className: "lede", textContent: "Everything the portal derived from this repository, grouped by what it is about. The bar under a kind shows each entity's lowest level across the scorecards that rate it; pick a kind to list its entities." }),
        el(
          "div",
          { className: "legend", style: "margin:0 0 12px" },
          ...["none", "bronze", "silver", "gold", "unscored"].map(function(l) {
            return el("span", {}, l === "unscored" ? el("span", { className: "dot unscored" }) : el("span", { className: "dot", style: "background:" + fill(l) }), l === "unscored" ? "not scored: out of every scorecard's scope, or none has run on it" : l);
          }),
          [...S.worstState.values()].indexOf("hollow") >= 0 ? el("span", {}, el("span", { className: "dot hatch", style: "--seg:" + fill("bronze") }), "hatched: not assessed by an agent yet") : null,
          [...S.worstState.values()].indexOf("unrun") >= 0 ? el("span", {}, el("span", { className: "dot dots", style: "--seg:" + fill("silver") }), "◌ dotted: provisional, checks not run in this build") : null
        ),
        grid
      );
      if (!selected) return top;
      var search = el("input", { type: "search", placeholder: "Filter " + selected + " by name…", ariaLabel: "Filter entities", value: query.q || "" });
      var list = el("div");
      var limit = 60;
      var drawList = function() {
        var q = search.value.toLowerCase();
        var ids = (catalog[selected] || []).filter(function(id) {
          return !q || id.toLowerCase().indexOf(q) >= 0;
        });
        if (!ids.length) {
          list.replaceChildren(
            el("div", { className: "empty", textContent: "No entities match “" + search.value + "”." }),
            el("button", {
              className: "link",
              textContent: "Clear filter",
              onclick: function() {
                search.value = "";
                search.oninput();
              }
            })
          );
          return;
        }
        var shown = ids;
        var kindCards = Object.keys(D.scorecards || {}).filter(function(n) {
          return (catalog[selected] || []).some(function(id) {
            return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
          });
        }).sort(function(a, b) {
          return title(D, a).localeCompare(title(D, b));
        });
        var cols = kindCards.slice(0, 4);
        var allUnrun = {};
        cols.forEach(function(n) {
          var inScope = (catalog[selected] || []).filter(function(id) {
            return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
          });
          allUnrun[n] = inScope.length > 1 && inScope.every(function(id) {
            return unrunIn(D.scorecards[n], D.scores[n][id]).length > 0;
          });
        });
        var tpl = "minmax(0,1fr)" + (cols.length ? " repeat(" + cols.length + ",minmax(96px,176px))" : " auto");
        var header = cols.length ? el(
          "div",
          { className: "ehdr", style: "grid-template-columns:" + tpl, ariaHidden: "true" },
          el("span", { textContent: kindTitle(D, selected) }),
          ...cols.map(function(n) {
            var mark = judgedMark(D, n);
            return el("span", { title: n }, title(D, n), mark ? " " : null, mark, allUnrun[n] ? el("span", { className: "unassessed-tag unrun", style: "display:block", title: "Every level in this column rests on checks not run in this build: it holds only if they pass" }, "all provisional: not run") : null);
          })
        ) : null;
        var rowOf = function(id) {
          var short = shortId(id);
          var r = link("entity/" + enc(id), "erow cols", [
            el("div", { className: "name" }, short, short !== id ? el("small", { textContent: id, title: id }) : null),
            ...cols.length ? cols.map(function(n) {
              var sc = D.scores[n] && D.scores[n][id];
              var lv = sc && sc.level !== "skipped" ? sc.level : null;
              var cell = el(
                "div",
                { className: "ecell" },
                lv ? levelPill(lv, unassessedIn(D, n, id), unrunIn(D.scorecards[n], sc), allUnrun[n]) : el("span", { className: "sub", title: "Not in this scorecard's scope", textContent: "not in scope" })
              );
              cell.dataset.sc = title(D, n);
              if (basisCounts(((D.scorecards || {})[n] || {}).rules).judged) cell.dataset.judged = "";
              return cell;
            }) : [el("div", { className: "badges" }, el("span", { className: "sub", textContent: "not scored" }))]
          ]);
          r.style.gridTemplateColumns = tpl;
          var lvls = cols.filter(function(n) {
            return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
          }).map(function(n) {
            return title(D, n) + ": " + D.scores[n][id].level;
          });
          if (lvls.length) {
            r.querySelectorAll(".ecell").forEach(function(c) {
              c.setAttribute("aria-hidden", "true");
            });
            r.append(el("span", { className: "sr", textContent: ". " + lvls.join(", ") }));
          }
          return r;
        };
        var built = shown.map(function(id) {
          var r = rowOf(id);
          return {
            id,
            r,
            sig: Array.prototype.map.call(r.querySelectorAll(".ecell"), function(c) {
              return c.innerHTML;
            }).join("|")
          };
        });
        var sigCount = {};
        built.forEach(function(b) {
          sigCount[b.sig] = (sigCount[b.sig] || 0) + 1;
        });
        var grouped = {};
        var rowsOut = [];
        built.forEach(function(b) {
          if (!cols.length || sigCount[b.sig] < 5) {
            rowsOut.push({ nodes: [b.r], n: 1 });
            return;
          }
          if (grouped[b.sig]) return;
          grouped[b.sig] = true;
          var members = built.filter(function(x) {
            return x.sig === b.sig;
          });
          var sum = el(
            "summary",
            { className: "erow cols", style: "grid-template-columns:" + tpl },
            el("div", { className: "name" }, kindCount(D, selected, members.length) + " with the same result ", el("span", { className: "chev", ariaHidden: "true", textContent: "›" })),
            ...Array.prototype.map.call(b.r.querySelectorAll(".ecell"), function(c) {
              return c.cloneNode(true);
            })
          );
          rowsOut.push({
            nodes: [
              el("details", { className: "egroup" }, sum, el("div", { className: "egroup-members" }, ...members.map(function(x) {
                return x.r;
              }))),
              memberLinks(members.map(function(x) {
                return x.id;
              }))
            ],
            n: members.length
          });
        });
        var paged = rowsOut.slice(0, limit);
        var hidden = rowsOut.slice(paged.length).reduce(function(n, o) {
          return n + o.n;
        }, 0);
        list.replaceChildren.apply(
          list,
          (header ? [header] : []).concat(
            paged.flatMap(function(o) {
              return o.nodes;
            })
          )
        );
        var judgedCols = cols.filter(function(n) {
          return basisCounts(((D.scorecards || {})[n] || {}).rules).judged > 0;
        });
        var unrunCols = cols.filter(function(n) {
          return allUnrun[n];
        });
        var dashed = unrunCols.length ? "Every level on the " + unrunCols.map(function(n) {
          return title(D, n);
        }).join(" and ") + (unrunCols.length > 1 ? " scorecards" : " scorecard") + " is provisional here: checks it needs were not run in this build." : "";
        var hollowJ = list.querySelector(".lvl.vacuous.unjudged");
        var hollowR = list.querySelector(".lvl.vacuous.unrun");
        if ((hollowJ || hollowR) && header)
          header.after(
            el(
              "div",
              { className: "hollow-key" },
              el("b", { textContent: "Levels that may not hold:" }),
              hollowJ ? el("span", {}, levelPill("bronze", true, null).firstChild, " no agent has reviewed it yet") : null,
              hollowR ? el("span", {}, levelPill("bronze", false, ["a check"], true).firstChild, " checks it needs were not run in this build") : null
            )
          );
        if (judgedCols.length || dashed)
          list.prepend(
            el(
              "div",
              { className: "sub basis-note" },
              judgedCols.length ? el(
                "p",
                {},
                el("b", { textContent: "How it's decided: " }),
                ...cols.flatMap(function(n, i) {
                  var bc = basisCounts(((D.scorecards || {})[n] || {}).rules);
                  var all = bc.measured + bc.judged;
                  return [i ? " · " : "", title(D, n) + " ", basisBadge(bc.judged ? "judged" : "measured"), " " + (bc.judged === all || !bc.judged ? "all " + plural(all, "rule") : bc.judged + " of " + plural(all, "rule") + ", the rest tool-checked")];
                })
              ) : null,
              dashed ? el("p", {}, el("b", { textContent: "Has it run: " }), dashed) : null
            )
          );
        if (kindCards.length > cols.length)
          list.append(el("p", { className: "sub", textContent: plural(kindCards.length - cols.length, "more scorecard") + " rate " + selected + "; open an entity to see them." }));
        if (hidden) {
          list.append(
            el(
              "div",
              { className: "more" },
              el("button", {
                className: "link",
                textContent: "Show all " + fmt(ids.length) + " (" + fmt(hidden) + " more)",
                onclick: function() {
                  limit = Infinity;
                  drawList();
                }
              })
            )
          );
        }
      };
      search.oninput = function() {
        var want = "#catalog/" + enc(selected) + (search.value ? "?" + new URLSearchParams({ q: search.value }) : "");
        if (location.hash !== want && history.replaceState) history.replaceState(history.state, "", want);
        drawList();
      };
      drawList();
      var levels = (scored[selected] ? kindParts(S, catalog[selected] || []) : []).map(function(p) {
        return Object.assign({}, p, { label: ownLevel(D, selected, p.level) });
      });
      var nScored = (catalog[selected] || []).filter(function(id) {
        return S.worst[id];
      }).length;
      var nAll = (catalog[selected] || []).length;
      var kpick = el(
        "select",
        {
          ariaLabel: "Kind",
          onchange: function() {
            location.hash = "catalog/" + enc(kpick.value);
          }
        },
        ...groups.map(function(g) {
          return el(
            "optgroup",
            { label: g.title },
            ...g.kinds.map(function(k) {
              return el("option", { value: k, selected: k === selected, textContent: kindTitle(D, k) + " (" + fmt((catalog[k] || []).length) + ")" });
            })
          );
        })
      );
      var listSection = el(
        "section",
        { id: "kind-list" },
        el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Kind" }), kpick),
        el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("catalog", "", "Catalog"), " / " + (groupOf[selected] && groupOf[selected].name ? groupOf[selected].title + " / " : "") + kindTitle(D, selected)),
        el("div", { className: "head" }, el("div", {}, el("h2", { textContent: kindTitle(D, selected), tabIndex: -1 }), kindTitle(D, selected) !== selected ? el("code", { className: "kid", textContent: selected }) : null), el("span", { className: "sub", textContent: (groupOf[selected] && groupOf[selected].name ? groupOf[selected].title + " · " : "") + plural(nAll, "entity", "entities") + scorecardSplit(D, selected) + (scored[selected] && nScored < nAll ? " · " + fmt(nScored) + " scored · " + fmt(nAll - nScored) + " not scored" : "") })),
        kindDescription(D, selected) ? el("p", { className: "lede" }, ...rich(kindDescription(D, selected))) : null,
        scored[selected] && nScored < nAll ? this.scopeNote(D, selected, nAll - nScored) : null,
        levels.length ? el("div", {}, barEl(levels), legendEl(levels)) : compared[selected] ? el("p", { className: "sub" }, "Compared rather than levelled: ", link("compare", "", "see how they stand →")) : el("p", { className: "sub", textContent: "No scorecard rates this kind." }),
        search,
        list
      );
      listSection.dataset.focus = "";
      var chips = el(
        "section",
        { className: "kchips-wrap" },
        el("div", { className: "head" }, el("h2", { textContent: "Catalog" }), el("span", { className: "sub" }, fmt(S.entityCount) + " entities in " + kinds.length + " kinds · ", link("catalog", "", "all kinds, with descriptions"))),
        ...groups.map(function(g) {
          return el(
            "div",
            { className: "kchips" },
            el("span", { className: "kchips-g", textContent: g.title }),
            el(
              "div",
              { className: "box" },
              ...g.kinds.map(function(k) {
                var a = link("catalog/" + enc(k), "", [kindTitle(D, k), el("b", { textContent: fmt((catalog[k] || []).length) })]);
                if (k === selected) a.setAttribute("aria-current", "true");
                return a;
              })
            )
          );
        })
      );
      return el("div", {}, chips, listSection);
    },
    // Why some entities of a rated kind carry no level: each scorecard on the kind that limits itself with appliesTo.
    scopeNote(D, kind, n) {
      var scoped = Object.entries(D.scorecards || {}).filter(function(e) {
        return e[1].kind === kind && e[1].appliesTo;
      });
      var who = plural(n, "entity is", "entities are");
      if (!scoped.length) return el("p", { className: "sub", textContent: fmt(n) + " " + who.replace(/^\d+ /, "") + " not scored: no scorecard on this kind has produced a level for them in this build." });
      return el(
        "p",
        { className: "sub scope-note" },
        el("b", { textContent: who.replace(/^\d+/, fmt(n)) + " not scored" }),
        ": they fall outside the scope that ",
        ...scoped.flatMap(function(e, i) {
          return [i ? i === scoped.length - 1 ? " and " : ", " : "", link("scorecard/" + enc(e[0]), "", e[1].title || e[0])];
        }),
        " sets for itself. ",
        el(
          "details",
          { className: "scope-rule" },
          el("summary", { textContent: "Show the scope rule" }),
          ...scoped.map(function(e) {
            return el("div", {}, el("span", { textContent: (e[1].title || e[0]) + " applies where " }), el("code", { textContent: e[1].appliesTo }));
          })
        )
      );
    }
  };

  // src/views/entity.js
  var entityViews = {
    rulesFor(sc, score, scName, unjudged, id) {
      var wrap = el("div");
      var kind = String(id || "").split(":")[0];
      var fixFor = function(text) {
        return id && kind ? String(text).split(kind + ":<name>").join(id) : text;
      };
      var rules = Object.entries(score.rules || {});
      var rank = function(o) {
        return o === "fail" ? 0 : o === "unknown" ? 1 : o === "waived" ? 2 : o === "skipped" ? 3 : 4;
      };
      var order = sc.levels || [];
      rules.sort(function(a, b) {
        var la = order.indexOf(((sc.rules || {})[a[0]] || {}).level);
        var lb = order.indexOf(((sc.rules || {})[b[0]] || {}).level);
        return rank(a[1]) - rank(b[1]) || la - lb || a[0].localeCompare(b[0]);
      });
      var passed = {};
      var npass = 0;
      var seen = {};
      var nextIdx = order.indexOf(score.level) + 1;
      var later = [];
      var laterLevels = /* @__PURE__ */ new Set();
      rules.forEach(function(ro) {
        var n = ro[0];
        var o = ro[1];
        var rule = sc.rules && sc.rules[n] || {};
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
                el("span", { className: "outcome unassessed", style: "margin:0", textContent: "passes, not assessed yet" }),
                basisBadge("judged"),
                rule.level ? el("span", { className: "sub", textContent: "counts for " + rule.level }) : ""
              ),
              el("div", { className: "fix" }, "No agent has assessed this entity, so there is nothing for this rule to fail on: it passes because nothing has been looked at, not because a review found nothing."),
              whyEl(rule)
            )
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
                rule.rationale ? el("span", { className: "rwhy", textContent: "Why: " + firstSentence(rule.rationale).replace(/`/g, "") }) : null
              ),
              scName ? el("div", { className: "sub" }, "Rule page: ", ruleLink(scName, n, null, true), " ", el("code", { className: "rid", textContent: n })) : null,
              rule.fix && o !== "unknown" ? el("div", { className: "fix" }, el("b", { textContent: "Fix: " }), ...rich(fixFor(rule.fix))) : "",
              whyEl(rule),
              evidenceEl(items, seen, scName ? "“" + ruleTitle(scName, n) + "”" : n)
            )
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
              rule.level ? el("span", { className: "sub", textContent: "needed for " + rule.level }) : ""
            ),
            rule.fix && o !== "unknown" ? el("div", { className: "fix" }, el("b", { textContent: "Fix: " }), ...rich(fixFor(rule.fix))) : "",
            whyEl(rule),
            evidenceEl(items, seen, scName ? "“" + ruleTitle(scName, n) + "”" : n)
          )
        );
      });
      if (later.length) {
        var further = order.filter(function(lv) {
          return laterLevels.has(lv);
        });
        wrap.append(el("div", { className: "later-h", textContent: "Further up: " + plural(later.length, "more rule") + " needed for " + further.join(" and ") + ". Open one for its fix and evidence." }), ...later);
      }
      if (npass) {
        wrap.append(el("div", { className: "sub", style: "margin-top:10px", textContent: plural(npass, "check passes", "checks pass") }));
        order.concat([""]).forEach(function(lv) {
          if (!passed[lv]) return;
          var all = Object.keys(sc.rules || {}).filter(function(n) {
            return (sc.rules[n].level || "") === lv && score.rules[n] !== "skipped";
          }).length;
          wrap.append(
            el(
              "div",
              { className: "passes" },
              lv ? badge(lv) : null,
              el("em", { textContent: passed[lv].length === all ? "all " + all + " met" : passed[lv].length + " of " + all }),
              ...passed[lv]
            )
          );
        });
      }
      return wrap;
    },
    nextStep(sc, score) {
      var order = sc.levels || [];
      var here = order.indexOf(score.level);
      var unsure = Object.keys(score.rules || {}).filter(function(n) {
        return score.rules[n] === "unknown" && order.indexOf(((sc.rules || {})[n] || {}).level) <= here;
      }).sort();
      var lowUnrun = unsure.map(function(n) {
        return ((sc.rules || {})[n] || {}).level;
      }).sort(function(a, b) {
        return order.indexOf(a) - order.indexOf(b);
      })[0];
      var caveat = unsure.length ? el(
        "div",
        { className: "callout", style: "background:color-mix(in srgb,var(--unknown) 12%,var(--card))" },
        el("span", {}, el("b", { className: "provisional", textContent: "Provisional " + score.level + ". " }), unrunWords(sc, unsure) + " " + levelName(score.level).charAt(0).toUpperCase() + levelName(score.level).slice(1) + " holds only if " + (unsure.length === 1 ? "it passes" : "they pass") + (lowUnrun && lowUnrun !== score.level ? ", and so does " + levelName(lowUnrun) : "") + ".")
      ) : null;
      var next = nextLevel(sc, score.level);
      if (!next) return el("div", {}, caveat, el("div", { className: "callout done" }, "✓ Top level reached: every rule passes."));
      var need = blockers(sc, score, next).filter(function(n) {
        return unsure.indexOf(n) < 0;
      });
      if (!need.length) return caveat;
      if (need.length === 1)
        return el("div", {}, caveat, el("div", { className: "callout near" }, el("span", {}, el("b", { textContent: "One rule from " }), badge(next), " · pass “" + ruleTitleOf(sc, need[0]) + "” ", basisBadge(basisOf((sc.rules || {})[need[0]]), "tip"), " and this entity moves up a level.")));
      return el("div", {}, caveat, el("div", { className: "callout" }, el("span", {}, el("b", { textContent: "Next: " }), badge(next), " needs: ", ...need.map(function(n, i) {
        return el("span", { className: "nr" }, ...keepWithLast(ruleTitleOf(sc, n), basisBadge(basisOf((sc.rules || {})[n]), "tip")), i < need.length - 1 ? ", " : "");
      }), ".")));
    },
    entity(D, S, id) {
      var kind = S.kindOf[id];
      var self = this;
      var cards = Object.keys(D.scorecards || {}).filter(function(name) {
        return D.scores[name] && D.scores[name][id];
      });
      var total = { pass: 0, hollow: 0, all: 0 };
      var bases = { measured: 0, judged: 0 };
      cards.forEach(function(n) {
        if (isComparative(D, n)) return;
        var rules = D.scores[n][id].rules || {};
        var unread = unassessedIn(D, n, id);
        Object.keys(rules).forEach(function(r) {
          var o = rules[r];
          if (o === "skipped") return;
          total.all++;
          bases[basisOf((D.scorecards[n].rules || {})[r])]++;
          if (isPassing(o) && unread && basisOf((D.scorecards[n].rules || {})[r]) === "judged") total.hollow++;
          else if (isPassing(o)) total.pass++;
        });
      });
      var judgedAt = [];
      var blocks = cards.map(function(name) {
        var sc = D.scorecards[name];
        var score = D.scores[name][id];
        if (isComparative(D, name)) {
          var c = D.comparisons[name];
          var peer = (c.peers || [])[0];
          var v = (c.verdict || {})[peer];
          var whoSubject = (c.titles || {})[c.subject] || shortId(c.subject);
          var whoPeer = (c.titles || {})[peer] || shortId(peer);
          return el(
            "div",
            { className: "sc-block" },
            el("div", { className: "head" }, el("div", {}, el("h3", { className: "sct" }, link("compare/" + enc(name), "", title(D, name))), " ", standingChip(v, whoSubject + " " + (v || "unknown"))), el("span", { className: "sub", textContent: "comparison · " + whoSubject + " vs " + whoPeer }))
          );
        }
        var here = { measured: 0, judged: 0 };
        var judgedHere = [];
        Object.keys(score.rules || {}).forEach(function(r) {
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
            el("span", { className: "sub" }, name + " · ", here.measured ? el("span", {}, basisIcon("measured"), " " + here.measured + " measured") : null, here.measured && here.judged ? " · " : "", here.judged ? el("span", { className: "judged-n" }, basisIcon("judged"), " " + here.judged + " agent-judged") : null)
          ),
          score.level === "skipped" ? el("div", { className: "sub", textContent: "Not in this scorecard's scope." }) : self.nextStep(sc, score),
          self.rulesFor(sc, score, name, unassessedIn(D, name, id), id)
        );
        if (judgedHere.length && score.level !== "skipped") judgedAt.push({ name, rules: judgedHere, block });
        return block;
      });
      var judgedList = judgedAt.length ? el(
        "p",
        { className: "sub judged-list" },
        el("b", { textContent: "Agent-judged here: " }),
        ...judgedAt.map(function(j, i) {
          return el(
            "span",
            {},
            i ? "; " : "",
            el("button", {
              className: "link",
              textContent: title(D, j.name),
              onclick: function() {
                j.block.scrollIntoView({ behavior: "smooth", block: "start" });
                var h = j.block.querySelector("h3 a");
                if (h) h.focus({ preventScroll: true });
              }
            }),
            " (" + j.rules.map(function(r) {
              return ruleTitle(j.name, r);
            }).join(", ") + ")"
          );
        })
      ) : null;
      var short = shortId(id);
      var notRun = 0;
      cards.forEach(function(n) {
        var sc = D.scorecards[n];
        var score = D.scores[n][id];
        if (isComparative(D, n) || score.level === "skipped") return;
        var here = (sc.levels || []).indexOf(S.worst[id]);
        if (here < 0) return;
        notRun += Object.keys(score.rules || {}).filter(function(r) {
          return score.rules[r] === "unknown" && (sc.levels || []).indexOf(((sc.rules || {})[r] || {}).level) <= here;
        }).length;
      });
      var unassessedCards = cards.filter(function(n) {
        return !isComparative(D, n) && D.scores[n][id].level !== "skipped" && unassessedIn(D, n, id);
      }).length;
      var atTop = cards.every(function(n) {
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
          S.worst[id] ? el(
            "div",
            { className: "elevel" },
            medal(S.worst[id], 52),
            el(
              "div",
              {},
              el("strong", { style: "color:" + ink(S.worst[id]) + (KNOWN.indexOf(ownLevel(D, kind, S.worst[id])) < 0 ? ";text-transform:none" : "") }, ownLevel(D, kind, S.worst[id]), atTop ? el("span", { className: "topmark", textContent: " ✓ top" + (KNOWN.indexOf(S.worst[id]) < 0 ? " level" : "") }) : null, provisional.length ? el("span", { className: "pv-head", textContent: notRun ? " (provisional)" : " (may not hold)" }) : null),
              provisional.length ? el("span", { className: "provisional", style: "display:block;font-size:13px", title: "The level holds only if the checks not run in this build pass and an agent's assessment finds nothing; the scorecards below name them.", textContent: provisional.join(", ") }) : null,
              el("span", { className: "sub", textContent: (cards.length > 1 ? "lowest of " + cards.length + " scorecards · " : "") + total.pass + " of " + plural(total.all, "check passes", "checks pass") + " on evidence" + (total.hollow ? " · " + total.hollow + " more " + (total.hollow === 1 ? "passes" : "pass") + " by default, because no agent has reviewed " + (total.hollow === 1 ? "it" : "them") + " yet" : "") })
            )
          ) : null
        ),
        total.all ? basisKey(bases, "check") : null,
        judgedList,
        cards.length ? el("div", {}, ...blocks) : el("div", { className: "empty", textContent: "Not scored by any scorecard in this snapshot." })
      );
    }
  };

  // src/views/rule.js
  var ruleViews = {
    rule(D, S, scName, ruleName) {
      var sc = D.scorecards[scName];
      var def = sc.rules[ruleName];
      var b = basisOf(def);
      var scores = (D.scores || {})[scName] || {};
      var tally = { pass: 0, waived: 0, fail: 0, unknown: 0 };
      var failing = [];
      Object.keys(scores).forEach(function(id) {
        var row = scores[id];
        var o = (row.rules || {})[ruleName];
        if (!o || o === "skipped" || row.level === "skipped") return;
        tally[o] = (tally[o] || 0) + 1;
        if (!isPassing(o)) failing.push(id);
      });
      var measureOf = function(id) {
        var m = (scores[id].measures || {})[ruleName];
        return typeof m === "number" ? m : null;
      };
      var measured = failing.some(function(id) {
        return measureOf(id) != null;
      });
      failing.sort(function(a, c) {
        return (scores[a].rules[ruleName] === "fail" ? 0 : 1) - (scores[c].rules[ruleName] === "fail" ? 0 : 1) || (measureOf(c) || 0) - (measureOf(a) || 0) || a.localeCompare(c);
      });
      var allFail = failing.every(function(id) {
        return scores[id].rules[ruleName] === "fail";
      });
      var all = tally.pass + tally.waived + tally.fail + tally.unknown;
      var parts = [
        { k: "pass", n: tally.pass + tally.waived, c: def.level ? fill(def.level) : "var(--pass)", t: def.level ? "pass, toward " + def.level : "pass" },
        { k: "fail", n: tally.fail, c: "var(--fail)", t: "fail" },
        { k: "unknown", n: tally.unknown, c: UNRUN, t: "not run in this build" }
      ].filter(function(p) {
        return p.n;
      });
      var spans = parts.map(function(p) {
        return el("span", { style: "flex-grow:0;flex-basis:0;background:" + p.c, title: p.n + " " + p.t });
      });
      grow(
        spans,
        parts.map(function(p) {
          return p.n;
        })
      );
      var bar = el("div", { className: "bar lg" }, ...spans);
      bar.setAttribute("role", "img");
      bar.setAttribute(
        "aria-label",
        parts.map(function(p) {
          return p.n + " " + p.t;
        }).join(", ")
      );
      var list = el("div");
      var limit = 40;
      var drawList = function() {
        list.replaceChildren(
          ...failing.slice(0, limit).map(function(id) {
            var o = scores[id].rules[ruleName];
            var ev = (scores[id].evidence || {})[ruleName];
            var items = ev == null ? [] : Array.isArray(ev) ? ev : [ev];
            return link("entity/" + enc(id), "erow", [
              el("div", { className: "name" }, id.indexOf(":") > 0 ? id.slice(id.indexOf(":") + 1) : shortId(id), el("span", { className: "chev", ariaHidden: "true", textContent: "›" }), el("small", { textContent: items.length ? (typeof items[0] === "string" ? items[0] : JSON.stringify(items[0])) + (items.length > 1 ? " (+" + (items.length - 1) + ")" : "") : id })),
              el("span", { className: "outcome " + o, textContent: o === "unknown" ? "not run in this build" : o })
            ]);
          })
        );
        if (failing.length > limit)
          list.append(
            el(
              "div",
              { className: "more" },
              el("button", {
                className: "link",
                textContent: "Show all " + failing.length,
                onclick: function() {
                  limit = Infinity;
                  drawList();
                }
              })
            )
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
          basisBadge(b)
        ),
        def.rationale || (def.links || []).length ? el(
          "div",
          { className: "rationale" },
          el("b", { textContent: "Why it exists" }),
          def.rationale ? el("div", {}, ...rich(def.rationale)) : null,
          refs(def.links),
          !(def.links || []).length && (sc.links || []).length ? el("div", { className: "sub", style: "margin-top:6px" }, "Read more, from the ", link("scorecard/" + enc(scName), "", title(D, scName)), " scorecard:") : null,
          !(def.links || []).length ? refs(sc.links) : null
        ) : el("p", { className: "sub", textContent: "This rule has no rationale yet." }),
        el("h3", { textContent: "How it is decided" }),
        el("p", { style: "margin:0" }, basisBadge(b), " ", BASIS[b].desc),
        el("div", { className: "cel", textContent: def.check }),
        def.fix ? el("h3", { textContent: "How to fix a failure" }) : null,
        def.fix ? el("p", { style: "margin:0" }, ...rich(def.fix)) : null,
        el("h3", { textContent: "Outcomes" }),
        all && def.level ? el("p", { className: "sub", style: "margin:0 0 6px", textContent: "A pass is drawn in the colour of " + def.level + ", the level this rule belongs to, as on the scorecard's bars." }) : null,
        all ? bar : el("p", { className: "empty", textContent: "No entity is in scope." }),
        all ? el(
          "div",
          { className: "legend" },
          ...parts.map(function(p) {
            return el("span", {}, el("span", { className: "dot", style: "background:" + p.c }), el("b", { textContent: fmt(p.n) }), " " + p.t);
          })
        ) : null
      );
      section.dataset.focus = "";
      return el(
        "div",
        {},
        section,
        failing.length ? el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Not passing" }), el("span", { className: "sub", textContent: plural(failing.length, "entity", "entities") + (measured ? ", furthest off first" : allFail ? "" : ", failures first") })), list) : all ? el("section", {}, el("p", { className: "empty good", textContent: "✓ Every entity in scope passes this rule." })) : null
      );
    }
  };

  // src/views/compare.js
  function tallyPhrase(n, total, key) {
    if (n.unknown === total) return "Not measured: none of " + plural(total, "rule") + " has values";
    if (n.behind && n.ahead) return n.ahead + " ahead, " + n.behind + " behind, of " + plural(total, "rule");
    return (n[key] || 0) + " of " + plural(total, "rule") + " " + (key === "parity" ? "at parity" : key);
  }
  var compareViews = {
    factorCards(D, names, current) {
      var all = D.comparisons || {};
      var order = { behind: 0, unknown: 1, parity: 2, ahead: 3 };
      var verdictOf = function(n) {
        var c = all[n];
        return (c.verdict || {})[(c.peers || [])[0]] || "unknown";
      };
      return el(
        "div",
        { className: "factors" },
        ...names.slice().sort(function(a, b) {
          return order[verdictOf(a)] - order[verdictOf(b)] || title(D, a).localeCompare(title(D, b));
        }).map(function(n) {
          var c = all[n];
          var def = (D.scorecards || {})[n] || {};
          var peer = (c.peers || [])[0];
          var v = verdictOf(n);
          var counts = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
          Object.keys(c.rules || {}).forEach(function(r) {
            var x = (c.rules[r] || {})[peer] || "unknown";
            counts[x] = (counts[x] || 0) + 1;
          });
          var segs = ["ahead", "parity", "behind", "unknown"].filter(function(k) {
            return counts[k];
          });
          var spans = segs.map(function(k) {
            return el("span", { style: "flex-grow:0;flex-basis:0;background:" + (k === "unknown" ? UNSCORED : standingColor(k)) });
          });
          grow(
            spans,
            segs.map(function(k) {
              return counts[k];
            })
          );
          var bar = el("div", { className: "bar xs" }, ...spans);
          bar.setAttribute("role", "img");
          var tallyText = segs.map(function(k) {
            return counts[k] + " " + k;
          }).join(" · ");
          bar.setAttribute("aria-label", "Rules: " + tallyText);
          var best = null;
          Object.keys(c.rules || {}).forEach(function(r) {
            var rd = (def.rules || {})[r] || {};
            if ((c.rules[r] || {})[peer] !== "ahead" || !rd.better) return;
            var x = ruleValue(c, r, c.subject);
            var y = ruleValue(c, r, peer);
            if (typeof x !== "number" || typeof y !== "number" || x <= 0 || y <= 0) return;
            var ratio = Math.max(x, y) / Math.min(x, y);
            if (!best || ratio > best.ratio) best = { ratio, r, word: cmpWord(rd, rd.better === "lower") };
          });
          var total = Object.keys(c.rules || {}).length;
          var fxKey = counts.behind ? "behind" : counts.ahead ? "ahead" : "parity";
          var fx = total ? el(
            "div",
            { className: "fx" },
            el("b", { className: counts.unknown === total ? "unmeasured" : counts.behind && counts.ahead ? "mixed" : "", textContent: tallyPhrase(counts, total, fxKey) }),
            best ? el("span", { textContent: "widest lead: " + lowerFirst(measureLabel(c, best.r)) + ", " + (best.ratio >= 10 ? Math.round(best.ratio).toLocaleString("en-US") : best.ratio.toFixed(1)) + "× " + best.word }) : null
          ) : null;
          var a = link("compare/" + enc(n), "factor", [
            el("div", { className: "fh" }, el("span", { textContent: def.title || n }), standingChip(v, factorChip(v, counts, total))),
            fx,
            def.rationale || def.description ? el("p", {}, ...rich(firstSentence(def.rationale || def.description))) : null,
            el("div", { className: "fbar" }, bar, el("span", { className: "fc", textContent: tallyText }))
          ]);
          a.style.setProperty("--st", standingColor(v));
          if (n === current) a.setAttribute("aria-current", "true");
          return a;
        })
      );
    },
    // One row per factor: the share of its rules behind grows left of the axis, ahead grows right; parity is the gap.
    headToHead(D, all, names, peer, current) {
      var rows = names.map(function(n) {
        var c = { ahead: 0, parity: 0, behind: 0, total: 0 };
        Object.keys(all[n].rules || {}).forEach(function(r) {
          var v = (all[n].rules[r] || {})[peer] || "unknown";
          if (v in c) c[v]++;
          c.total++;
        });
        return { n, c };
      }).sort(function(a, b) {
        return b.c.behind / Math.max(1, b.c.total) - a.c.behind / Math.max(1, a.c.total) || a.c.ahead / Math.max(1, a.c.total) - b.c.ahead / Math.max(1, b.c.total);
      });
      var pct = function(k, c) {
        return Math.round(100 * c[k] / Math.max(1, c.total));
      };
      var box = el(
        "div",
        { className: "h2h" },
        el("div", { className: "h2h-head", ariaHidden: "true" }, el("span", {}), el("span", { className: "behind", textContent: STANDING_MARK.behind + " behind" }), el("span", { className: "ahead", textContent: "ahead " + STANDING_MARK.ahead })),
        ...rows.map(function(r) {
          var a = link("compare/" + enc(r.n), "h2h-row", [
            el("span", { className: "h2h-name", textContent: title(D, r.n) }),
            el("span", { className: "h2h-l" }, el("span", { style: "width:" + pct("behind", r.c) + "%" })),
            el("span", { className: "h2h-r" }, el("span", { style: "width:" + pct("ahead", r.c) + "%" }))
          ]);
          a.setAttribute("aria-label", title(D, r.n) + ": " + r.c.behind + " behind, " + r.c.parity + " at parity, " + r.c.ahead + " ahead, of " + r.c.total + " rules");
          a.title = r.c.behind + " behind · " + r.c.parity + " at parity · " + r.c.ahead + " ahead, of " + r.c.total + " rules";
          if (r.n === current) a.setAttribute("aria-current", "true");
          return a;
        }),
        el("div", { className: "h2h-foot", textContent: "Share of each factor's rules; the gap is parity" })
      );
      return box;
    },
    compare(D, name) {
      var all = D.comparisons || {};
      var names = Object.keys(all).sort();
      if (!names.length) return el("section", {}, el("div", { className: "empty", textContent: "No comparative scorecards in this portal." }));
      var first = all[names[0]];
      var subject = first.subject;
      var peer = (first.peers || [])[0];
      var who = function(id) {
        for (var i = 0; i < names.length; i++) {
          var t = (all[names[i]].titles || {})[id];
          if (t && t !== id) return t;
        }
        return shortId(id);
      };
      var verdictOf = function(n) {
        return (all[n].verdict || {})[peer] || "unknown";
      };
      var counts = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
      names.forEach(function(n) {
        counts[verdictOf(n)]++;
      });
      if (name) return this.compareDetail(D, name, names, who);
      var behindNames = names.filter(function(n) {
        return verdictOf(n) === "behind";
      });
      var ruleCounts = { ahead: 0, parity: 0, behind: 0 };
      var ruleTotal = 0;
      var behindRules = {};
      names.forEach(function(n) {
        var rs = all[n].rules || {};
        behindRules[n] = 0;
        Object.keys(rs).forEach(function(r) {
          var v = (rs[r] || {})[peer];
          ruleTotal++;
          if (v in ruleCounts) ruleCounts[v]++;
          if (v === "behind") behindRules[n]++;
        });
      });
      var unmeasured = counts.unknown >= names.length;
      var majority = byMajority(D, names);
      var other = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
      names.forEach(function(n) {
        var rs = all[n].rules || {};
        var k = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
        Object.keys(rs).forEach(function(r) {
          var v = (rs[r] || {})[peer];
          k[v in k ? v : "unknown"]++;
        });
        var s = majority ? k.behind ? "behind" : k.unknown ? "unknown" : k.ahead ? "ahead" : "parity" : k.unknown ? "unknown" : k.ahead > k.behind ? "ahead" : k.behind > k.ahead ? "behind" : "parity";
        other[s]++;
      });
      var otherDiffers = ["ahead", "parity", "behind"].some(function(s) {
        return other[s] !== counts[s];
      });
      var headline = unmeasured ? "Not measured yet: none of the " + plural(names.length, "factor") + " has values for both products." : "Ahead on " + counts.ahead + " of " + plural(names.length, "factor") + (counts.behind ? ", behind on " + counts.behind : "") + (counts.parity ? ", at parity on " + counts.parity : "") + (counts.unknown ? ", " + counts.unknown + " not measured" : "") + (majority ? ", each by the majority of its rules." : ", where a factor is behind if any of its rules is.");
      var lean = counts.behind > counts.ahead ? "behind" : counts.ahead > counts.behind ? "ahead" : "parity";
      var hero = el(
        "section",
        { className: "cmp-hero" },
        el(
          "div",
          { className: "cmp-main" },
          el("div", { className: "kicker", textContent: "Head to head" }),
          el("h2", { className: "big", textContent: who(subject) + " vs " + who(peer) }),
          el("p", { className: "verdict", style: "margin:0;font-size:18px" }, el("b", { textContent: headline }), behindNames.length ? el("span", {}, " Behind on ", ...behindNames.map(function(n, i) {
            return el("span", {}, i ? ", " : "", link("compare/" + enc(n), "", title(D, n)), " (" + behindRules[n] + " of " + Object.keys(all[n].rules || {}).length + " rules)");
          }), behindNames.length === 1 ? ": " + andList(Object.keys(all[behindNames[0]].rules || {}).filter(function(r) {
            return (all[behindNames[0]].rules[r] || {})[peer] === "behind";
          }).sort().map(function(r) {
            return ruleTitle(behindNames[0], r);
          })) + ". If your teams rely on those, start with " + title(D, behindNames[0]) + "." : ": if your teams rely on what those factors measure, read them first.") : unmeasured ? " Run the comparison's harness and commit its records to fill this in." : counts.unknown ? " Not behind on any measured factor." : " Never behind."),
          unmeasured ? el("p", { className: "rule-tally", textContent: "Rule by rule: none of " + plural(ruleTotal, "rule") + " has values for both products yet." }) : el(
            "p",
            { className: "rule-tally" },
            "Rule by rule: ",
            el("b", { className: "ahead", textContent: ruleCounts.ahead + " ahead" }),
            " · ",
            el("b", { className: "parity", textContent: ruleCounts.parity + " at parity" }),
            " · ",
            el("b", { className: "behind", textContent: ruleCounts.behind + " behind" }),
            ". By this comparison's rule, " + factorRule(majority) + (names.some(function(n) {
              var rs = Object.keys(all[n].rules || {}).map(function(r) {
                return (all[n].rules[r] || {})[peer];
              });
              return rs.indexOf("ahead") >= 0 && rs.indexOf("behind") >= 0;
            }) ? ", so a factor marked “mixed” is ahead on some of its rules and behind on others" : "") + ". ",
            el("b", {
              className: "other-rule",
              textContent: "Counted the other way, where " + factorRule(!majority) + (otherDiffers ? ", it would be " + andList(["ahead", "behind", "parity"].filter(function(s) {
                return other[s] || s === "ahead";
              }).map(function(s) {
                return (s === "parity" ? "at parity on " : s + " on ") + other[s];
              })) + ". " : ", the factor tally is the same. ")
            }),
            "Factors are not weighted, so the factor count is a tally, not an overall score."
          ),
          el(
            "details",
            { className: "sub cmp-def cmp-brief", style: "margin:6px 0 0" },
            el("summary", { textContent: "How a factor and a rule are counted" }),
            "A factor is a scorecard that scores both products on the same rules. On a rule, a product is ahead when its measured value is better (within the rule's tolerance is parity) or it passes a bar the other misses. " + who(subject) + (majority ? " is ahead on a factor when it is ahead on more of its rules than it is behind on, behind when it is behind on more, and at parity when they are even; a factor with any unmeasured rule is not measured. “Mixed” means ahead on some rules and behind on others; “narrowly ahead” means ahead on fewer than half of that factor's rules, at parity on the rest." : " is behind on a factor if it is behind on any one of its rules, and ahead if it is ahead on some and behind on none; “narrowly ahead” means ahead on fewer than half of that factor's rules, at parity on the rest.")
          )
        ),
        el(
          "div",
          { className: "cmp-side" },
          el("div", { className: "kicker", textContent: "Factors" }),
          ruleTag(majority),
          el(
            "div",
            { className: "tally3" },
            ...["ahead", "parity", "behind"].map(function(k) {
              var mixed = unmeasured ? 0 : names.filter(function(n) {
                var vs = Object.keys(all[n].rules || {}).map(function(r) {
                  return (all[n].rules[r] || {})[peer];
                });
                return verdictOf(n) === k && vs.indexOf("ahead") >= 0 && vs.indexOf("behind") >= 0;
              }).length;
              return el("div", {}, el("strong", { textContent: unmeasured ? "—" : String(counts[k]), style: "color:" + (unmeasured ? "var(--muted)" : standingColor(k)) }), el("span", { textContent: STANDING_MARK[k] + " " + k }), mixed ? el("small", { className: "mixed", textContent: mixed + " mixed" }) : null);
            })
          ),
          unmeasured ? null : el("div", { className: "kicker", style: "margin-top:10px", textContent: "Rules" }),
          unmeasured ? null : el(
            "div",
            { className: "tally3 rules" },
            ...["ahead", "parity", "behind"].map(function(k) {
              return el("div", {}, el("strong", { textContent: String(ruleCounts[k]), style: "color:" + standingColor(k) }), el("span", { textContent: STANDING_MARK[k] + " " + k }));
            })
          ),
          this.headToHead(D, all, names, peer)
        )
      );
      hero.style.setProperty("--st", standingColor(lean));
      hero.dataset.focus = "";
      var read = {};
      var fields = {};
      names.forEach(function(n) {
        Object.values(all[n].reads || {}).forEach(function(fs) {
          (fs || []).forEach(function(f) {
            read[f] = true;
          });
        });
        Object.assign(fields, all[n].fields || {});
      });
      var facts = first.facts || {};
      var ids = [subject].concat(first.peers || []);
      var context = {};
      ids.forEach(function(id) {
        Object.keys(facts[id] || {}).forEach(function(f) {
          if (!read[f] && f !== "title") context[f] = true;
        });
      });
      var show = function(v) {
        return typeof v === "number" ? num(v) : typeof v === "boolean" ? v ? "yes" : "no" : String(v);
      };
      var setup = el(
        "div",
        { className: "setup" },
        ...Object.keys(context).sort().map(function(f) {
          var vals = ids.map(function(id) {
            return (facts[id] || {})[f];
          });
          var same = vals.every(function(v) {
            return JSON.stringify(v) === JSON.stringify(vals[0]);
          });
          return el(
            "div",
            {},
            el("b", { textContent: humanize(f) }),
            same ? el("span", { textContent: show(vals[0]) }) : el(
              "span",
              {},
              ...ids.map(function(id, i) {
                return vals[i] == null ? null : el("span", { style: "display:block" }, el("span", { className: "sub", textContent: who(id) + ": " }), show(vals[i]));
              })
            ),
            fields[f] ? el("small", { textContent: firstSentence(fields[f]), title: fields[f] }) : null,
            vals.some(function(v) {
              return /\+uncommitted\b/.test(String(v));
            }) ? el("small", { className: "warn-note", textContent: "“+uncommitted”: the harness had local changes when it ran, so this exact code is not in the repository and a rerun from the commit may differ." }) : null
          );
        })
      );
      var trackName = ((D.scorecards || {})[names[0]] || {}).track;
      var track = trackName && (D.tracks || {})[trackName];
      var bases = { measured: 0, judged: 0 };
      names.forEach(function(n) {
        var b = basisCounts(((D.scorecards || {})[n] || {}).rules);
        bases.measured += b.measured;
        bases.judged += b.judged;
      });
      var lead = track && track.description ? firstSentence(track.description) : "";
      var rest = lead ? track.description.trim().slice(lead.length).trim() : "";
      var restSentences = rest ? sentences(rest) : [];
      var scope = restSentences.filter(function(x) {
        return /^Not (covered|measured)\s*:/i.test(x);
      })[0];
      restSentences = restSentences.filter(function(x) {
        return x !== scope;
      });
      var method = el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "How it was measured" }), track ? link(trackPath(D, trackName), "", "Track: " + (track.title || trackName) + " →") : null),
        restSentences.length ? el("ul", { className: "method-lede" }, ...restSentences.map(function(x) {
          return el("li", {}, ...rich(x));
        })) : null,
        Object.keys(context).length ? setup : null,
        unmeasured ? el("p", { className: "sub", style: "margin:8px 0 0", textContent: "Nothing is measured in this build. Once the harness runs and its records are committed, the rules are decided like this:" }) : null,
        basisKey(bases)
      );
      var brief = setupBrief(context, ids, facts, who);
      var sources = track ? refs(track.links) : null;
      if (brief.length || sources)
        hero.firstChild.querySelector(".rule-tally").after(
          el(
            "div",
            { className: "sub cmp-prov" },
            sources ? el("span", { className: "cmp-src" }, el("b", { textContent: "Evidence: " }), sources, unmeasured ? " (no results are recorded in this build yet)" : null) : null
          )
        );
      hero.firstChild.querySelector(".rule-tally").after(
        el(
          "p",
          { className: "cmp-method", style: "margin:8px 0 0" },
          el("b", { textContent: lead ? "Method: " : "How it was measured: " }),
          ...lead ? rich(lead + " ") : [(brief.length ? brief.join(" · ") + ". " : "") + "Every rule is " + (bases.judged ? "measured or agent-judged" : "measured by script") + ". "],
          scope ? el("span", { style: "display:block;margin-top:4px" }, el("b", { textContent: "Not covered: " }), ...rich(scope.replace(/^Not (covered|measured)\s*:\s*/i, "") + " ")) : null,
          el("button", {
            className: "link",
            textContent: track && track.description ? "The full method, and who checked it ↓" : "Versions, setup and sources ↓",
            onclick: function() {
              method.scrollIntoView({ behavior: "smooth", block: "start" });
            }
          })
        )
      );
      var behindRows = [];
      var behindIn = [];
      var behindBases = {};
      names.forEach(function(n) {
        var def = (D.scorecards || {})[n] || {};
        Object.keys(all[n].rules || {}).forEach(function(r) {
          if ((all[n].rules[r] || {})[peer] === "behind") behindBases[basisOf((def.rules || {})[r] || {})] = true;
        });
      });
      var oneBasis = Object.keys(behindBases).length === 1 ? Object.keys(behindBases)[0] : null;
      names.forEach(function(n) {
        var c = all[n];
        var def = (D.scorecards || {})[n] || {};
        Object.keys(c.rules || {}).forEach(function(r) {
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
                el("div", { className: "bh-val" }, valueLine(c, r, rd, ids, who))
              ),
              el(
                "div",
                {},
                rd.rationale ? el("p", { className: "why" }, el("b", { textContent: "Why it matters: " }), ...rich(rd.rationale), (rd.links || []).length ? " " : null, refs(rd.links)) : null,
                rd.fix ? el("p", { className: "why bh-fix" }, el("b", { textContent: "To close it: " }), ...rich(rd.fix)) : null
              )
            )
          );
          if (behindIn.indexOf(n) < 0) behindIn.push(n);
        });
      });
      var behind = el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "Where " + who(subject) + " is behind" }), behindIn.length ? el(
          "span",
          { className: "sub" },
          plural(behindRows.length, "rule") + (oneBasis ? ", all " : ""),
          oneBasis ? basisBadge(oneBasis, true) : null,
          "; in full: ",
          ...behindIn.flatMap(function(n, i) {
            return [i ? ", " : "", link("compare/" + enc(n), "", title(D, n) + " →")];
          })
        ) : null),
        behindRows.length ? el("div", {}, ...behindRows) : unmeasured ? el("p", { className: "empty", textContent: "Nothing to compare yet: no rule has values for both products." }) : el("p", { className: "empty good", textContent: "✓ Not behind on any rule." })
      );
      var ratioRows = function(side) {
        var rows = [];
        names.forEach(function(n) {
          var c = all[n];
          var def = (D.scorecards || {})[n] || {};
          Object.keys(c.rules || {}).forEach(function(r) {
            var rd = (def.rules || {})[r] || {};
            if ((c.rules[r] || {})[peer] !== side || !rd.better) return;
            var a = ruleValue(c, r, subject);
            var b = ruleValue(c, r, peer);
            if (typeof a !== "number" || typeof b !== "number" || a <= 0 || b <= 0) return;
            rows.push({ n, c, r, rd, a, b, ratio: Math.max(a, b) / Math.min(a, b) });
          });
        });
        return rows.sort(function(x, y) {
          return y.ratio - x.ratio;
        });
      };
      var leadRows = ratioRows("ahead");
      var gapRows = ratioRows("behind");
      var unitOf = function(rd, v) {
        return num(v) + (rd.unit ? " " + rd.unit : "");
      };
      var times = function(l) {
        return l.ratio >= 10 ? Math.round(l.ratio).toLocaleString("en-US") : l.ratio.toFixed(1);
      };
      var widest = function(label, l, gap) {
        return el("p", { className: "cmp-lead" + (gap ? " gaps" : "") }, el("b", { textContent: label }), link("compare/" + enc(l.n), "", [lowerFirst(measureLabel(l.c, l.r)) + ", ", el("span", { className: "lead-x", textContent: (gap ? STANDING_MARK.behind + " " : "") + times(l) + "× " + cmpWord(l.rd, l.a < l.b) })]), " (" + who(subject) + " " + unitOf(l.rd, l.a) + ", " + who(peer) + " " + unitOf(l.rd, l.b) + ").");
      };
      var tiles = function(rows, gap) {
        return el(
          "div",
          { className: gap ? "leads gaps" : "leads" },
          ...rows.slice(0, 3).map(function(l) {
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
              el("span", { className: "sub", textContent: vs + " · " + title(D, l.n) })
            ]);
          })
        );
      };
      var verdictEl = hero.querySelector(".verdict");
      if (gapRows[0]) verdictEl.after(widest("Widest gap: ", gapRows[0], true));
      if (leadRows[0]) verdictEl.after(widest("Widest lead: ", leadRows[0]));
      if (gapRows.length) behind.querySelector(".head").after(el("p", { className: "sub", style: "margin:0 0 6px", textContent: "Measured values only, largest ratio first; every rule behind is listed below." }), tiles(gapRows, true));
      var leads = leadRows.length ? el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "Where " + who(subject) + " leads by the most" }), el("span", { className: "sub", textContent: "Measured values only, largest ratio first" })),
        tiles(leadRows)
      ) : null;
      var factors = el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Factors" }), el("span", { className: "sub", textContent: unmeasured ? "By name" : "Behind first" })), this.factorCards(D, names));
      var jumps = [
        ["Factors", factors],
        ["Behind", behind],
        ["Leads", leads],
        ["Method", method]
      ].filter(function(j) {
        return j[1];
      });
      var jump = el(
        "nav",
        { className: "jump", ariaLabel: "On this page" },
        el("span", { className: "sub", textContent: "On this page: " }),
        ...jumps.map(function(j, i) {
          return el("span", {}, i ? " · " : "", el("button", { className: "link", textContent: j[0], onclick: function() {
            j[1].scrollIntoView({ behavior: "smooth", block: "start" });
          } }));
        })
      );
      return el("div", {}, hero, jump, factors, behind, leads, method);
    },
    compareDetail(D, name, names, who) {
      var c = D.comparisons[name];
      var def = (D.scorecards || {})[name] || {};
      var peer = (c.peers || [])[0];
      var ids = [c.subject].concat(c.peers || []);
      var v = (c.verdict || {})[peer] || "unknown";
      var order = { behind: 0, unknown: 1, ahead: 2, parity: 3 };
      var rules = Object.keys(def.rules || {}).sort(function(a, b) {
        var sa = (c.rules[a] || {})[peer] || "unknown";
        var sb = (c.rules[b] || {})[peer] || "unknown";
        return order[sa] - order[sb] || a.localeCompare(b);
      });
      var rows = rules.map(function(r) {
        var rd = def.rules[r];
        var st = (c.rules[r] || {})[peer] || "unknown";
        return el(
          "div",
          { className: "measure" },
          el(
            "div",
            { className: "mt" },
            measureLabel(c, r),
            el("small", {}, el("code", { textContent: r }), basisBadge(basisOf(rd), true), rd.better ? el("span", { textContent: rd.better + " is better" }) : null)
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
                ((c.reads || {})[r] || []).length ? "The benchmark harness recorded " + ((c.reads || {})[r] || []).map(function(f) {
                  return "“" + ((c.fields || {})[f] || humanize(f)).replace(/\.$/, "") + "”";
                }).join(" and ") + " for each product, on the versions and machine under " : "Scored on the versions and machine under ",
                link("compare", "", "How it was measured"),
                ". " + (rd.better ? "The " + rd.better + " value is ahead." : "Passing beats failing.")
              ),
              el("div", { className: "cel", ariaLabel: "Expression", textContent: rd.check + (rd.value ? "\nvalue: " + rd.value : "") })
            ),
            rd.fix && st === "behind" ? el("p", { className: "sub", style: "margin:4px 0 0" }, el("b", { textContent: "What would close it: " }), ...rich(rd.fix)) : null
          )
        );
      });
      var atParity = rules.filter(function(r) {
        return ((c.rules[r] || {})[peer] || "unknown") === "parity";
      }).length;
      var fold = atParity >= 3 && atParity < rules.length;
      var read = {};
      names.forEach(function(n) {
        Object.values((D.comparisons[n] || {}).reads || {}).forEach(function(fs) {
          (fs || []).forEach(function(f) {
            read[f] = true;
          });
        });
      });
      var context = {};
      ids.forEach(function(id) {
        Object.keys((c.facts || {})[id] || {}).forEach(function(f) {
          if (!read[f] && f !== "title") context[f] = true;
        });
      });
      var timed = Object.values(def.rules || {}).some(function(rd) {
        return TIMED_UNITS.indexOf(String(rd.unit || "").toLowerCase()) >= 0;
      });
      if (!timed)
        Object.keys(context).forEach(function(f) {
          if (ids.some(function(id) {
            return typeof ((c.facts || {})[id] || {})[f] === "number";
          })) delete context[f];
        });
      var brief = setupBrief(context, ids, c.facts || {}, who);
      var tallyBy = /* @__PURE__ */ new Map();
      rules.forEach(function(r) {
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
        el("p", { className: "sub", textContent: "Each rule scores both products. Where both have a measured value, the better value decides; otherwise passing beats failing. " + who(c.subject) + (c.verdictRule === "majority" ? " is ahead on the factor when more of its rules are ahead than behind, and behind when more are behind." : " is behind on the factor if it is behind on any rule.") }),
        basisKey(basisCounts(def.rules)),
        el("div", {}, ...fold ? rows.slice(0, rows.length - atParity) : rows),
        fold ? el(
          "details",
          {
            className: "parity-fold",
            ontoggle: function() {
              this.firstChild.textContent = (this.open ? "Hide the " : "Show the ") + atParity + " rules at parity (neither product leads)";
            }
          },
          el("summary", { textContent: "Show the " + atParity + " rules at parity (neither product leads)" }),
          ...rows.slice(rows.length - atParity)
        ) : null
      );
      head.dataset.focus = "";
      return el("div", {}, head, el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Every factor" }), link("compare", "", "← Back to the summary")), el("div", { className: "h2h-wide" }, this.headToHead(D, D.comparisons, names, peer, name))));
    }
  };

  // src/element.js
  var ToolboxReport = class extends HTMLElement {
    connectedCallback() {
      if (this._booted) return;
      this._booted = true;
      var root = this.attachShadow({ mode: "open" });
      root.append(template.content.cloneNode(true));
      root.getElementById("skip").addEventListener("click", function() {
        var body = root.getElementById("body");
        body.tabIndex = -1;
        body.focus();
      });
      var ftip = el("div", { className: "ftip", hidden: true, ariaHidden: "true" });
      root.append(ftip);
      var tipFor = null;
      var tipTimer = 0;
      var hideTip = function() {
        clearTimeout(tipTimer);
        tipFor = null;
        ftip.hidden = true;
      };
      var showTip = function(e) {
        if (e.target === ftip) return clearTimeout(tipTimer);
        var t = e.target && e.target.closest && e.target.closest(".basis[data-tip],abbr.gloss,a.rl[data-tip]");
        if (t === tipFor) clearTimeout(tipTimer);
        if (!t || !t.dataset.tip || t === tipFor) return;
        clearTimeout(tipTimer);
        tipFor = t;
        ftip.textContent = t.dataset.tip;
        ftip.hidden = false;
        var r = t.getBoundingClientRect();
        var vw = document.documentElement.clientWidth;
        var vh = document.documentElement.clientHeight;
        var left = Math.max(12, Math.min(r.left, vw - 12 - ftip.offsetWidth));
        var top = r.bottom + 4 + ftip.offsetHeight > vh - 8 ? r.top - 4 - ftip.offsetHeight : r.bottom + 4;
        ftip.style.left = left + "px";
        ftip.style.top = Math.max(8, top) + "px";
      };
      root.addEventListener("mouseover", showTip);
      root.addEventListener("focusin", showTip);
      root.addEventListener("mouseout", function(e) {
        if (tipFor && (e.target === ftip || tipFor.contains(e.target)) && root.activeElement !== tipFor) tipTimer = setTimeout(hideTip, 200);
      });
      root.addEventListener("focusout", function(e) {
        if (tipFor && tipFor.contains(e.target)) hideTip();
      });
      window.addEventListener("keydown", function(e) {
        if (e.key === "Escape" && tipFor) hideTip();
      });
      window.addEventListener("scroll", hideTip, { passive: true, capture: true });
      window.addEventListener("hashchange", hideTip);
      var self = this;
      var inline = this.querySelector('script[type="application/json"]');
      var load = inline ? Promise.resolve(JSON.parse(inline.textContent)) : fetch(this.getAttribute("src")).then(function(r) {
        if (!r.ok) throw new Error("report JSON HTTP " + r.status);
        return r.json();
      });
      load.then(function(data) {
        self.mount(root, data);
      }).catch(function(e) {
        root.getElementById("body").replaceChildren(el("section", {}, el("div", { className: "empty", textContent: "Failed to load report: " + e.message })));
      });
    }
    mount(root, D) {
      var self = this;
      aliasLevels(D);
      ruleTitles(D);
      var S = summarize(D);
      root.getElementById("top").replaceChildren(boxLogo(), el("h1", { textContent: D.portal || "Toolbox report" }));
      root.getElementById("meta").textContent = fmt(S.entityCount) + " entities in the box (" + fmt(Object.keys(S.worst).length) + " scored) · " + plural(
        S.rows.filter(function(r) {
          return !r.compare;
        }).length,
        "scorecard"
      ) + (Object.keys(D.comparisons || {}).length ? " · " + plural(Object.keys(D.comparisons).length, "comparison factor") : "") + (D.snapshot ? " · facts snapshot #" + D.snapshot : " · no facts collected yet");
      var settle;
      var draw = function() {
        clearTimeout(settle);
        self.render(root, D, S);
      };
      var keep = function() {
        clearTimeout(settle);
        if (history.replaceState) history.replaceState(Object.assign({}, history.state, { y: window.scrollY }), "");
      };
      window.addEventListener("hashchange", draw);
      window.addEventListener("scroll", function() {
        clearTimeout(settle);
        settle = setTimeout(keep, 200);
      });
      window.addEventListener("click", keep, true);
      draw();
    }
    render(root, D, S) {
      var route = parseHash();
      if (route.mode === "tracks" && route.name) route.name = trackByPath(D, route.name);
      var modes = root.getElementById("modes");
      var body = root.getElementById("body");
      var self = this;
      var go = function(path) {
        var next = "#" + path;
        if (location.hash === next) self.render(root, D, S);
        else location.hash = path;
      };
      var modeOf = function(r) {
        if (r.mode === "entity") return "catalog";
        if (r.mode === "campaign") return "campaigns";
        return r.mode === "scorecard" || r.mode === "rule" ? "scorecards" : r.mode;
      };
      var activeMode = route.bare && !(D.scorecards || {})[route.name] ? null : modeOf(route);
      var campaigns = campaignNamesSorted(D);
      var tabs = [
        { label: "Overview", path: "overview", id: "overview" }
      ];
      var trackCount = tracksSorted(D, S).length;
      if (trackCount) tabs.push({ label: "Tracks", path: "tracks", id: "tracks", count: trackCount });
      tabs.push({ label: "Catalog", path: "catalog", id: "catalog", count: S.entityCount });
      if (campaigns.length || activeMode === "campaigns") tabs.push({ label: "Campaigns", path: "campaigns", id: "campaigns", count: campaigns.length });
      tabs.push(
        {
          label: "Scorecards",
          id: "scorecards",
          count: S.rows.length,
          path: "scorecards"
        }
      );
      var comparisons = Object.keys(D.comparisons || {});
      if (comparisons.length) tabs.push({ label: "Compare", path: "compare", id: "compare", count: comparisons.length });
      var levelCards = S.rows.filter(function(r) {
        return !r.compare;
      });
      tabs.forEach(function(t) {
        if (t.id === "scorecards") {
          t.count = levelCards.length;
        }
      });
      modes.style.setProperty("--tabcols", String(tabs.length <= 3 ? tabs.length : tabs.length === 4 ? 2 : 3));
      modes.replaceChildren(
        ...tabs.map(function(t) {
          var selected = activeMode === t.id;
          return el(
            "button",
            {
              role: "tab",
              id: "tab-" + t.id,
              ariaSelected: String(selected),
              tabIndex: selected || !activeMode && t.id === "overview" ? 0 : -1,
              onclick: function(ev) {
                if (ev.currentTarget === root.activeElement) self._tabFocus = true;
                go(t.path);
              }
            },
            t.label,
            t.count != null ? el("span", { className: "count", textContent: fmt(t.count) }) : null
          );
        })
      );
      requestAnimationFrame(function() {
        var cur = modes.querySelector('[aria-selected="true"]');
        if (cur && modes.scrollWidth > modes.clientWidth) {
          var at = cur.getBoundingClientRect().left - modes.getBoundingClientRect().left + modes.scrollLeft;
          modes.scrollLeft = Math.max(0, at + cur.offsetWidth / 2 - modes.clientWidth / 2);
        }
        var fade = function() {
          modes.classList.toggle("more", modes.scrollLeft + modes.clientWidth < modes.scrollWidth - 4);
          modes.classList.toggle("less", modes.scrollLeft > 4);
        };
        fade();
        modes.onscroll = fade;
      });
      modes.onkeydown = function(ev) {
        var keys = { ArrowLeft: -1, ArrowRight: 1, Home: "home", End: "end" };
        var dir = keys[ev.key];
        if (dir == null) return;
        ev.preventDefault();
        var buttons = Array.prototype.slice.call(modes.querySelectorAll('[role="tab"]'));
        var i = buttons.indexOf(root.activeElement);
        if (i < 0)
          i = buttons.findIndex(function(b) {
            return b.getAttribute("aria-selected") === "true";
          });
        var next = dir === "home" ? 0 : dir === "end" ? buttons.length - 1 : (i + dir + buttons.length) % buttons.length;
        self._tabFocus = true;
        buttons[next].click();
      };
      var view;
      var missing = missingTarget(D, route);
      if (missing) view = notFound(missing);
      else if (route.mode === "overview") view = this.overview(D, S);
      else if (route.mode === "catalog") view = this.catalog(D, S, route.kind, route.query);
      else if (route.mode === "entity") view = this.entity(D, S, route.id);
      else if (route.mode === "campaigns" || route.mode === "campaign") view = this.campaigns(D, S, route.name);
      else if (route.mode === "compare") view = this.compare(D, route.name);
      else if (route.mode === "tracks") view = this.tracks(D, S, route.name);
      else if (route.mode === "rule") view = this.rule(D, S, route.scorecard, route.rule);
      else if (route.mode === "scorecards") view = this.scorecardIndex(D, S, route.query);
      else view = this.scorecard(D, S, route.name, route.query);
      view.classList.add("view");
      body.replaceChildren(view);
      var h2s = view.querySelectorAll("h2");
      var named = view.querySelector("[data-focus] h2") || (route.name || route.kind ? h2s[1] : null) || h2s[0];
      document.title = (named && route.mode !== "overview" ? named.textContent + " · " : "") + (D.portal || "Toolbox report");
      var target = view.querySelector("[data-focus]");
      var hostTop = root.host.getBoundingClientRect().top + window.scrollY;
      if (this._tabFocus) {
        this._tabFocus = false;
        var sel = modes.querySelector('[aria-selected="true"]');
        if (sel) sel.focus({ preventScroll: true });
        if (this._lastRoute !== location.hash) window.scrollTo(0, Math.max(0, hostTop));
      } else if (this._lastMode !== void 0 && this._lastRoute !== location.hash) {
        var targetTop = target ? target.getBoundingClientRect().top + window.scrollY : hostTop;
        window.scrollTo(0, targetTop - hostTop < window.innerHeight * 0.6 ? Math.max(0, hostTop) : targetTop - 8);
        var heading = target && target.querySelector("h2") || named;
        if (heading) {
          if (!heading.hasAttribute("tabindex")) heading.tabIndex = -1;
          heading.focus({ preventScroll: true });
        }
      }
      var back = history.state && typeof history.state.y === "number" ? history.state.y : null;
      if (back !== null && this._lastRoute !== void 0 && this._lastRoute !== location.hash) window.scrollTo(0, back);
      this._lastMode = route.mode;
      this._lastRoute = location.hash;
    }
  };
  Object.assign(ToolboxReport.prototype, overviewViews, scorecardsViews, tracksViews, campaignsViews, catalogViews, entityViews, ruleViews, compareViews);
  if (!customElements.get("toolbox-report")) customElements.define("toolbox-report", ToolboxReport);
})();
