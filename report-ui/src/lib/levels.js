import { KNOWN } from "./dom.js";

// Custom level names (a one-level scorecard's "ten", say) borrow the colour of the
// standard level at the same height, counted from the top.
export var ALIAS = {};
export var LEVEL_NOTE = {};
export var LEVEL_DEF = {};
export var ONLY_LEVEL = new Set();
export function aliasLevels(D) {
  Object.entries(D.scorecards || {}).forEach(function (e) {
    var sc = e[1];
    var levels = sc.levels || [];
    levels.forEach(function (l, i) {
      if (KNOWN.indexOf(l) < 0 && !ALIAS[l]) {
        ALIAS[l] = ["gold", "silver", "bronze"][Math.min(2, levels.length - 1 - i)];
        var said = (sc.description || "").split(/(?<=\.)\s+/).filter(function (s) {
          return s.toLowerCase().split(/[^a-z0-9-]+/).indexOf(String(l).toLowerCase()) >= 0;
        })[0];
        if (said) LEVEL_DEF[l] = said.replace(/`/g, "");
        LEVEL_NOTE[l] = "“" + l + "” is a level " + (sc.title || e[0]) + " names itself" + (levels.length === 1 ? ": its only level, so reaching it is the top." : ", ranked like " + ALIAS[l] + ".") + (said ? " " + LEVEL_DEF[l] : "");
        if (levels.length === 1) ONLY_LEVEL.add(l);
      }
    });
  });
}

export function known(l) {
  return KNOWN.indexOf(l) >= 0 ? l : ALIAS[l] || null;
}

// Two textures only: hatched for "not assessed by an agent yet", dotted for "checks not run".
// Not scored is absence, so it is a plain pale fill.
export var UNSCORED = "color-mix(in srgb,var(--faint) 45%,var(--bg))";
export var UNRUN = "radial-gradient(circle,var(--faint) 1.3px,transparent 1.7px) 0 0/5px 5px";

export function fill(l) {
  if (l === "unscored") return UNSCORED;
  return "var(--" + (known(l) || "accent") + ")";
}

export function ink(l) {
  return known(l) ? "var(--" + known(l) + "-ink)" : "var(--accent)";
}
