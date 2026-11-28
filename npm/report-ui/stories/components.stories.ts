import { badge, medal } from "../src/lib/bars";
import { basisBadge } from "../src/lib/basis";
import { goalBar, kpiBar } from "../src/lib/campaigns";
import { el } from "../src/lib/dom";
import { ring, tallyRing } from "../src/lib/rings";
import { component } from "./report";

export default { title: "Components" };

const row = (...kids) => el("div", { style: "display:flex;gap:16px;align-items:center;flex-wrap:wrap;margin-bottom:16px" }, ...kids);
const LEVELS = ["none", "bronze", "silver", "gold"];

export const Medals = component(() => row(...LEVELS.map((l) => el("div", {}, medal(l, 48), el("div", { textContent: l })))));

export const LevelBadges = component(() => row(...LEVELS.concat(["skipped"]).map((l) => badge(l))));

export const Rings = component(() => row(ring(0, { size: 60 }), ring(42, { size: 60 }), ring(100, { size: 60 }), ring(null, { size: 60, label: "not scored" })));

export const TallyRings = component(() => [
  row(tallyRing({ ahead: 4, parity: 1, behind: 1, unknown: 0 }, 6, 88), tallyRing({ ahead: 0, parity: 0, behind: 6, unknown: 0 }, 6, 88)),
  row(tallyRing({ ahead: 2, parity: 0, behind: 1, unknown: 3 }, 6, 88), tallyRing({ ahead: 0, parity: 0, behind: 0, unknown: 6 }, 6, 88)),
]);

export const Basis = component(() => row(basisBadge("measured"), basisBadge("judged"), basisBadge("measured", true), basisBadge("judged", true)));

export const CampaignBars = component(() => [
  goalBar("", 64, 80, true),
  goalBar("on", 100, 80, true),
  goalBar("empty", 0, 80, true, [{ met: true }, { met: false }, { met: true }]),
  kpiBar(37, "37% of rules pass"),
  kpiBar(62, "62% pass, 20% only because no agent has assessed them yet", false, 20),
  kpiBar(62, "Below target, 20% only because no agent has assessed them yet", true, 20),
]);
