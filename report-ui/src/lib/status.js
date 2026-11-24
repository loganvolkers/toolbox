import { levelName } from "./campaigns.js";
import { el, enc } from "./dom.js";
import { ring, title } from "./rings.js";
import { floorOf } from "./rules.js";
import { RAMP } from "./tracks.js";
import { andList, fmt } from "./words.js";

export function daysLeft(end) {
  if (!end) return null;
  var ms = Date.parse(end + "T23:59:59Z") - Date.now();
  return isNaN(ms) ? null : Math.ceil(ms / 86400000);
}

export function isPassing(o) {
  return o === "pass" || o === "waived";
}

// A check that was not run ("unknown") is not a failure: the scorer lets it hold a level provisionally.
export function isFailing(o) {
  return o != null && !isPassing(o) && o !== "skipped" && o !== "unknown";
}

// The rules an entity still fails at or below `level` (what stands between it and that level).
export function blockers(sc, score, level) {
  var order = sc.levels || [];
  var cap = order.indexOf(level);
  return Object.keys(score.rules || {})
    .filter(function (n) {
      var rl = ((sc.rules || {})[n] || {}).level;
      return isFailing(score.rules[n]) && order.indexOf(rl) <= cap;
    })
    .sort();
}

// One bar per scorecard on a track, filled to its share at the top level; the dashed line is their
// average, which is the track's ring.
export function ringWords(t) {
  return t.capped.length ? "average share at the top level, across " + fmt(t.averaged) + " of its " + fmt(t.scoredRows) + " scorecards" : "average share at the top level, across its " + fmt(t.averaged) + " scorecards";
}

// Why a track's ring leaves a scorecard out: its 100% is a ladder that stops short, not progress.
export function ringWhy(D, t) {
  if (!t.capped.length) return "";
  var lv = levelName(floorOf(D, t.capped[0]));
  return (
    andList(
      t.capped.map(function (n) {
        return title(D, n);
      }),
    ) +
    (t.capped.length === 1 ? " is" : " are") +
    " left out: " +
    (t.capped.length === 1 ? "it holds the repository level at " + lv + " until it gains a higher level, so its 100% is a ceiling, not progress" : "each holds the repository level until it gains a higher level, so their 100% is a ceiling, not progress")
  );
}

export function trackSlots(D, t) {
  var cards = t.cards.filter(function (c) {
    return !c.row.compare && c.row.scored;
  });
  if (!cards.length || t.pct == null) return null;
  var left = function (c) {
    return t.capped.indexOf(c.row.name) >= 0;
  };
  var said = cards.map(function (c) {
    return title(D, c.row.name) + ": " + c.row.topShare + "% at the top level (" + levelName(c.row.top || "top") + ")" + (left(c) ? ", left out of the average because it caps the repository" : "");
  });
  var named = function (c) {
    var top = c.row.top || "top";
    return title(D, c.row.name) + ": " + c.row.topShare + "%" + (top === "gold" ? "" : " at " + levelName(top)) + (left(c) ? ", caps the repository, left out of the average" : "");
  };
  var low = cards.reduce(function (a, c, i) {
    return left(c) || (a >= 0 && cards[a].row.topShare <= c.row.topShare) ? a : i;
  }, -1);
  var high = cards.reduce(function (a, c, i) {
    return left(c) || (a >= 0 && cards[a].row.topShare >= c.row.topShare) ? a : i;
  }, -1);
  var rest =
    (low >= 0 ? "Lowest bar, " + named(cards[low]) : "Each bar is one scorecard") +
    (high >= 0 && high !== low && cards[high].row.topShare > cards[low].row.topShare ? " · highest, " + named(cards[high]) : "") +
    " · dashed line: their average, " + t.pct + "%";
  var cap = el("div", { className: "slotcap", ariaHidden: "true", textContent: rest });
  var bars = cards.map(function (c, i) {
    return el("span", {
      className: "slot" + (left(c) ? " capped" : ""),
      style: "--h:" + c.row.topShare + "%;--c:" + (left(c) ? "var(--unknown)" : RAMP),
      onpointerenter: function () {
        show(i);
      },
      onclick: function () {
        if (on === i) location.hash = "scorecard/" + enc(c.row.name);
        else show(i);
      },
    });
  });
  var on = -1;
  var show = function (i) {
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
        onpointerleave: function () {
          show(-1);
        },
        onfocus: function () {
          if (on < 0) show(low >= 0 ? low : 0);
        },
        onblur: function () {
          show(-1);
        },
        onkeydown: function (e) {
          if (e.key === "Enter" && on >= 0) {
            location.hash = "scorecard/" + enc(cards[on].row.name);
            return;
          }
          if (e.key !== "ArrowRight" && e.key !== "ArrowLeft") return;
          e.preventDefault();
          show(on < 0 ? (e.key === "ArrowRight" ? 0 : bars.length - 1) : (on + (e.key === "ArrowRight" ? 1 : bars.length - 1)) % bars.length);
        },
      },
      ...bars,
      el("span", { className: "avg", ariaHidden: "true" }),
    ),
    cap,
  );
  return wrap;
}
