export var SVGNS = "http://www.w3.org/2000/svg";
export var KNOWN = ["none", "bronze", "silver", "gold", "skipped"];

// What a view may hand to el(), svg() or link() as a child: falsy values and "" are skipped.
export type Kid = Node | string | number | null | undefined | false;
export type Props = Record<string, unknown> | null | undefined;

export var TAG = /^[a-z][a-z0-9]*$/;
export function el<K extends keyof HTMLElementTagNameMap>(t: K, a?: Props, ...kids: Kid[]): HTMLElementTagNameMap[K];
export function el(t: string, a?: Props, ...kids: Kid[]): HTMLElement;
export function el(t: string, a?: Props, ...kids: Kid[]): HTMLElement {
  if (!TAG.test(t)) throw new Error("bad tag name");
  var e = document.createElement(t);
  Object.assign(e, a || {});
  kids.forEach(function (k) {
    if (k === "" || k == null || k === false) return;
    e.append(typeof k === "number" ? String(k) : k);
  });
  return e;
}

export function svg(t: string, attrs?: Record<string, string | number> | null, ...kids: Kid[]): SVGElement {
  var e = document.createElementNS(SVGNS, t) as SVGElement;
  Object.keys(attrs || {}).forEach(function (k) {
    e.setAttribute(k, String((attrs as Record<string, string | number>)[k]));
  });
  kids.forEach(function (k) {
    if (k) e.append(typeof k === "number" ? String(k) : k);
  });
  return e;
}

export function link(path: string, cls?: string | null, kids?: Kid | Kid[]): HTMLAnchorElement {
  var a = el("a", { href: "#" + path, className: cls || "" });
  (Array.isArray(kids) ? kids : [kids]).forEach(function (k) {
    if (k != null && k !== "" && k !== false) a.append(typeof k === "number" ? String(k) : k);
  });
  return a;
}

export function enc(s: string | number): string {
  return encodeURIComponent(s);
}
