export var SVGNS = "http://www.w3.org/2000/svg";
export var KNOWN = ["none", "bronze", "silver", "gold", "skipped"];

export var TAG = /^[a-z][a-z0-9]*$/;
export function el(t, a) {
  if (!TAG.test(t)) throw new Error("bad tag name");
  var e = document.createElement(t);
  var kids = Array.prototype.slice.call(arguments, 2);
  Object.assign(e, a || {});
  kids.forEach(function (k) {
    if (k === "" || k == null || k === false) return;
    e.append(k);
  });
  return e;
}

export function svg(t, attrs) {
  var e = document.createElementNS(SVGNS, t);
  Object.keys(attrs || {}).forEach(function (k) {
    e.setAttribute(k, attrs[k]);
  });
  Array.prototype.slice.call(arguments, 2).forEach(function (k) {
    if (k) e.append(k);
  });
  return e;
}

export function link(path, cls, kids) {
  var a = el("a", { href: "#" + path, className: cls || "" });
  (Array.isArray(kids) ? kids : [kids]).forEach(function (k) {
    if (k != null && k !== "") a.append(k);
  });
  return a;
}

export function enc(s) {
  return encodeURIComponent(s);
}
