export type PortalRoute =
  | { page: "home" | "ops"; api: string }
  | { page: "search"; api: string; query: string }
  | { page: "catalog"; api: string; id: string; filter: string }
  | { page: "entity" | "scorecard" | "campaign"; api: string; id: string }
  | { page: "assessments"; api: string; mode: "assessments" | "assessment" | "rubrics" | "rubric"; id?: string; filter?: string }
  | { page: "missing"; api: null };

const encode = encodeURIComponent;

export function portalRoute(url: URL): PortalRoute {
  let segments: string[];
  try {
    segments = url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  } catch {
    return { page: "missing", api: null };
  }
  const [section, id, detail] = segments;
  if (!section) return { page: "home", api: "/v1/views/home" };
  if (section === "ops" && !id) return { page: "ops", api: "/v1/views/ops" };
  if (section === "assessments" && segments.length <= 3) {
    const api = "/v1/views/report";
    if (!id) return { page: "assessments", mode: "assessments", api, filter: url.searchParams.get("rubric") ?? undefined };
    if (id === "rubrics") return { page: "assessments", mode: detail ? "rubric" : "rubrics", api, id: detail };
    if (id === "run" && detail) return { page: "assessments", mode: "assessment", api, id: detail };
  }
  if (section === "search" && !id) {
    const query = url.searchParams.get("q") ?? "";
    return { page: "search", query, api: "/v1/entities?q=" + encode(query) + "&limit=50" };
  }
  if (!id || segments.length !== 2) return { page: "missing", api: null };
  if (section === "c") {
    const filter = url.searchParams.get("filter") ?? "";
    const api = "/v1/entities?kind=" + encode(id) + "&limit=200" + (filter ? "&filter=" + encode(filter) : "");
    return { page: "catalog", id, filter, api };
  }
  if (section === "e") return { page: "entity", id, api: "/v1/views/entity/" + encode(id) };
  if (section === "s") return { page: "scorecard", id, api: "/v1/views/scorecard/" + encode(id) };
  if (section === "cgn") return { page: "campaign", id, api: "/v1/views/campaign/" + encode(id) };
  return { page: "missing", api: null };
}
