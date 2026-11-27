import { h, ref } from "vue";
import PageFrame from "@seikika/ui/components/PageFrame.vue";
import PortalHeader from "../../ui/src/components/PortalHeader.vue";
import PortalView from "../../ui/src/components/PortalView.vue";
import type { ApiResult } from "../../ui/src/api";
import { portalRoute } from "../../ui/src/routes";
import assessments from "../fixtures/s4s-assessments.json";
import "../../ui/src/app.css";

export default { title: "Hosted portal" };

const entity = {
  id: "service:checkout",
  kind: "Service",
  fields: {
    title: "Checkout",
    description: "Processes payments for the storefront.",
    ownership: { owner: "group:payments", status: "verified" },
    grafanaUrl: "https://example.com/d/checkout",
    runbookUrl: "https://example.com/runbooks/checkout",
  },
  provenance: { title: "git:catalog", grafanaUrl: "git:catalog" },
};
const scorecards = { readiness: { level: "silver", rules: { runbook: { outcome: "pass" } } } };
const campaign = { name: "production", title: "Production readiness", status: "in progress", percent: 50,
  targetPercent: 80, meeting: 1, inScope: 2, entities: [{ id: entity.id, meeting: false, gaps: ["owner review"] }] };
const detail = {
  entity, scorecards, snapshot: "demo-1",
  relations: { out: [{ relation: "dependsOn", to: "service:ledger" }], in: [] },
  actions: [{ name: "refresh-slo", description: "Read the latest SLO data", allowed: true }],
  freshness: [{ connector: "git", kind: "service", last_read: "2026-11-25" }],
};

function story(path: string, body: ApiResult["body"], status = 200) {
  return {
    render: () => ({
      setup() {
        const route = portalRoute(new URL(path, "https://portal.example.com"));
        const message = ref("");
        return () => h(PageFrame, { class: "portal-ui" }, {
          navigation: () => h(PortalHeader, { user: "alex", assessments: route.page === "assessments" }),
          default: () => h(PortalView, {
            route,
            result: body ? { status, body, snapshot: "demo-1" } : null,
            refresh: { running: false, failed: false, message: message.value },
            onRefresh: () => { message.value = "Refreshed at snapshot demo-2"; },
          }),
        });
      },
    }),
  };
}

export const Home = story("/", {
  portal: { title: "Acme portal" }, viewer: { user: "alex" }, teams: ["group:payments"],
  owned: [{ id: entity.id, levels: { readiness: "silver" }, failing: [] }],
  kinds: { Service: 1 }, scorecards: [{ name: "readiness", kind: "Service" }], campaigns: [campaign],
});
export const Catalog = story("/c/Service", { items: [entity] });
export const Search = story("/search?q=checkout", { items: [entity] });
export const SearchEmpty = story("/search?q=missing", { items: [] });
export const Entity = story("/e/service%3Acheckout", detail);
export const EntityWithoutScore = story("/e/service%3Acheckout", { ...detail, scorecards: {} });
export const EntityRestricted = story("/e/service%3Acheckout", {
  ...detail,
  scorecards: { readiness: { level: "silver", rules: { redacted: true, visible_to: "the payments team", requires: "owner" } } },
  actions: [{ name: "refresh-slo", allowed: false, permission: "owner" }],
});
export const Scorecard = story("/s/readiness", {
  scorecard: { title: "Production readiness", kind: "Service", owner: "group:payments" },
  by_level: { silver: [entity.id] }, failing_rules: { "slo-probe": 1 },
});
export const Campaign = story("/cgn/production", { campaign });
export const Operations = story("/ops", {
  snapshot: "demo-1", input_version: 3, recompute_ms: 12,
  connectors: [{ name: "git", runtime: "stdio", registered: true, last_read: { service: "2026-11-25" } }],
  pending_commands: [], unverified_ownership: [], orphans: [], diagnostics: [],
});
export const Assessments = story("/assessments", assessments);
export const AssessmentRun = story("/assessments/run/" + encodeURIComponent("assessment:crate-design/crate:seikika-expr"), assessments);
export const AssessmentRubric = story("/assessments/rubrics/" + encodeURIComponent("rubric:seikika/assessment-quality"), assessments);
export const Loading = story("/e/service%3Acheckout", null);
export const SignIn = story("/", { title: "Authentication required" }, 401);
export const Missing = story("/missing", null);
