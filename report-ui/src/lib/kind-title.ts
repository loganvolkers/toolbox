import type { ReportData } from "../types";

const acronyms: Record<string, string> = {
  Api: "API", Cli: "CLI", Json: "JSON", Mcp: "MCP", Sdlc: "SDLC", Ui: "UI", Url: "URL",
};

/** Use the configured kind title or turn a catalog identifier into readable words. */
export function kindTitle(data: ReportData, kind?: string | null): string {
  const name = kind ?? "";
  const title = data.kinds?.[name]?.title;
  if (title) return title;
  const words = name.match(/OpenSSF|[A-Z]{2,}(?![a-z])|[A-Z][a-z0-9]*|[a-z0-9]+/g) ?? [name];
  return words.map((word, index) => {
    if (Object.hasOwn(acronyms, word)) return acronyms[word];
    if (word === "OpenSSF" || /^[A-Z]{2,}$/.test(word)) return word;
    return index ? word.toLowerCase() : word;
  }).join(" ");
}
