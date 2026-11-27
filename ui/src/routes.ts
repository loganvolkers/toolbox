// Keep links published by earlier portal builds working with the shared report routes.
export function restorePortalRoute(url: URL) {
  const segments = url.pathname.split("/").filter(Boolean);
  if (!segments.length || segments.length === 1 && segments[0] === "report") return;
  const [section, id, detail] = segments;
  let path = "missing";
  if (segments.length === 1) {
    if (section === "ops") path = "operations";
    if (section === "search" || section === "assessments") path = section;
  } else if (segments.length === 2) {
    const routes = { c: "catalog", e: "entity", s: "scorecard", cgn: "campaign" } as const;
    if (Object.hasOwn(routes, section)) path = routes[section as keyof typeof routes] + "/" + id;
    if (section === "assessments" && id === "rubrics") path = "rubrics";
  } else if (segments.length === 3 && section === "assessments") {
    if (id === "rubrics") path = "rubric/" + detail;
    if (id === "run") path = "assessment/" + detail;
  }
  const params = new URLSearchParams(url.search);
  params.delete("as");
  for (const key of params.keys()) url.searchParams.delete(key);
  url.pathname = "/";
  url.hash = path + (params.size ? "?" + params : "");
  history.replaceState(history.state, "", url);
}
