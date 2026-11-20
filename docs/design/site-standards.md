# Site Standards: Cataloguing the Public Docs

*Status: deterministic catalog and scorecard built, 2026-11-20 (D87). The report element takes
host slots (D89). The public docs are a
Starlight site in `site/`, published to GitHub Pages. Toolbox for Toolbox catalogues each content
page and scores it with `maintainer-site`. This is the maintainer SDLC: the site is part of this
repository, and a page is held to the same shape of standard as a skill or a plugin. Related:
[repo-portals.md](./repo-portals.md), [package-standards.md](./package-standards.md),
[toolbox-for-toolbox.md](./toolbox-for-toolbox.md).*

---

## 1. Conclusion

**A docs page is a catalog entity. The site is not a second product.**

The pages under `site/src/content/docs` are what GitHub Pages serves. The scorecard checks the
facts a reader hits: a title, a description, links that resolve, a place in the sidebar, and no
secret-shaped string. It does not render the site, and it does not grade prose.

| Question | What answers it | In this repo |
|---|---|---|
| Does the page have a title, a description, and links that resolve? | Frontmatter, and each markdown link | bronze |
| Can a reader reach it from the sidebar, and is it not a draft? | `astro.config` autogenerate or an explicit slug | silver |
| Is there no secret-shaped string in the page? | AWS access-key and GitHub token shapes | gold |

A root-relative link has to include the site `base` (`/toolbox/concepts/scorecards/`). Starlight
does not rewrite a markdown link that already starts with `/`, and GitHub Pages serves this
repository under `/toolbox`. An `https` link is left to the reader. A draft, or a page the sidebar
does not list, stops at bronze: it exists, and it is not yet part of the site.

## 2. What this site is

| Source | What it decides | Adopted |
|---|---|---|
| **Astro Starlight** | Content is markdown under `src/content/docs`, with `title` and `description` in frontmatter, and a sidebar declared in `astro.config` | The site is `site/`. The sidebar is an autogenerate entry per section |
| **GitHub Pages** | A project site is served at `https://<user>.github.io/<repo>/` | `site` is `https://loganvolkers.github.io` and `base` is `/toolbox`. A workflow builds with `npm ci` and deploys with the Pages actions, pinned to commits |
| **The one-file HTML report** ([repo-portals.md](./repo-portals.md)) | `toolbox report -o html` is one self-contained page. `-o web` also writes the `<toolbox-report>` element and `report.json` | The site publishes that directory (D88). `src/pages/report-hosted.astro` fills the header, nav, and footer slots (D89). CI checks the three generated files against a fresh run |
| **Packages** ([packages.md](./packages.md)) | Where shared standard packages live is Q15, still open | The site is documentation. It is not a package registry, and `site/package.json` is `private` |

## 3. The catalog

Globs: `site/src/content/docs/**/*.md` and `**/*.mdx`. Id `site-page:<path>`.

| Fact | Meaning |
|---|---|
| `title`, `descriptionChars` | Frontmatter. Empty title and a zero-length description fail bronze |
| `draft`, `sidebarHidden` | `draft: true`, or `sidebar.hidden` / `hidden` |
| `inSidebar` | False when the page is a draft or hidden. True for the home page. Otherwise the slug is under an `autogenerate` directory, or the config quotes the slug |
| `missingFiles` | A markdown link whose target is neither a content page, nor a file under `site/public`, nor an Astro page under `site/src/pages`, nor a file next to the page. A root-relative link that omits `base` is missing |
| `secretHits` | `AKIA…` and `ghp_…` shapes, `path:line kind` |

The live pages are the site a reader gets. The ladder (a gold page, a draft, a page with no title)
lives only in `portal/fixtures/workspace.ndjson`, so a fixture is not published. Starlight's own
404 route stays the framework default; a content page named `404` collides with it.

`maintainer-site` is a constituent of `maintainer-repo` (D86). A new page must reach bronze.

## 4. What is not gated

- A visual review, a screenshot, or a browser pass. The build in CI is the check that the site compiles.
- Prose quality, reading level, or whether a page cites a decision. Decision citations stay on the docs scorecard (D84).
- Link status of `https` URLs. Those change outside the repository.
- The Pages environment existing in the GitHub repository settings. The workflow is the part this repo owns.

## Sources

- Astro Starlight, content pages and sidebar: the `site/` config in this repository, built with `npm run build`.
- GitHub Pages project sites: `actions/upload-pages-artifact` and `actions/deploy-pages` in `.github/workflows/pages.yml`.
- [repo-portals.md](./repo-portals.md) §4, the one-file HTML report and the `<toolbox-report>` slots (D88, D89).
- [packages.md](./packages.md), Q15, which this site does not decide.
