import type { ToolboxReport } from "../element";
import type { Summary } from "../lib/summary";
import type { ReportData } from "../types";
// hero, tiles, campaignCard, scorecardTable, overview: methods of <toolbox-report>, installed on its prototype by element.js.
import { badge, barEl, levelLegend, medal, splitUnassessed } from "../lib/bars";
import { lockIcon } from "../lib/basis";
import { campaignStanding, campaignState, goalBar, includeProgress, kpiBar, kpiPips, nextMedalEl, scCaption, stuckNote } from "../lib/campaigns";
import { cardName, num } from "../lib/compare";
import { el, enc, link } from "../lib/dom";
import { kindCount, kindDescription, kindGroupsOf, kindTitle } from "../lib/kinds";
import { fill, ink, known } from "../lib/levels";
import { ring, title, topNote } from "../lib/rings";
import { LADDER, floorOf, floorPill } from "../lib/rules";
import { daysLeft } from "../lib/status";
import { campaignNamesSorted, healthVerdict, noFacts, share as shareOf } from "../lib/summary";
import { furthestTrack, trackStats, trackTitle, tracksSorted } from "../lib/tracks";
import { andList, fmt, plural, scPurpose } from "../lib/words";

export const overviewViews = {
  hero(D: ReportData, S: Summary, campaignNames) {
    const R = S.repo;
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
            el("code", { textContent: "toolbox doctor" }),
            " in the repository to see which connector or tool is missing, then ",
            el("code", { textContent: "toolbox report" }),
            " again. Connectors are listed under ",
            el("code", { textContent: "connectors:" }),
            " in the portal's ",
            el("code", { textContent: "toolbox.yaml" }),
            ". The " + plural(S.rows.length, "scorecard") + " below will fill in.",
          ),
        ),
      );
      start.setAttribute("aria-label", "Status");
      return start;
    }
    if (!R) {
      var at = verdict.indexOf(": ");
      var needs = at > 0 && verdict.indexOf("Needs") === 0;
      var almost = at > 0 && verdict.indexOf("Almost") === 0;
      var tone = needs ? "none" : almost ? "bronze" : "gold";
      // The furthest track's lowest scorecard, as its track card names it; with no tracks, every scorecard's.
      var far = furthestTrack(D, S);
      var next = needs || almost ? nextMedalEl(D, trackStats(D, S, far || undefined).cards, "Next: ", far ? " It comes first because " + trackTitle(D, far) + " is the track furthest from its goal." : "") : null;
      var plain = el("section", { className: "hero plain", style: "--lvl:" + fill(tone) + ";--lvl-ink:" + ink(tone) }, el("div", {}, el("div", { className: "kicker", textContent: "Status" }), el("h2", { className: "levelword", textContent: at > 0 ? verdict.slice(0, at) : verdict }), at > 0 ? el("p", { className: "verdict", textContent: verdict.slice(at + 2).replace(/^./, function (ch) { return ch.toUpperCase(); }) }) : null, next, S.rows.length ? el("p", { className: "sub", style: "margin:8px 0 0", textContent: "No overall level: this portal does not combine its scorecards into one level for the repository, so each scorecard stands on its own." }) : null));
      plain.setAttribute("aria-label", "Status");
      return plain;
    }
    var level = R.floor;
    var section = el("section", { className: "hero", style: "--lvl:" + fill(level) + ";--lvl-ink:" + ink(level) });
    section.setAttribute("aria-label", "Status");
    var holdingNames = R.holding.map(function (c) {
      return title(D, c.name);
    });
    var rated = " The repository is rated by the weakest of its " + plural(R.constituents.length, "standard") + ".";
    var sentence = R.next ? "Held at " + level + " by " + plural(R.holding.length, "standard") + ", listed below." + rated : "Every one of its " + plural(R.constituents.length, "standard") + " is at the top level.";
    var wins = R.ladder
      .slice()
      .reverse()
      .filter(function (l) {
        return l !== "none" && l !== level;
      })
      .map(function (l) {
        var n = R.constituents.filter(function (c) {
          return c.level === l;
        }).length;
        return n ? plural(n, "standard") + " at " + l : null;
      })
      .filter(Boolean);
    var capped = R.holding.filter(function (c) {
      return c.level === c.top;
    });
    var liftNames = R.holding
      .filter(function (c) {
        return c.level !== c.top;
      })
      .map(function (c) {
        return title(D, c.name);
      });
    if (R.next && capped.length)
      // Lead with the cause: the capping standard is fully met, it just has no higher level to reach.
      sentence =
        "Capped at " +
        level +
        ": " +
        capped
          .map(function (c) {
            return title(D, c.name);
          })
          .join(", ") +
        (capped.length === 1 ? " is fully met, but its top level is " : " are fully met, but their top level is ") +
        level +
        ", so the repository cannot rise until " +
        (capped.length === 1 ? "its owner adds" : "their owners add") +
        " a " +
        R.next +
        " level." +
        (liftNames.length ? " After that, " + plural(liftNames.length, "more standard needs", "more standards need") + " " + R.next + "." : "") +
        rated;
    var nextBox = !R.next
      ? el("div", { className: "next" }, el("b", { textContent: "Top of the ladder. " }), "Every repository standard is at its highest level.")
      : capped.length
        ? null
        : el("div", { className: "next" }, el("b", { textContent: "Next: " }), "reach ", badge(R.next), " by lifting " + plural(holdingNames.length, "standard") + " to " + R.next + ", listed below.");
    var climb: HTMLElement | null = null;
    if (R.next) {
      var rank = function (l) {
        return R.ladder.indexOf(l);
      };
      var there = R.constituents.filter(function (c) {
        return rank(c.level) >= rank(R.next);
      }).length;
      var share = shareOf(there, R.constituents.length);
      var bar = el("div", { className: "climb-bar" }, el("span", { style: "width:" + share + "%;background:" + ink(R.next) }));
      bar.setAttribute("role", "img");
      var blockedBy = capped.length
        ? capped
            .map(function (c) {
              return title(D, c.name);
            })
            .join(", ")
        : "";
      bar.setAttribute("aria-label", there + " of " + R.constituents.length + " repository standards at " + R.next + " or above" + (blockedBy ? "; blocked until " + blockedBy + " gains a " + R.next + " level" : ""));
      climb = el(
        "div",
        { className: "climb" + (blockedBy ? " blocked" : "") },
        el("div", { className: "climb-head" }, el("span", {}, "Toward ", badge(R.next)), el("span", { className: "sub", textContent: there + " of " + R.constituents.length + " standards there (" + share + "%)" })),
        blockedBy ? el("div", { className: "climb-track" }, bar, el("span", { className: "climb-lock", title: "No " + R.next + " level exists yet on " + blockedBy }, lockIcon(), " blocked by a missing level")) : bar,          el(
          "ul",
          {},
          ...R.holding.map(function (c) {
            var stuck = c.level === c.top;
            return el("li", { className: stuck ? "stuck" : "" }, el("span", { className: "mark", ariaHidden: "true", textContent: stuck ? "⊘" : "↑" }), el("span", {}, link("scorecard/" + enc(c.name), "", title(D, c.name)), stuck ? stuckNote(D, c.name, R.next) : el("span", { className: "sub", textContent: " lift from " + c.level + " to " + R.next })));
          }),
        ),
      );
    }
    var left = el(
      "div",
      {},
      el(
        "div",
        { className: "badge-big" },
        medal(level, 72),
        el("div", {}, el("div", { className: "kicker", textContent: "Repository level" }), el("h2", { className: "levelword", textContent: level })),
      ),
      el("p", { className: "verdict", textContent: sentence }),
      verdict.indexOf(": ") > 0
        ? el(
            "p",
            { className: "attn" },
            el("b", { textContent: verdict.slice(0, verdict.indexOf(": ") + 1) + " " }),
            verdict.slice(verdict.indexOf(": ") + 2) + " ",
            el("button", {
              className: "link",
              textContent: "See them ↓",
              onclick: function (this: HTMLElement) {
                var h = (this.getRootNode() as ParentNode).querySelector("#need-att");
                if (h) h.scrollIntoView({ behavior: "smooth", block: "start" });
              },
            }),
          )
        : el("p", { className: "attn", textContent: verdict }),
      wins.length ? el("p", { className: "wins" }, el("b", { textContent: "✓ Already: " }), wins.join(" · ")) : null,
      nextBox,
      climb,
    );
    var ladder = el("div", { className: "ladder", style: "--steps:" + R.ladder.length });
    ladder.setAttribute("aria-label", "Repository standards by lowest level");
    R.ladder.forEach(function (l) {
      var here = R.constituents.filter(function (c) {
        return c.level === l;
      });
      var rung = el(
        "div",
        { className: "rung" + (l === level ? " here" : "") + (here.length ? "" : " vacant"), style: "--lvl:" + fill(l) },
        el("div", { className: "rh" }, medal(l, 22), l, el("span", { className: "n", textContent: String(here.length) })),
      );
      if (!here.length) rung.append(l === "none" ? el("div", { className: "empty good", textContent: "✓ No repository standard at none" }) : el("div", { className: "empty", textContent: "—" }));
      here.forEach(function (c) {
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
          "Only these standards set the repository level. The other scorecards rate single entities, such as one rule or one skill, and are listed below." +
            (R.ceiling !== R.ladder[R.ladder.length - 1] && R.ceiling !== level
              ? " " +
                plural(
                  R.constituents.filter(function (c) {
                    return c.top === R.ceiling;
                  }).length,
                  "standard tops",
                  "standards top",
                ) +
                " out at " +
                R.ceiling +
                ", so " +
                R.ceiling +
                " is the highest level the repository can reach."
              : ""),
        ),
      ),
    );
    return section;
  },

  tiles(D: ReportData, S: Summary, campaignNames) {
    var campaigns = D.campaigns || {};
    var c = S.checks;
    var total = c.pass + c.vacuous + c.fail + c.unknown;
    var pct = shareOf(c.pass, total);
    var checkNote = [c.vacuous ? fmt(c.vacuous) + " more pass by default, because no agent has reviewed them yet" : null, c.unknown ? fmt(c.unknown) + " not run" : null].filter(Boolean);
    var on = campaignNames.filter(function (n) {
      return campaignStanding(D, campaigns[n]) === "on";
    }).length;
    var debt = S.rows.filter(function (r) {
      return r.none > 0 && !r.compare;
    }).length;
    var scoredEntities = Object.keys(S.worst).length;
    var tops = new Set(
      S.rows
        .filter(function (r) {
          return !r.compare;
        })
        .map(function (r) {
          return known((r.sc.levels || []).slice(-1)[0]) || "gold";
        }),
    );
    var topList = LADDER.filter(function (l) {
      return tops.has(l);
    });
    var topMedal = topList[topList.length - 1] || "gold";
    var goldIds = S.atTop;
    var golden = goldIds.length;
    var unjudged = new Set();
    Object.keys(D.unassessed || {}).forEach(function (card) {
      var rows = (D.scores || {})[card] || {};
      (D.unassessed || {})[card].forEach(function (id) {
        if (rows[id] && rows[id].level !== "skipped") unjudged.add(id);
      });
    });
    var goldUnjudged = goldIds.filter(function (id) {
      return unjudged.has(id);
    }).length;
    // Counted per entity, one numerous kind can carry the figure; say so when it is a large share.
    var perKind: Record<string, number> = {};
    Object.keys(S.worst).forEach(function (id) {
      var k = S.kindOf[id] || "";
      perKind[k] = (perKind[k] || 0) + 1;
    });
    var bigKind = Object.entries(perKind).sort(function (a, b) {
      return b[1] - a[1];
    })[0];
    var kindNote = bigKind && bigKind[0] && bigKind[1] * 3 >= scoredEntities && Object.keys(perKind).length > 1 ? "Counted per entity; the largest group is " + kindCount(D, bigKind[0], bigKind[1]) + "." : null;
    var tile = function (path, cls, lead, big, text) {
      return link(path, "tile " + (cls || ""), [lead, el("div", {}, el("strong", { textContent: big }), el("span", {}, ...[].concat(text)))]);
    };
    var shown = [
      total ? tile("scorecards", "good", kpiBar(pct, pct + "% of checks pass"), pct + "%", [fmt(c.pass) + " of " + fmt(total) + " checks pass on evidence", checkNote.length ? el("small", { className: "tile-note", textContent: checkNote.join("; ") }) : null]) : null,
      scoredEntities ? tile("catalog", "", medal(topMedal, 40), fmt(golden), ["of " + fmt(scoredEntities) + " scored entities are at the top level of every scorecard that rates them", topList.length > 1 ? el("small", { className: "tile-note", textContent: "Top levels differ by scorecard: " + andList(topList) + "." }) : null, goldUnjudged ? el("small", { className: "tile-note", textContent: fmt(goldUnjudged) + " of those still await an agent's assessment" }) : null, kindNote ? el("small", { className: "tile-note", textContent: kindNote }) : null]) : null,
      scoredEntities ? tile("scorecards", debt ? "bad" : "good", medal(debt ? "none" : "gold", 40), String(debt), debt === 1 ? "scorecard has entities at none" : debt ? "scorecards have entities at none" : "scorecards with entities at none") : null,
      campaignNames.length ? tile("campaigns", on === campaignNames.length ? "good" : "", kpiPips(on, campaignNames.length, on + " of " + campaignNames.length + " campaigns on target"), on + " of " + campaignNames.length, "campaigns on target") : null,
    ].filter(Boolean);
    return shown.length ? el("div", { className: "tiles" }, ...shown) : null;
  },

  campaignCard(D: ReportData, name, current?) {
    var c = (D.campaigns || {})[name];
    var state = campaignState(c);
    var pct = c.percent == null ? 0 : c.percent;
    var target = c.targetPercent == null ? 100 : c.targetPercent;
    var days = daysLeft(c.end);
    var each = includeProgress(D, c);
    var toGo = each.reduce(function (n, p) {
      return n + (p.met ? 0 : p.below);
    }, 0);
    var status =
      state === "empty"
        ? toGo
          ? el("span", { className: "pill bad", textContent: plural(toGo, "entity", "entities") + " to go" })
          : el("span", { className: "pill good", textContent: "on target" })
        : state === "on"
          ? el("span", { className: "pill good", textContent: "on target" })
          : el("span", { className: "pill bad", textContent: plural((c.inScope || 0) - (c.meeting || 0), "entity", "entities") + " to go" });
    var a = link("campaign/" + enc(name), "card", [
      goalBar(state, pct, target, false, each),
      el(
        "div",
        {},
        el("div", { className: "t", textContent: c.title || name }),
        el("div", { className: "s" }, status, days != null && days >= 0 && c.status !== "ended" ? " · " + plural(days, "day") + " left" : c.status === "ended" ? " · ended" : ""),
      ),
    ]);
    if (current) a.setAttribute("aria-current", "true");
    return a;
  },

  scorecardTable(D: ReportData, rows, floors?) {
    var why = function (r, indent) {
      var p = scPurpose(r.sc);
      return p ? el("span", { className: "rp" + (indent ? " in" : ""), textContent: p }) : null;
    };
    var wrap = el("div", { className: "rows" });
    wrap.append(el("div", { className: "rows-head" }, el("span", { textContent: "Scorecard" }), el("span", { className: "dist", textContent: "Entities by level" }), el("span", { textContent: rows.some(function (r) { return r.scored; }) ? "At none" : "Status" })));
    rows.forEach(function (r) {
      wrap.append(
        link("scorecard/" + enc(r.name), "row", [
          floors
            ? el("div", { className: "medal-col" }, medal(floorOf(D, r.name) || "not scored", 30), el("div", {}, ...cardName(D, r.name), el("span", { className: "weakest" }, "weakest ", floorOf(D, r.name) ? floorPill(D, r.name) : "not scored", topNote(D, r.name, floorOf(D, r.name)))))
            : el("div", {}, ...cardName(D, r.name)),
          el("div", { className: "dist" }, barEl(splitUnassessed(D, r.name, r.parts)), el("div", { className: "sub", style: "font-size:12px;margin-top:4px" }, ...scCaption(D, r, true))),
          el("div", { className: "num" + (r.none ? " bad" : "") }, el("strong", { textContent: r.scored ? String(r.none) : "—" }), el("span", { textContent: r.scored ? "at none" : "not scored" })),
          why(r, floors),
        ]),
      );
    });
    return wrap;
  },

  overview(D: ReportData, S: Summary) {
    var catalog = D.catalog || {};
    var campaignNames = campaignNamesSorted(D);
    var self = this;
    var attention = S.rows.filter(function (r) {
      return r.none > 0 && !r.compare;
    });
    var clear = S.rows.filter(function (r) {
      return r.none === 0 && !r.compare && r.scored;
    });
    var unscored = S.rows.filter(function (r) {
      return !r.compare && !r.scored;
    });
    var clearHollow = clear.filter(function (r) {
      return splitUnassessed(D, r.name, r.parts).some(function (p) {
        return p.hollow;
      });
    });
    var clearBelow = clear.filter(function (r) {
      return clearHollow.indexOf(r) < 0 && r.topShare < 100;
    });
    var clearLine = el(
      "span",
      {},
      "✓ " + plural(clear.length - clearHollow.length, "scorecard has", "scorecards have") + " no entity at none",
      clearHollow.length
        ? el("span", { className: "unjudged-n" }, "; " + fmt(clearHollow.length) + " more only because no agent has assessed some of their entities yet (", ...clearHollow.flatMap(function (r, i) {
            return [i ? ", " : "", link("scorecard/" + enc(r.name), "", title(D, r.name))];
          }), ")")
        : null,
      clearBelow.length
        ? el("span", {}, "; " + (clear.length - clearHollow.length === 1 ? "it still has entities below its top level" : clearBelow.length === 1 ? "one of them still has entities below its top level" : fmt(clearBelow.length) + " of them still have entities below their top level") + " (", ...clearBelow.flatMap(function (r, i) {
            return [i ? ", " : "", link("scorecard/" + enc(r.name), "", title(D, r.name))];
          }), ")")
        : null,
      ". ",
    );
    var showAll = false;
    var clearWrap = el("div");
    var drawClear = function () {
      clearWrap.replaceChildren(
        showAll
          ? el(
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
                  onclick: function () {
                    showAll = false;
                    drawClear();
                    var b = clearWrap.querySelector("button");
                    if (b) b.focus();
                  },
                }),
              ),
              self.scorecardTable(D, clear),
            )
          : el(
              "p",
              { className: "empty good" },
              clearLine,
              el("button", {
                className: "link",
                textContent: "Show them",
                ariaExpanded: "false",
                onclick: function () {
                  showAll = true;
                  drawClear();
                  var b = clearWrap.querySelector("button");
                  if (b) b.focus();
                },
              }),
            ),
      );
    };
    drawClear();
    var attFold = attention.length > 4 ? 3 : attention.length;
    var attTable = attention.length ? this.scorecardTable(D, attention.slice(0, attFold)) : null;
    var attMore =
      attention.length > attFold
        ? el(
            "div",
            { className: "more" },
            el("button", {
              className: "link",
              ariaExpanded: "false",
              textContent: "Show the other " + fmt(attention.length - attFold) + " that need attention",
              onclick: function (this: HTMLElement) {
                var open = this.getAttribute("aria-expanded") !== "true";
                var shown = open ? self.scorecardTable(D, attention) : self.scorecardTable(D, attention.slice(0, attFold));
                (attTable as HTMLElement).replaceWith(shown);
                attTable = shown;
                this.setAttribute("aria-expanded", String(open));
                this.textContent = open ? "Show only the first " + attFold : "Show the other " + fmt(attention.length - attFold) + " that need attention";
                var next = open && shown.querySelectorAll<HTMLElement>("a.row")[attFold];
                if (next) next.focus();
              },
            }),
          )
        : null;
    return el(
      "div",
      {},
      this.hero(D, S, campaignNames),
      tracksSorted(D, S).length
        ? el(
            "section",
            {},
            el("div", { className: "head" }, el("h2", { textContent: "Tracks" }), link("tracks", "", "All tracks →")),
            el("p", { className: "lede", textContent: (furthestTrack(D, S) ? plural(tracksSorted(D, S).length, "long-running line") + " of work; " + trackTitle(D, furthestTrack(D, S)) + " is furthest from its goal. " : "") + "Each ring is its scorecards' average share of entities at the top level." + (tracksSorted(D, S).some(function (t) {
                return t.compare;
              })
                ? " The comparison track's ring counts factors ahead of a rival instead."
                : "") + " Open a track for its scorecards and campaigns." }),
            this.trackStrip(D, S, tracksSorted(D, S), "Tracks", true),
          )
        : null,
      this.tiles(D, S, campaignNames),
      campaignNames.length
        ? el(
            "section",
            {},
            el("div", { className: "head" }, el("h2", { textContent: "Campaigns" }), el("span", { className: "sub", textContent: "Goals with a target, behind target first; a bar turns green once it passes its target mark" })),
            el(
              "div",
              { className: "cards" },
              ...campaignNames.map(function (n) {
                return self.campaignCard(D, n);
              }),
            ),
          )
        : null,
      el(
        "section",
        {},
        el(
          "div",
          { className: "head" },
          el("h2", { id: "need-att", textContent: attention.length ? "Scorecards that need attention" : "Scorecards" }),
          attention.length ? el("span", { className: "sub", textContent: "Most entities at none first" }) : null,
        ),
        el("p", { className: "lede", textContent: "Each scorecard rates every entity of one kind against levels of rules. An entity is at none until it passes every rule of the first level." }),
        unscored.length === S.rows.length ? null : levelLegend(D, S.rows),
        attTable,
        attMore,
        clear.length ? clearWrap : null,
        unscored.length ? el("div", {}, el("p", { className: "empty", textContent: plural(unscored.length, "scorecard has", "scorecards have") + " no scored entity yet:" }), this.scorecardTable(D, unscored)) : null,
      ),
      !S.entityCount ? null : el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "What's in the box" }), link("catalog", "", "Browse the catalog →")),
        ...kindGroupsOf(D).map(function (g) {
          return el(
            "div",
            { className: "boxrow" },
            el("div", { className: "bg" }, g.title, el("small", { textContent: plural(g.count, "entity", "entities") })),
            el(
              "div",
              { className: "box" },
              ...g.kinds.map(function (k) {
                var a = link("catalog/" + enc(k), "", [kindTitle(D, k), el("b", { textContent: fmt(catalog[k].length) })]);
                if (kindDescription(D, k)) a.title = kindDescription(D, k).replace(/`/g, "");
                return a;
              }),
            ),
          );
        }),
      ),
    );
  },
} satisfies ThisType<ToolboxReport>;
