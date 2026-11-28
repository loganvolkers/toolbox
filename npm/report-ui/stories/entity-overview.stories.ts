import { page } from "./report";

export default { title: "Published report" };

export const ServiceWithoutScore = page({
  portal: "Storefront",
  snapshot: "demo-1",
  catalog: { Service: ["service:checkout"], Group: ["group:payments"] },
  kinds: { Service: { title: "Service" }, Group: { title: "Team" } },
  entityOverview: {
    "service:checkout": {
      title: "Checkout",
      description: "Processes payments for the storefront.",
      ownership: { owner: "group:payments", status: "verified" },
      grafanaUrl: "https://example.com/d/checkout",
      runbookUrl: "https://example.com/runbooks/checkout",
    },
  },
  scorecards: {},
  scores: {},
}, "entity/service%3Acheckout");
