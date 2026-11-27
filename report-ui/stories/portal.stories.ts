import { h, provide } from "vue";
import PortalApp from "../../ui/src/components/PortalApp.vue";
import type { PortalApi } from "../../ui/src/api";
import { portalApi } from "../../ui/src/requests";
import assessments from "../fixtures/s4s-assessments.json";
import type { ReportData } from "../src/types";
import "../../ui/src/app.css";

export default { title: "Hosted portal" };

const id = "service:checkout";
const fields = {
  title: "Checkout",
  description: "Processes payments for the storefront.",
  ownership: { owner: "group:payments", status: "verified" },
  grafanaUrl: "https://example.com/d/checkout",
  runbookUrl: "https://example.com/runbooks/checkout",
};
const entity = { id, kind: "Service", fields };
const snapshot = "demo-1";
const report: ReportData = {
  portal: "Acme portal",
  viewer: "user:alex",
  snapshot,
  catalog: { Group: ["group:payments"], Service: [id] },
  entityOverview: { [id]: fields },
  entities: { [id]: { fields, provenance: { title: "git:catalog", grafanaUrl: "git:catalog" } } },
  scorecards: { readiness: {
    title: "Production readiness", kind: "Service", levels: ["silver"],
    rules: { runbook: { title: "Runbook", level: "silver", check: "has(entity.runbookUrl)", basis: "measured" } },
  } },
  scores: { readiness: { [id]: { level: "silver", rules: { runbook: "pass" } } } },
  campaigns: { production: { title: "Production readiness", percent: 50, targetPercent: 80, meeting: 1,
    inScope: 2, includes: [{ scorecard: "readiness", atLeast: "silver" }], entities: { [id]: { meeting: true } } } },
};

function story(route: string, data: unknown = report, status = 200) {
  const request: PortalApi = async (path, init) => {
    const json = (body: unknown, status = 200, at = snapshot) => ({ status, body, snapshot: at });
    const url = new URL(path, "https://portal.example.com");
    if (url.pathname === "/v1/views/report") return json(data, status);
    if (url.pathname === "/v1/entities") return json({ items: url.searchParams.get("q") === "missing" ? [] : [entity] });
    if (url.pathname === "/v1/views/home") {
      if (url.searchParams.has("public_catalog")) return (data as ReportData | null)?.publicCatalog ? json(data) : json(null, 204);
      return json({ owned: [{ id, levels: { readiness: "silver" }, failing: [] }] });
    }
    if (url.pathname === "/v1/views/ops") return json({
      snapshot, input_version: 3, recompute_ms: 12,
      connectors: [{ name: "git", runtime: "stdio", registered: true, last_read: { service: "2026-11-25" } }],
      pending_commands: [], unverified_ownership: [], orphans: [], diagnostics: [],
    });
    if (url.pathname.startsWith("/v1/views/entity/")) return json({
      actions: [{ name: "refresh-slo", description: "Read the latest SLO data", allowed: true }],
      freshness: [{ connector: "git", kind: "service", last_read: "2026-11-25" }],
    });
    if (init?.method === "POST" && url.pathname.endsWith("/refresh")) return json({ commands: ["demo"] }, 202);
    if (url.pathname.startsWith("/v1/commands/")) return json({ status: "succeeded", snapshot });
    if (url.pathname.endsWith("/explain")) return json({ authority: { status: "verified", explanation: "The ownership claim matches the repository." } });
    if (url.pathname === "/v1/snapshot") return json({ snapshot });
    return json(null, 404);
  };
  return {
    render: () => ({
      setup() {
        provide(portalApi, request);
        return () => h(PortalApp, { initialRoute: route });
      },
    }),
  };
}

const entityRoute = "entity/" + encodeURIComponent(id);
export const Home = story("overview");
export const Catalog = story("catalog/Service");
export const Search = story("search?q=checkout");
export const SearchEmpty = story("search?q=missing");
export const Entity = story(entityRoute);
export const EntityWithoutScore = story(entityRoute, { ...report, scores: {} });
export const EntityRestricted = story(entityRoute, {
  ...report,
  entities: { [id]: { fields: { ...fields, contact: { redacted: true, visible_to: "the payments team", requires: "can_view_details" } } } },
  scores: { readiness: { [id]: { level: "silver" } } },
});
export const Scorecard = story("scorecard/readiness");
export const Campaign = story("campaign/production");
export const Operations = story("operations");
export const Assessments = story("assessments", assessments);
export const AssessmentRun = story("assessment/" + encodeURIComponent("assessment:crate-design/crate:seikika-expr"), assessments);
export const AssessmentRubric = story("rubric/" + encodeURIComponent("rubric:seikika/assessment-quality"), assessments);
export const Loading = story(entityRoute, null, 503);
export const SignIn = story("overview", { title: "Authentication required" }, 401);
export const Missing = story("entity/service%3Amissing");

export const PublicCatalog = story("overview", {
  portal: "Public demo",
  publicCatalog: true,
  catalog: { Repository: ["repository:ledger"] },
  entityOverview: { "repository:ledger": {
    title: "example/ledger", repositoryUrl: "https://github.com/example/ledger",
  } },
  scorecards: {},
  scores: {},
});
