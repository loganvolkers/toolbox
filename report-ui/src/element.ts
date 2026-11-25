import type { ReportData } from "./types";
import type { Summary } from "./lib/summary";
import { boxLogo } from "./lib/bars";
import { el, link } from "./lib/dom";
import { aliasLevels } from "./lib/levels";
import { missingTarget, notFound, parseHash, trackByPath } from "./lib/routes";
import { registerIds } from "./lib/rings";
import { ruleTitles } from "./lib/rules";
import { template } from "./lib/styles";
import { campaignNamesSorted, summarize } from "./lib/summary";
import { tracksSorted } from "./lib/tracks";
import { fmt, plural } from "./lib/words";
import { overviewViews } from "./views/overview";
import { scorecardsViews } from "./views/scorecards";
import { tracksViews } from "./views/tracks";
import { campaignsViews } from "./views/campaigns";
import { catalogViews } from "./views/catalog";
import { entityViews } from "./views/entity";
import { ruleViews } from "./views/rule";
import { compareViews } from "./views/compare";

type Views = typeof overviewViews &
  typeof scorecardsViews &
  typeof tracksViews &
  typeof campaignsViews &
  typeof catalogViews &
  typeof entityViews &
  typeof ruleViews &
  typeof compareViews;

// The views are installed on the prototype below, so the class declares them through this interface.
export interface ToolboxReport extends Views {}

// Look up a part of the shadow tree's template, which always has it.
function part(root: ShadowRoot, id: string): HTMLElement {
  return root.getElementById(id) as HTMLElement;
}

export class ToolboxReport extends HTMLElement {
  declare _booted: boolean;
  declare _tabFocus: boolean;
  declare _lastMode: string | undefined;
  declare _lastRoute: string | undefined;

  connectedCallback() {
    if (this._booted) return;
    this._booted = true;
    var root = this.attachShadow({ mode: "open" });
    root.append(template.content.cloneNode(true));
    // A button, not a link: the hash is the route, and the target is inside the shadow root.
    part(root, "skip").addEventListener("click", function () {
      var body = part(root, "body");
      body.tabIndex = -1;
      body.focus();
    });
    // One tooltip at the top of the shadow tree, fixed to the viewport, so no collapsed or scrolling box
    // clips it. Its trigger already carries the same words for screen readers.
    var ftip = el("div", { className: "ftip", hidden: true, ariaHidden: "true" });
    root.append(ftip);
    var tipFor: HTMLElement | null = null;
    var tipTimer: ReturnType<typeof setTimeout> | undefined;
    var hideTip = function () {
      clearTimeout(tipTimer);
      tipFor = null;
      ftip.hidden = true;
    };
    var showTip = function (e: Event) {
      if (e.target === ftip) return clearTimeout(tipTimer);
      var src = e.target as Element | null;
      var t = src && src.closest ? (src.closest(".basis[data-tip],abbr.gloss,a.rl[data-tip]") as HTMLElement | null) : null;
      if (t === tipFor) clearTimeout(tipTimer);
      if (!t || !t.dataset.tip || t === tipFor) return;
      clearTimeout(tipTimer);
      tipFor = t;
      ftip.textContent = t.dataset.tip || "";
      ftip.hidden = false;
      var r = t.getBoundingClientRect();
      var vw = document.documentElement.clientWidth;
      var vh = document.documentElement.clientHeight;
      var left = Math.max(12, Math.min(r.left, vw - 12 - ftip.offsetWidth));
      var top = r.bottom + 4 + ftip.offsetHeight > vh - 8 ? r.top - 4 - ftip.offsetHeight : r.bottom + 4;
      ftip.style.left = left + "px";
      ftip.style.top = Math.max(8, top) + "px";
    };
    root.addEventListener("mouseover", showTip);
    root.addEventListener("focusin", showTip);
    // A short grace period lets the pointer cross the gap onto the tooltip without it closing.
    root.addEventListener("mouseout", function (e) {
      if (tipFor && (e.target === ftip || tipFor.contains(e.target as Node)) && root.activeElement !== tipFor) tipTimer = setTimeout(hideTip, 200);
    });
    root.addEventListener("focusout", function (e) {
      if (tipFor && tipFor.contains(e.target as Node)) hideTip();
    });
    window.addEventListener("keydown", function (e) {
      if (e.key === "Escape" && tipFor) hideTip();
    });
    window.addEventListener("scroll", hideTip, { passive: true, capture: true });
    window.addEventListener("hashchange", hideTip);
    var self = this;
    var inline = this.querySelector('script[type="application/json"]');
    var load = inline
      ? Promise.resolve(JSON.parse(inline.textContent || "null") as ReportData)
      : fetch(this.getAttribute("src") || "").then(function (r): Promise<ReportData> {
          if (!r.ok) throw new Error("report JSON HTTP " + r.status);
          return r.json();
        });
    load
      .then(function (data) {
        self.mount(root, data);
      })
      .catch(function (e) {
        part(root, "body").replaceChildren(el("section", {}, el("div", { className: "empty", textContent: "Failed to load report: " + e.message })));
      });
  }


  mount(root: ShadowRoot, D: ReportData) {
    var self = this;
    aliasLevels(D);
    ruleTitles(D);
    registerIds(([] as string[]).concat(...Object.values(D.catalog || {})));
    var S = summarize(D);
    part(root, "top").replaceChildren(boxLogo(), el("h1", { textContent: D.portal || "Toolbox report" }));
    part(root, "meta").textContent =
      fmt(S.entityCount) +
      " entities in the box (" +
      fmt(Object.keys(S.worst).length) +
      " scored) · " +
      plural(
        S.rows.filter(function (r) {
          return !r.compare;
        }).length,
        "scorecard",
      ) +
      (Object.keys(D.comparisons || {}).length ? " · " + plural(Object.keys(D.comparisons || {}).length, "comparison factor") : "") +
      (D.snapshot ? " · facts snapshot #" + D.snapshot : " · no facts collected yet");
    // Each history entry keeps its scroll position, so Back returns to the same place in a long list.
    var settle: ReturnType<typeof setTimeout> | undefined;
    var draw = function () {
      clearTimeout(settle);
      self.render(root, D, S);
    };
    var keep = function () {
      clearTimeout(settle);
      if (history.replaceState) history.replaceState(Object.assign({}, history.state, { y: window.scrollY }), "");
    };
    window.addEventListener("hashchange", draw);
    window.addEventListener("scroll", function () {
      clearTimeout(settle);
      settle = setTimeout(keep, 200);
    });
    // A click that navigates can land before the scroll settles; capture runs before the hash changes.
    window.addEventListener("click", keep, true);
    draw();
  }


  render(root: ShadowRoot, D: ReportData, S: Summary) {
    var route = parseHash();
    if (route.mode === "tracks" && route.name) route.name = trackByPath(D, route.name);
    var modes = part(root, "modes");
    var body = part(root, "body");
    var self = this;
    var go = function (path) {
      var next = "#" + path;
      if (location.hash === next) self.render(root, D, S);
      else location.hash = path;
    };
    var modeOf = function (r) {
      if (r.mode === "entity") return "catalog";
      if (r.mode === "campaign") return "campaigns";
      return r.mode === "scorecard" || r.mode === "rule" ? "scorecards" : r.mode;
    };
    var activeMode = route.bare && !(D.scorecards || {})[route.name || ""] ? null : modeOf(route);
    var campaigns = campaignNamesSorted(D);
    var tabs: { label: string; path: string; id: string; count?: number }[] = [
      { label: "Overview", path: "overview", id: "overview" },
    ];
    var trackCount = tracksSorted(D, S).length;
    if (trackCount) tabs.push({ label: "Tracks", path: "tracks", id: "tracks", count: trackCount });
    tabs.push({ label: "Catalog", path: "catalog", id: "catalog", count: S.entityCount });
    if (campaigns.length || activeMode === "campaigns") tabs.push({ label: "Campaigns", path: "campaigns", id: "campaigns", count: campaigns.length });
    tabs.push(
      {
        label: "Scorecards",
        id: "scorecards",
        count: S.rows.length,
        path: "scorecards",
      },
    );
    var comparisons = Object.keys(D.comparisons || {});
    if (comparisons.length) tabs.push({ label: "Compare", path: "compare", id: "compare", count: comparisons.length });
    var levelCards = S.rows.filter(function (r) {
      return !r.compare;
    });
    tabs.forEach(function (t) {
      if (t.id === "scorecards") {
        t.count = levelCards.length;
      }
    });
    // On a phone the tabs form an even grid: four as two rows of two, never one left alone on a row.
    modes.style.setProperty("--tabcols", String(tabs.length <= 3 ? tabs.length : tabs.length === 4 ? 2 : 3));
    modes.replaceChildren(
      ...tabs.map(function (t) {
        var selected = activeMode === t.id;
        return el(
          "button",
          {
            role: "tab",
            id: "tab-" + t.id,
            ariaSelected: String(selected),
            tabIndex: selected || (!activeMode && t.id === "overview") ? 0 : -1,
            onclick: function (ev) {
              if (ev.currentTarget === root.activeElement) self._tabFocus = true;
              go(t.path);
            },
          },
          t.label,
          t.count != null ? el("span", { className: "count", textContent: fmt(t.count) }) : null,
        );
      }),
    );
    requestAnimationFrame(function () {
      var cur = modes.querySelector<HTMLElement>('[aria-selected="true"]');
      if (cur && modes.scrollWidth > modes.clientWidth) {
        var at = cur.getBoundingClientRect().left - modes.getBoundingClientRect().left + modes.scrollLeft;
        modes.scrollLeft = Math.max(0, at + cur.offsetWidth / 2 - modes.clientWidth / 2);
      }
      var fade = function () {
        modes.classList.toggle("more", modes.scrollLeft + modes.clientWidth < modes.scrollWidth - 4);
        modes.classList.toggle("less", modes.scrollLeft > 4);
      };
      fade();
      modes.onscroll = fade;
    });
    modes.onkeydown = function (ev) {
      var keys: Record<string, number | "home" | "end"> = { ArrowLeft: -1, ArrowRight: 1, Home: "home", End: "end" };
      var dir = keys[ev.key];
      if (dir == null) return;
      ev.preventDefault();
      var buttons = Array.from(modes.querySelectorAll<HTMLElement>('[role="tab"]'));
      var i = buttons.indexOf(root.activeElement as HTMLElement);
      if (i < 0)
        i = buttons.findIndex(function (b) {
          return b.getAttribute("aria-selected") === "true";
        });
      var next = dir === "home" ? 0 : dir === "end" ? buttons.length - 1 : (i + dir + buttons.length) % buttons.length;
      self._tabFocus = true;
      buttons[next].click();
    };
    var view: HTMLElement;
    var missing = missingTarget(D, route);
    if (missing) view = notFound(missing);
    else if (route.mode === "overview") view = this.overview(D, S);
    else if (route.mode === "catalog") view = this.catalog(D, S, route.kind, route.query);
    else if (route.mode === "entity") view = this.entity(D, S, route.id);
    else if (route.mode === "campaigns" || route.mode === "campaign") view = this.campaigns(D, S, route.name);
    else if (route.mode === "compare") view = this.compare(D, route.name);
    else if (route.mode === "tracks") view = this.tracks(D, S, route.name);
    else if (route.mode === "rule") view = this.rule(D, S, route.scorecard, route.rule);
    else if (route.mode === "scorecards") view = this.scorecardIndex(D, S, route.query);
    else view = this.scorecard(D, S, route.name, route.query);
    view.classList.add("view");
    body.replaceChildren(view);
    var h2s = view.querySelectorAll<HTMLElement>("h2");
    var named = view.querySelector<HTMLElement>("[data-focus] h2") || (route.name || route.kind ? h2s[1] : null) || h2s[0];
    document.title = (named && route.mode !== "overview" ? named.textContent + " · " : "") + (D.portal || "Toolbox report");
    var target = view.querySelector<HTMLElement>("[data-focus]");
    var hostTop = root.host.getBoundingClientRect().top + window.scrollY;
    if (this._tabFocus) {
      this._tabFocus = false;
      var sel = modes.querySelector<HTMLElement>('[aria-selected="true"]');
      if (sel) sel.focus({ preventScroll: true });
      if (this._lastRoute !== location.hash) window.scrollTo(0, Math.max(0, hostTop));
    } else if (this._lastMode !== undefined && this._lastRoute !== location.hash) {
      // A focus target near the top opens the page from its header, so the portal's name stays in sight.
      var targetTop = target ? target.getBoundingClientRect().top + window.scrollY : hostTop;
      window.scrollTo(0, targetTop - hostTop < window.innerHeight * 0.6 ? Math.max(0, hostTop) : targetTop - 8);
      var heading = (target && target.querySelector<HTMLElement>("h2")) || named;
      if (heading) {
        if (!heading.hasAttribute("tabindex")) heading.tabIndex = -1;
        heading.focus({ preventScroll: true });
      }
    }
    var back = history.state && typeof history.state.y === "number" ? history.state.y : null;
    if (back !== null && this._lastRoute !== undefined && this._lastRoute !== location.hash) window.scrollTo(0, back);
    this._lastMode = route.mode;
    this._lastRoute = location.hash;
  }
}

Object.assign(ToolboxReport.prototype, overviewViews, scorecardsViews, tracksViews, campaignsViews, catalogViews, entityViews, ruleViews, compareViews);

if (!customElements.get("toolbox-report")) customElements.define("toolbox-report", ToolboxReport);
