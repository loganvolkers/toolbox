// trackCard, trackGrid, tracks, track, trackStrip: methods of <toolbox-report>, installed on its prototype by element.js.
import { badge, levelLegend, medal } from "../lib/bars.js";
import { lockIcon, refs } from "../lib/basis.js";
import { levelName, nextMedal, nextMedalEl } from "../lib/campaigns.js";
import { el, enc, link } from "../lib/dom.js";
import { LEVEL_DEF, ONLY_LEVEL } from "../lib/levels.js";
import { capsRepo, isTop, ring, tallyRing, tallyWords, title } from "../lib/rings.js";
import { trackPath } from "../lib/routes.js";
import { LADDER, rankLevel } from "../lib/rules.js";
import { ringWhy, ringWords, trackSlots } from "../lib/status.js";
import { prose, rich } from "../lib/text.js";
import { furthestTrack, standingChip, trackColor, trackKey, trackStats, tracksSorted } from "../lib/tracks.js";
import { firstSentence, plural } from "../lib/words.js";

export const tracksViews = {
  trackCard(D, S, t, flag) {
    var def = t.def;
    var color = trackColor(t);
    var kids = [
      flag ? el("span", { className: "flag", textContent: flag }) : null,
      el(
        "div",
        { className: "th" },
        t.compare ? tallyRing(t.verdicts, t.rows.length, 60) : ring(t.pct, { size: 60, color: color, label: t.pct == null ? "not scored" : t.pct + "% at the top level" }),
        el("div", {}, el("div", { className: "tt", textContent: def.title || t.name }), el("div", { className: "sub", textContent: t.compare ? "factors ahead, a segment per factor" : t.averaged > 1 ? ringWords(t) : "of entities at the top level" })),
      ),
      def.description ? el("div", { className: "td", textContent: prose(firstSentence(def.description)) }) : null,
    ];
    var slots = !t.compare && trackSlots(D, t);
    if (slots) kids.push(slots);
    if (slots && t.capped.length) kids.push(el("div", { className: "capnote tcap" }, lockIcon(), " " + ringWhy(D, t)));
    if (t.compare) {
      kids.push(
        el(
          "div",
          { className: "chips" },
          ...["ahead", "parity", "behind", "unknown"]
            .filter(function (v) {
              return t.verdicts[v];
            })
            .map(function (v) {
              return standingChip(v, t.verdicts[v] + " " + v);
            }),
        ),
      );
    } else if (t.cards.length) {
      var shelf = {};
      var shelfTop = {};
      var shelfOrder = [];
      t.cards.forEach(function (c) {
        var l = c.floor || "not scored";
        if (!shelf[l]) {
          shelf[l] = 0;
          shelfTop[l] = 0;
          shelfOrder.push(l);
        }
        shelf[l]++;
        if (c.floor && isTop(D, c.row.name, c.floor)) shelfTop[l]++;
      });
      shelfOrder.sort(function (a, b) {
        return rankLevel(a === "not scored" ? null : a) - rankLevel(b === "not scored" ? null : b);
      });
      kids.push(
        el(
          "div",
          { className: "shelf", role: "img", ariaLabel: "Scorecards by their weakest entity's level: " + shelfOrder.map(function (l) { return shelf[l] + " " + l + (shelfTop[l] && shelfTop[l] < shelf[l] ? ", " + shelfTop[l] + " of them at its top level" : shelfTop[l] ? ", at its top level" : ""); }).join("; ") },
          el("span", { className: "sub", textContent: "Medals:" }),
          ...shelfOrder.map(function (l) {
            var own = ONLY_LEVEL.has(l) && LADDER.indexOf(l) < 0;
            var owner = own
              ? t.cards.find(function (c) {
                  return c.floor === l;
                })
              : null;
            var gloss = null;
            if (own) {
              var def = "The one level " + (owner ? title(D, owner.row.name) : "its scorecard") + " has, so reaching it is the top." + (LEVEL_DEF[l] ? " " + LEVEL_DEF[l] : "");
              gloss = el("abbr", { className: "gloss", tabIndex: 0 }, levelName(l), el("span", { className: "sr", textContent: " (" + def + ")" }));
              gloss.dataset.tip = def;
            }
            return el("span", { className: "sh" + (own ? " own" : "") }, medal(l === "not scored" ? "none" : l, 20), el("b", { textContent: String(shelf[l]) }), " ", gloss || levelName(l), (own ? " ✓ top" : shelfTop[l] === shelf[l] || ONLY_LEVEL.has(l) ? " ✓ top" : shelfTop[l] ? " (" + shelfTop[l] + " ✓ top)" : ""));
          }),
        ),
      );
      var nm = nextMedal(D, t.cards);
      var wk = t.cards[0] && t.cards[0].floor && !isTop(D, t.cards[0].row.name, t.cards[0].floor) ? t.cards[0] : null;
      if (wk && (!nm || nm.name !== wk.row.name)) kids.push(el("div", { className: "tnext" }, el("b", { textContent: "Lowest level: " }), link("scorecard/" + enc(wk.row.name), "", title(D, wk.row.name)), " at ", badge(wk.floor)));
      kids.push(
        nm
          ? el("div", { className: "tnext" }, el("b", { textContent: wk && nm.name === wk.row.name ? "Lowest level, and the next medal: " : "Next medal: " }), link("scorecard/" + enc(nm.name), "", title(D, nm.name)), wk && nm.name === wk.row.name ? el("span", {}, " at ", badge(wk.floor), " → " + nm.up) : " to " + nm.up, " · " + plural(nm.low, "entity", "entities") + " to lift")
          : el("div", { className: "tnext good", textContent: "✓ Every scorecard is at its top level" }),
      );
    }
    var weakest = !t.compare && t.cards[0] && t.cards[0].floor ? title(D, t.cards[0].row.name) + " at " + t.cards[0].floor + (capsRepo(D, t.cards[0].row.name, t.cards[0].floor) ? ", its top level, which caps the repository" : isTop(D, t.cards[0].row.name, t.cards[0].floor) ? ", its top level" : "") : null;
    kids.push(
      el(
        "div",
        { className: "foot" },
        el("span", { textContent: plural(t.rows.length, t.compare ? "factor" : "scorecard") + (t.campaigns.length ? " · " + plural(t.campaigns.length, "campaign") : "") }),
        t.compare ? el("span", { textContent: t.behind.length ? "behind on " + t.behind.join(", ") : t.verdicts.unknown >= t.rows.length ? "not measured yet" : t.verdicts.unknown ? "not behind on any measured factor" : "never behind" }) : null,
      ),
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
      ...shown.map(function (t) {
        return self.trackCard(D, S, t, t.name === furthest ? "Furthest from goal" : null);
      }),
    );
  },

  tracks(D, S, name) {
    if (name) return this.track(D, S, name);
    var all = tracksSorted(D, S);
    if (!all.length) return el("section", {}, el("div", { className: "empty", textContent: "This portal declares no tracks." }));
    var loose = S.rows.filter(function (r) {
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
          all.filter(function (t) {
            return !t.compare;
          }),
        ),
      ),
      all.some(function (t) {
        return t.compare;
      })
        ? el(
            "section",
            {},
            el("div", { className: "head" }, el("h2", { textContent: "Measured against a rival" })),
            el("p", { className: "lede", textContent: "Its ring counts the factors where the product leads its rival, so it stands outside the order above." }),
            this.trackGrid(
              D,
              S,
              all.filter(function (t) {
                return t.compare;
              }),
            ),
          )
        : null,
      loose.length
        ? el(
            "section",
            {},
            el("div", { className: "head" }, el("h2", { textContent: "Not on a track" })),
            levelLegend(D, loose),
            this.scorecardTable(D, loose),
          )
        : null,
    );
  },

  track(D, S, name) {
    var t = trackStats(D, S, name);
    var def = t.def;
    var self = this;
    var color = trackColor(t);
    var owner = def.owner ? def.owner.replace(/^group:/, "") : "";
    var weakest = t.cards[0];
    var goal = t.compare
      ? el("p", {}, el("b", { textContent: tallyWords(t.verdicts, t.rows.length).replace(/^a/, "A") + ". " }), link("compare", "", "Open the comparison →"))
      : el(
          "p",
          {},
          el("b", { textContent: "Goal: every entity at the top level of each scorecard below. " }),
          t.pct == null ? "Nothing is scored yet." : "On average " + t.pct + "% are there" + (t.none ? "; " + plural(t.none, "entity is", "entities are") + " still at none" : "") + "." + (t.capped.length ? " " + ringWhy(D, t) + "." : ""),
          weakest && weakest.floor ? el("span", {}, " The weakest scorecard is ", link("scorecard/" + enc(weakest.row.name), "", title(D, weakest.row.name)), ", at ", badge(weakest.floor), capsRepo(D, weakest.row.name, weakest.floor) ? ", its only top level, so it caps the repository there until it gains a higher one." : isTop(D, weakest.row.name, weakest.floor) ? ", already its top level." : ".") : null,
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
        t.compare ? tallyRing(t.verdicts, t.rows.length, 88) : ring(t.pct, { size: 88, color: color, label: t.pct == null ? "not scored" : t.pct + "% at the top level" }),
      ),
      def.description ? el("p", { className: "lede", style: "max-width:80ch" }, ...rich(def.description)) : null,
      refs(def.links),
      goal,
      trackNext ? el("div", { style: "max-width:80ch;margin-top:8px" }, trackNext) : null,
    );
    section.dataset.focus = "";
    var parts = [section];
    if (t.compare) {
      parts.push(el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Factors" })), this.factorCards(D, t.rows.map(function (r) {
        return r.name;
      }))));
    } else if (t.rows.length) {
      var rows = t.cards.map(function (c) {
        return c.row;
      });
      parts.push(
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Scorecards" }), el("span", { className: "sub", textContent: "Lowest level first" })),
          levelLegend(D, rows),
          this.scorecardTable(D, rows, true),
        ),
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
            ...t.campaigns.map(function (n) {
              return self.campaignCard(D, n);
            }),
          ),
        ),
      );
    }
    var others = tracksSorted(D, S).filter(function (o) {
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
      ...shown.map(function (o) {
        var standing = o.compare ? tallyWords(o.verdicts, o.rows.length) : o.pct == null ? "not scored" : o.pct + "% at the top level";
        var flag = lead && o.name === furthest;
        var a = link(trackPath(D, o.name), flag ? "far" : "", [
          o.compare ? tallyRing(o.verdicts, o.rows.length, 30, true) : ring(o.pct, { size: 30, color: trackColor(o), label: standing, bare: true }),
          el("span", {}, el("span", { textContent: o.def.title || o.name }), lead ? el("small", { textContent: (o.compare ? standing : o.pct == null ? "not scored" : o.pct + "%") + (flag ? " · furthest from goal" : "") }) : null),
        ]);
        a.title = standing + (o.capped && o.capped.length ? ". " + ringWhy(D, o) : "");
        a.setAttribute("aria-label", (o.def.title || o.name) + ": " + standing + (flag ? ". Furthest from goal" : ""));
        return a;
      }),
    );
  },
};
