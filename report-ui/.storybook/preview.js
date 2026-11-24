import "../src/index.js";

const page = document.createElement("style");
page.textContent =
  "body{margin:0;background:#f7f6f2;color:#1d1d1b}" +
  "@media (prefers-color-scheme:dark){body{background:#141413;color:#ecebe7}}";
document.head.append(page);

export default {
  parameters: { layout: "fullscreen" },
};
