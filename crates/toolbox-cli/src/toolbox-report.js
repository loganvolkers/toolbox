// <toolbox-report>: Overview, Catalog, Campaigns, Scorecards, and Compare as a custom element.
//
// Data is a child <script type="application/json">, or the src attribute (a URL of that JSON).
// A host page projects its own chrome into the slots header, nav, and footer. Those children
// stay in light DOM; this file does not read or concatenate them. Mode chrome and scorecard
// navigation live inside the shadow tree and are not slots.
(function () {
  if (customElements.get("toolbox-report")) return;

  var CSS = [
    ":host{display:block;color:var(--fg);background:var(--bg);font:15px/1.55 system-ui,-apple-system,'Segoe UI',sans-serif;",
    "--bg:#f7f6f2;--fg:#1d1d1b;--muted:#5f5f5a;--faint:#8a8a84;--line:#e4e2dc;--card:#fff;--tint:#f1efe9;",
    "--accent:#2f5d8a;--accent-bg:#e8eff6;--pass:#21683e;--fail:#b23a2f;--unknown:#7a6420;",
    "--none:#d0493d;--none-ink:#a8352a;--bronze:#bf7b3a;--bronze-ink:#86501b;",
    "--silver:#9aa6b2;--silver-ink:#525d69;--gold:#e1b030;--gold-ink:#7a5a00;--skipped:#c9c8c2;--skipped-ink:#5f5f5a;",
    "--judged:#6a45b8;--judged-bg:#f1ecfb;",
    "--shadow:0 1px 2px rgba(30,30,25,.06),0 4px 14px rgba(30,30,25,.05)}",
    "@media (prefers-color-scheme:dark){:host{--bg:#141413;--fg:#ecebe7;--muted:#b0afa8;--faint:#8b8a84;--line:#2f2f2c;",
    "--card:#1d1d1b;--tint:#252523;--accent:#8fbbe6;--accent-bg:#1e2a36;--pass:#6fc493;--fail:#f08a80;--unknown:#d6c177;",
    "--none:#e0665b;--none-ink:#f39a91;--bronze:#d39a5c;--bronze-ink:#e9b985;--silver:#aab5c1;--silver-ink:#cbd4dd;",
    "--gold:#e6c04c;--gold-ink:#f0d57e;--skipped:#5c5c58;--skipped-ink:#b0afa8;--judged:#c3aef5;--judged-bg:#2a2340;--shadow:none}}",
    "*{box-sizing:border-box}",
    "h2[tabindex='-1']:focus{outline:none}",
    "main{max-width:1120px;margin:0 auto;padding:28px 20px 72px}",
    "a{color:var(--accent)}",
    "a:focus-visible,button:focus-visible,summary:focus-visible,input:focus-visible{outline:2px solid var(--accent);outline-offset:2px;border-radius:6px}",
    ".top{display:flex;align-items:center;gap:14px;margin-bottom:6px}",
    ".top svg{flex:none}",
    "h1{font-size:26px;line-height:1.2;margin:0;letter-spacing:-.01em}",
    "h2{font-size:19px;line-height:1.3;margin:0;letter-spacing:-.005em}",
    "h3{font-size:12px;margin:18px 0 8px;color:var(--muted);font-weight:650;text-transform:uppercase;letter-spacing:.06em}",
    ".meta{color:var(--muted);font-size:13px}",
    ".modes{display:flex;gap:4px;margin:20px 0 22px;border-bottom:1px solid var(--line);overflow-x:auto;scrollbar-width:none}",
    ".modes::-webkit-scrollbar{display:none}",
    ".modes.more{-webkit-mask-image:linear-gradient(90deg,#000 85%,transparent);mask-image:linear-gradient(90deg,#000 85%,transparent)}",
    ".modes.less{-webkit-mask-image:linear-gradient(90deg,transparent,#000 15%);mask-image:linear-gradient(90deg,transparent,#000 15%)}",
    ".modes.more.less{-webkit-mask-image:linear-gradient(90deg,transparent,#000 15%,#000 85%,transparent);mask-image:linear-gradient(90deg,transparent,#000 15%,#000 85%,transparent)}",
    ".modes button{border:0;background:none;color:var(--muted);padding:10px 14px;font:inherit;font-weight:600;cursor:pointer;border-bottom:3px solid transparent;margin-bottom:-1px;white-space:nowrap}",
    ".modes button:hover{color:var(--fg)}",
    ".modes button[aria-selected=true]{color:var(--fg);border-bottom-color:var(--accent)}",
    ".modes .count{display:inline-block;min-width:20px;margin-left:6px;padding:0 6px;border-radius:10px;background:var(--tint);color:var(--muted);font-size:12px;font-weight:600}",
    "section{background:var(--card);border:1px solid var(--line);border-radius:14px;padding:20px;margin-bottom:18px;box-shadow:var(--shadow)}",
    ".head{display:flex;flex-wrap:wrap;gap:6px 16px;align-items:baseline;justify-content:space-between;margin-bottom:10px}",
    ".sub{color:var(--muted);font-size:13px}",
    ".lede{color:var(--muted);margin:4px 0 12px;max-width:72ch}",
    ".hero{display:grid;grid-template-columns:minmax(0,5fr) minmax(0,7fr);gap:24px;align-items:start;padding:24px;",
    "background:linear-gradient(135deg,color-mix(in srgb,var(--lvl) 16%,var(--card)) 0%,var(--card) 55%)}",
    ".hero .badge-big{display:flex;align-items:center;gap:16px}",
    ".hero .kicker{font-size:12px;font-weight:650;text-transform:uppercase;letter-spacing:.08em;color:var(--muted)}",
    ".hero .levelword{font-size:42px;line-height:1.05;font-weight:750;color:var(--lvl-ink);text-transform:capitalize;letter-spacing:-.02em}",
    ".hero .verdict{font-size:17px;line-height:1.5;margin:16px 0 0}",
    ".wins{margin:8px 0 0;font-size:14px;color:var(--pass)}.wins b{font-weight:650}",
    ".next{margin-top:14px;padding:12px 14px;border-radius:10px;background:var(--tint);font-size:14px}",
    ".next b{font-weight:650}",
    ".ladder{display:grid;grid-template-columns:repeat(var(--steps),minmax(0,1fr));gap:8px}",
    ".rung{border-radius:10px;padding:10px;background:var(--tint);min-height:120px;border:1px solid transparent}",
    ".rung.here{border-color:var(--lvl);background:color-mix(in srgb,var(--lvl) 12%,var(--card))}",
    ".rung .rh{display:flex;align-items:center;gap:6px;font-size:13px;font-weight:650;text-transform:capitalize;margin-bottom:8px}",
    ".rung .rh .n{margin-left:auto;color:var(--muted);font-weight:600}",
    ".rung a{display:block;font-size:13px;line-height:1.3;padding:5px 7px;margin-top:4px;border-radius:6px;background:var(--card);color:var(--fg);text-decoration:none;border:1px solid var(--line)}",
    ".rung a:hover{border-color:var(--accent)}",
    ".rung a.done{border-color:color-mix(in srgb,var(--pass) 45%,var(--line))}",
    ".rung .topmark{display:block;font-size:11.5px;font-weight:650;color:var(--pass);margin-top:2px}",
    ".rung .capmark{display:block;font-size:11.5px;font-weight:650;color:var(--unknown);margin-top:2px}.rung a.capped{border-color:var(--unknown);border-left-width:3px}.weakest .topmark{color:var(--pass);font-weight:650}.weakest .capnote{color:var(--unknown);font-weight:650}",
    ".rung .empty{font-size:12px}",
    ".tiles{display:grid;grid-template-columns:repeat(auto-fit,minmax(190px,1fr));gap:12px;margin-bottom:18px}",
    ".tile{background:var(--card);border:1px solid var(--line);border-radius:12px;padding:14px 16px;display:flex;gap:12px;align-items:center;box-shadow:var(--shadow);text-decoration:none;color:var(--fg)}",
    "a.tile:hover{border-color:var(--accent)}",
    ".tile strong{display:block;font-size:24px;line-height:1.15;font-variant-numeric:tabular-nums}",
    ".tile span{color:var(--muted);font-size:13px;line-height:1.3;display:block}",
    ".tile.bad strong{color:var(--fail)}.tile.good strong{color:var(--pass)}",
    ".bar{display:flex;height:10px;border-radius:5px;overflow:hidden;background:var(--line);gap:2px}",
    ".bar span{display:block;height:100%;transition:flex-grow .6s cubic-bezier(.2,.7,.2,1)}",
    ".bar.lg{height:16px;border-radius:8px}",
    ".bar.xs{height:6px;gap:1px}",
    ".legend{display:flex;flex-wrap:wrap;gap:6px 14px;font-size:12px;color:var(--muted);margin-top:6px}",
    ".legend b{color:var(--fg);font-weight:650;font-variant-numeric:tabular-nums}",
    ".dot{display:inline-block;width:9px;height:9px;border-radius:50%;margin-right:5px;vertical-align:-1px}.legend.trackkey{margin:2px 0 18px}.medal-key{display:inline-flex;align-items:center;gap:4px;white-space:nowrap}",
    ".lvl{display:inline-flex;align-items:center;gap:5px;padding:1px 8px 1px 6px;border-radius:999px;font-size:12px;font-weight:650;text-transform:capitalize;white-space:nowrap;",
    "color:var(--ink);background:color-mix(in srgb,var(--fill) 18%,transparent);border:1px solid color-mix(in srgb,var(--fill) 50%,transparent)}",
    ".lvl i{width:8px;height:8px;border-radius:50%;background:var(--fill);display:inline-block}",
    ".lvl.none i{background:transparent;border:1.5px solid var(--fill)}",
    ".rows{display:flex;flex-direction:column}",
    "a.row{display:grid;grid-template-columns:minmax(0,2.2fr) minmax(0,2.4fr) 90px;gap:16px;align-items:center;padding:11px 10px;border-top:1px solid var(--line);color:var(--fg);text-decoration:none;border-radius:8px}",
    "a.row:hover{background:var(--tint)}",
    ".row .t{font-weight:600;line-height:1.3}",
    ".row .c{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);margin-top:2px;overflow-wrap:anywhere}",
    ".row .num{text-align:right;font-variant-numeric:tabular-nums}",
    ".row .num strong{display:block;font-size:18px;line-height:1.1}",
    ".row .num.bad strong{color:var(--fail)}.row .num.ok strong{color:var(--pass)}",
    ".row .num span{font-size:11px;color:var(--muted)}",
    ".rows-head{display:grid;grid-template-columns:minmax(0,2.2fr) minmax(0,2.4fr) 90px;gap:16px;padding:0 10px 6px;font-size:12px;color:var(--muted);font-weight:600}",
    ".rows-head span:last-child{text-align:right}",
    ".cards{display:grid;grid-template-columns:repeat(auto-fill,minmax(250px,1fr));gap:12px}",
    "a.card{display:flex;gap:14px;align-items:center;padding:14px;border:1px solid var(--line);border-radius:12px;background:var(--card);color:var(--fg);text-decoration:none;transition:transform .15s,border-color .15s}",
    "a.card:hover{border-color:var(--accent);transform:translateY(-1px)}",
    "a.card[aria-current=true]{border-color:var(--accent);box-shadow:0 0 0 1px var(--accent)}",
    "a.card .t{font-weight:600;line-height:1.3}",
    "a.card .s{font-size:12px;color:var(--muted);margin-top:3px}",
    ".ring{flex:none}",
    ".ring circle{transition:stroke-dashoffset .8s cubic-bezier(.2,.7,.2,1)}",
    ".ring text{font:700 13px system-ui,sans-serif;fill:var(--fg)}",
    ".pill{display:inline-block;padding:1px 8px;border-radius:999px;font-size:12px;font-weight:600;background:var(--tint);color:var(--muted)}",
    ".pill.good{background:color-mix(in srgb,var(--pass) 16%,transparent);color:var(--pass)}",
    ".pill.bad{background:color-mix(in srgb,var(--fail) 14%,transparent);color:var(--fail)}.pill.unknown{background:transparent;border:1px dashed var(--unknown);color:var(--unknown)}",
    "input[type=search]{width:100%;margin:10px 0;padding:9px 12px;border:1px solid var(--line);border-radius:8px;background:var(--bg);color:var(--fg);font:inherit}",
    ".filters{display:flex;flex-wrap:wrap;gap:8px 16px;margin:12px 0 4px;align-items:center}",
    ".seg{display:inline-flex;border:1px solid var(--line);border-radius:8px;overflow:hidden}",
    ".seg button{border:0;border-right:1px solid var(--line);background:var(--card);color:var(--fg);padding:6px 12px;font:inherit;font-size:13px;cursor:pointer;min-height:32px}.seg button::first-letter{text-transform:uppercase}",
    ".seg button:last-child{border-right:0}",
    ".chip-filter{display:inline-flex;align-items:center;gap:4px;min-height:32px;padding:4px 12px;border:1px solid var(--accent);border-radius:999px;background:var(--tint);color:var(--fg);font:inherit;font-size:13px;cursor:pointer}",
    ".chip-filter:hover{background:var(--card)}",
    "a.card,a.track,a.kind,a.tile,a.factor,a.erow,a.row,.rung a,.box a,.rank a,.fail-row,.seg button,.modes button,details.ent>summary,.chip-filter{transition:background-color .15s,border-color .15s,color .15s,transform .15s,box-shadow .15s}",
    "@keyframes enter{from{opacity:0;transform:translateY(4px)}to{opacity:1;transform:none}}",
    ".view>*{animation:enter .22s ease-out both}",
    ".seg button[aria-pressed=true]{background:var(--accent-bg);color:var(--accent);font-weight:650}",
    "details.ent{border-top:1px solid var(--line)}",
    "details.ent>summary{cursor:pointer;display:grid;grid-template-columns:86px minmax(min(14rem,100%),1fr) auto;gap:12px;align-items:center;padding:10px 6px;list-style:none;border-radius:8px}",
    "details.ent>summary:hover{background:var(--tint)}",
    "details.ent>summary::-webkit-details-marker{display:none}",
    "details.ent>summary .name{font-weight:600;overflow-wrap:anywhere;min-width:0}details.ent>summary .name .tail{white-space:nowrap}",
    "details.ent>summary .name small{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);font-weight:400;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:100%}",
    "details.ent>summary .tally{font-size:12px;color:var(--muted);text-align:right;min-width:0}",
    "details.ent>summary .tally b{color:var(--fg);font-weight:650}details.ent>summary .tally b.bad{color:var(--fail)}",
    "details.ent[open]>summary{background:var(--tint)}",
    ".ent-body{padding:6px 6px 14px 104px}",
    ".chev{display:inline-block;transition:transform .15s;color:var(--faint);margin-left:6px}",
    "details[open]>summary .chev{transform:rotate(90deg)}",
    ".rule{padding:10px 12px;border-radius:8px;border:1px solid var(--line);border-left:4px solid var(--fail);margin-top:8px;background:var(--card)}",
    ".callout.judged-note{background:var(--judged-bg)}",
    ".bar span.hollow{background:repeating-linear-gradient(135deg,var(--seg) 0 3px,transparent 3px 7px);box-shadow:inset 0 0 0 1.5px var(--judged)}.unjudged-n,.judged-n{color:var(--judged)}.dot.hatch{border-radius:2px;background:repeating-linear-gradient(135deg,var(--seg) 0 2px,transparent 2px 4px);box-shadow:inset 0 0 0 1.5px var(--judged)}.judged-list{margin:8px 0 0}",
    ".rule.unknown{border-left-color:var(--unknown)}.rule.waived{border-left-color:var(--bronze)}.rule.unassessed{border-left-color:var(--judged)}",
    ".rule .rn{font-weight:650;font-size:inherit;margin:0;display:inline}.sc-block h3.sct{display:inline;margin:0;font-size:16px}",
    ".rule .fix{margin-top:4px;font-size:14px}",
    ".rule ul{margin:6px 0 0;padding-left:18px;font-size:13px;color:var(--muted)}",
    ".rule li{overflow-wrap:anywhere}",
    ".rule .evidence details{font-size:13px;color:var(--muted);margin:2px 0 0 18px}.rule .evidence details ul{margin-top:2px}.rule .evidence summary{cursor:pointer;min-height:24px;line-height:24px}",
    ".rule code,.evid code{font:12.5px ui-monospace,SFMono-Regular,Menlo,monospace;overflow-wrap:anywhere}",
    ".outcome{font-size:12px;font-weight:650;margin-left:6px}",
    ".outcome.fail{color:var(--fail)}.outcome.unknown{color:var(--unknown)}.outcome.waived{color:var(--bronze-ink)}.outcome.skipped{color:var(--muted)}.outcome.unassessed{color:var(--judged)}",
    ".passes{display:flex;flex-wrap:wrap;align-items:center;gap:6px;margin-top:8px}",
    ".passes>span:not(.lvl){font-size:12px;padding:2px 8px 2px 6px;border-radius:999px;background:color-mix(in srgb,var(--pass) 12%,transparent);color:var(--pass);font-weight:600}",
    ".passes>span:not(.lvl)::before{content:'\\2713  '}",
    ".passes>span:not(.lvl){display:inline-flex;align-items:center;gap:2px}.passes>span a{color:inherit;text-decoration:none}.passes>span a:hover{text-decoration:underline}",
    ".passes .basis.mini{border:0;background:none;padding:0;min-width:0;color:inherit}.passes>span.judged:not(.lvl){background:var(--judged-bg);color:var(--judged)}",
    ".passes em{font-style:normal;font-size:12px;color:var(--muted);margin-right:4px}",
    ".callout{display:flex;gap:10px;align-items:flex-start;padding:10px 12px;border-radius:10px;background:var(--accent-bg);font-size:14px;margin:8px 0}",
    ".callout.done{background:color-mix(in srgb,var(--pass) 12%,var(--card))}",
    ".callout.near{background:color-mix(in srgb,var(--pass) 7%,var(--card));border-left:3px solid var(--pass)}",
    ".close-note{color:var(--fg)}.close-note b{color:var(--pass)}",
    ".fails{display:flex;flex-direction:column;gap:2px}",
    ".fail-row{display:grid;grid-template-columns:minmax(200px,2fr) minmax(0,3fr) 70px;gap:12px;align-items:center;padding:7px 8px;border:0;border-radius:8px;background:none;color:var(--fg);font:inherit;font-size:13px;text-align:left;cursor:pointer}",
    ".fail-row:hover{background:var(--tint)}",
    ".fail-row .rn{display:flex;flex-wrap:wrap;gap:4px 8px;align-items:center;font-weight:600;overflow-wrap:break-word}",
    ".fail-row .fc{text-align:right;color:var(--muted);font-variant-numeric:tabular-nums}",
    ".scope-note details{display:inline}.scope-note summary{display:inline-flex;align-items:center;min-height:24px;cursor:pointer;color:var(--accent)}.scope-note details[open] div{margin-top:4px}",
    ".basis.tip{position:relative;cursor:help;line-height:22px}.basis.tip:hover::after,.basis.tip:focus::after{content:attr(data-tip);position:absolute;left:0;top:calc(100% + 4px);z-index:5;width:max-content;max-width:min(280px,70vw);padding:6px 9px;border-radius:8px;background:var(--fg);color:var(--bg);font-size:12.5px;font-weight:450;line-height:1.4;letter-spacing:0;text-transform:none;white-space:normal;box-shadow:0 4px 14px rgba(0,0,0,.18)}",
    "abbr.gloss{position:relative;display:inline-block;min-height:24px;line-height:24px;vertical-align:bottom;text-decoration:underline dotted;text-underline-offset:3px;cursor:help}abbr.gloss:hover::after,abbr.gloss:focus::after{content:attr(data-tip);position:absolute;left:0;top:calc(100% + 4px);z-index:5;width:max-content;max-width:min(280px,70vw);padding:6px 9px;border-radius:8px;background:var(--fg);color:var(--bg);font-size:12.5px;font-weight:450;line-height:1.4;white-space:normal;text-decoration:none;box-shadow:0 4px 14px rgba(0,0,0,.18)}",
    ".jumpbar{display:flex;flex-wrap:wrap;align-items:center;gap:4px 14px;margin:14px 0 0;padding:6px 10px;border-radius:8px;background:var(--tint);font-size:13px}.jumpbar button.link{min-height:24px}",
    ".levels-need{display:grid;grid-template-columns:repeat(auto-fit,minmax(200px,1fr));gap:10px;margin:8px 0 4px;align-items:start}",
    ".need{padding:10px 12px;border-radius:10px;background:var(--tint);font-size:13px}",
    ".need .lvl{margin-bottom:6px}",
    ".need ul{margin:0;padding-left:16px}",
    ".need li{margin:6px 0}",
    ".need li .why{display:block;color:var(--muted);font-size:12.5px;line-height:1.4}.levels-need.clamped li .why{display:-webkit-box;-webkit-line-clamp:2;-webkit-box-orient:vertical;overflow:hidden}.need-head{display:flex;flex-wrap:wrap;align-items:center;justify-content:space-between;gap:4px 8px;margin-bottom:6px}.need-head .lvl{margin-bottom:0}.need-basis{font-size:12px;color:var(--muted);display:inline-flex;align-items:center;gap:4px}.need-toggle{display:inline-flex;align-items:center;min-height:24px;font-size:13px;margin:0 0 4px}",
    "code{font-family:ui-monospace,SFMono-Regular,Menlo,monospace;font-size:.92em}",
    ".picker{display:none}",
    ".picker select{width:100%;padding:9px 10px;border:1px solid var(--line);border-radius:8px;background:var(--bg);color:var(--fg);font:inherit;min-height:40px}",
    ".kinds{display:grid;grid-template-columns:repeat(auto-fill,minmax(170px,1fr));gap:10px}",
    "a.kind{display:block;padding:12px;border:1px solid var(--line);border-radius:10px;background:var(--card);color:var(--fg);text-decoration:none}",
    "a.kind:hover{border-color:var(--accent)}",
    "a.kind[aria-current=true]{border-color:var(--accent);box-shadow:0 0 0 1px var(--accent)}",
    "a.kind strong{display:block;font-size:14px}",
    "a.kind span{display:block;color:var(--muted);font-size:12px;margin-bottom:6px}",
    ".box{display:flex;flex-wrap:wrap;gap:6px}",
    ".box a{font-size:13px;padding:4px 10px;border-radius:999px;background:var(--tint);color:var(--fg);text-decoration:none;border:1px solid transparent}",
    ".box a:hover{border-color:var(--accent)}",
    ".box a b{font-variant-numeric:tabular-nums;margin-left:4px;color:var(--muted);font-weight:600}",
    "a.erow{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:12px;align-items:center;padding:10px 8px;border-top:1px solid var(--line);color:var(--fg);text-decoration:none;border-radius:8px}",
    "a.erow:hover{background:var(--tint)}",
    "a.erow .name{font-weight:600;overflow-wrap:anywhere;min-width:0}",
    "a.erow .name small{display:block;font:12px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);font-weight:400;white-space:nowrap;overflow:hidden;text-overflow:ellipsis;max-width:100%}",
    "a.erow .badges{display:flex;flex-wrap:wrap;gap:4px;justify-content:flex-end}",
    "code.kid{display:block;font:11.5px ui-monospace,SFMono-Regular,Menlo,monospace;font-weight:400;color:var(--muted)}",
    ".kchips{display:grid;grid-template-columns:minmax(9rem,12rem) 1fr;gap:4px 12px;align-items:baseline;padding:6px 0;border-top:1px solid var(--line)}.kchips-g{font-size:12px;font-weight:650;color:var(--muted);text-transform:uppercase;letter-spacing:.05em}.kchips .box a{display:inline-flex;align-items:center;min-height:28px}.kchips .box a[aria-current=true]{border-color:var(--accent);background:var(--card);font-weight:650}",
    ".ehdr{display:grid;gap:12px;padding:6px 8px;font-size:12px;font-weight:650;color:var(--muted)}.ehdr span{overflow-wrap:break-word}",
    ".weakest{display:block;margin-top:3px;font-size:12px;color:var(--muted)}.weakest .lvl{font-size:11px}",
    ".ecell{font-size:12px}.unassessed-tag{display:block;margin-top:2px;font-size:11px;color:var(--judged)}.unassessed-tag.unrun{color:var(--unknown)}.lvl.vacuous{background:transparent;border-style:dashed}.lvl.vacuous.unjudged{border-color:var(--judged)}.lvl.vacuous.unrun{border-style:dotted;border-color:var(--unknown)}.unassessed-tag svg{vertical-align:-1px}.pillwrap{display:inline-flex;flex-direction:column;align-items:flex-start;vertical-align:middle}",
    "@media (max-width:820px){.unassessed-tag{display:inline;margin:0}}",
    ".crumb{font-size:13px;color:var(--muted);margin-bottom:12px}",
    ".crumb a{color:var(--muted);display:inline-block;padding:4px 0;margin:-4px 0}",
    "a.rl{display:inline-flex;align-items:center;min-height:24px;vertical-align:middle}.refs a{display:inline-flex;align-items:center;min-height:26px}",
    ".passes>span a{display:inline-block;padding:4px 0;margin:-4px 0}",
    ".ehead{display:flex;flex-wrap:wrap;gap:16px;align-items:center;justify-content:space-between}",
    ".ehead h2{font-size:24px}",
    ".ehead code{font:12.5px ui-monospace,SFMono-Regular,Menlo,monospace;color:var(--muted);overflow-wrap:anywhere}",
    ".sc-block{border-top:1px solid var(--line);padding-top:14px;margin-top:14px}",
    ".sc-block .head a{font-weight:650;font-size:16px;text-decoration:none}",
    ".sc-block .head a:hover{text-decoration:underline}",
    ".rank{position:relative;display:flex;flex-direction:column;max-height:min(62vh,560px);overflow:auto;margin-top:4px;scrollbar-width:thin;scrollbar-color:var(--faint) transparent;border-top:1px solid var(--line);border-bottom:1px solid var(--line)}",
    ".rank.more{mask-image:linear-gradient(to bottom,#000 calc(100% - 36px),transparent)}.rank.less{mask-image:linear-gradient(to bottom,transparent,#000 36px)}.rank.less.more{mask-image:linear-gradient(to bottom,transparent,#000 36px,#000 calc(100% - 36px),transparent)}",
    ".rank a{display:flex;justify-content:space-between;gap:12px;align-items:baseline;padding:8px 10px;border-radius:8px;color:var(--fg);text-decoration:none}",
    ".rank a:hover{background:var(--tint)}",
    ".rank-head{font-size:11px;font-weight:650;letter-spacing:.04em;text-transform:uppercase;color:var(--muted);padding:10px 10px 4px}",
    ".rank a[aria-current=true]{background:var(--accent-bg);font-weight:650}",
    ".rank .debt{font-size:12px;color:var(--muted);white-space:nowrap}",
    ".rank .debt.bad{color:var(--fail);font-weight:650}.rank .debt.ok{color:var(--pass)}",
    ".split{display:grid;grid-template-columns:300px minmax(0,1fr);gap:18px;align-items:start}",
    ".split>section:first-child{position:sticky;top:12px}",
    ".gap{display:grid;grid-template-columns:minmax(0,1fr) minmax(0,2fr);gap:12px;padding:9px 6px;border-top:1px solid var(--line);font-size:14px}",
    ".gap a{font-weight:600}",
    ".gap .why{color:var(--muted);font-size:13px;overflow-wrap:anywhere}",
    ".empty{color:var(--muted);font-size:14px;padding:10px 0}",
    ".empty.good{color:var(--pass);font-weight:600}",
    "button.link{background:none;border:0;padding:4px 0;margin:-4px 0;color:var(--accent);font:inherit;cursor:pointer;text-decoration:underline;text-underline-offset:2px}",
    ":where(section a:not([class]),section a[class=\"\"]){display:inline-block;padding:4px 0;margin:-4px 0}",
    ".more{margin-top:10px}",
    ".milestones{position:relative;height:18px;margin-top:2px}",
    ".milestones span{position:absolute;top:0;transform:translateX(-50%);font-size:11px;color:var(--muted);white-space:nowrap}",
    ".milestones span::before{content:'';display:block;width:2px;height:6px;background:var(--faint);margin:0 auto 1px}",
    ".milestones span.met{color:var(--pass);font-weight:650}",
    ".tablewrap{overflow-x:auto}",
    "table.cmp{width:100%;border-collapse:collapse;font-size:14px}",
    "table.cmp th{text-align:left;font-size:12px;font-weight:650;color:var(--muted);padding:6px 10px 6px 0;border-bottom:1px solid var(--line)}",
    "table.cmp td{padding:10px 10px 10px 0;border-top:1px solid var(--line);vertical-align:top}",
    "table.cmp td.n{font-variant-numeric:tabular-nums;color:var(--muted)}",
    "table.cmp a{font-weight:600}",
    ".standing{display:inline-block;padding:1px 8px;border-radius:999px;font-size:12px;font-weight:650;background:var(--tint);color:var(--muted)}",
    ".standing.ahead{background:color-mix(in srgb,var(--pass) 16%,transparent);color:var(--pass)}",
    ".standing.behind{background:color-mix(in srgb,var(--fail) 14%,transparent);color:var(--fail)}",
    ".standing.parity{background:var(--accent-bg);color:var(--accent)}",
    ".value{display:block;font-size:12px;color:var(--muted);margin-top:2px}",
    ".basis{display:inline-flex;align-items:center;gap:4px;padding:0 7px 0 5px;border-radius:6px;font-size:12px;font-weight:650;white-space:nowrap;line-height:20px;border:1px solid var(--line);color:var(--muted);background:var(--card);vertical-align:1px;text-transform:none;letter-spacing:0}",
    ".basis.judged{color:var(--judged);border-color:color-mix(in srgb,var(--judged) 45%,transparent);background:var(--judged-bg)}",
    ".basis.mini{padding:0 5px 0 3px;gap:3px;font-size:11px;font-weight:600;line-height:18px}",
    ".basis svg{display:block;flex:none}",
    ".basis-key{display:grid;grid-template-columns:repeat(auto-fit,minmax(260px,1fr));gap:8px 18px;padding:10px 12px;border-radius:10px;border:1px dashed var(--line);font-size:13px;color:var(--muted);margin:10px 0}",
    ".basis-key>div{display:flex;gap:8px;align-items:flex-start}.basis-key.one{display:block;padding:6px 12px}",
    ".basis-key b{color:var(--fg);font-weight:650}",
    ".why{margin-top:6px;font-size:13.5px;line-height:1.5}",
    ".why>b{font-weight:650}",
    ".refs{display:flex;flex-wrap:wrap;align-items:center;gap:4px 14px;margin:4px 0 2px;font-size:13px;line-height:26px}",
    ".refs a{text-underline-offset:2px}",
    ".refs a::after{content:' \\2197';font-size:11px}",
    ".rationale{margin:6px 0 12px;padding:10px 14px;border-left:3px solid var(--accent);background:var(--tint);border-radius:0 10px 10px 0;max-width:80ch;font-size:14px}",
    ".rationale>b{display:block;font-size:12px;color:var(--muted);text-transform:uppercase;letter-spacing:.06em;margin-bottom:2px}",
    ".cel{margin:6px 0;padding:10px 12px;border-radius:8px;background:var(--tint);overflow-x:auto;font:12.5px/1.5 ui-monospace,SFMono-Regular,Menlo,monospace;white-space:pre-wrap;overflow-wrap:anywhere}",
    ".tracks{display:grid;grid-template-columns:repeat(auto-fill,minmax(300px,1fr));gap:14px;margin-top:6px}",
    "a.track{position:relative;display:flex;flex-direction:column;gap:10px;padding:16px;border:1px solid var(--line);border-radius:14px;color:var(--fg);text-decoration:none;transition:transform .15s,border-color .15s;",
    "background:linear-gradient(160deg,color-mix(in srgb,var(--tc) 13%,var(--card)) 0%,var(--card) 62%)}",
    "a.track:hover{border-color:var(--accent);transform:translateY(-2px)}",
    ".track .th{display:flex;gap:14px;align-items:center}",
    ".track .tt{font-weight:700;font-size:16px;line-height:1.25}",
    ".track .td{font-size:13px;color:var(--muted)}",
    ".trackstrip{display:flex;flex-wrap:wrap;align-items:center;gap:8px 10px;margin-top:28px;padding-top:16px;border-top:1px solid var(--line)}",
    ".trackstrip a{display:inline-flex;align-items:center;gap:8px;min-height:36px;padding:4px 12px 4px 4px;border:1px solid var(--line);border-radius:999px;color:var(--fg);text-decoration:none;font-size:13px}",
    ".trackstrip a{transition:border-color .15s,transform .15s,box-shadow .15s}.trackstrip a:hover,.trackstrip a.far:hover{border-color:var(--accent);transform:translateY(-2px);box-shadow:0 4px 12px color-mix(in srgb,var(--fg) 10%,transparent)}",
    ".leads{display:grid;grid-template-columns:repeat(auto-fit,minmax(220px,1fr));gap:12px}a.lead-tile{display:flex;flex-direction:column;gap:4px;padding:14px 16px;border:1px solid var(--line);border-left:4px solid var(--pass);border-radius:12px;color:var(--fg);text-decoration:none;background:var(--card);transition:transform .15s,border-color .15s}a.lead-tile:hover{transform:translateY(-2px);border-color:var(--accent);border-left-color:var(--pass)}.lead-x{font-size:30px;font-weight:750;line-height:1;color:var(--pass)}.lead-dir{font-size:12px;font-weight:650;text-transform:uppercase;letter-spacing:.06em;color:var(--muted)}.lead-what{font-weight:650;line-height:1.35}",
    ".climb{margin-top:12px;padding:10px 12px;border:1px solid var(--line);border-radius:10px;background:var(--card)}.climb-head{display:flex;flex-wrap:wrap;justify-content:space-between;align-items:center;gap:4px 10px;font-weight:650}.climb-bar{height:8px;margin:8px 0;border-radius:4px;background:var(--bg);box-shadow:inset 0 0 0 1px var(--line);overflow:hidden}.climb-bar span{display:block;height:100%;border-radius:3px}.climb ul{list-style:none;margin:0;padding:0;display:grid;gap:4px;font-size:14px}.climb li{display:grid;grid-template-columns:auto 1fr;align-items:baseline;gap:4px}.climb .box{width:12px;height:12px;border:1.5px solid var(--muted);border-radius:3px;margin-right:4px;transform:translateY(1px)}.climb li.stuck .box{border-color:var(--unknown);border-style:dashed}",
    ".climb-track{display:flex;align-items:center;gap:8px}.climb-track .climb-bar{flex:1}.climb-lock{display:inline-flex;align-items:center;gap:3px;font-size:12px;font-weight:650;color:var(--muted);white-space:nowrap}",
    ".trackstrip.lead{margin-top:4px;padding-top:0;border-top:0}",
    ".trackstrip.lead a{padding:6px 14px 6px 6px;border-radius:14px;background:var(--card)}",
    ".trackstrip small{display:block;font-size:12px;color:var(--muted)}",
    ".trackstrip a.far{border-color:var(--fail)}.trackstrip a.far small{color:var(--fail);font-weight:650}",
    ".track .foot{display:flex;flex-wrap:wrap;justify-content:space-between;gap:4px 10px;font-size:12px;color:var(--muted);margin-top:auto}",
    ".medals{display:flex;flex-wrap:wrap;gap:4px;margin:0;padding:0;list-style:none}",
    ".medals>li{display:inline-flex;align-items:center;gap:4px;padding:2px 8px 2px 3px;border-radius:999px;background:var(--tint);font-size:12px;line-height:1.3}",
    ".sr{position:absolute;width:1px;height:1px;overflow:hidden;clip:rect(0 0 0 0);white-space:nowrap}",
    ".medals>li.more{padding:2px 8px;color:var(--muted)}",
    ".flag{position:absolute;top:-10px;right:14px;font-size:12px;font-weight:700;padding:1px 9px;border-radius:999px;background:var(--fail);color:var(--card)}",
    ".chips{display:flex;flex-wrap:wrap;gap:4px}",
    ".chips .standing{font-size:12px}",
    ".kgroup{margin-top:16px}",
    ".kgroup:first-of-type{margin-top:4px}",
    ".kgh{display:flex;flex-wrap:wrap;gap:4px 12px;align-items:baseline}",
    ".kgh h3{margin:0;color:var(--fg);font-size:13px}",
    ".kgd{margin:2px 0 8px;max-width:80ch}",
    "a.kind small{display:block;color:var(--muted);font-size:12px;line-height:1.35;margin:-2px 0 6px}",
    ".boxrow{display:grid;grid-template-columns:200px minmax(0,1fr);gap:12px;align-items:start;padding:10px 0;border-top:1px solid var(--line)}",
    ".boxrow:first-of-type{border-top:0}",
    ".boxrow .bg{font-weight:650;font-size:14px;line-height:1.3}",
    ".boxrow .bg small{display:block;font-weight:400;color:var(--muted);font-size:12px}",
    ".cmp-hero{display:grid;grid-template-columns:minmax(0,1fr) auto;gap:20px;align-items:center;background:linear-gradient(135deg,color-mix(in srgb,var(--st) 14%,var(--card)) 0%,var(--card) 60%)}",
    ".cmp-hero .big{font-size:28px;line-height:1.2;font-weight:750;letter-spacing:-.01em;margin:4px 0 8px}",
    ".cmp-hero .kicker{font-size:12px;font-weight:650;text-transform:uppercase;letter-spacing:.08em;color:var(--muted)}",
    ".parity-fold{margin-top:8px;border-top:1px solid var(--line);padding-top:8px}.parity-fold>summary{cursor:pointer;color:var(--accent);font-size:14px;min-height:24px;display:flex;align-items:center}",
    ".method-lede{max-width:80ch;margin:0 0 12px;font-size:15px;line-height:1.55}",
    ".cmp-side{display:grid;gap:12px;justify-items:center}.h2h{display:grid;gap:2px;width:300px;font-size:12px}.h2h-head,.h2h-row{display:grid;grid-template-columns:110px 1fr 1fr;align-items:center;gap:0}.h2h-head{color:var(--muted);font-weight:650}.h2h-head .behind{text-align:right;padding-right:6px;color:var(--fail)}.h2h-head .ahead{padding-left:6px;color:var(--pass)}.h2h-row{min-height:24px;color:var(--fg);text-decoration:none;border-radius:6px;padding:0 2px}.h2h-row:hover,.h2h-row:focus-visible{background:color-mix(in srgb,var(--accent) 10%,transparent)}.h2h-name{overflow:hidden;text-overflow:ellipsis;white-space:nowrap;padding-right:6px}.h2h-l,.h2h-r{display:flex;height:10px}.h2h-l{justify-content:flex-end;border-right:1.5px solid var(--muted)}.h2h-l span{background:var(--fail);border-radius:3px 0 0 3px}.h2h-r span{background:var(--pass);border-radius:0 3px 3px 0}.h2h-foot{color:var(--muted);font-size:11px;text-align:center;margin-top:2px}",
    ".tally3{display:flex;gap:18px;text-align:center}.rule-tally{margin:6px 0 0;font-size:14px}.rule-tally .ahead{color:var(--pass)}.rule-tally .behind{color:var(--fail)}.rule-tally .parity{color:var(--muted)}",
    ".tally3 strong{display:block;font-size:30px;line-height:1.1;font-variant-numeric:tabular-nums}",
    ".tally3 span{font-size:12px;color:var(--muted);font-weight:650;text-transform:uppercase;letter-spacing:.05em}",
    ".setup{display:grid;grid-template-columns:repeat(auto-fit,minmax(210px,1fr));gap:10px;margin:10px 0;align-items:start}",
    ".setup>div{padding:10px 12px;border-radius:10px;background:var(--tint);font-size:14px}",
    ".setup b{display:block;font-size:12px;color:var(--muted);font-weight:650;text-transform:uppercase;letter-spacing:.05em;margin-bottom:2px}",
    ".setup small{display:block;color:var(--muted);font-size:12px;margin-top:4px;line-height:1.4}",
    ".factors{display:grid;grid-template-columns:repeat(auto-fill,minmax(260px,1fr));gap:12px}",
    "a.factor{display:flex;flex-direction:column;gap:8px;padding:14px;border:1px solid var(--line);border-left:5px solid var(--st);border-radius:12px;background:var(--card);color:var(--fg);text-decoration:none;transition:transform .15s,border-color .15s}",
    "a.factor:hover{border-color:var(--accent);border-left-color:var(--st);transform:translateY(-1px)}",
    "a.factor[aria-current=true]{box-shadow:0 0 0 1px var(--accent)}",
    ".factor .fh{display:flex;justify-content:space-between;gap:8px;align-items:center;font-weight:700}",
    ".factor p{margin:0;font-size:13px;color:var(--muted)}.fx{display:flex;flex-direction:column;gap:1px;line-height:1.2}.fx b{font-size:22px;font-weight:750;color:var(--st);font-variant-numeric:tabular-nums}.fx span{font-size:12px;color:var(--muted)}",
    ".factor .fc{font-size:12px;color:var(--muted)}",
    ".measure{display:grid;grid-template-columns:minmax(0,5fr) minmax(0,6fr) minmax(130px,auto);gap:10px 18px;align-items:center;padding:14px 4px;border-top:1px solid var(--line)}",
    ".measure .mt{font-weight:650;line-height:1.35}",
    ".measure .mt small{display:flex;flex-wrap:wrap;gap:4px 8px;align-items:center;font-weight:400;color:var(--muted);font-size:12px;margin-top:4px}",
    ".measure .ms{text-align:right}",
    ".measure .full{grid-column:1/-1;margin-top:-4px}",
    ".measure details summary{cursor:pointer;font-size:12.5px;color:var(--muted);padding:4px 0;min-height:24px}",
    ".vals{display:flex;flex-direction:column;gap:6px}",
    ".val{display:grid;grid-template-columns:92px minmax(0,1fr) auto;gap:8px;align-items:center;font-size:13px}",
    ".val .who{color:var(--muted);overflow:hidden;text-overflow:ellipsis;white-space:nowrap}",
    ".val .vb{height:12px;border-radius:6px;background:var(--tint);overflow:hidden}",
    ".val .vb span{display:block;height:100%;border-radius:6px;width:0;transition:width .7s cubic-bezier(.2,.7,.2,1)}",
    ".val .v{font-variant-numeric:tabular-nums;font-weight:650;white-space:nowrap}",
    ".val .v.pass{color:var(--pass)}.val .v.fail{color:var(--fail)}",
    ".mag{display:block;font-size:12px;color:var(--muted);margin-top:4px}",
    ".behind-row{display:grid;grid-template-columns:minmax(0,2fr) minmax(0,3fr);gap:6px 18px;padding:12px 4px;border-top:1px solid var(--line);font-size:14px}",
    ".behind-row .why{margin:6px 0 0;color:var(--muted)}",
    ".provisional{color:var(--unknown);font-weight:650}",
    "::slotted(*){display:block;max-width:1120px;margin:0 auto;padding:12px 20px 0;box-sizing:border-box}",
    "@media (max-width:820px){.hero{grid-template-columns:1fr}.split{grid-template-columns:1fr}.split>section:first-child{display:none}.picker{display:flex;flex-direction:column;gap:4px;margin:0 0 16px}}",
    ".ering{display:flex;flex-direction:column;align-items:center;gap:4px;flex:none}",
    ".elevel{display:flex;align-items:center;gap:12px;flex:0 1 auto;min-width:0;max-width:100%}",
    ".elevel>div{min-width:0;overflow-wrap:anywhere}",
    ".elevel strong{display:block;font-size:30px;line-height:1.1;font-weight:750;text-transform:capitalize;letter-spacing:-.01em}",
    ".elevel .sub{display:block}",
    "@media (max-width:600px){main{padding:18px 12px 56px}.hero .levelword{font-size:34px}",
    ".modes{gap:0;flex-wrap:wrap}.modes button{padding:10px 9px}.modes .count{display:none}",
    ".kinds{grid-template-columns:repeat(2,minmax(0,1fr));gap:8px}a.kind{padding:9px}.fail-row{grid-template-columns:minmax(0,1fr) 64px}.fail-row .bar{display:none}",
    "a.row,.rows-head{grid-template-columns:minmax(0,1fr) 64px}.row .dist,.rows-head .dist{display:none}",
    ".ladder{grid-template-columns:1fr 1fr}.rung{min-height:0}.rung.vacant{grid-column:1/-1;display:flex;align-items:center;gap:12px;padding:6px 10px}.rung.vacant .rh{margin:0}.rung.vacant .empty{padding:0;margin:0}",
    "details.ent>summary{grid-template-columns:76px minmax(0,1fr)}details.ent>summary .tally{grid-column:2;text-align:left}",
    ".ent-body{padding-left:6px}.gap{grid-template-columns:1fr}.hide-sm{display:none}",
    ".kchips{grid-template-columns:1fr}.boxrow{grid-template-columns:1fr;gap:6px}.measure{grid-template-columns:1fr}.measure .ms{text-align:left}.cmp-hero{grid-template-columns:1fr}.h2h{display:none}",
    ".ehdr{display:none}a.erow.cols{grid-template-columns:1fr!important;gap:4px}.ecell{display:flex;gap:6px;align-items:center}.ecell::before{content:attr(data-sc);color:var(--muted)}.ecell[data-judged]::before{content:attr(data-sc) \" \\2726\"}",
    ".behind-row{grid-template-columns:1fr}.val{grid-template-columns:76px minmax(0,1fr) auto}}",
    ".skip{position:absolute;left:8px;top:-48px;z-index:10;padding:8px 12px;border:0;border-radius:8px;background:var(--fg);color:var(--bg);font:inherit;font-weight:650;cursor:pointer}.skip:focus{top:8px}",
    "@media (prefers-reduced-motion:reduce){*{transition:none!important;animation:none!important}}",
  ].join("");

  var template = document.createElement("template");
  template.innerHTML =
    "<style>" +
    CSS +
    "</style>" +
    '<slot name="header"></slot><slot name="nav"></slot>' +
    '<main><button type="button" class="skip" id="skip">Skip to content</button><div class="top" id="top"></div><div class="meta" id="meta"></div>' +
    '<div class="modes" id="modes" role="tablist" aria-label="Report views"></div>' +
    '<div id="body"></div></main><slot name="footer"></slot>';

  var SVGNS = "http://www.w3.org/2000/svg";
  var KNOWN = ["none", "bronze", "silver", "gold", "skipped"];

  var TAG = /^[a-z][a-z0-9]*$/;
  function el(t, a) {
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

  function svg(t, attrs) {
    var e = document.createElementNS(SVGNS, t);
    Object.keys(attrs || {}).forEach(function (k) {
      e.setAttribute(k, attrs[k]);
    });
    Array.prototype.slice.call(arguments, 2).forEach(function (k) {
      if (k) e.append(k);
    });
    return e;
  }

  function link(path, cls, kids) {
    var a = el("a", { href: "#" + path, className: cls || "" });
    (Array.isArray(kids) ? kids : [kids]).forEach(function (k) {
      if (k != null && k !== "") a.append(k);
    });
    return a;
  }

  function enc(s) {
    return encodeURIComponent(s);
  }

  // Manifest prose (a rule's fix) marks code with backticks.
  // Decision ids belong in a standard's links, not in the sentence a reader is reading.
  // A parenthetical made only of citations a reader can't resolve here: decision, question and
  // lesson ids, doc paths and plan sections. The standard's links carry the sources.
  var CITATION = "(?:D\\d+|D0[0-9A-Z]{20,}|Q\\d+|lessons? L\\d+(?: and L\\d+)?|plan §[\\d.]+|[\\w./-]+\\.md(?:#[\\w-]+| §[\\d.]+)?(?: for [a-z ]+)?|step \\d+)";
  var DECISION_REFS = new RegExp("\\s*\\(" + CITATION + "(?:[,;] ?" + CITATION + ")*\\)", "g");
  function prose(s) {
    return String(s || "").replace(DECISION_REFS, "");
  }

  // Words a standard's own prose uses that a reader outside the project would have to look up.
  var GLOSSARY = {
    ratchet: "The CI gate that compares each build with the committed results and fails when any score falls.",
    cel: "Common Expression Language: the small expression language a rule's check is written in.",
    fixture: "A sample input in the standard's tests, built to pass or to fail a rule, so the rule is shown to decide both ways.",
    harness: "The program that runs something: an agent harness (such as Claude Code, Cursor or Codex) runs an agent, and a benchmark harness runs and times the products it compares.",
  };
  var GLOSS_RE = /\b(ratchet|CEL|fixtures?|harness)\b/i;

  function glossed(text) {
    var m = GLOSS_RE.exec(text);
    if (!m) return [text];
    var word = m[1].toLowerCase();
    var def = GLOSSARY[word] || GLOSSARY[word.replace(/s$/, "")];
    if (!def || (word === "cel" && m[1] !== "CEL")) return [text];
    // Focusable, with the definition in the accessible name, so keyboard and touch readers reach it too.
    var term = el("abbr", { className: "gloss", tabIndex: 0 }, m[1], el("span", { className: "sr", textContent: " (" + def + ")" }));
    term.dataset.tip = def;
    return [text.slice(0, m.index), term, text.slice(m.index + m[1].length)];
  }

  function rich(s) {
    var first = true;
    return prose(s)
      .split("`")
      .flatMap(function (part, i) {
        if (i % 2) return [el("code", { textContent: part })];
        if (!first) return [part];
        var out = glossed(part);
        if (out.length > 1) first = false;
        return out;
      });
  }

  // Prose that names another scorecard by id links to it under its title, so the reader needn't decode ids.
  function richCards(D, s, self) {
    return rich(s).flatMap(function (part) {
      if (typeof part !== "string") return [part];
      return part.split(/([a-z0-9]+(?:-[a-z0-9]+)+)/).map(function (bit, i) {
        return i % 2 && bit !== self && (D.scorecards || {})[bit] ? link("scorecard/" + enc(bit), "", title(D, bit)) : bit;
      });
    });
  }

  // Custom level names (a one-level scorecard's "ten", say) borrow the colour of the
  // standard level at the same height, counted from the top.
  var ALIAS = {};
  var LEVEL_NOTE = {};
  var LEVEL_DEF = {};
  function aliasLevels(D) {
    Object.entries(D.scorecards || {}).forEach(function (e) {
      var sc = e[1];
      var levels = sc.levels || [];
      levels.forEach(function (l, i) {
        if (KNOWN.indexOf(l) < 0 && !ALIAS[l]) {
          ALIAS[l] = ["gold", "silver", "bronze"][Math.min(2, levels.length - 1 - i)];
          var said = (sc.description || "").split(/(?<=\.)\s+/).filter(function (s) {
            return s.toLowerCase().split(/[^a-z0-9-]+/).indexOf(String(l).toLowerCase()) >= 0;
          })[0];
          if (said) LEVEL_DEF[l] = said.replace(/`/g, "");
          LEVEL_NOTE[l] = "“" + l + "” is a level " + (sc.title || e[0]) + " names itself" + (levels.length === 1 ? ": its only level, so reaching it is the top." : ", ranked like " + ALIAS[l] + ".") + (said ? " " + LEVEL_DEF[l] : "");
        }
      });
    });
  }

  function known(l) {
    return KNOWN.indexOf(l) >= 0 ? l : ALIAS[l] || null;
  }

  // Dots mean "no result"; stripes are kept for "not assessed by an agent yet".
  var UNSCORED = "radial-gradient(circle,var(--faint) 1.3px,transparent 1.7px) 0 0/5px 5px";

  function fill(l) {
    if (l === "unscored") return UNSCORED;
    return "var(--" + (known(l) || "accent") + ")";
  }

  function ink(l) {
    return known(l) ? "var(--" + known(l) + "-ink)" : "var(--accent)";
  }

  // The first whole sentence of a description, for cards: a card shows a complete thought, never a cut one.
  function firstSentence(text) {
    var t = String(text || "").trim();
    var depth = 0;
    var code = false;
    for (var i = 0; i < t.length - 2; i++) {
      var ch = t.charAt(i);
      if (ch === "`") code = !code;
      else if (!code && (ch === "[" || ch === "(")) depth++;
      else if (!code && (ch === "]" || ch === ")")) depth = Math.max(0, depth - 1);
      else if (!code && !depth && (ch === "." || ch === "!" || ch === "?") && /\s/.test(t.charAt(i + 1)) && /[A-Z]/.test(t.charAt(i + 2))) return t.slice(0, i + 1);
    }
    return t;
  }

  // "Bytes of…" reads as "bytes of…" mid-sentence; "MCP…" keeps its capitals.
  function lowerFirst(t) {
    return /^[A-Z][a-z]/.test(t || "") ? t.charAt(0).toLowerCase() + t.slice(1) : t;
  }

  function plural(n, word, many) {
    return fmt(n) + " " + (n === 1 ? word : many || word + "s");
  }

  function fmt(n) {
    return Number(n).toLocaleString("en-US");
  }

  function levelOrder(levels) {
    return ["none"].concat(levels || [], ["skipped"]);
  }

  function tallyLevels(rows, order) {
    var tally = {};
    rows.forEach(function (pair) {
      tally[pair[1].level] = (tally[pair[1].level] || 0) + 1;
    });
    return order
      .filter(function (l) {
        return tally[l];
      })
      .map(function (l) {
        return { level: l, n: tally[l] };
      });
  }

  // Grow from zero on the next frame so a re-render reads as movement, not a jump.
  function grow(nodes, targets) {
    requestAnimationFrame(function () {
      requestAnimationFrame(function () {
        nodes.forEach(function (n, i) {
          n.style.flexGrow = targets[i];
        });
      });
    });
  }

  // Skipped entities are outside a scorecard's scope, so they take no length in a bar.
  function barEl(all, cls, label) {
    var parts = all.filter(function (p) {
      return p.level !== "skipped";
    });
    var spans = parts.map(function (p) {
      return p.hollow
        ? el("span", { className: "hollow", style: "flex-grow:0;flex-basis:0;--seg:" + fill(p.level), title: p.level + ": " + p.n + ", not assessed by an agent yet" })
        : el("span", { style: "flex-grow:0;flex-basis:0;background:" + fill(p.level), title: p.level + ": " + p.n });
    });
    grow(
      spans,
      parts.map(function (p) {
        return p.n;
      }),
    );
    var bar = el("div", { className: "bar" + (cls ? " " + cls : "") }, ...spans);
    bar.setAttribute("role", "img");
    bar.setAttribute(
      "aria-label",
      label ||
        parts
          .map(function (p) {
            return p.n + " " + p.level + (p.hollow ? " not assessed" : "");
          })
          .join(", "),
    );
    return bar;
  }

  // A scorecard's level parts with the entities no agent has assessed split out, so a bar draws them hollow.
  // The level key above a scorecard table, with the hatched swatch when any row draws one.
  function levelLegend(D, rows) {
    var hollow = (rows || []).some(function (r) {
      return !r.compare && unassessedOf(D, r.name);
    });
    return el(
      "div",
      { className: "legend", style: "margin-bottom:8px" },
      ...["none", "bronze", "silver", "gold"].map(function (l) {
        return el("span", {}, el("span", { className: "dot", style: "background:" + fill(l) }), l);
      }),
      hollow ? el("span", {}, el("span", { className: "dot hatch", style: "--seg:" + fill("bronze") }), "hatched: ✦ not assessed by an agent yet") : null,
    );
  }

  function splitUnassessed(D, name, parts) {
    var ids = (D.unassessed || {})[name] || [];
    var rows = (D.scores || {})[name] || {};
    var by = {};
    ids.forEach(function (id) {
      var s = rows[id];
      if (s && s.level !== "skipped") by[s.level] = (by[s.level] || 0) + 1;
    });
    var out = [];
    parts.forEach(function (p) {
      var u = Math.min(by[p.level] || 0, p.n);
      if (p.n - u) out.push({ level: p.level, n: p.n - u });
      if (u) out.push({ level: p.level, n: u, hollow: true });
    });
    return out;
  }

  function unassessedOf(D, name) {
    var rows = (D.scores || {})[name] || {};
    return ((D.unassessed || {})[name] || []).filter(function (id) {
      return rows[id] && rows[id].level !== "skipped";
    }).length;
  }

  function legendEl(parts) {
    return el(
      "div",
      { className: "legend" },
      ...parts.map(function (p) {
        if (p.level === "skipped") return el("span", {}, el("b", { textContent: fmt(p.n) }), " not in scope");
        if (p.level === "unscored") return el("span", {}, el("span", { className: "dot", style: "background:" + UNSCORED }), el("b", { textContent: fmt(p.n) }), " not scored");
        return el(
          "span",
          {},
          el("span", { className: "dot", style: "background:" + fill(p.level) }),
          el("b", { textContent: fmt(p.n) }),
          " " + p.level,
        );
      }),
    );
  }

  function badge(level) {
    var span = el("span", { className: "lvl " + level, style: "--fill:" + fill(level) + ";--ink:" + ink(level) }, el("i"), level);
    span.setAttribute("aria-label", "level " + level);
    if (LEVEL_NOTE[level]) span.title = LEVEL_NOTE[level];
    return span;
  }

  // An agent-judged level with no assessment behind it is drawn dashed, the same everywhere it is shown.
  // Checks at or below an entity's level that were not run in this build: the level holds only if they pass.
  function unrunIn(sc, score) {
    var order = (sc && sc.levels) || [];
    var here = order.indexOf(score.level);
    return Object.keys(score.rules || {}).filter(function (r) {
      return score.rules[r] === "unknown" && order.indexOf(((sc.rules || {})[r] || {}).level) <= here;
    });
  }

  // How many in-scope entities of a scorecard hold a level that rests on checks not run in this build.
  function provisionalOf(D, name) {
    var sc = (D.scorecards || {})[name];
    var rows = (D.scores || {})[name] || {};
    return Object.keys(rows).filter(function (id) {
      return rows[id].level !== "skipped" && unrunIn(sc, rows[id]).length > 0;
    }).length;
  }

  // A level that may not hold is dashed everywhere it is drawn, with the reason beside it. `quiet`
  // keeps the outline but drops the tag, for a column whose header already says it once.
  function levelPill(level, unassessed, unrun, quiet) {
    var pill = badge(level);
    if (!unassessed && !(unrun && unrun.length)) return pill;
    pill.classList.add("vacuous", unassessed ? "unjudged" : "unrun");
    var why = unassessed
      ? "No agent has assessed this entity, so its agent-judged rules have nothing to read: this level is provisional"
      : plural(unrun.length, "check") + " this level needs " + (unrun.length === 1 ? "was" : "were") + " not run in this build (" + unrun.join(", ") + "): the level holds only if " + (unrun.length === 1 ? "it passes" : "they pass");
    pill.setAttribute("aria-label", "level " + level + ", " + (unassessed ? "not assessed" : "provisional"));
    if (quiet) return el("span", { className: "pillwrap", title: why }, pill);
    return el("span", { className: "pillwrap", title: why }, pill, el("span", { className: "unassessed-tag" + (unassessed ? "" : " unrun"), ariaHidden: "true" }, unassessed ? basisIcon("judged") : null, unassessed ? " not assessed" : "provisional: not run"));
  }

  function medal(level, size) {
    var s = size || 56;
    var none = level === "none" || !known(level);
    var c = fill(level);
    var g = svg(
      "svg",
      { width: s, height: s, viewBox: "0 0 56 56", "aria-hidden": "true" },
      svg("path", { d: "M18 4h8l-4 22h-8z", fill: none ? "var(--line)" : "var(--accent)", opacity: ".85" }),
      svg("path", { d: "M30 4h8l2 22h-8z", fill: none ? "var(--line)" : "var(--accent)", opacity: ".6" }),
      svg("circle", {
        cx: 28,
        cy: 34,
        r: 17,
        fill: none ? "var(--card)" : c,
        stroke: none ? "var(--muted)" : c,
        "stroke-width": none ? 2.5 : 0,
        "stroke-dasharray": none ? "4 3" : "",
      }),
      none ? null : svg("circle", { cx: 28, cy: 34, r: 12, fill: "none", stroke: "#fff", "stroke-opacity": ".55", "stroke-width": 1.5 }),
      none
        ? null
        : svg("path", {
            d: "M28 26.5l2.3 4.7 5.2.8-3.8 3.7.9 5.1-4.6-2.4-4.6 2.4.9-5.1-3.8-3.7 5.2-.8z",
            fill: "#fff",
            "fill-opacity": ".8",
          }),
    );
    return g;
  }

  function boxLogo() {
    return svg(
      "svg",
      { width: 34, height: 34, viewBox: "0 0 34 34", "aria-hidden": "true" },
      svg("rect", { x: 3, y: 11, width: 28, height: 19, rx: 4, fill: "var(--accent)" }),
      svg("path", { d: "M12 11V8a2 2 0 0 1 2-2h6a2 2 0 0 1 2 2v3", fill: "none", stroke: "var(--accent)", "stroke-width": 2.4 }),
      svg("rect", { x: 3, y: 17, width: 28, height: 3, fill: "var(--card)", opacity: ".35" }),
      svg("rect", { x: 14.5, y: 15.5, width: 5, height: 6, rx: 1.2, fill: "var(--gold)" }),
    );
  }

  function ring(pct, opts) {
    var o = opts || {};
    var s = o.size || 52;
    var r = s / 2 - 5;
    var circ = 2 * Math.PI * r;
    var color = o.color || "var(--accent)";
    var arc = svg("circle", {
      cx: s / 2,
      cy: s / 2,
      r: r,
      fill: "none",
      stroke: color,
      "stroke-width": 6,
      "stroke-linecap": "round",
      "stroke-dasharray": circ,
      "stroke-dashoffset": circ,
      transform: "rotate(-90 " + s / 2 + " " + s / 2 + ")",
    });
    var label = String(o.text != null ? o.text : pct == null ? "—" : pct + "%");
    var target = null;
    if (o.target != null && o.target < 100) {
      var a = (o.target / 100) * 2 * Math.PI - Math.PI / 2;
      target = svg("line", {
        x1: s / 2 + (r - 5) * Math.cos(a),
        y1: s / 2 + (r - 5) * Math.sin(a),
        x2: s / 2 + (r + 5) * Math.cos(a),
        y2: s / 2 + (r + 5) * Math.sin(a),
        stroke: "var(--fg)",
        "stroke-width": 2,
      });
    }
    var g = svg(
      "svg",
      { class: "ring", width: s, height: s, viewBox: "0 0 " + s + " " + s, role: "img", "aria-label": o.label || pct + "%" },
      svg("circle", { cx: s / 2, cy: s / 2, r: r, fill: "none", stroke: "var(--line)", "stroke-width": 6 }),
      pct == null ? null : arc,
      target,
      o.bare ? null : svg("text", { x: "50%", y: "50%", "text-anchor": "middle", "dominant-baseline": "central", style: s > 70 ? "font-size:18px" : label.length > 3 ? "font-size:11px" : "" }, label),
    );
    if (pct != null) {
      requestAnimationFrame(function () {
        requestAnimationFrame(function () {
          arc.setAttribute("stroke-dashoffset", circ * (1 - Math.max(0, Math.min(100, pct)) / 100));
        });
      });
    }
    return g;
  }

  // A comparison's standing: one segment per factor, coloured ahead, at parity or behind, so it is
  // never read as the progress ring's share.
  function tallyRing(v, n, size, bare) {
    var s = size || 52;
    var c = s / 2;
    var r = c - 5;
    var circ = 2 * Math.PI * r;
    var seg = circ / Math.max(n, 1);
    var gap = n > 1 ? Math.min(4, seg / 3) : 0;
    var segs = [];
    ["ahead", "parity", "behind", "unknown"].forEach(function (k) {
      for (var j = 0; j < (v[k] || 0); j++) {
        segs.push(
          svg("circle", {
            cx: c,
            cy: c,
            r: r,
            fill: "none",
            stroke: standingColor(k),
            "stroke-width": 6,
            "stroke-dasharray": seg - gap + " " + (circ - seg + gap),
            "stroke-dashoffset": -segs.length * seg,
            transform: "rotate(-90 " + c + " " + c + ")",
          }),
        );
      }
    });
    var text = v.ahead + "/" + n;
    return svg(
      "svg",
      { class: "ring tally", width: s, height: s, viewBox: "0 0 " + s + " " + s, role: "img", "aria-label": tallyWords(v, n) },
      ...segs,
      bare ? null : svg("text", { x: "50%", y: "50%", "text-anchor": "middle", "dominant-baseline": "central", style: s > 70 ? "font-size:18px" : text.length > 3 ? "font-size:11px" : "" }, text),
    );
  }

  function tallyWords(v, n) {
    return ["ahead on " + v.ahead, v.parity ? "at parity on " + v.parity : null, "behind on " + v.behind].filter(Boolean).join(", ") + " of " + n + " factors";
  }

  function shortId(id) {
    var s = String(id || "");
    var slash = s.lastIndexOf("/");
    var colon = s.lastIndexOf(":");
    var i = Math.max(slash, colon);
    return i >= 0 && i < s.length - 1 ? s.slice(i + 1) : s;
  }

  function isTop(D, name, level) {
    var levels = ((D.scorecards || {})[name] || {}).levels || [];
    return !!level && level === levels[levels.length - 1];
  }

  // A repository standard whose only top level is the repository's level holds the repository there:
  // that is a ceiling, not an achievement.
  function capsRepo(D, name, level) {
    var R = repoLevel(D);
    return !!(R && R.next && isTop(D, name, level) && level === R.floor && (D.repoRollup || []).indexOf(name) >= 0);
  }

  function topNote(D, name, level) {
    if (capsRepo(D, name, level)) return el("span", { className: "capnote", textContent: " ▲ ceiling: its top level is " + level + ", so the repository can't rise above it", title: "Its top level is " + level + ", so the repository cannot rise above " + level + " until this standard gains a higher level" });
    var top = isTop(D, name, level) ? el("span", { className: "topmark", textContent: " ✓ top level" }) : "";
    return LEVEL_DEF[level] ? el("span", {}, top, el("span", { className: "sub leveldef", textContent: " · " + LEVEL_DEF[level] })) : top;
  }

  function title(D, name) {
    var sc = (D.scorecards || {})[name] || {};
    return sc.title || name;
  }

  var BASIS = {
    measured: {
      label: "Measured",
      desc: "A tool, connector or script read the fact. The same input gives the same result.",
    },
    judged: {
      label: "Agent-judged",
      desc: "Passes on an agent's written assessment against a rubric, reviewed by a second agent. It changes when a verdict does.",
    },
  };

  function basisIcon(b) {
    if (b === "judged")
      return svg("svg", { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": "true" }, svg("path", { d: "M6 .6l1.5 3.8 3.9 1.6-3.9 1.6L6 11.4 4.5 7.6.6 6l3.9-1.6z", fill: "currentColor" }));
    return svg(
      "svg",
      { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": "true" },
      svg("path", { d: "M1.4 9.2a4.6 4.6 0 0 1 9.2 0", fill: "none", stroke: "currentColor", "stroke-width": 1.6, "stroke-linecap": "round" }),
      svg("path", { d: "M6 9.2l2.3-3.3", stroke: "currentColor", "stroke-width": 1.6, "stroke-linecap": "round" }),
      svg("circle", { cx: 6, cy: 9.2, r: 1.2, fill: "currentColor" }),
    );
  }

  function lockIcon() {
    return svg(
      "svg",
      { width: 12, height: 12, viewBox: "0 0 12 12", "aria-hidden": "true" },
      svg("rect", { x: 2, y: 5.2, width: 8, height: 6, rx: 1.2, fill: "currentColor" }),
      svg("path", { d: "M3.8 5.2V3.8a2.2 2.2 0 0 1 4.4 0v1.4", fill: "none", stroke: "currentColor", "stroke-width": 1.4 }),
    );
  }

  function basisOf(rule) {
    return (rule || {}).basis === "judged" ? "judged" : "measured";
  }

  function basisBadge(b, mini, inControl) {
    // Mini badges sit inside links and dense rows: the same word, smaller, joins the row's accessible name
    // rather than adding a focus stop of its own, and the page's key explains it in full.
    if (mini) return el("span", { className: "basis " + b + " mini", title: BASIS[b].label + ": " + BASIS[b].desc }, basisIcon(b), el("span", {}, el("span", { className: "sr", textContent: " (" }), BASIS[b].label, el("span", { className: "sr", textContent: ")" })));
    if (inControl) return el("span", { className: "basis " + b, title: BASIS[b].label + ": " + BASIS[b].desc }, basisIcon(b), el("span", { className: "sr", textContent: " (" }), BASIS[b].label, el("span", { className: "sr", textContent: ")" }));
    // A full badge stands on its own, so keyboard and touch readers can open what it means.
    var full = el("span", { className: "basis tip " + b, tabIndex: 0 }, basisIcon(b), BASIS[b].label, el("span", { className: "sr", textContent: ": " + BASIS[b].desc }));
    full.dataset.tip = BASIS[b].desc;
    return full;
  }

  // One basis for every rule: the badge and its count, in the line that says what the scorecard rates.
  function basisInline(counts, noun) {
    var only = counts && (counts.measured || 0) + (counts.judged || 0) > 0 && !(counts.measured && counts.judged) ? (counts.measured ? "measured" : "judged") : null;
    if (!only) return null;
    return el("span", { className: "basis-inline" }, " · ", basisBadge(only), " " + (counts[only] === 1 ? "its one " + (noun || "rule") : "all " + plural(counts[only], noun || "rule")));
  }

  // What the two badges mean, with how many rules (or checks) each covers when counts are given.
  function basisKey(counts, noun) {
    var row = function (b) {
      var n = counts ? counts[b] || 0 : null;
      return el("div", {}, basisBadge(b), el("span", {}, n != null ? el("b", { textContent: plural(n, noun || "rule") + ". " }) : null, BASIS[b].desc));
    };
    var only = counts && (counts.measured || 0) + (counts.judged || 0) > 0 && !(counts.measured && counts.judged) ? (counts.measured ? "measured" : "judged") : null;
    // One basis only: say so in a line, and name the other so its absence is visible.
    var k = only
      ? el("div", { className: "basis-key one" }, el("div", {}, basisBadge(only), el("span", {}, el("b", { textContent: (counts[only] === 1 ? "" : "All ") + plural(counts[only], noun || "rule") + ". " }), BASIS[only].desc + " None here is " + BASIS[only === "measured" ? "judged" : "measured"].label.toLowerCase() + ".")))
      : el("div", { className: "basis-key" }, row("measured"), row("judged"));
    k.setAttribute("role", "note");
    k.setAttribute("aria-label", "How a rule is decided");
    return k;
  }

  function basisCounts(rules) {
    var n = { measured: 0, judged: 0 };
    Object.keys(rules || {}).forEach(function (r) {
      n[basisOf(rules[r])]++;
    });
    return n;
  }

  function refs(links) {
    var safe = (links || []).filter(function (l) {
      return l && /^https?:\/\//.test(l.url || "");
    });
    if (!safe.length) return null;
    return el(
      "span",
      { className: "refs" },
      ...safe.map(function (l) {
        // A title that leads with a decision id reads as its claim; the id stays in the hover text.
        var m = /^(D\d+|D0[0-9A-Z]{20,}) (.+)$/.exec(l.title || "");
        var a = el("a", { href: l.url, textContent: m ? "Decision: " + m[2] : l.title || l.url, target: "_blank", rel: "noopener" });
        if (m) a.title = m[1] + ": " + m[2];
        return a;
      }),
    );
  }

  function evidenceEl(items, seen, rule) {
    if (!items.length) return "";
    var text = function (i) {
      return typeof i === "string" ? i : JSON.stringify(i);
    };
    var key = items.map(text).join("\n");
    if (seen && seen[key]) return el("p", { className: "sub", textContent: "Evidence: the same " + plural(items.length, "item") + " as " + seen[key] + " above." });
    if (seen) seen[key] = rule;
    var li = function (i) {
      return el("li", {}, el("code", { textContent: text(i) }));
    };
    var SHOW = 3;
    if (items.length <= SHOW + 1) return el("ul", { className: "evidence" }, ...items.map(li));
    return el(
      "div",
      { className: "evidence" },
      el("ul", {}, ...items.slice(0, SHOW).map(li)),
      el("details", {}, el("summary", { textContent: "Show " + (items.length - SHOW) + " more" }), el("ul", {}, ...items.slice(SHOW, 50).map(li), items.length > 50 ? el("li", { textContent: "… and " + (items.length - 50) + " more" }) : null)),
    );
  }

  function whyEl(def, label) {
    if (!def || (!def.rationale && !(def.links || []).length)) return null;
    return el("div", { className: "why" }, def.rationale ? el("b", { textContent: label || "Why: " }) : null, ...(def.rationale ? rich(def.rationale) : []), refs(def.links));
  }

  function ruleLink(scName, rule, text) {
    return link("rule/" + enc(scName) + "/" + enc(rule), "rl", text || rule);
  }

  function isComparative(D, name) {
    return !!(D.comparisons || {})[name];
  }

  var LADDER = ["none", "bronze", "silver", "gold"];

  function rankLevel(l) {
    if (l == null) return 9;
    var k = known(l);
    return k ? LADDER.indexOf(k) : 0;
  }

  // A scorecard's lowest level among the entities it really scores: demo fixtures, out-of-scope
  // entities and entities whose every rule is unknown don't count.
  function floorOf(D, name) {
    var sc = (D.scorecards || {})[name];
    if (!sc) return null;
    var order = ["none"].concat(sc.levels || []);
    var best = null;
    Object.keys((D.scores || {})[name] || {}).forEach(function (id) {
      if (id.indexOf("/demo/") >= 0) return;
      var row = D.scores[name][id];
      if (!row || row.level === "skipped") return;
      var rules = row.rules || {};
      var keys = Object.keys(rules);
      if (
        keys.length &&
        keys.every(function (k) {
          return rules[k] === "unknown";
        })
      )
        return;
      var r = order.indexOf(row.level);
      if (best == null || r < best.rank) best = { level: row.level, rank: r };
    });
    return best ? best.level : null;
  }

  function trackTitle(D, name) {
    var t = (D.tracks || {})[name];
    return (t && t.title) || name;
  }

  // A track's standing: for level scorecards, the share of entities at the top level averaged
  // over its scorecards; for comparisons, the factors at parity or ahead.
  function trackStats(D, S, name) {
    var rows = S.rows.filter(function (r) {
      return r.sc.track === name;
    });
    var compare =
      rows.length > 0 &&
      rows.every(function (r) {
        return isComparative(D, r.name);
      });
    var none = 0;
    var shares = [];
    var cards = rows.map(function (r) {
      if (!r.compare) {
        none += r.none;
        if (r.scored) shares.push(r.topShare);
      }
      return { row: r, floor: r.compare ? null : floorOf(D, r.name) };
    });
    cards.sort(function (a, b) {
      return rankLevel(a.floor) - rankLevel(b.floor) || b.row.none - a.row.none || title(D, a.row.name).localeCompare(title(D, b.row.name));
    });
    var verdicts = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
    var behind = [];
    rows.forEach(function (r) {
      if (!r.compare) return;
      var c = D.comparisons[r.name];
      var v = (c.verdict || {})[(c.peers || [])[0]] || "unknown";
      verdicts[v] = (verdicts[v] || 0) + 1;
      if (v === "behind") behind.push(title(D, r.name));
    });
    var pct = compare
      ? Math.round(((verdicts.ahead + verdicts.parity) / rows.length) * 100)
      : shares.length
        ? Math.round(
            shares.reduce(function (a, b) {
              return a + b;
            }, 0) / shares.length,
          )
        : null;
    var campaigns = Object.keys(D.campaigns || {}).filter(function (n) {
      return D.campaigns[n].track === name;
    });
    return { name: name, def: (D.tracks || {})[name] || {}, rows: rows, cards: cards, compare: compare, none: none, pct: pct, averaged: shares.length, verdicts: verdicts, behind: behind, campaigns: campaigns };
  }

  function tracksSorted(D, S) {
    return Object.keys(D.tracks || {})
      .map(function (n) {
        return trackStats(D, S, n);
      })
      .filter(function (t) {
        return t.rows.length || t.campaigns.length;
      })
      .sort(function (a, b) {
        return (a.compare ? 1 : 0) - (b.compare ? 1 : 0) || (a.pct == null ? 101 : a.pct) - (b.pct == null ? 101 : b.pct) || b.none - a.none || a.name.localeCompare(b.name);
      });
  }

  function unassessedNote(D, card, scored) {
    var ids = (D.unassessed || {})[card] || [];
    var inScope = ids.filter(function (id) {
      return ((D.scores[card] || {})[id] || {}).level !== "skipped";
    }).length;
    if (!inScope) return null;
    var rules = ((D.scorecards || {})[card] || {}).rules || {};
    var passing = [];
    var failing = [];
    Object.keys(rules)
      .filter(function (r) {
        return basisOf(rules[r]) === "judged";
      })
      .forEach(function (r) {
        var fails = ids.some(function (id) {
          var o = (((D.scores[card] || {})[id] || {}).rules || {})[r];
          return o && o !== "pass";
        });
        (fails ? failing : passing).push(r);
      });
    var names = function (rs) {
      return rs.map(function (r) {
        return ruleLink(card, r);
      });
    };
    var list = function (rs) {
      var out = [];
      names(rs).forEach(function (a, i) {
        if (i) out.push(i === rs.length - 1 ? " and " : ", ");
        out.push(a);
      });
      return out;
    };
    var why = [];
    if (passing.length) why.push(...list(passing), passing.length === 1 ? " passes" : " pass", " only because there is nothing to fail on");
    if (passing.length && failing.length) why.push("; ");
    if (failing.length) why.push(...list(failing), failing.length === 1 ? " fails" : " fail", " until an assessment exists");
    why.push(passing.length ? ". A level they reach means nothing has been found, not that a review found nothing." : ".");
    return el(
      "div",
      { className: "callout judged-note" },
      el("span", {}, el("b", { textContent: (inScope === scored ? "No entity here has an" : fmt(inScope) + " of " + fmt(scored) + " entities have no") + " agent's assessment yet. " }), "On those, of the ", basisBadge("judged", true), " rules, ", ...why),
    );
  }

  function unassessedIn(D, card, id) {
    return ((D.unassessed || {})[card] || []).indexOf(id) >= 0;
  }

  // What a track card's ring colours and a comparison's segments mean, in place of a paragraph.
  function trackKey(tracks) {
    var sw = function (c, t) {
      return el("span", {}, el("span", { className: "dot", style: "background:" + c }), t);
    };
    var compare = tracks.some(function (t) {
      return t.compare;
    });
    return el(
      "div",
      { className: "legend trackkey" },
      el("b", { textContent: "Ring, share at the top level:" }),
      sw(RAMP[3], "below 25%"),
      sw(RAMP[2], "25–49%"),
      sw(RAMP[1], "50–79%"),
      sw(RAMP[0], "80–100%"),
      compare ? el("b", { textContent: "Segmented ring, one per factor:" }) : null,
      compare ? sw(standingColor("ahead"), "ahead") : null,
      compare ? sw(standingColor("parity"), "at parity") : null,
      compare ? sw(standingColor("behind"), "behind") : null,
      el("span", { className: "medal-key" }, el("b", { textContent: "Medal:" }), medal("bronze", 16), "its weakest entity's level ·", medal("none", 16), "none"),
    );
  }

  function furthestTrack(D, S) {
    var t = tracksSorted(D, S).find(function (t) {
      return !t.compare && t.pct != null && t.pct < 100;
    });
    return t ? t.name : null;
  }

  // One ordered ramp, red to green, for a share at the top level; blue is left for parity.
  var RAMP = ["var(--pass)", "color-mix(in srgb,var(--pass) 55%,var(--gold))", "var(--bronze)", "var(--fail)"];

  function trackColor(t) {
    if (t.compare) return "var(--accent)";
    if (t.pct == null) return "var(--faint)";
    return RAMP[t.pct >= 80 ? 0 : t.pct >= 50 ? 1 : t.pct >= 25 ? 2 : 3];
  }

  var STANDING_MARK = { ahead: "▲", parity: "=", behind: "▼", unknown: "?" };

  function standingChip(s, text) {
    var v = s || "unknown";
    return el("span", { className: "standing " + v, textContent: STANDING_MARK[v] + " " + (text || v) });
  }

  // "Toolbox is behind Backstage", in words that read as a sentence.
  function standingPhrase(a, s, b) {
    if (s === "ahead") return a + " ahead of " + b;
    if (s === "behind") return a + " behind " + b;
    if (s === "parity") return a + " at parity with " + b;
    return a + " vs " + b + ": not measured";
  }

  function standingColor(s) {
    return s === "ahead" ? "var(--pass)" : s === "behind" ? "var(--fail)" : s === "parity" ? "var(--accent)" : "var(--unknown)";
  }

  // Kinds in their declared groups, then any kind no group names.
  function kindGroupsOf(D) {
    var catalog = D.catalog || {};
    var seen = {};
    var groups = (D.kindGroups || [])
      .map(function (g) {
        var kinds = (g.kinds || []).filter(function (k) {
          seen[k] = true;
          return catalog[k];
        });
        return { name: g.name, title: g.title || g.name, description: g.description, kinds: kinds };
      })
      .filter(function (g) {
        return g.kinds.length;
      });
    var rest = catalogKindsSorted(D).filter(function (k) {
      return !seen[k];
    });
    if (rest.length) groups.push({ name: "", title: groups.length ? "Other kinds" : "Kinds", kinds: rest });
    groups.forEach(function (g) {
      g.count = g.kinds.reduce(function (n, k) {
        return n + catalog[k].length;
      }, 0);
    });
    return groups;
  }

  function kindDescription(D, k) {
    return (((D.kinds || {})[k] || {}).description || "").trim();
  }

  var ACRONYMS = { Api: "API", Cli: "CLI", Json: "JSON", Mcp: "MCP", Sdlc: "SDLC", Ui: "UI", Url: "URL" };
  // A kind's id (SdlcLayer) in the reader's words (SDLC layer), unless the kind names a title.
  function kindTitle(D, k) {
    var t = ((D.kinds || {})[k] || {}).title;
    if (t) return t;
    var words = String(k || "").match(/OpenSSF|[A-Z]{2,}(?![a-z])|[A-Z][a-z0-9]*|[a-z0-9]+/g) || [k];
    return words
      .map(function (w, i) {
        if (ACRONYMS[w]) return ACRONYMS[w];
        if (w === "OpenSSF" || /^[A-Z]{2,}$/.test(w)) return w;
        return i ? w.toLowerCase() : w;
      })
      .join(" ");
  }

  // "15 action dependencies", "26 SDLC stages": a count of a kind, in running text.
  function kindCount(D, k, n) {
    var t = kindTitle(D, k);
    if (!/^[A-Z]{2,}|^OpenSSF/.test(t)) t = t.charAt(0).toLowerCase() + t.slice(1);
    if (n !== 1) t = /[^aeiou]y$/.test(t) ? t.slice(0, -1) + "ies" : /(s|x|ch|sh)$/.test(t) ? t + "es" : t + "s";
    return fmt(n) + " " + t;
  }

  // The Scorecard kind counts more than the Scorecards tab: say what the surplus is wherever its total appears.
  function scorecardSplit(D, k) {
    if (k !== "Scorecard") return "";
    var ids = (D.catalog || {})[k] || [];
    var hits = {};
    var split = ids.map(function (id) {
      var m = /^[^:]+:([^/]+)\/(.+)$/.exec(id);
      if (m && (D.scorecards || {})[m[2]]) hits[m[1]] = (hits[m[1]] || 0) + 1;
      return m;
    });
    var mine = Object.keys(hits).sort(function (a, b) {
      return hits[b] - hits[a];
    })[0];
    var own = 0;
    var cmp = 0;
    split.forEach(function (m) {
      if (!m || m[1] !== mine || !(D.scorecards || {})[m[2]]) return;
      if ((D.comparisons || {})[m[2]]) cmp++;
      else own++;
    });
    var parts = [plural(own, "standard"), cmp ? plural(cmp, "comparison factor") : null, ids.length - own - cmp ? fmt(ids.length - own - cmp) + " in example portals" : null].filter(Boolean);
    return parts.length > 1 ? " (" + parts.join(", ") + ")" : "";
  }

  function humanize(s) {
    var t = String(s || "")
      .replace(/([a-z0-9])([A-Z])/g, "$1 $2")
      .replace(/[-_]+/g, " ")
      .toLowerCase();
    return t.charAt(0).toUpperCase() + t.slice(1);
  }

  function num(v) {
    return Number(v).toLocaleString("en-US", { maximumFractionDigits: 2 });
  }

  // A comparison rule in the reader's words: the description of the one fact it reads.
  function measureLabel(c, r) {
    var fs = (c.reads || {})[r] || [];
    var d = fs.length === 1 ? (c.fields || {})[fs[0]] : null;
    return d ? d.replace(/\.$/, "") : humanize(r);
  }

  function ruleValue(c, r, id) {
    var s = (c.scores || {})[id];
    var v = s && s.values ? s.values[r] : undefined;
    if (v != null) return v;
    var fs = (c.reads || {})[r] || [];
    return fs.length === 1 ? ((c.facts || {})[id] || {})[fs[0]] : undefined;
  }

  function valueLine(c, r, rd, ids, who) {
    var vals = ids.map(function (id) {
      return ruleValue(c, r, id);
    });
    var numeric = vals.every(function (v) {
      return v == null || typeof v === "number";
    });
    var peer = (c.peers || [])[0];
    var st = (c.rules[r] || {})[peer];
    var leader = st === "ahead" ? c.subject : st === "behind" ? peer : null;
    var max = Math.max.apply(
      null,
      vals
        .filter(function (v) {
          return typeof v === "number";
        })
        .concat([0]),
    );
    return el(
      "div",
      { className: "vals" },
      ...ids.map(function (id, i) {
        var outcome = (((c.scores || {})[id] || {}).rules || {})[r] || "unknown";
        var v = vals[i];
        var color = id === leader ? "var(--pass)" : st === "parity" ? "var(--accent)" : "var(--faint)";
        var text =
          v == null
            ? outcome === "pass"
              ? "✓ meets it"
              : outcome === "fail"
                ? "✗ misses it"
                : "not measured"
            : typeof v === "boolean"
              ? v
                ? "✓ yes"
                : "✗ no"
              : typeof v === "number"
                ? num(v) + (rd.unit ? " " + rd.unit : "")
                : String(v);
        var fillEl = null;
        if (numeric && typeof v === "number" && max > 0) {
          fillEl = el("span", { style: "background:" + color });
          var w = Math.max(2, (v / max) * 100);
          requestAnimationFrame(function () {
            requestAnimationFrame(function () {
              fillEl.style.width = w + "%";
            });
          });
        }
        return el(
          "div",
          { className: "val" },
          el("span", { className: "who", textContent: who(id), title: who(id) }),
          fillEl ? el("span", { className: "vb" }, fillEl) : el("span"),
          // Colour marks the leader only; the rule's absolute bar, when it has one, is in the hover text.
          el("span", { className: "v " + (id === leader ? "pass" : ""), textContent: text, title: outcome === "pass" ? "Meets the rule's bar" : outcome === "fail" ? "Misses the rule's bar" : "" }),
        );
      }),
    );
  }

  // How far apart two measured values are, in the reader's terms ("8.6× lower").
  function magnitude(c, r, rd, who) {
    var peer = (c.peers || [])[0];
    var a = ruleValue(c, r, c.subject);
    var b = ruleValue(c, r, peer);
    if (typeof a !== "number" || typeof b !== "number") return null;
    if (a === b) return el("span", { className: "mag", textContent: "same value" });
    if (a <= 0 || b <= 0) return null;
    var ratio = Math.max(a, b) / Math.min(a, b);
    var x = ratio >= 10 ? Math.round(ratio).toLocaleString("en-US") : ratio.toFixed(1);
    var lead = rd.better === "lower" ? (a < b ? c.subject : peer) : rd.better === "higher" ? (a > b ? c.subject : peer) : c.subject;
    var la = lead === c.subject ? a : b;
    var lb = lead === c.subject ? b : a;
    return el("span", { className: "mag", textContent: who(lead) + "'s is " + x + "× " + (la < lb ? "lower" : "higher") });
  }

  function gapText(D, g, id) {
    var m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
    if (m) {
      var sc = (D.scorecards || {})[m[1]];
      var score = ((D.scores || {})[m[1]] || {})[id];
      var need = sc && score ? blockers(sc, score, m[2]) : [];
      return (need.length ? "Needs " + need.join(", ") + " for " + m[2] : m[2].charAt(0).toUpperCase() + m[2].slice(1) + " is the goal") + " on " + title(D, m[1]) + "; " + m[3] + " now.";
    }
    m = /^(.+?):rule (.+) \((\S+)\)$/.exec(g);
    if (m) return title(D, m[1]) + ": rule " + m[2] + " is " + m[3] + ".";
    m = /^(.+?): unknown scorecard$/.exec(g);
    if (m) return "Scorecard " + m[1] + " is unknown.";
    return g;
  }

  // Beside a scorecard's name in any list: whether an agent's judgement decides some of its rules.
  function judgedMark(D, name) {
    var n = basisCounts(((D.scorecards || {})[name] || {}).rules);
    if (!n.judged) return null;
    var b = basisBadge("judged", true);
    b.title = (n.measured ? n.judged + " of " + (n.judged + n.measured) + " rules are" : "Every rule is") + " agent-judged: " + BASIS.judged.desc;
    return b;
  }

  function cardName(D, name) {
    var t = title(D, name);
    var mark = judgedMark(D, name);
    var head = el("span", { className: "t" }, t, mark ? " " : null, mark);
    return t === name ? [head] : [head, el("code", { className: "c", textContent: name })];
  }

  function daysLeft(end) {
    if (!end) return null;
    var ms = Date.parse(end + "T23:59:59Z") - Date.now();
    return isNaN(ms) ? null : Math.ceil(ms / 86400000);
  }

  function isPassing(o) {
    return o === "pass" || o === "waived";
  }

  // A check that was not run ("unknown") is not a failure: the scorer lets it hold a level provisionally.
  function isFailing(o) {
    return o != null && !isPassing(o) && o !== "skipped" && o !== "unknown";
  }

  // The rules an entity still fails at or below `level` (what stands between it and that level).
  function blockers(sc, score, level) {
    var order = sc.levels || [];
    var cap = order.indexOf(level);
    return Object.keys(score.rules || {})
      .filter(function (n) {
        var rl = ((sc.rules || {})[n] || {}).level;
        return isFailing(score.rules[n]) && order.indexOf(rl) <= cap;
      })
      .sort();
  }

  function nextLevel(sc, level) {
    var order = sc.levels || [];
    if (level === "none") return order[0] || null;
    var i = order.indexOf(level);
    return i >= 0 && i < order.length - 1 ? order[i + 1] : null;
  }

  function campaignState(c) {
    if (!c.inScope && c.measure == null) return "empty";
    return c.onTarget ? "on" : "off";
  }

  function scorecardRows(D) {
    return Object.keys(D.scorecards || {})
      .map(function (name) {
        var sc = D.scorecards[name];
        var ents = Object.entries((D.scores || {})[name] || {});
        var parts = tallyLevels(ents, levelOrder(sc.levels));
        var count = function (l) {
          var p = parts.find(function (x) {
            return x.level === l;
          });
          return p ? p.n : 0;
        };
        var top = (sc.levels || [])[(sc.levels || []).length - 1];
        var scored = ents.filter(function (p) {
          return p[1].level !== "skipped";
        }).length;
        return {
          name: name,
          sc: sc,
          compare: isComparative(D, name),
          parts: parts,
          none: count("none"),
          bronze: count("bronze"),
          top: top,
          topShare: scored ? Math.round((count(top) / scored) * 100) : 0,
          scored: scored,
          total: ents.length,
          weight: isComparative(D, name) ? -1 : count("none") * 1000 + count("bronze"),
        };
      })
      .sort(function (a, b) {
        return b.weight - a.weight || a.name.localeCompare(b.name);
      });
  }

  function campaignNamesSorted(D) {
    var campaigns = D.campaigns || {};
    var rank = { off: 0, empty: 1, on: 2 };
    return Object.keys(campaigns)
      .filter(function (n) {
        var st = campaigns[n].status;
        return st === "active" || st === "scheduled" || st === "ended";
      })
      .sort(function (a, b) {
        var ca = campaigns[a];
        var cb = campaigns[b];
        return rank[campaignState(ca)] - rank[campaignState(cb)] || (ca.percent || 0) - (cb.percent || 0) || a.localeCompare(b);
      });
  }

  function scoredKinds(D) {
    var set = {};
    Object.keys(D.scorecards || {}).forEach(function (name) {
      var k = D.scorecards[name].kind;
      if (k && !isComparative(D, name)) set[k] = true;
    });
    return set;
  }

  function catalogKindsSorted(D) {
    var catalog = D.catalog || {};
    var scored = scoredKinds(D);
    return Object.keys(catalog).sort(function (a, b) {
      var as = scored[a] ? 0 : 1;
      var bs = scored[b] ? 0 : 1;
      if (as !== bs) return as - bs;
      return catalog[b].length - catalog[a].length || a.localeCompare(b);
    });
  }

  // Facts every view shares, computed once per load.
  function summarize(D) {
    var rows = scorecardRows(D);
    var byEntity = {};
    var checks = { pass: 0, fail: 0, unknown: 0 };
    rows.forEach(function (r) {
      if (r.compare) return;
      var order = levelOrder(r.sc.levels);
      Object.entries((D.scores || {})[r.name] || {}).forEach(function (pair) {
        var id = pair[0];
        var s = pair[1];
        if (s.level === "skipped") return;
        var cur = byEntity[id];
        var rank = order.indexOf(s.level);
        var norm = ["none", "bronze", "silver", "gold"].indexOf(s.level) >= 0 ? s.level : rank <= 0 ? "none" : "gold";
        if (!cur || ["none", "bronze", "silver", "gold"].indexOf(norm) < ["none", "bronze", "silver", "gold"].indexOf(cur)) byEntity[id] = norm;
        Object.values(s.rules || {}).forEach(function (o) {
          if (isPassing(o)) checks.pass++;
          else if (o === "fail") checks.fail++;
          else if (o === "unknown") checks.unknown++;
        });
      });
    });
    var catalog = D.catalog || {};
    var entityCount = Object.keys(catalog).reduce(function (n, k) {
      return n + catalog[k].length;
    }, 0);
    var kindOf = {};
    Object.keys(catalog).forEach(function (k) {
      catalog[k].forEach(function (id) {
        kindOf[id] = k;
      });
    });
    return { rows: rows, worst: byEntity, checks: checks, entityCount: entityCount, kindOf: kindOf, repo: repoLevel(D) };
  }

  function repoLevel(D) {
    var rollup = Array.isArray(D.repoRollup) ? D.repoRollup : [];
    if (!rollup.length) return null;
    var ladder = ["none", "bronze", "silver", "gold"];
    var constituents = rollup
      .filter(function (name) {
        return (D.scorecards || {})[name];
      })
      .map(function (name) {
        return { name: name, level: floorOf(D, name) || "none" };
      });
    if (!constituents.length) return null;
    constituents.forEach(function (c) {
      var lv = (D.scorecards[c.name].levels || []).slice(-1)[0];
      c.top = known(lv) || lv;
    });
    var ceiling = constituents.reduce(function (m, c) {
      return ladder.indexOf(c.top) >= 0 && ladder.indexOf(c.top) < ladder.indexOf(m) ? c.top : m;
    }, "gold");
    var floor = constituents.reduce(function (m, c) {
      return ladder.indexOf(c.level) < ladder.indexOf(m) ? c.level : m;
    }, "gold");
    var next = ladder[ladder.indexOf(floor) + 1] || null;
    return {
      ladder: ladder,
      floor: floor,
      ceiling: ceiling,
      next: next,
      constituents: constituents,
      holding: constituents.filter(function (c) {
        return c.level === floor;
      }),
    };
  }

  function healthVerdict(D, S, campaignNames) {
    var campaigns = D.campaigns || {};
    var off = campaignNames.filter(function (n) {
      return campaignState(campaigns[n]) === "off";
    });
    var debt = S.rows.filter(function (r) {
      return r.none > 0 && !r.compare;
    });
    var blind = campaignNames.filter(function (n) {
      return campaignState(campaigns[n]) === "empty";
    });
    if (!campaignNames.length && !S.rows.length) return "No campaigns or scorecards in this snapshot.";
    if (!off.length && !debt.length && !blind.length) return "On track: every active campaign meets its target, and no scorecard has entities at none.";
    var bits = [];
    if (debt.length) bits.push(plural(debt.length, "scorecard") + " with entities at none");
    if (off.length) bits.push(plural(off.length, "campaign") + " behind target");
    if (blind.length) bits.push(plural(blind.length, "campaign") + " that can't be measured");
    return "Needs attention: " + (bits.length > 1 ? bits.slice(0, -1).join(", ") + " and " + bits[bits.length - 1] : bits[0]) + ".";
  }

  function parseHash() {
    var h = (location.hash || "").replace(/^#/, "");
    var query = {};
    var qi = h.indexOf("?");
    if (qi >= 0) {
      var params = new URLSearchParams(h.slice(qi + 1));
      ["q", "level", "outcome", "rule"].forEach(function (k) {
        if (params.has(k)) query[k] = params.get(k);
      });
      h = h.slice(0, qi);
    }
    var route = parsePath(h);
    route.query = query;
    return route;
  }

  function parsePath(h) {
    if (!h || h === "overview") return { mode: "overview" };
    if (h === "catalog") return { mode: "catalog" };
    if (h.indexOf("catalog/") === 0) return { mode: "catalog", kind: decodeURIComponent(h.slice(8)) };
    if (h.indexOf("entity/") === 0) return { mode: "entity", id: decodeURIComponent(h.slice(7)) };
    if (h === "tracks") return { mode: "tracks" };
    if (h.indexOf("track/") === 0) return { mode: "tracks", name: decodeURIComponent(h.slice(6)) };
    if (h.indexOf("rule/") === 0) {
      var rest = h.slice(5);
      var cut = rest.indexOf("/");
      return { mode: "rule", scorecard: decodeURIComponent(rest.slice(0, cut)), rule: decodeURIComponent(rest.slice(cut + 1)) };
    }
    if (h === "campaigns") return { mode: "campaigns" };
    if (h.indexOf("campaign/") === 0) return { mode: "campaign", name: decodeURIComponent(h.slice(9)) };
    if (h === "compare") return { mode: "compare" };
    if (h.indexOf("compare/") === 0) return { mode: "compare", name: decodeURIComponent(h.slice(8)) };
    if (h.indexOf("scorecard/") === 0) return { mode: "scorecard", name: decodeURIComponent(h.slice(10)) };
    if (h === "scorecards") return { mode: "scorecard" };
    return { mode: "scorecard", name: decodeURIComponent(h), bare: true };
  }

  function missingTarget(D, route) {
    if (route.bare && !(D.scorecards || {})[route.name]) return "The link “#" + route.name + "” doesn't match anything in snapshot " + D.snapshot + ".";
    if (route.mode === "scorecard" && route.name && !(D.scorecards || {})[route.name]) return "No scorecard is named “" + route.name + "”.";
    if (route.mode === "tracks" && route.name && !(D.tracks || {})[route.name]) return "No track is named “" + route.name + "”.";
    if (route.mode === "rule") {
      var rsc = (D.scorecards || {})[route.scorecard];
      if (!rsc) return "No scorecard is named “" + route.scorecard + "”.";
      if (!(rsc.rules || {})[route.rule]) return title(D, route.scorecard) + " has no rule named “" + route.rule + "”.";
    }
    if (route.mode === "campaign" && !(D.campaigns || {})[route.name]) return "No campaign is named “" + route.name + "”.";
    if (route.mode === "compare" && route.name && !(D.comparisons || {})[route.name]) return "No comparison is named “" + route.name + "”.";
    if (route.mode === "catalog" && route.kind && !(D.catalog || {})[route.kind]) return "No kind is named “" + route.kind + "”.";
    if (route.mode === "entity") {
      var known = Object.values(D.catalog || {}).some(function (ids) {
        return ids.indexOf(route.id) >= 0;
      });
      if (!known) return "No entity has the id “" + route.id + "” in this snapshot.";
    }
    return null;
  }

  function notFound(message) {
    return el(
      "section",
      {},
      el("h2", { textContent: "Not found" }),
      el("p", { className: "lede", textContent: message + " The link may be from an older report." }),
      el("p", {}, link("overview", "", "Go to the overview"), " · ", link("catalog", "", "Browse the catalog")),
    );
  }

  class ToolboxReport extends HTMLElement {
    connectedCallback() {
      if (this._booted) return;
      this._booted = true;
      var root = this.attachShadow({ mode: "open" });
      root.append(template.content.cloneNode(true));
      // A button, not a link: the hash is the route, and the target is inside the shadow root.
      root.getElementById("skip").addEventListener("click", function () {
        var body = root.getElementById("body");
        body.tabIndex = -1;
        body.focus();
      });
      var self = this;
      var inline = this.querySelector('script[type="application/json"]');
      var load = inline
        ? Promise.resolve(JSON.parse(inline.textContent))
        : fetch(this.getAttribute("src")).then(function (r) {
            if (!r.ok) throw new Error("report JSON HTTP " + r.status);
            return r.json();
          });
      load
        .then(function (data) {
          self.mount(root, data);
        })
        .catch(function (e) {
          root.getElementById("body").replaceChildren(el("section", {}, el("div", { className: "empty", textContent: "Failed to load report: " + e.message })));
        });
    }

    mount(root, D) {
      var self = this;
      aliasLevels(D);
      var S = summarize(D);
      root.getElementById("top").replaceChildren(boxLogo(), el("h1", { textContent: D.portal || "Toolbox report" }));
      root.getElementById("meta").textContent =
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
        (Object.keys(D.comparisons || {}).length ? " · " + plural(Object.keys(D.comparisons).length, "comparison factor") : "") +
        " · snapshot " +
        D.snapshot +
        " · generated by toolbox report";
      // Each history entry keeps its scroll position, so Back returns to the same place in a long list.
      var settle;
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

    render(root, D, S) {
      var route = parseHash();
      var modes = root.getElementById("modes");
      var body = root.getElementById("body");
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
      var activeMode = route.bare && !(D.scorecards || {})[route.name] ? null : modeOf(route);
      var campaigns = campaignNamesSorted(D);
      var tabs = [
        { label: "Overview", path: "overview", id: "overview" },
      ];
      var trackCount = tracksSorted(D, S).length;
      if (trackCount) tabs.push({ label: "Tracks", path: "tracks", id: "tracks", count: trackCount });
      tabs.push(
        { label: "Catalog", path: "catalog", id: "catalog", count: S.entityCount },
        { label: "Campaigns", path: "campaigns", id: "campaigns", count: campaigns.length },
        {
          label: "Scorecards",
          id: "scorecards",
          count: S.rows.length,
          path: S.rows[0] ? "scorecard/" + enc(S.rows[0].name) : "overview",
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
          if (levelCards[0]) t.path = "scorecard/" + enc(levelCards[0].name);
        }
      });
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
        var cur = modes.querySelector('[aria-selected="true"]');
        if (cur && modes.scrollWidth > modes.clientWidth) modes.scrollLeft = Math.max(0, cur.offsetLeft - modes.clientWidth / 3);
        var fade = function () {
          modes.classList.toggle("more", modes.scrollLeft + modes.clientWidth < modes.scrollWidth - 4);
          modes.classList.toggle("less", modes.scrollLeft > 4);
        };
        fade();
        modes.onscroll = fade;
      });
      modes.onkeydown = function (ev) {
        var keys = { ArrowLeft: -1, ArrowRight: 1, Home: "home", End: "end" };
        var dir = keys[ev.key];
        if (dir == null) return;
        ev.preventDefault();
        var buttons = Array.prototype.slice.call(modes.querySelectorAll('[role="tab"]'));
        var i = buttons.indexOf(root.activeElement);
        if (i < 0)
          i = buttons.findIndex(function (b) {
            return b.getAttribute("aria-selected") === "true";
          });
        var next = dir === "home" ? 0 : dir === "end" ? buttons.length - 1 : (i + dir + buttons.length) % buttons.length;
        self._tabFocus = true;
        buttons[next].click();
      };
      var view;
      var missing = missingTarget(D, route);
      if (missing) view = notFound(missing);
      else if (route.mode === "overview") view = this.overview(D, S);
      else if (route.mode === "catalog") view = this.catalog(D, S, route.kind, route.query);
      else if (route.mode === "entity") view = this.entity(D, S, route.id);
      else if (route.mode === "campaigns" || route.mode === "campaign") view = this.campaigns(D, S, route.name);
      else if (route.mode === "compare") view = this.compare(D, route.name);
      else if (route.mode === "tracks") view = this.tracks(D, S, route.name);
      else if (route.mode === "rule") view = this.rule(D, S, route.scorecard, route.rule);
      else view = this.scorecard(D, S, route.name, route.query);
      view.classList.add("view");
      body.replaceChildren(view);
      var h2s = view.querySelectorAll("h2");
      var named = view.querySelector("[data-focus] h2") || (route.name || route.kind ? h2s[1] : null) || h2s[0];
      document.title = (named && route.mode !== "overview" ? named.textContent + " · " : "") + (D.portal || "Toolbox report");
      var target = view.querySelector("[data-focus]");
      if (this._tabFocus) {
        this._tabFocus = false;
        var sel = modes.querySelector('[aria-selected="true"]');
        if (sel) sel.focus();
      } else if (this._lastMode !== undefined && this._lastRoute !== location.hash) {
        // A focus target near the top opens the page from its header, so the portal's name stays in sight.
        var hostTop = root.host.getBoundingClientRect().top + window.scrollY;
        var targetTop = target ? target.getBoundingClientRect().top + window.scrollY : hostTop;
        window.scrollTo(0, targetTop - hostTop < window.innerHeight * 0.6 ? Math.max(0, hostTop) : targetTop - 8);
        var heading = (target && target.querySelector("h2")) || named;
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

    hero(D, S, campaignNames) {
      var R = S.repo;
      var verdict = healthVerdict(D, S, campaignNames);
      if (!R) {
        return el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Fleet health" })), el("p", { className: "verdict", textContent: verdict }));
      }
      var level = R.floor;
      var section = el("section", { className: "hero", style: "--lvl:" + fill(level) + ";--lvl-ink:" + ink(level) });
      section.setAttribute("aria-label", "Fleet health");
      var holdingNames = R.holding.map(function (c) {
        return title(D, c.name);
      });
      var sentence =
        (D.portal || "This repository") +
        " is at " +
        level +
        ": the lowest level among " +
        plural(R.constituents.length, "repository standard") +
        ". " +
        (R.next ? plural(R.holding.length, "standard holds", "standards hold") + " it there." : "Every standard is at the top level.");
      var wins = R.ladder
        .slice()
        .reverse()
        .filter(function (l) {
          return l !== "none" && l !== level;
        })
        .map(function (l) {
          var n = R.constituents.filter(function (c) {
            return c.level === l;
          }).length;
          return n ? plural(n, "standard") + " at " + l : null;
        })
        .filter(Boolean);
      var capped = R.holding.filter(function (c) {
        return c.level === c.top;
      });
      var liftNames = R.holding
        .filter(function (c) {
          return c.level !== c.top;
        })
        .map(function (c) {
          return title(D, c.name);
        });
      var nextBox = !R.next
        ? el("div", { className: "next" }, el("b", { textContent: "Top of the ladder. " }), "Every repository standard is at its highest level.")
        : capped.length
          ? el(
              "div",
              { className: "next" },
              el("b", { textContent: "Capped at " + level + ": " }),
              capped
                .map(function (c) {
                  return title(D, c.name);
                })
                .join(", ") +
                (capped.length === 1 ? " has" : " have") +
                " no level above " +
                level +
                ", so the repository stays at " +
                level +
                " until " +
                (capped.length === 1 ? "that standard gains" : "those standards gain") +
                " a " +
                R.next +
                " level." +
                (liftNames.length ? " The rest of the work is in the checklist below." : ""),
            )
          : el("div", { className: "next" }, el("b", { textContent: "Next: " }), "reach ", badge(R.next), " by lifting " + plural(holdingNames.length, "standard") + " to " + R.next + ", listed below.");
      var climb = null;
      if (R.next) {
        var rank = function (l) {
          return R.ladder.indexOf(l);
        };
        var there = R.constituents.filter(function (c) {
          return rank(c.level) >= rank(R.next);
        }).length;
        var share = Math.round((100 * there) / Math.max(1, R.constituents.length));
        var bar = el("div", { className: "climb-bar" }, el("span", { style: "width:" + share + "%;background:" + ink(R.next) }));
        bar.setAttribute("role", "img");
        var blockedBy = capped.length
          ? capped
              .map(function (c) {
                return title(D, c.name);
              })
              .join(", ")
          : "";
        bar.setAttribute("aria-label", there + " of " + R.constituents.length + " repository standards at " + R.next + " or above" + (blockedBy ? "; blocked until " + blockedBy + " gains a " + R.next + " level" : ""));
        climb = el(
          "div",
          { className: "climb" + (blockedBy ? " blocked" : "") },
          el("div", { className: "climb-head" }, el("span", {}, "Toward ", badge(R.next)), el("span", { className: "sub", textContent: there + " of " + R.constituents.length + " standards there (" + share + "%)" })),
          blockedBy ? el("div", { className: "climb-track" }, bar, el("span", { className: "climb-lock", title: "No " + R.next + " level exists yet on " + blockedBy }, lockIcon(), " blocked")) : bar,          el(
            "ul",
            {},
            ...R.holding.map(function (c) {
              var stuck = c.level === c.top;
              return el("li", { className: stuck ? "stuck" : "" }, el("span", { className: "box", ariaHidden: "true" }), el("span", {}, link("scorecard/" + enc(c.name), "", title(D, c.name)), el("span", { className: "sub", textContent: stuck ? " needs a " + R.next + " level first" : " lift from " + c.level + " to " + R.next })));
            }),
          ),
        );
      }
      var left = el(
        "div",
        {},
        el(
          "div",
          { className: "badge-big" },
          medal(level, 72),
          el("div", {}, el("div", { className: "kicker", textContent: "Repository level" }), el("h2", { className: "levelword", textContent: level })),
        ),
        el("p", { className: "verdict", textContent: sentence }),
        wins.length ? el("p", { className: "wins" }, el("b", { textContent: "✓ Already: " }), wins.join(" · ")) : null,
        nextBox,
        climb,
        el("p", { className: "sub", style: "margin:12px 0 0", textContent: verdict }),
      );
      var ladder = el("div", { className: "ladder", style: "--steps:" + R.ladder.length });
      ladder.setAttribute("aria-label", "Repository standards by lowest level");
      R.ladder.forEach(function (l) {
        var here = R.constituents.filter(function (c) {
          return c.level === l;
        });
        var rung = el(
          "div",
          { className: "rung" + (l === level ? " here" : "") + (here.length ? "" : " vacant"), style: "--lvl:" + fill(l) },
          el("div", { className: "rh" }, medal(l, 22), l, el("span", { className: "n", textContent: String(here.length) })),
        );
        if (!here.length) rung.append(l === "none" ? el("div", { className: "empty good", textContent: "✓ No standard at none" }) : el("div", { className: "empty", textContent: "—" }));
        here.forEach(function (c) {
          var done = c.level === c.top;
          var caps = done && R.next && c.level === level;
          var a = link("scorecard/" + enc(c.name), caps ? "capped" : done ? "done" : "", [title(D, c.name), caps ? el("span", { className: "capmark", textContent: "▲ ceiling: the repository can't rise above it" }) : done ? el("span", { className: "topmark", textContent: "✓ top level" }) : null]);
          a.title = caps ? c.name + ": at its only top level, " + c.level + ", which holds the repository there" : done ? c.name + ": already at its top level, " + c.level : c.name;
          rung.append(a);
        });
        ladder.append(rung);
      });
      section.append(
        left,
        el(
          "div",
          {},
          el("h3", { style: "margin-top:0", textContent: "Where each repository standard stands" }),
          ladder,
          el(
            "p",
            { className: "sub", style: "margin:10px 0 0" },
            "Only these standards set the repository level. The other scorecards rate single entities, such as one rule or one skill, and are listed below." +
              (R.ceiling !== R.ladder[R.ladder.length - 1] && R.ceiling !== level
                ? " " +
                  plural(
                    R.constituents.filter(function (c) {
                      return c.top === R.ceiling;
                    }).length,
                    "standard tops",
                    "standards top",
                  ) +
                  " out at " +
                  R.ceiling +
                  ", so " +
                  R.ceiling +
                  " is the highest level the repository can reach."
                : ""),
          ),
        ),
      );
      return section;
    }

    tiles(D, S, campaignNames) {
      var campaigns = D.campaigns || {};
      var c = S.checks;
      var total = c.pass + c.fail + c.unknown;
      var pct = total ? Math.round((c.pass / total) * 100) : 0;
      var on = campaignNames.filter(function (n) {
        return campaignState(campaigns[n]) === "on";
      }).length;
      var debt = S.rows.filter(function (r) {
        return r.none > 0 && !r.compare;
      }).length;
      var scoredEntities = Object.keys(S.worst).length;
      var golden = Object.keys(S.worst).filter(function (id) {
        return S.worst[id] === "gold";
      }).length;
      var tile = function (path, cls, lead, big, text) {
        return link(path, "tile " + (cls || ""), [lead, el("div", {}, el("strong", { textContent: big }), el("span", { textContent: text }))]);
      };
      return el(
        "div",
        { className: "tiles" },
        tile("scorecards", "good", ring(pct, { size: 46, color: "var(--pass)", text: "", label: pct + "% of checks pass" }), pct + "%", fmt(c.pass) + " of " + fmt(total) + " checks pass" + (c.unknown ? " (" + fmt(c.unknown) + " unknown)" : "")),
        tile("catalog", "", medal("gold", 40), fmt(golden), "of " + fmt(scoredEntities) + " scored entities are gold on every scorecard"),
        tile("scorecards", debt ? "bad" : "good", medal(debt ? "none" : "gold", 40), String(debt), debt ? "scorecards have entities at none" : "scorecards with entities at none"),
        tile("campaigns", on === campaignNames.length ? "good" : "", ring(campaignNames.length ? Math.round((on / campaignNames.length) * 100) : null, { size: 46, color: "var(--pass)", text: "", label: on + " of " + campaignNames.length + " campaigns on target" }), on + " of " + campaignNames.length, "campaigns on target"),
      );
    }

    campaignCard(D, name, current) {
      var c = (D.campaigns || {})[name];
      var state = campaignState(c);
      var pct = c.percent == null ? 0 : c.percent;
      var target = c.targetPercent == null ? 100 : c.targetPercent;
      var days = daysLeft(c.end);
      var color = state === "on" ? "var(--pass)" : state === "empty" ? "var(--faint)" : "var(--muted)";
      var status =
        state === "empty"
          ? el("span", { className: "pill unknown", textContent: "can't be measured" })
          : state === "on"
            ? el("span", { className: "pill good", textContent: "on target" })
            : el("span", { className: "pill bad", textContent: (c.inScope || 0) - (c.meeting || 0) + " to go" });
      var a = link("campaign/" + enc(name), "card", [
        ring(state === "empty" ? null : pct, { color: color, target: target, label: state === "empty" ? "can't be measured" : pct + "% of " + target + "% target" }),
        el(
          "div",
          {},
          el("div", { className: "t", textContent: c.title || name }),
          el("div", { className: "s" }, status, days != null && days >= 0 && c.status !== "ended" ? " · " + plural(days, "day") + " left" : c.status === "ended" ? " · ended" : ""),
        ),
      ]);
      if (current) a.setAttribute("aria-current", "true");
      return a;
    }

    scorecardTable(D, rows, floors) {
      var wrap = el("div", { className: "rows" });
      wrap.append(el("div", { className: "rows-head" }, el("span", { textContent: "Standard" }), el("span", { className: "dist", textContent: "Entities by level" }), el("span", { textContent: "At none" })));
      rows.forEach(function (r) {
        wrap.append(
          link("scorecard/" + enc(r.name), "row", [
            floors
              ? el("div", { style: "display:flex;gap:10px;align-items:center" }, medal(floorOf(D, r.name) || "none", 30), el("div", {}, ...cardName(D, r.name), el("span", { className: "weakest" }, "weakest ", floorOf(D, r.name) ? badge(floorOf(D, r.name)) : "not scored", topNote(D, r.name, floorOf(D, r.name)))))
              : el("div", {}, ...cardName(D, r.name)),
            el("div", { className: "dist" }, barEl(splitUnassessed(D, r.name, r.parts)), el("div", { className: "sub", style: "font-size:12px;margin-top:4px", textContent: (r.sc.kind ? kindCount(D, r.sc.kind, r.total) + scorecardSplit(D, r.sc.kind) : r.total + " entities") + (r.total > r.scored ? ", " + fmt(r.scored) + " in scope" : "") + " · " + r.topShare + "% " + (r.top || "top") }, provisionalOf(D, r.name) ? el("span", { className: "provisional", title: "Their levels hold only if the checks not run in this build pass", textContent: " · " + fmt(provisionalOf(D, r.name)) + " provisional (checks not run)" }) : null, unassessedOf(D, r.name) ? el("span", { className: "unjudged-n", title: "No agent has assessed them yet, so their agent-judged rules pass or fail for want of anything to read; drawn hollow in the bar", textContent: " · " + fmt(unassessedOf(D, r.name)) + " not assessed (✦ agent-judged)" }) : null)),
            el("div", { className: "num" + (r.none ? " bad" : "") }, el("strong", { textContent: String(r.none) }), el("span", { textContent: "at none" })),
          ]),
        );
      });
      return wrap;
    }

    overview(D, S) {
      var catalog = D.catalog || {};
      var campaignNames = campaignNamesSorted(D);
      var self = this;
      var attention = S.rows.filter(function (r) {
        return r.none > 0 && !r.compare;
      });
      var clear = S.rows.filter(function (r) {
        return r.none === 0 && !r.compare;
      });
      var showAll = false;
      var clearWrap = el("div");
      var drawClear = function () {
        clearWrap.replaceChildren(
          showAll
            ? el(
                "div",
                {},
                el(
                  "p",
                  { className: "empty good" },
                  "✓ " + plural(clear.length, "scorecard has", "scorecards have") + " no entity at none. ",
                  el("button", {
                    className: "link",
                    textContent: "Hide them",
                    ariaExpanded: "true",
                    onclick: function () {
                      showAll = false;
                      drawClear();
                      var b = clearWrap.querySelector("button");
                      if (b) b.focus();
                    },
                  }),
                ),
                self.scorecardTable(D, clear),
              )
            : el(
                "p",
                { className: "empty good" },
                "✓ " + plural(clear.length, "scorecard has", "scorecards have") + " no entity at none. ",
                el("button", {
                  className: "link",
                  textContent: "Show them",
                  ariaExpanded: "false",
                  onclick: function () {
                    showAll = true;
                    drawClear();
                    var b = clearWrap.querySelector("button");
                    if (b) b.focus();
                  },
                }),
              ),
        );
      };
      drawClear();
      return el(
        "div",
        {},
        this.hero(D, S, campaignNames),
        this.tiles(D, S, campaignNames),
        tracksSorted(D, S).length
          ? el(
              "section",
              {},
              el("div", { className: "head" }, el("h2", { textContent: "Tracks" }), link("tracks", "", "All tracks →")),
              el("p", { className: "lede", textContent: "Long-running lines of work, furthest from its goal first: the ring is the share of entities at the top level, averaged over the track's scorecards." + (tracksSorted(D, S).some(function (t) {
                  return t.compare;
                })
                  ? " A comparison track comes last, and its ring counts factors ahead of a rival instead."
                  : "") + " Open a track for its scorecards and campaigns." }),
              this.trackStrip(D, S, tracksSorted(D, S), "Tracks", true),
            )
          : null,
        campaignNames.length
          ? el(
              "section",
              {},
              el("div", { className: "head" }, el("h2", { textContent: "Campaigns" }), el("span", { className: "sub", textContent: "Time-bound goals, behind target first; a ring turns green once it reaches its target" })),
              el(
                "div",
                { className: "cards" },
                ...campaignNames.map(function (n) {
                  return self.campaignCard(D, n);
                }),
              ),
            )
          : null,
        el(
          "section",
          {},
          el(
            "div",
            { className: "head" },
            el("h2", { textContent: attention.length ? "Scorecards that need attention" : "Scorecards" }),
            el("span", { className: "sub", textContent: "Needs attention first" }),
          ),
          el("p", { className: "lede", textContent: "Each scorecard rates every entity of one kind against levels of rules. An entity is at none until it passes every rule of the first level." }),
          levelLegend(D, S.rows),
          attention.length ? this.scorecardTable(D, attention) : null,
          clearWrap,
        ),
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "What's in the box" }), link("catalog", "", "Browse the catalog →")),
          ...kindGroupsOf(D).map(function (g) {
            return el(
              "div",
              { className: "boxrow" },
              el("div", { className: "bg" }, g.title, el("small", { textContent: plural(g.count, "entity", "entities") })),
              el(
                "div",
                { className: "box" },
                ...g.kinds.map(function (k) {
                  var a = link("catalog/" + enc(k), "", [kindTitle(D, k), el("b", { textContent: fmt(catalog[k].length) })]);
                  if (kindDescription(D, k)) a.title = kindDescription(D, k).replace(/`/g, "");
                  return a;
                }),
              ),
            );
          }),
        ),
      );
    }

    trackCard(D, S, t, flag) {
      var def = t.def;
      var color = trackColor(t);
      var kids = [
        flag ? el("span", { className: "flag", textContent: flag }) : null,
        el(
          "div",
          { className: "th" },
          t.compare ? tallyRing(t.verdicts, t.rows.length, 60) : ring(t.pct, { size: 60, color: color, label: t.pct == null ? "not scored" : t.pct + "% at the top level" }),
          el("div", {}, el("div", { className: "tt", textContent: def.title || t.name }), el("div", { className: "sub", textContent: t.compare ? "factors ahead, a segment per factor" : t.averaged > 1 ? "of entities at the top level, averaged over " + plural(t.averaged, "scorecard") : "of entities at the top level" })),
        ),
        def.description ? el("div", { className: "td", textContent: prose(firstSentence(def.description)) }) : null,
      ];
      if (t.compare) {
        kids.push(
          el(
            "div",
            { className: "chips" },
            ...["ahead", "parity", "behind", "unknown"]
              .filter(function (v) {
                return t.verdicts[v];
              })
              .map(function (v) {
                return standingChip(v, t.verdicts[v] + " " + v);
              }),
          ),
        );
      } else if (t.cards.length) {
        var MAX = 5;
        var medals = el(
          "ul",
          { className: "medals" },
          ...t.cards.slice(0, MAX).map(function (c) {
            return el("li", { title: "Lowest entity at " + (c.floor || "not scored") }, medal(c.floor || "none", 18), el("span", { textContent: title(D, c.row.name) }), judgedMark(D, c.row.name), el("span", { className: "sr", textContent: ", lowest level " + (c.floor || "not scored") }));
          }),
          t.cards.length > MAX ? el("li", { className: "more", textContent: "+" + (t.cards.length - MAX) + " more on the track page" }) : null,
        );
        kids.push(medals);
      }
      var weakest = !t.compare && t.cards[0] && t.cards[0].floor ? title(D, t.cards[0].row.name) + " at " + t.cards[0].floor + (capsRepo(D, t.cards[0].row.name, t.cards[0].floor) ? ", its top level, which caps the repository" : isTop(D, t.cards[0].row.name, t.cards[0].floor) ? ", its top level" : "") : null;
      kids.push(
        el(
          "div",
          { className: "foot" },
          el("span", { textContent: plural(t.rows.length, t.compare ? "factor" : "scorecard") + (t.campaigns.length ? " · " + plural(t.campaigns.length, "campaign") : "") }),
          el("span", { textContent: t.compare ? (t.behind.length ? "behind on " + t.behind.join(", ") : "never behind") : weakest ? "weakest: " + weakest : "" }),
        ),
      );
      var a = link("track/" + enc(t.name), "track", kids);
      a.style.setProperty("--tc", color);
      var standing = t.compare ? tallyWords(t.verdicts, t.rows.length) : t.pct == null ? "not scored" : t.pct + "% of entities at the top level" + (t.averaged > 1 ? ", averaged over " + plural(t.averaged, "scorecard") : "");
      a.setAttribute("aria-label", (def.title || t.name) + ": " + standing + (flag ? ". " + flag : "") + (weakest ? ". Weakest: " + weakest : ""));
      return a;
    }

    trackGrid(D, S, shown) {
      var self = this;
      var furthest = furthestTrack(D, S);
      return el(
        "div",
        { className: "tracks" },
        ...shown.map(function (t) {
          return self.trackCard(D, S, t, t.name === furthest ? "Furthest from goal" : null);
        }),
      );
    }

    tracks(D, S, name) {
      if (name) return this.track(D, S, name);
      var all = tracksSorted(D, S);
      if (!all.length) return el("section", {}, el("div", { className: "empty", textContent: "This portal declares no tracks." }));
      var loose = S.rows.filter(function (r) {
        return !r.sc.track || !(D.tracks || {})[r.sc.track];
      });
      return el(
        "div",
        {},
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Tracks" }), el("span", { className: "sub", textContent: "Furthest from its goal first" })),
          el("p", { className: "lede", textContent: "A track is a long-running line of work: the scorecards that measure it and the campaigns that push on it. Its goal is every entity at the top level; each medal is one scorecard at its weakest entity's level." }),
          trackKey(all),
          this.trackGrid(
            D,
            S,
            all.filter(function (t) {
              return !t.compare;
            }),
          ),
        ),
        all.some(function (t) {
          return t.compare;
        })
          ? el(
              "section",
              {},
              el("div", { className: "head" }, el("h2", { textContent: "Measured against a rival" })),
              el("p", { className: "lede", textContent: "A comparison track levels nothing: its ring counts the factors on which the product leads its rival, so it sits apart from the tracks above and has no place in their order." }),
              this.trackGrid(
                D,
                S,
                all.filter(function (t) {
                  return t.compare;
                }),
              ),
            )
          : null,
        loose.length
          ? el(
              "section",
              {},
              el("div", { className: "head" }, el("h2", { textContent: "Not on a track" })),
              levelLegend(D, loose),
              this.scorecardTable(D, loose),
            )
          : null,
      );
    }

    track(D, S, name) {
      var t = trackStats(D, S, name);
      var def = t.def;
      var self = this;
      var color = trackColor(t);
      var owner = def.owner ? def.owner.replace(/^group:/, "") : "";
      var weakest = t.cards[0];
      var goal = t.compare
        ? el("p", {}, el("b", { textContent: tallyWords(t.verdicts, t.rows.length).replace(/^a/, "A") + ". " }), link("compare", "", "Open the comparison →"))
        : el(
            "p",
            {},
            el("b", { textContent: "Goal: every entity at the top level of each scorecard below. " }),
            t.pct == null ? "Nothing is scored yet." : "On average " + t.pct + "% are there" + (t.none ? "; " + plural(t.none, "entity is", "entities are") + " still at none" : "") + ".",
            weakest && weakest.floor ? el("span", {}, " The weakest scorecard is ", link("scorecard/" + enc(weakest.row.name), "", title(D, weakest.row.name)), ", at ", badge(weakest.floor), capsRepo(D, weakest.row.name, weakest.floor) ? ", its only top level, so it caps the repository there until it gains a higher one." : isTop(D, weakest.row.name, weakest.floor) ? ", already its top level." : ".") : null,
          );
      var section = el(
        "section",
        {},
        el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("tracks", "", "Tracks"), " / " + (def.title || name)),
        el(
          "div",
          { className: "ehead" },
          el("div", {}, el("h2", { textContent: def.title || name, tabIndex: -1 }), el("div", { className: "sub", textContent: [owner ? "owned by " + owner : null, plural(t.rows.length, t.compare ? "factor" : "scorecard"), t.campaigns.length ? plural(t.campaigns.length, "campaign") : null].filter(Boolean).join(" · ") })),
          t.compare ? tallyRing(t.verdicts, t.rows.length, 88) : ring(t.pct, { size: 88, color: color, label: t.pct == null ? "not scored" : t.pct + "% at the top level" }),
        ),
        def.description ? el("p", { className: "lede", style: "max-width:80ch" }, ...rich(def.description)) : null,
        refs(def.links),
        goal,
      );
      section.dataset.focus = "";
      var parts = [section];
      if (t.compare) {
        parts.push(el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Factors" })), this.factorCards(D, t.rows.map(function (r) {
          return r.name;
        }))));
      } else if (t.rows.length) {
        var rows = t.cards.map(function (c) {
          return c.row;
        });
        parts.push(
          el(
            "section",
            {},
            el("div", { className: "head" }, el("h2", { textContent: "Scorecards" }), el("span", { className: "sub", textContent: "Weakest first" })),
            levelLegend(D, rows),
            this.scorecardTable(D, rows, true),
          ),
        );
      }
      if (t.campaigns.length) {
        parts.push(
          el(
            "section",
            {},
            el("div", { className: "head" }, el("h2", { textContent: "Campaigns on this track" }), el("span", { className: "sub", textContent: "Time-bound pushes toward the goal" })),
            el(
              "div",
              { className: "cards" },
              ...t.campaigns.map(function (n) {
                return self.campaignCard(D, n);
              }),
            ),
          ),
        );
      }
      var others = tracksSorted(D, S).filter(function (o) {
        return o.name !== name;
      });
      if (others.length) parts.push(this.trackStrip(D, S, others, "Other tracks"));
      return el("div", {}, ...parts);
    }

    // Tracks as a row of small rings; `lead` drops the separator and names each standing.
    trackStrip(D, S, shown, label, lead) {
      var furthest = furthestTrack(D, S);
      return el(
        "nav",
        { className: "trackstrip" + (lead ? " lead" : ""), ariaLabel: label },
        lead ? null : el("span", { className: "sub", textContent: label }),
        ...shown.map(function (o) {
          var standing = o.compare ? tallyWords(o.verdicts, o.rows.length) : o.pct == null ? "not scored" : o.pct + "% at the top level";
          var flag = lead && o.name === furthest;
          var a = link("track/" + enc(o.name), flag ? "far" : "", [
            o.compare ? tallyRing(o.verdicts, o.rows.length, 30, true) : ring(o.pct, { size: 30, color: trackColor(o), label: standing, bare: true }),
            el("span", {}, el("span", { textContent: o.def.title || o.name }), lead ? el("small", { textContent: (o.compare ? standing : o.pct == null ? "not scored" : o.pct + "%") + (flag ? " · furthest from goal" : "") }) : null),
          ]);
          a.title = standing;
          a.setAttribute("aria-label", (o.def.title || o.name) + ": " + standing + (flag ? ". Furthest from goal" : ""));
          return a;
        }),
      );
    }

    campaigns(D, S, name) {
      var all = D.campaigns || {};
      var names = campaignNamesSorted(D);
      if (!names.length) names = Object.keys(all).sort();
      if (!names.length) return el("section", {}, el("div", { className: "empty", textContent: "No campaigns in this portal." }));
      var self = this;
      var selected = name && all[name] ? name : names[0];
      var c = all[selected];
      var state = campaignState(c);
      var pct = c.percent == null ? 0 : c.percent;
      var target = c.targetPercent == null ? 100 : c.targetPercent;
      var days = daysLeft(c.end);
      var includes = el("ul", { style: "margin:4px 0 0;padding-left:18px" });
      (c.includes || []).forEach(function (i) {
        includes.append(
          el(
            "li",
            {},
            link("scorecard/" + enc(i.scorecard), "", title(D, i.scorecard)),
            i.atLeast ? el("span", {}, " at ", badge(i.atLeast), " or above") : "",
            i.rules && i.rules.length ? el("span", { className: "sub", textContent: " · rules " + i.rules.join(", ") }) : "",
          ),
        );
      });
      var bar = el("div", { className: "bar lg" });
      var fillSpan = el("span", { style: "flex-grow:0;flex-basis:0;background:" + (state === "on" ? "var(--pass)" : "var(--muted)") });
      bar.append(fillSpan, el("span", { style: "flex-grow:" + Math.max(0, 100 - pct) + ";background:transparent" }));
      grow([fillSpan], [pct]);
      var marks = el("div", { className: "milestones" });
      (c.milestones || []).forEach(function (m) {
        marks.append(el("span", { className: m.met ? "met" : "", style: "left:" + m.targetPercent + "%", title: "Milestone “" + m.name + "”: " + m.targetPercent + "% of entities meeting the campaign" + (m.met ? ", reached" : ", not reached yet"), textContent: (m.met ? "✓ " : "") + m.name + " " + m.targetPercent + "%" + (m.met ? " reached" : "") }));
      });
      if (target < 100) marks.append(el("span", { style: "left:" + target + "%", textContent: "target " + target + "%" }));
      var gaps = Object.entries(c.entities || {})
        .filter(function (pair) {
          return !pair[1].meeting;
        })
        .sort(function (a, b) {
          return a[0].localeCompare(b[0]);
        });
      var gapList = el("div");
      if (state === "empty") {
        var kinds = (c.includes || []).map(function (i) {
          return ((D.scorecards || {})[i.scorecard] || {}).kind;
        });
        gapList.append(
          el(
            "div",
            { className: "next", style: "margin:0 0 10px" },
            el("b", { textContent: "This campaign can't be measured. " }),
            "Its scope is the entities rated by every scorecard it includes, and no entity is rated by all of them" +
              (kinds.length > 1 && kinds.some(function (k) { return k !== kinds[0]; }) ? " (they rate " + kinds.filter(Boolean).join(" and ") + ")" : "") +
              ". The work it tracks is still open:",
          ),
          ...(c.includes || []).map(function (i) {
            var row = S.rows.find(function (r) {
              return r.name === i.scorecard;
            });
            var order = row ? levelOrder(row.sc.levels) : [];
            var goal = i.atLeast ? order.indexOf(i.atLeast) : order.length - 2;
            var below = Object.values((D.scores || {})[i.scorecard] || {}).filter(function (sc) {
              return sc.level !== "skipped" && order.indexOf(sc.level) < goal;
            }).length;
            return el(
              "div",
              { className: "gap" },
              link("scorecard/" + enc(i.scorecard), "", title(D, i.scorecard)),
              el("div", { className: "why", textContent: row ? plural(below, "entity", "entities") + " below " + (order[goal] || "the goal") + " of " + (row.sc.kind ? kindCount(D, row.sc.kind, row.scored) : fmt(row.scored)) : "not in this report" }),
            );
          }),
          el("p", { className: "sub", textContent: "To measure it, include scorecards that rate the same kind." }),
        );
      } else if (!gaps.length) {
        gapList.append(el("div", { className: "empty good", textContent: "✓ Every in-scope entity meets the goal." }));
      } else {
        var needCounts = {};
        var needFix = {};
        gaps.forEach(function (pair) {
          (pair[1].gaps || []).forEach(function (g) {
            var m = /^(.+?):atLeast (\S+) \((\S+)\)$/.exec(g);
            var gsc = m && (D.scorecards || {})[m[1]];
            var score = m && ((D.scores || {})[m[1]] || {})[pair[0]];
            if (gsc && score)
              blockers(gsc, score, m[2]).forEach(function (r) {
                needCounts[r] = (needCounts[r] || 0) + 1;
                needFix[r] = { sc: m[1], def: (gsc.rules || {})[r] || {} };
              });
          });
        });
        var common = Object.entries(needCounts).sort(function (a, b) {
          return b[1] - a[1] || a[0].localeCompare(b[0]);
        })[0];
        if (common)
          gapList.append(
            el(
              "div",
              { className: "next", style: "margin:0 0 10px" },
              el("b", { textContent: "Biggest lever: " }),
              ruleLink(needFix[common[0]].sc, common[0]),
              " ",
              basisBadge(basisOf(needFix[common[0]].def)),
              " blocks " + common[1] + " of " + gaps.length + (gaps.length === 1 ? " entity" : " entities") + ". ",
              ...rich(needFix[common[0]].def.fix || ""),
              needFix[common[0]].def.rationale ? el("span", { className: "why", style: "display:block;margin-top:4px" }, el("b", { textContent: "Why: " }), ...rich(needFix[common[0]].def.rationale)) : null,
            ),
          );
        gaps.forEach(function (pair) {
          gapList.append(
            el(
              "div",
              { className: "gap" },
              el("div", {}, link("entity/" + enc(pair[0]), "", shortId(pair[0])), el("div", { className: "sub", style: "font-size:12px", textContent: pair[0] })),
              el("div", { className: "why", textContent: (pair[1].gaps || []).map(function (g) { return gapText(D, g, pair[0]); }).join("; ") }),
            ),
          );
        });
      }
      var track = c.track && D.tracks && D.tracks[c.track] ? D.tracks[c.track].title || c.track : c.track;
      var picker = el(
        "select",
        {
          ariaLabel: "Campaign",
          onchange: function () {
            location.hash = "campaign/" + enc(picker.value);
          },
        },
        ...names.map(function (n) {
          var cn = all[n];
          return el("option", { value: n, selected: n === selected, textContent: (cn.title || n) + " (" + (campaignState(cn) === "empty" ? "can't be measured" : (cn.percent || 0) + "%") + ")" });
        }),
      );
      return el(
        "div",
        { className: "split" },
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Campaigns" })),
          el("p", { className: "sub", style: "margin-top:0", textContent: "Behind target first" }),
          el(
            "div",
            { className: "cards", style: "grid-template-columns:1fr" },
            ...names.map(function (n) {
              return self.campaignCard(D, n, n === selected);
            }),
          ),
        ),
        el(
          "section",
          {},
          el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Campaign" }), picker),
          el(
            "div",
            { className: "ehead" },
            el("div", {}, el("h2", { textContent: c.title || selected }), el("div", { className: "sub" }, c.status || "", track ? el("span", {}, " · track ", link("track/" + enc(c.track), "", track)) : null, c.owner ? " · owned by " + c.owner.replace(/^group:/, "") : null)),
            ring(state === "empty" ? null : pct, { size: 84, color: state === "on" ? "var(--pass)" : "var(--muted)", target: target, label: state === "empty" ? "can't be measured" : pct + "% of " + target + "% target" }),
          ),
          c.description ? el("p", { className: "lede" }, ...rich(c.description)) : null,
          state === "empty"
            ? null
            : el(
                "p",
                { style: "margin:8px 0 6px" },
                el("b", { textContent: (c.meeting || 0) + " of " + (c.inScope || 0) }),
                " entities meet the goal · target " + target + "%" + (c.end ? " by " + c.end : "") + (days != null && days >= 0 ? " (" + plural(days, "day") + " left)" : "") + ". The ring turns green once it reaches the target.",
              ),
          state === "empty" ? null : bar,
          state === "empty" ? null : marks,
          el("h3", { textContent: "Goal" }),
          includes,
          el("h3", { textContent: state === "empty" ? "Scope" : "Not meeting yet" + (gaps.length ? " (" + gaps.length + ")" : "") }),
          gapList,
        ),
      );
    }

    catalog(D, S, kind, query) {
      query = query || {};
      var catalog = D.catalog || {};
      var groups = kindGroupsOf(D);
      var kinds = [].concat.apply(
        [],
        groups.map(function (g) {
          return g.kinds;
        }),
      );
      if (!kinds.length) return el("section", {}, el("div", { className: "empty", textContent: "Catalog is empty in this report." }));
      var selected = kind && catalog[kind] ? kind : null;
      var scored = scoredKinds(D);
      var compared = {};
      Object.keys(D.comparisons || {}).forEach(function (n) {
        var k = ((D.scorecards || {})[n] || {}).kind;
        if (k) compared[k] = true;
      });
      var groupOf = {};
      groups.forEach(function (g) {
        g.kinds.forEach(function (k) {
          groupOf[k] = g;
        });
      });
      var tile = function (k) {
          var ids = catalog[k];
          var unscored = scored[k]
            ? ids.filter(function (id) {
                return !S.worst[id];
              }).length
            : 0;
          var a = link("catalog/" + enc(k), "kind", [
            el("strong", {}, kindTitle(D, k)),
            kindDescription(D, k) ? el("small", {}, ...rich(firstSentence(kindDescription(D, k)))) : null,
            el("span", { textContent: plural(ids.length, "entity", "entities") + scorecardSplit(D, k) + (scored[k] ? (unscored ? " · " + fmt(unscored) + " not scored" : "") : compared[k] ? " · compared" : " · not scored") }),
          ]);
          if (kindDescription(D, k)) a.title = kindDescription(D, k).replace(/`/g, "");
          if (scored[k]) {
            var parts = tallyLevels(
              ids.map(function (id) {
                return [id, { level: S.worst[id] || "unscored" }];
              }),
              ["none", "bronze", "silver", "gold", "unscored"],
            );
            if (parts.length) a.append(barEl(parts, "xs"));
          }
          if (k === selected) a.setAttribute("aria-current", "true");
          return a;
      };
      var grid = el(
        "div",
        {},
        ...groups.map(function (g) {
          return el(
            "div",
            { className: "kgroup" },
            el("div", { className: "kgh" }, el("h3", { textContent: g.title }), el("span", { className: "sub", textContent: plural(g.count, "entity", "entities") + " in " + plural(g.kinds.length, "kind") })),
            g.description ? el("p", { className: "sub kgd", textContent: g.description }) : null,
            el("div", { className: "kinds" }, ...g.kinds.map(tile)),
          );
        }),
      );
      var top = el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "Catalog" }), el("span", { className: "sub", textContent: fmt(S.entityCount) + " entities in " + kinds.length + " kinds and " + plural(groups.length, "group") })),
        el("p", { className: "lede", textContent: "Everything the portal derived from this repository, grouped by what it is about. The bar under a kind shows each entity's lowest level across the scorecards that rate it; pick a kind to list its entities." }),
        el(
          "div",
          { className: "legend", style: "margin:0 0 12px" },
          ...["none", "bronze", "silver", "gold", "unscored"].map(function (l) {
            return el("span", {}, el("span", { className: "dot", style: "background:" + fill(l) }), l === "unscored" ? "not scored" : l);
          }),
        ),
        grid,
      );
      if (!selected) return top;
      var search = el("input", { type: "search", placeholder: "Filter " + selected + " by name…", ariaLabel: "Filter entities", value: query.q || "" });
      var list = el("div");
      var limit = 60;
      var drawList = function () {
        var q = search.value.toLowerCase();
        var ids = (catalog[selected] || []).filter(function (id) {
          return !q || id.toLowerCase().indexOf(q) >= 0;
        });
        if (!ids.length) {
          list.replaceChildren(
            el("div", { className: "empty", textContent: "No entities match “" + search.value + "”." }),
            el("button", {
              className: "link",
              textContent: "Clear filter",
              onclick: function () {
                search.value = "";
                search.oninput();
              },
            }),
          );
          return;
        }
        var shown = ids.slice(0, limit);
        var kindCards = Object.keys(D.scorecards || {})
          .filter(function (n) {
            return (catalog[selected] || []).some(function (id) {
              return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
            });
          })
          .sort(function (a, b) {
            return title(D, a).localeCompare(title(D, b));
          });
        var cols = kindCards.slice(0, 4);
        var allUnrun = {};
        cols.forEach(function (n) {
          var inScope = (catalog[selected] || []).filter(function (id) {
            return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
          });
          allUnrun[n] = inScope.length > 1 && inScope.every(function (id) {
            return unrunIn(D.scorecards[n], D.scores[n][id]).length > 0;
          });
        });
        var tpl = "minmax(0,1fr)" + (cols.length ? " repeat(" + cols.length + ",minmax(84px,128px))" : " auto");
        var header = cols.length
          ? el(
              "div",
              { className: "ehdr", style: "grid-template-columns:" + tpl, ariaHidden: "true" },
              el("span", { textContent: kindTitle(D, selected) }),
              ...cols.map(function (n) {
                var judged = basisCounts(((D.scorecards || {})[n] || {}).rules).judged > 0;
                return el("span", { title: n }, title(D, n), judged ? " " : null, judged ? basisBadge("judged", true) : null, allUnrun[n] ? el("span", { className: "unassessed-tag unrun", style: "display:block", title: "Every level in this column rests on checks not run in this build: it holds only if they pass" }, "all provisional: not run") : null);
              }),
            )
          : null;
        list.replaceChildren.apply(
          list,
          (header ? [header] : []).concat(
            shown.map(function (id) {
              var short = shortId(id);
              var r = link("entity/" + enc(id), "erow cols", [
                el("div", { className: "name" }, short, short !== id ? el("small", { textContent: id, title: id }) : null),
                ...(cols.length
                  ? cols.map(function (n) {
                      var sc = D.scores[n] && D.scores[n][id];
                      var lv = sc && sc.level !== "skipped" ? sc.level : null;
                      var cell = el(
                        "div",
                        { className: "ecell" },
                        lv ? levelPill(lv, unassessedIn(D, n, id), unrunIn(D.scorecards[n], sc), allUnrun[n]) : el("span", { className: "sub", title: "Not in this scorecard's scope", textContent: "not in scope" }),
                      );
                      cell.dataset.sc = title(D, n);
                      if (basisCounts(((D.scorecards || {})[n] || {}).rules).judged) cell.dataset.judged = "";
                      return cell;
                    })
                  : [el("div", { className: "badges" }, el("span", { className: "sub", textContent: "not scored" }))]),
              ]);
              r.style.gridTemplateColumns = tpl;
              var lvls = cols
                .filter(function (n) {
                  return D.scores[n] && D.scores[n][id] && D.scores[n][id].level !== "skipped";
                })
                .map(function (n) {
                  return title(D, n) + ": " + D.scores[n][id].level;
                });
              if (lvls.length) r.setAttribute("aria-label", short + ". " + lvls.join(", "));
              return r;
            }),
          ),
        );
        var judgedCols = cols.filter(function (n) {
          return basisCounts(((D.scorecards || {})[n] || {}).rules).judged > 0;
        });
        var unrunCols = cols.filter(function (n) {
          return allUnrun[n];
        });
        var dashed = (unrunCols.length ? " Every level on the " + unrunCols.map(function (n) { return title(D, n); }).join(" and ") + (unrunCols.length > 1 ? " scorecards" : " scorecard") + " is provisional here: checks it needs were not run in this build." : "") + (list.querySelector(".lvl.vacuous") ? " A level drawn hollow may not hold: a violet dashed outline marked “not assessed” means no agent's assessment has been read yet, and a dotted outline marked “provisional: not run” means a check it needs was not run in this build." : "");
        if (judgedCols.length || dashed)
          list.prepend(
            el(
              "p",
              { className: "sub basis-note" },
              judgedCols.length ? basisBadge("judged") : null,
              (judgedCols.length
                ? " " +
                  judgedCols
                    .map(function (n) {
                      return title(D, n);
                    })
                    .join(", ") +
                  " " +
                  (judgedCols.length === 1 ? "has" : "have") +
                  " agent-judged rules: an agent's assessment decides them."
                : "") + dashed,
            ),
          );
        if (kindCards.length > cols.length)
          list.append(el("p", { className: "sub", textContent: plural(kindCards.length - cols.length, "more scorecard") + " rate " + selected + "; open an entity to see them." }));
        if (ids.length > limit) {
          list.append(
            el(
              "div",
              { className: "more" },
              el("button", {
                className: "link",
                textContent: "Show all " + fmt(ids.length) + " (" + fmt(ids.length - limit) + " more)",
                onclick: function () {
                  limit = Infinity;
                  drawList();
                },
              }),
            ),
          );
        }
      };
      search.oninput = function () {
        var want = "#catalog/" + enc(selected) + (search.value ? "?" + new URLSearchParams({ q: search.value }) : "");
        if (location.hash !== want && history.replaceState) history.replaceState(history.state, "", want);
        drawList();
      };
      drawList();
      var levels = scored[selected]
        ? tallyLevels(
            (catalog[selected] || []).map(function (id) {
              return [id, { level: S.worst[id] || "unscored" }];
            }),
            ["none", "bronze", "silver", "gold", "unscored"],
          )
        : [];
      var nScored = (catalog[selected] || []).filter(function (id) {
        return S.worst[id];
      }).length;
      var nAll = (catalog[selected] || []).length;
      var listSection = el(
        "section",
        { id: "kind-list" },
        el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("catalog", "", "Catalog"), " / " + (groupOf[selected] && groupOf[selected].name ? groupOf[selected].title + " / " : "") + kindTitle(D, selected)),
        el("div", { className: "head" }, el("div", {}, el("h2", { textContent: kindTitle(D, selected), tabIndex: -1 }), kindTitle(D, selected) !== selected ? el("code", { className: "kid", textContent: selected }) : null), el("span", { className: "sub", textContent: (groupOf[selected] && groupOf[selected].name ? groupOf[selected].title + " · " : "") + plural(nAll, "entity", "entities") + scorecardSplit(D, selected) + (scored[selected] && nScored < nAll ? " · " + fmt(nScored) + " scored · " + fmt(nAll - nScored) + " not scored" : "") })),
        kindDescription(D, selected) ? el("p", { className: "lede" }, ...rich(kindDescription(D, selected))) : null,
        scored[selected] && nScored < nAll ? this.scopeNote(D, selected, nAll - nScored) : null,
        levels.length
          ? el("div", {}, barEl(levels), legendEl(levels))
          : compared[selected]
            ? el("p", { className: "sub" }, "Compared rather than levelled: ", link("compare", "", "see how they stand →"))
            : el("p", { className: "sub", textContent: "No scorecard rates this kind." }),
        search,
        list,
      );
      listSection.dataset.focus = "";
      // With a kind open, the kind grid folds to one chip row per group so the list starts near the top.
      var chips = el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "Catalog" }), el("span", { className: "sub" }, fmt(S.entityCount) + " entities in " + kinds.length + " kinds · ", link("catalog", "", "all kinds, with descriptions"))),
        ...groups.map(function (g) {
          return el(
            "div",
            { className: "kchips" },
            el("span", { className: "kchips-g", textContent: g.title }),
            el(
              "div",
              { className: "box" },
              ...g.kinds.map(function (k) {
                var a = link("catalog/" + enc(k), "", [kindTitle(D, k), el("b", { textContent: fmt((catalog[k] || []).length) })]);
                if (k === selected) a.setAttribute("aria-current", "true");
                return a;
              }),
            ),
          );
        }),
      );
      return el("div", {}, chips, listSection);
    }

    // Why some entities of a rated kind carry no level: each scorecard on the kind that limits itself with appliesTo.
    scopeNote(D, kind, n) {
      var scoped = Object.entries(D.scorecards || {}).filter(function (e) {
        return e[1].kind === kind && e[1].appliesTo;
      });
      var who = plural(n, "entity is", "entities are");
      if (!scoped.length) return el("p", { className: "sub", textContent: fmt(n) + " " + who.replace(/^\d+ /, "") + " not scored: no scorecard on this kind has produced a level for them in this build." });
      return el(
        "p",
        { className: "sub scope-note" },
        el("b", { textContent: who.replace(/^\d+/, fmt(n)) + " not scored" }),
        ": they fall outside the scope that ",
        ...scoped.flatMap(function (e, i) {
          return [i ? (i === scoped.length - 1 ? " and " : ", ") : "", link("scorecard/" + enc(e[0]), "", e[1].title || e[0])];
        }),
        " sets for itself. ",
        el(
          "details",
          { className: "scope-rule" },
          el("summary", { textContent: "Show the scope rule" }),
          ...scoped.map(function (e) {
            return el("div", {}, el("span", { textContent: (e[1].title || e[0]) + " applies where " }), el("code", { textContent: e[1].appliesTo }));
          }),
        ),
      );
    }

    rulesFor(sc, score, scName, unjudged) {
      var wrap = el("div");
      var rules = Object.entries(score.rules || {});
      var rank = function (o) {
        return o === "fail" ? 0 : o === "unknown" ? 1 : o === "waived" ? 2 : o === "skipped" ? 3 : 4;
      };
      var order = sc.levels || [];
      rules.sort(function (a, b) {
        var la = order.indexOf(((sc.rules || {})[a[0]] || {}).level);
        var lb = order.indexOf(((sc.rules || {})[b[0]] || {}).level);
        return rank(a[1]) - rank(b[1]) || la - lb || a[0].localeCompare(b[0]);
      });
      var passed = {};
      var npass = 0;
      var seen = {};
      rules.forEach(function (ro) {
        var n = ro[0];
        var o = ro[1];
        var rule = (sc.rules && sc.rules[n]) || {};
        if (o === "pass" && unjudged && basisOf(rule) === "judged") {
          wrap.append(
            el(
              "div",
              { className: "rule unassessed" },
              el(
                "div",
                { style: "display:flex;flex-wrap:wrap;gap:4px 8px;align-items:center" },
                el("h4", { className: "rn" }, scName ? ruleLink(scName, n) : n),
                el("span", { className: "outcome unassessed", style: "margin:0", textContent: "passes, not assessed yet" }),
                basisBadge("judged"),
                rule.level ? el("span", { className: "sub", textContent: "counts for " + rule.level }) : "",
              ),
              el("div", { className: "fix" }, "No agent has assessed this entity, so there is nothing for this rule to fail on: it passes because nothing has been looked at, not because a review found nothing."),
              whyEl(rule),
            ),
          );
          return;
        }
        if (o === "pass") {
          var lv = rule.level || "";
          (passed[lv] = passed[lv] || []).push(el("span", { className: basisOf(rule) === "judged" ? "judged" : "", title: rule.rationale || rule.fix || rule.check || "" }, scName ? ruleLink(scName, n) : n, " ", basisBadge(basisOf(rule), true)));
          npass++;
          return;
        }
        var ev = (score.evidence || {})[n];
        var items = ev == null ? [] : Array.isArray(ev) ? ev : [ev];
        wrap.append(
          el(
            "div",
            { className: "rule " + o },
            el(
              "div",
              { style: "display:flex;flex-wrap:wrap;gap:4px 8px;align-items:center" },
              el("h4", { className: "rn" }, scName ? ruleLink(scName, n) : n),
              el("span", { className: "outcome " + o, style: "margin:0", textContent: o === "unknown" ? "not run in this build" : o }),
              basisBadge(basisOf(rule)),
              rule.level ? el("span", { className: "sub", textContent: "needed for " + rule.level }) : "",
            ),
            rule.fix && o !== "unknown" ? el("div", { className: "fix" }, el("b", { textContent: "Fix: " }), ...rich(rule.fix)) : "",
            whyEl(rule),
            evidenceEl(items, seen, n),
          ),
        );
      });
      if (npass) {
        wrap.append(el("div", { className: "sub", style: "margin-top:10px", textContent: plural(npass, "check passes", "checks pass") }));
        order.concat([""]).forEach(function (lv) {
          if (!passed[lv]) return;
          var all = Object.keys(sc.rules || {}).filter(function (n) {
            return (sc.rules[n].level || "") === lv && score.rules[n] !== "skipped";
          }).length;
          wrap.append(
            el(
              "div",
              { className: "passes" },
              lv ? badge(lv) : null,
              el("em", { textContent: passed[lv].length === all ? "all " + all + " met" : passed[lv].length + " of " + all }),
              ...passed[lv],
            ),
          );
        });
      }
      return wrap;
    }

    nextStep(sc, score) {
      var order = sc.levels || [];
      var here = order.indexOf(score.level);
      var unsure = Object.keys(score.rules || {})
        .filter(function (n) {
          return score.rules[n] === "unknown" && order.indexOf(((sc.rules || {})[n] || {}).level) <= here;
        })
        .sort();
      var caveat = unsure.length
        ? el(
            "div",
            { className: "callout", style: "background:color-mix(in srgb,var(--unknown) 12%,var(--card))" },
            el("span", {}, el("b", { className: "provisional", textContent: "Provisional " + score.level + ". " }), plural(unsure.length, "check", "checks") + " this level needs " + (unsure.length === 1 ? "was" : "were") + " not run in this build (" + unsure.join(", ") + "). The level holds only if " + (unsure.length === 1 ? "it passes" : "they pass") + "."),
          )
        : null;
      var next = nextLevel(sc, score.level);
      if (!next) return el("div", {}, caveat, el("div", { className: "callout done" }, "✓ Top level reached: every rule passes."));
      var need = blockers(sc, score, next).filter(function (n) {
        return unsure.indexOf(n) < 0;
      });
      if (!need.length) return caveat;
      if (need.length === 1)
        return el("div", {}, caveat, el("div", { className: "callout near" }, el("span", {}, el("b", { textContent: "One rule from " }), badge(next), " · pass " + need[0] + " ", basisBadge(basisOf((sc.rules || {})[need[0]]), true), " and this entity moves up a level.")));
      return el("div", {}, caveat, el("div", { className: "callout" }, el("span", {}, el("b", { textContent: "Next: " }), badge(next), " needs: ", ...need.map(function (n, i) {
        return el("span", { className: "nr" }, (i ? ", " : "") + n + " ", basisBadge(basisOf((sc.rules || {})[n]), true));
      }), ".")));
    }

    entity(D, S, id) {
      var kind = S.kindOf[id];
      var self = this;
      var cards = Object.keys(D.scorecards || {}).filter(function (name) {
        return D.scores[name] && D.scores[name][id];
      });
      var total = { pass: 0, all: 0 };
      var bases = { measured: 0, judged: 0 };
      cards.forEach(function (n) {
        if (isComparative(D, n)) return;
        var rules = D.scores[n][id].rules || {};
        Object.keys(rules).forEach(function (r) {
          var o = rules[r];
          if (o === "skipped") return;
          total.all++;
          bases[basisOf((D.scorecards[n].rules || {})[r])]++;
          if (isPassing(o)) total.pass++;
        });
      });
      var judgedAt = [];
      var blocks = cards.map(function (name) {
        var sc = D.scorecards[name];
        var score = D.scores[name][id];
        if (isComparative(D, name)) {
          var c = D.comparisons[name];
          var peer = (c.peers || [])[0];
          var v = (c.verdict || {})[peer];
          var whoSubject = (c.titles || {})[c.subject] || shortId(c.subject);
          var whoPeer = (c.titles || {})[peer] || shortId(peer);
          return el(
            "div",
            { className: "sc-block" },
            el("div", { className: "head" }, el("div", {}, el("h3", { className: "sct" }, link("compare/" + enc(name), "", title(D, name))), " ", standingChip(v, whoSubject + " " + (v || "unknown"))), el("span", { className: "sub", textContent: "comparison · " + whoSubject + " vs " + whoPeer })),
          );
        }
        var here = { measured: 0, judged: 0 };
        var judgedHere = [];
        Object.keys(score.rules || {}).forEach(function (r) {
          var b = basisOf((sc.rules || {})[r]);
          here[b]++;
          if (b === "judged") judgedHere.push(r);
        });
        var block = el(
          "div",
          { className: "sc-block" },
          el(
            "div",
            { className: "head" },
            el("div", {}, el("h3", { className: "sct" }, link("scorecard/" + enc(name), "", title(D, name))), " ", score.level === "skipped" ? badge(score.level) : levelPill(score.level, unassessedIn(D, name, id), unrunIn(sc, score))),
            el("span", { className: "sub" }, name + " · ", here.measured ? el("span", {}, basisIcon("measured"), " " + here.measured + " measured") : null, here.measured && here.judged ? " · " : "", here.judged ? el("span", { className: "judged-n" }, basisIcon("judged"), " " + here.judged + " agent-judged") : null),
          ),
          score.level === "skipped" ? el("div", { className: "sub", textContent: "Not in this scorecard's scope." }) : self.nextStep(sc, score),
          self.rulesFor(sc, score, name, unassessedIn(D, name, id)),
        );
        if (judgedHere.length && score.level !== "skipped") judgedAt.push({ name: name, rules: judgedHere, block: block });
        return block;
      });
      var judgedList = judgedAt.length
        ? el(
            "p",
            { className: "sub judged-list" },
            el("b", { textContent: "Agent-judged here: " }),
            ...judgedAt.map(function (j, i) {
              return el(
                "span",
                {},
                i ? "; " : "",
                el("button", {
                  className: "link",
                  textContent: title(D, j.name),
                  onclick: function () {
                    j.block.scrollIntoView({ behavior: "smooth", block: "start" });
                    var h = j.block.querySelector("h3 a");
                    if (h) h.focus({ preventScroll: true });
                  },
                }),
                " (" + j.rules.join(", ") + ")",
              );
            }),
          )
        : null;
      var short = shortId(id);
      var notRun = 0;
      cards.forEach(function (n) {
        var sc = D.scorecards[n];
        var score = D.scores[n][id];
        if (isComparative(D, n) || score.level !== S.worst[id]) return;
        var here = (sc.levels || []).indexOf(score.level);
        notRun += Object.keys(score.rules || {}).filter(function (r) {
          return score.rules[r] === "unknown" && (sc.levels || []).indexOf(((sc.rules || {})[r] || {}).level) <= here;
        }).length;
      });
      var unassessedCards = cards.filter(function (n) {
        return !isComparative(D, n) && D.scores[n][id].level !== "skipped" && unassessedIn(D, n, id);
      }).length;
      var provisional = [notRun ? plural(notRun, "check") + " not run" : null, unassessedCards ? plural(unassessedCards, "agent-judged scorecard") + " not assessed" : null].filter(Boolean);
      return el(
        "section",
        {},
        el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("catalog", "", "Catalog"), kind ? el("span", {}, " / ", link("catalog/" + enc(kind), "", kindTitle(D, kind))) : "", " / " + short),
        el(
          "div",
          { className: "ehead" },
          el("div", {}, el("h2", { textContent: short }), short !== id ? el("code", { textContent: id }) : null, el("div", { className: "sub", textContent: (kind ? kindTitle(D, kind) : "unknown kind") + " · " + plural(cards.length, "scorecard") })),
          S.worst[id]
            ? el(
                "div",
                { className: "elevel" },
                medal(S.worst[id], 52),
                el(
                  "div",
                  {},
                  el("strong", { textContent: S.worst[id], style: "color:" + ink(S.worst[id]) }),
                  provisional.length ? el("span", { className: "provisional", style: "display:block;font-size:13px", title: "The level holds only if the checks not run in this build pass and an agent's assessment finds nothing; the scorecards below name them.", textContent: "Provisional: " + provisional.join(", ") }) : null,
                  el("span", { className: "sub", textContent: (cards.length > 1 ? "lowest of " + cards.length + " scorecards · " : "") + total.pass + " of " + total.all + " checks pass" }),
                ),
              )
            : null,
        ),
        total.all ? basisKey(bases, "check") : null,
        judgedList,
        cards.length ? el("div", {}, ...blocks) : el("div", { className: "empty", textContent: "Not scored by any scorecard in this snapshot." }),
      );
    }

    rule(D, S, scName, ruleName) {
      var sc = D.scorecards[scName];
      var def = sc.rules[ruleName];
      var b = basisOf(def);
      var scores = (D.scores || {})[scName] || {};
      var tally = { pass: 0, waived: 0, fail: 0, unknown: 0 };
      var failing = [];
      Object.keys(scores).forEach(function (id) {
        var row = scores[id];
        var o = (row.rules || {})[ruleName];
        if (!o || o === "skipped" || row.level === "skipped") return;
        tally[o] = (tally[o] || 0) + 1;
        if (!isPassing(o)) failing.push(id);
      });
      failing.sort(function (a, c) {
        return (scores[a].rules[ruleName] === "fail" ? 0 : 1) - (scores[c].rules[ruleName] === "fail" ? 0 : 1) || a.localeCompare(c);
      });
      var all = tally.pass + tally.waived + tally.fail + tally.unknown;
      var parts = [
        { k: "pass", n: tally.pass + tally.waived, c: "var(--pass)", t: "pass" },
        { k: "fail", n: tally.fail, c: "var(--fail)", t: "fail" },
        { k: "unknown", n: tally.unknown, c: UNSCORED, t: "not run in this build" },
      ].filter(function (p) {
        return p.n;
      });
      var spans = parts.map(function (p) {
        return el("span", { style: "flex-grow:0;flex-basis:0;background:" + p.c, title: p.n + " " + p.t });
      });
      grow(
        spans,
        parts.map(function (p) {
          return p.n;
        }),
      );
      var bar = el("div", { className: "bar lg" }, ...spans);
      bar.setAttribute("role", "img");
      bar.setAttribute(
        "aria-label",
        parts
          .map(function (p) {
            return p.n + " " + p.t;
          })
          .join(", "),
      );
      var list = el("div");
      var limit = 40;
      var drawList = function () {
        list.replaceChildren(
          ...failing.slice(0, limit).map(function (id) {
            var o = scores[id].rules[ruleName];
            var ev = (scores[id].evidence || {})[ruleName];
            var items = ev == null ? [] : Array.isArray(ev) ? ev : [ev];
            return link("entity/" + enc(id), "erow", [
              el("div", { className: "name" }, shortId(id), el("small", { textContent: items.length ? (typeof items[0] === "string" ? items[0] : JSON.stringify(items[0])) + (items.length > 1 ? " (+" + (items.length - 1) + ")" : "") : id })),
              el("span", { className: "outcome " + o, textContent: o === "unknown" ? "not run in this build" : o }),
            ]);
          }),
        );
        if (failing.length > limit)
          list.append(
            el(
              "div",
              { className: "more" },
              el("button", {
                className: "link",
                textContent: "Show all " + failing.length,
                onclick: function () {
                  limit = Infinity;
                  drawList();
                },
              }),
            ),
          );
      };
      drawList();
      var section = el(
        "section",
        {},
        el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("scorecards", "", "Scorecards"), " / ", link("scorecard/" + enc(scName), "", title(D, scName)), " / " + ruleName),
        el(
          "div",
          { className: "ehead" },
          el("div", {}, el("h2", { textContent: ruleName, tabIndex: -1 }), el("div", { className: "sub" }, "A rule of ", link("scorecard/" + enc(scName), "", title(D, scName)), def.level ? el("span", {}, " · needed for ", badge(def.level)) : null)),
          basisBadge(b),
        ),
        def.rationale || (def.links || []).length
          ? el(
              "div",
              { className: "rationale" },
              el("b", { textContent: "Why it exists" }),
              def.rationale ? el("div", {}, ...rich(def.rationale)) : null,
              refs(def.links),
              !(def.links || []).length && (sc.links || []).length ? el("div", { className: "sub", style: "margin-top:6px" }, "Read more, from the ", link("scorecard/" + enc(scName), "", title(D, scName)), " standard:") : null,
              !(def.links || []).length ? refs(sc.links) : null,
            )
          : el("p", { className: "sub", textContent: "This rule has no rationale yet." }),
        el("h3", { textContent: "How it is decided" }),
        el("p", { style: "margin:0" }, basisBadge(b), " ", BASIS[b].desc),
        el("div", { className: "cel", textContent: def.check }),
        def.fix ? el("h3", { textContent: "How to fix a failure" }) : null,
        def.fix ? el("p", { style: "margin:0" }, ...rich(def.fix)) : null,
        el("h3", { textContent: "Outcomes" }),
        all ? bar : el("p", { className: "empty", textContent: "No entity is in scope." }),
        all
          ? el(
              "div",
              { className: "legend" },
              ...parts.map(function (p) {
                return el("span", {}, el("span", { className: "dot", style: "background:" + p.c }), el("b", { textContent: fmt(p.n) }), " " + p.t);
              }),
            )
          : null,
      );
      section.dataset.focus = "";
      return el(
        "div",
        {},
        section,
        failing.length
          ? el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Not passing" }), el("span", { className: "sub", textContent: plural(failing.length, "entity", "entities") + ", failures first" })), list)
          : all
            ? el("section", {}, el("p", { className: "empty good", textContent: "✓ Every entity in scope passes this rule." }))
            : null,
      );
    }

    factorCards(D, names, current) {
      var all = D.comparisons || {};
      var order = { behind: 0, unknown: 1, parity: 2, ahead: 3 };
      var verdictOf = function (n) {
        var c = all[n];
        return (c.verdict || {})[(c.peers || [])[0]] || "unknown";
      };
      return el(
        "div",
        { className: "factors" },
        ...names
          .slice()
          .sort(function (a, b) {
            return order[verdictOf(a)] - order[verdictOf(b)] || title(D, a).localeCompare(title(D, b));
          })
          .map(function (n) {
            var c = all[n];
            var def = (D.scorecards || {})[n] || {};
            var peer = (c.peers || [])[0];
            var v = verdictOf(n);
            var counts = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
            Object.keys(c.rules || {}).forEach(function (r) {
              var x = (c.rules[r] || {})[peer] || "unknown";
              counts[x] = (counts[x] || 0) + 1;
            });
            var segs = ["ahead", "parity", "behind", "unknown"].filter(function (k) {
              return counts[k];
            });
            var spans = segs.map(function (k) {
              return el("span", { style: "flex-grow:0;flex-basis:0;background:" + standingColor(k) });
            });
            grow(
              spans,
              segs.map(function (k) {
                return counts[k];
              }),
            );
            var bar = el("div", { className: "bar xs" }, ...spans);
            bar.setAttribute("role", "img");
            var tallyText = segs
              .map(function (k) {
                return counts[k] + " " + k;
              })
              .join(" · ");
            bar.setAttribute("aria-label", "Rules: " + tallyText);
            // Every card leads with the same number, rules behind (else ahead, else at parity); its widest measured lead follows.
            var best = null;
            Object.keys(c.rules || {}).forEach(function (r) {
              var rd = (def.rules || {})[r] || {};
              if ((c.rules[r] || {})[peer] !== "ahead" || !rd.better) return;
              var x = ruleValue(c, r, c.subject);
              var y = ruleValue(c, r, peer);
              if (typeof x !== "number" || typeof y !== "number" || x <= 0 || y <= 0) return;
              var ratio = Math.max(x, y) / Math.min(x, y);
              if (!best || ratio > best.ratio) best = { ratio: ratio, r: r, better: rd.better };
            });
            var total = Object.keys(c.rules || {}).length;
            var fxKey = counts.behind ? "behind" : counts.ahead ? "ahead" : "parity";
            var fx = total
              ? el(
                  "div",
                  { className: "fx" },
                  el("b", { textContent: counts[fxKey] + " of " + total + " rules " + (fxKey === "parity" ? "at parity" : fxKey) }),
                  best ? el("span", { textContent: "widest lead: " + (best.ratio >= 10 ? Math.round(best.ratio).toLocaleString("en-US") : best.ratio.toFixed(1)) + "× " + best.better + " " + lowerFirst(measureLabel(c, best.r)) }) : null,
                )
              : null;
            var a = link("compare/" + enc(n), "factor", [
              el("div", { className: "fh" }, el("span", { textContent: def.title || n }), standingChip(v)),
              fx,
              def.rationale || def.description ? el("p", {}, ...rich(firstSentence(def.rationale || def.description))) : null,
              bar,
              el("div", { className: "fc", textContent: "Rules: " + tallyText }),
            ]);
            a.style.setProperty("--st", standingColor(v));
            if (n === current) a.setAttribute("aria-current", "true");
            return a;
          }),
      );
    }

    // One row per factor: the share of its rules behind grows left of the axis, ahead grows right; parity is the gap.
    headToHead(D, all, names, peer) {
      var rows = names
        .map(function (n) {
          var c = { ahead: 0, parity: 0, behind: 0, total: 0 };
          Object.keys(all[n].rules || {}).forEach(function (r) {
            var v = ((all[n].rules[r] || {})[peer]) || "unknown";
            if (v in c) c[v]++;
            c.total++;
          });
          return { n: n, c: c };
        })
        .sort(function (a, b) {
          return b.c.behind / Math.max(1, b.c.total) - a.c.behind / Math.max(1, a.c.total) || a.c.ahead / Math.max(1, a.c.total) - b.c.ahead / Math.max(1, b.c.total);
        });
      var pct = function (k, c) {
        return Math.round((100 * c[k]) / Math.max(1, c.total));
      };
      var box = el(
        "div",
        { className: "h2h" },
        el("div", { className: "h2h-head", ariaHidden: "true" }, el("span", {}), el("span", { className: "behind", textContent: STANDING_MARK.behind + " behind" }), el("span", { className: "ahead", textContent: "ahead " + STANDING_MARK.ahead })),
        ...rows.map(function (r) {
          var a = link("compare/" + enc(r.n), "h2h-row", [
            el("span", { className: "h2h-name", textContent: title(D, r.n) }),
            el("span", { className: "h2h-l" }, el("span", { style: "width:" + pct("behind", r.c) + "%" })),
            el("span", { className: "h2h-r" }, el("span", { style: "width:" + pct("ahead", r.c) + "%" })),
          ]);
          a.setAttribute("aria-label", title(D, r.n) + ": " + r.c.behind + " behind, " + r.c.parity + " at parity, " + r.c.ahead + " ahead, of " + r.c.total + " rules");
          a.title = r.c.behind + " behind · " + r.c.parity + " at parity · " + r.c.ahead + " ahead, of " + r.c.total + " rules";
          return a;
        }),
        el("div", { className: "h2h-foot", textContent: "Share of each factor's rules; the gap is parity" }),
      );
      return box;
    }

    compare(D, name) {
      var all = D.comparisons || {};
      var names = Object.keys(all).sort();
      if (!names.length) return el("section", {}, el("div", { className: "empty", textContent: "No comparative scorecards in this portal." }));
      var first = all[names[0]];
      var subject = first.subject;
      var peer = (first.peers || [])[0];
      var who = function (id) {
        for (var i = 0; i < names.length; i++) {
          var t = (all[names[i]].titles || {})[id];
          if (t) return t;
        }
        return shortId(id);
      };
      var verdictOf = function (n) {
        return ((all[n].verdict || {})[peer]) || "unknown";
      };
      var counts = { ahead: 0, parity: 0, behind: 0, unknown: 0 };
      names.forEach(function (n) {
        counts[verdictOf(n)]++;
      });
      if (name) return this.compareDetail(D, name, names, who);
      var behindNames = names.filter(function (n) {
        return verdictOf(n) === "behind";
      });
      var headline =
        "Ahead on " +
        counts.ahead +
        " of " +
        names.length +
        " factors" +
        (counts.parity ? ", at parity on " + counts.parity : "") +
        (counts.behind ? ", behind on " + counts.behind : "") +
        (counts.unknown ? ", " + counts.unknown + " not measured" : "") +
        ".";
      var lean = counts.behind > counts.ahead ? "behind" : counts.ahead > counts.behind ? "ahead" : "parity";
      var ruleCounts = { ahead: 0, parity: 0, behind: 0 };
      var behindRules = {};
      names.forEach(function (n) {
        var rs = all[n].rules || {};
        behindRules[n] = 0;
        Object.keys(rs).forEach(function (r) {
          var v = (rs[r] || {})[peer];
          if (v in ruleCounts) ruleCounts[v]++;
          if (v === "behind") behindRules[n]++;
        });
      });
      var hero = el(
        "section",
        { className: "cmp-hero" },
        el(
          "div",
          {},
          el("div", { className: "kicker", textContent: "Head to head" }),
          el("h2", { className: "big", textContent: who(subject) + " vs " + who(peer) }),
          el("p", { className: "verdict", style: "margin:0;font-size:17px" }, el("b", { textContent: headline }), behindNames.length ? el("span", {}, " Behind on ", ...behindNames.map(function (n, i) {
            return el("span", {}, i ? ", " : "", link("compare/" + enc(n), "", title(D, n)), " (" + behindRules[n] + " of " + Object.keys(all[n].rules || {}).length + " rules)");
          }), "; check it first if it matters to you.") : " Never behind."),
          el(
            "p",
            { className: "rule-tally" },
            "Rule by rule: ",
            el("b", { className: "ahead", textContent: ruleCounts.ahead + " ahead" }),
            " · ",
            el("b", { className: "parity", textContent: ruleCounts.parity + " at parity" }),
            " · ",
            el("b", { className: "behind", textContent: ruleCounts.behind + " behind" }),
            ". Factors are not weighted, so the factor count is a tally, not an overall score.",
          ),
          el(
            "p",
            { className: "sub", style: "margin:8px 0 0" },
            "A factor is a scorecard that scores both products on the same rules. On a rule, a product is ahead when its measured value is better (within the rule's tolerance is parity) or it passes a bar the other misses. " + who(subject) + " is behind on a factor if it is behind on any one of its rules, and ahead if it is ahead on some and behind on none.",
          ),
        ),
        el(
          "div",
          { className: "cmp-side" },
          el(
            "div",
            { className: "tally3" },
            ...["ahead", "parity", "behind"].map(function (k) {
              return el("div", {}, el("strong", { textContent: String(counts[k]), style: "color:" + standingColor(k) }), el("span", { textContent: STANDING_MARK[k] + " " + k }));
            }),
          ),
          this.headToHead(D, all, names, peer),
        ),
      );
      hero.style.setProperty("--st", standingColor(lean));
      hero.dataset.focus = "";

      // The setup: facts no rule reads (version, corpus, harness, runs) describe how it was measured.
      var read = {};
      var fields = {};
      names.forEach(function (n) {
        Object.values(all[n].reads || {}).forEach(function (fs) {
          (fs || []).forEach(function (f) {
            read[f] = true;
          });
        });
        Object.assign(fields, all[n].fields || {});
      });
      var facts = first.facts || {};
      var ids = [subject].concat(first.peers || []);
      var context = {};
      ids.forEach(function (id) {
        Object.keys(facts[id] || {}).forEach(function (f) {
          if (!read[f] && f !== "title") context[f] = true;
        });
      });
      var show = function (v) {
        return typeof v === "number" ? num(v) : typeof v === "boolean" ? (v ? "yes" : "no") : String(v);
      };
      var setup = el(
        "div",
        { className: "setup" },
        ...Object.keys(context)
          .sort()
          .map(function (f) {
            var vals = ids.map(function (id) {
              return (facts[id] || {})[f];
            });
            var same = vals.every(function (v) {
              return JSON.stringify(v) === JSON.stringify(vals[0]);
            });
            return el(
              "div",
              {},
              el("b", { textContent: humanize(f) }),
              same
                ? el("span", { textContent: show(vals[0]) })
                : el(
                    "span",
                    {},
                    ...ids.map(function (id, i) {
                      return vals[i] == null ? null : el("span", { style: "display:block" }, el("span", { className: "sub", textContent: who(id) + ": " }), show(vals[i]));
                    }),
                  ),
              fields[f] ? el("small", { textContent: fields[f] }) : null,
            );
          }),
      );
      var trackName = ((D.scorecards || {})[names[0]] || {}).track;
      var track = trackName && (D.tracks || {})[trackName];
      var bases = { measured: 0, judged: 0 };
      names.forEach(function (n) {
        var b = basisCounts(((D.scorecards || {})[n] || {}).rules);
        bases.measured += b.measured;
        bases.judged += b.judged;
      });
      // The hero carries the description's first sentence; the method section continues from there.
      var lead = track && track.description ? firstSentence(track.description) : "";
      var rest = lead ? track.description.trim().slice(lead.length).trim() : "";
      var method = el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "How it was measured" }), track ? link("track/" + enc(trackName), "", "Track: " + (track.title || trackName) + " →") : null),
        rest ? el("p", { className: "method-lede" }, ...rich(rest)) : null,
        Object.keys(context).length ? setup : null,
        basisKey(bases),
        track ? refs(track.links) : null,
      );
      var brief = Object.keys(context)
        .sort()
        .map(function (f) {
          var vals = ids.map(function (id) {
            return (facts[id] || {})[f];
          });
          var same = vals.every(function (v) {
            return JSON.stringify(v) === JSON.stringify(vals[0]);
          });
          if (same) return humanize(f).toLowerCase() + " " + show(vals[0]);
          return (
            humanize(f).toLowerCase() +
            " " +
            ids
              .map(function (id, i) {
                return vals[i] == null ? null : who(id) + " " + show(vals[i]);
              })
              .filter(Boolean)
              .join(" vs ")
          );
        })
        .filter(Boolean);
      hero.firstChild.append(
        el(
          "p",
          { className: "sub", style: "margin:6px 0 0" },
          el("b", { textContent: lead ? "Method: " : "How it was measured: " }),
          ...(lead ? rich(lead + " ") : [(brief.length ? brief.join(" · ") + ". " : "") + "Every rule is " + (bases.judged ? "measured or agent-judged" : "measured by script") + ". "]),
          el("button", {
            className: "link",
            textContent: track && track.description ? "The full method, and who checked it ↓" : "Versions, setup and sources ↓",
            onclick: function () {
              method.scrollIntoView({ behavior: "smooth", block: "start" });
            },
          }),
        ),
      );

      var behindRows = [];
      names.forEach(function (n) {
        var c = all[n];
        var def = (D.scorecards || {})[n] || {};
        Object.keys(c.rules || {}).forEach(function (r) {
          if ((c.rules[r] || {})[peer] !== "behind") return;
          var rd = (def.rules || {})[r] || {};
          behindRows.push(
            el(
              "div",
              { className: "behind-row" },
              el(
                "div",
                {},
                el("div", { style: "font-weight:650", textContent: measureLabel(c, r) }),
                el("div", { className: "sub" }, link("compare/" + enc(n), "", def.title || n), " · ", el("code", { textContent: r })),
              ),
              el("div", {}, valueLine(c, r, rd, ids, who), rd.rationale ? el("p", { className: "why" }, ...rich(rd.rationale)) : null),
            ),
          );
        });
      });
      var behind = el(
        "section",
        {},
        el("div", { className: "head" }, el("h2", { textContent: "Where " + who(subject) + " is behind" }), el("span", { className: "sub", textContent: plural(behindRows.length, "rule") })),
        behindRows.length ? el("div", {}, ...behindRows) : el("p", { className: "empty good", textContent: "✓ Not behind on any rule." }),
      );
      var leadRows = [];
      names.forEach(function (n) {
        var c = all[n];
        var def = (D.scorecards || {})[n] || {};
        Object.keys(c.rules || {}).forEach(function (r) {
          var rd = (def.rules || {})[r] || {};
          if ((c.rules[r] || {})[peer] !== "ahead" || !rd.better) return;
          var a = ruleValue(c, r, subject);
          var b = ruleValue(c, r, peer);
          if (typeof a !== "number" || typeof b !== "number" || a <= 0 || b <= 0) return;
          leadRows.push({ n: n, c: c, r: r, rd: rd, a: a, b: b, ratio: Math.max(a, b) / Math.min(a, b) });
        });
      });
      leadRows.sort(function (x, y) {
        return y.ratio - x.ratio;
      });
      var unitOf = function (rd, v) {
        return num(v) + (rd.unit ? " " + rd.unit : "");
      };
      var leads = leadRows.length
        ? el(
            "section",
            {},
            el("div", { className: "head" }, el("h2", { textContent: "Where " + who(subject) + " leads by the most" }), el("span", { className: "sub", textContent: "Measured values only, largest ratio first" })),
            el(
              "div",
              { className: "leads" },
              ...leadRows.slice(0, 3).map(function (l) {
                var x = l.ratio >= 10 ? Math.round(l.ratio).toLocaleString("en-US") : l.ratio.toFixed(1);
                var vs = who(subject) + " " + unitOf(l.rd, l.a) + " vs " + who(peer) + " " + unitOf(l.rd, l.b);
                var tile = link("compare/" + enc(l.n), "lead-tile", [
                  el("span", { className: "lead-x", textContent: x + "×" }),
                  el("span", { className: "lead-dir", textContent: l.rd.better }),
                  el("span", { className: "lead-what", textContent: measureLabel(l.c, l.r) }),
                  el("span", { className: "sub", textContent: vs + " · " + title(D, l.n) }),
                ]);
                tile.setAttribute("aria-label", x + "× " + l.rd.better + ": " + measureLabel(l.c, l.r) + ". " + vs + ". Factor: " + title(D, l.n));
                return tile;
              }),
            ),
          )
        : null;
      return el(
        "div",
        {},
        hero,
        el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Factors" }), el("span", { className: "sub", textContent: "Behind first" })), this.factorCards(D, names)),
        behind,
        leads,
        method,
      );
    }

    compareDetail(D, name, names, who) {
      var c = D.comparisons[name];
      var def = (D.scorecards || {})[name] || {};
      var peer = (c.peers || [])[0];
      var ids = [c.subject].concat(c.peers || []);
      var v = (c.verdict || {})[peer] || "unknown";
      var order = { behind: 0, unknown: 1, ahead: 2, parity: 3 };
      var rules = Object.keys(def.rules || {}).sort(function (a, b) {
        var sa = (c.rules[a] || {})[peer] || "unknown";
        var sb = (c.rules[b] || {})[peer] || "unknown";
        return order[sa] - order[sb] || a.localeCompare(b);
      });
      var rows = rules.map(function (r) {
        var rd = def.rules[r];
        var st = (c.rules[r] || {})[peer] || "unknown";
        return el(
          "div",
          { className: "measure" },
          el(
            "div",
            { className: "mt" },
            measureLabel(c, r),
            el("small", {}, el("code", { textContent: r }), basisBadge(basisOf(rd)), rd.better ? el("span", { textContent: rd.better + " is better" }) : null),
          ),
          valueLine(c, r, rd, ids, who),
          el("div", { className: "ms" }, standingChip(st, st === "parity" ? "parity" : who(c.subject) + " " + st), magnitude(c, r, rd, who)),
          el(
            "div",
            { className: "full" },
            whyEl(rd),
            el(
              "details",
              {},
              el("summary", { textContent: "How it is scored" }),
              el(
                "p",
                { className: "sub", style: "margin:6px 0 0" },
                ((c.reads || {})[r] || []).length
                  ? "The benchmark harness recorded " +
                      ((c.reads || {})[r] || [])
                        .map(function (f) {
                          return "“" + ((c.fields || {})[f] || humanize(f)).replace(/\.$/, "") + "”";
                        })
                        .join(" and ") +
                      " for each product, on the versions and machine under "
                  : "Scored on the versions and machine under ",
                link("compare", "", "How it was measured"),
                ". " + (rd.better ? "The " + rd.better + " value is ahead." : "Passing beats failing."),
              ),
              el("div", { className: "cel", ariaLabel: "Expression", textContent: rd.check + (rd.value ? "\nvalue: " + rd.value : "") }),
              rd.fix && (st === "behind" || st === "unknown") ? el("p", { className: "sub", style: "margin:4px 0 0" }, el("b", { textContent: "What would close it: " }), ...rich(rd.fix)) : null,
            ),
          ),
        );
      });
      // Parity sorts last; when it is most of a long factor, it folds so the rules that differ lead.
      var atParity = rules.filter(function (r) {
        return ((c.rules[r] || {})[peer] || "unknown") === "parity";
      }).length;
      var fold = atParity >= 3 && atParity < rules.length;
      var head = el(
        "section",
        {},
        el("nav", { className: "crumb", ariaLabel: "Breadcrumb" }, link("compare", "", "Compare"), " / " + (def.title || name)),
        el("div", { className: "head" }, el("h2", { textContent: def.title || name, tabIndex: -1 }), standingChip(v, standingPhrase(who(c.subject), v, who(peer)))),
        def.description ? el("p", { className: "lede" }, ...rich(def.description)) : null,
        def.rationale || (def.links || []).length ? el("div", { className: "rationale" }, el("b", { textContent: "Why this factor matters" }), def.rationale ? el("div", {}, ...rich(def.rationale)) : null, refs(def.links)) : null,
        el("p", { className: "sub", textContent: "Each rule scores both products. Where both have a measured value, the better value decides; otherwise passing beats failing. " + who(c.subject) + " is behind on the factor if it is behind on any rule." }),
        basisKey(basisCounts(def.rules)),
        el("div", {}, ...(fold ? rows.slice(0, rows.length - atParity) : rows)),
        fold
          ? el(
              "details",
              { className: "parity-fold" },
              el("summary", { textContent: "Show the " + atParity + " rules at parity (neither product leads)" }),
              ...rows.slice(rows.length - atParity),
            )
          : null,
      );
      head.dataset.focus = "";
      return el("div", {}, head, el("section", {}, el("div", { className: "head" }, el("h2", { textContent: "Every factor" }), link("compare", "", "← Back to the summary")), this.factorCards(D, names, name)));
    }

    scorecard(D, S, name, query) {
      query = query || {};
      var ranked = S.rows;
      var active = name && D.scorecards[name] ? name : ranked[0] && ranked[0].name;
      if (!active) return el("section", {}, el("div", { className: "empty", textContent: "No scorecards in this report." }));
      var self = this;
      var sc = D.scorecards[active];
      var rows = Object.entries(D.scores[active] || {});
      var order = levelOrder(sc.levels);
      var parts = tallyLevels(rows, order);

      var cardSearch = el("input", { type: "search", placeholder: "Find a scorecard by name or id…", ariaLabel: "Find a scorecard" });
      var cardNav = el("nav", { className: "rank", ariaLabel: "Scorecards, most debt first" });
      var drawCards = function () {
        var q = cardSearch.value.toLowerCase();
        var shown = ranked.filter(function (r) {
          return !q || r.name.toLowerCase().indexOf(q) >= 0 || title(D, r.name).toLowerCase().indexOf(q) >= 0 || (r.sc.kind || "").toLowerCase().indexOf(q) >= 0;
        });
        if (!shown.length) {
          cardNav.replaceChildren(
            el("div", { className: "empty", textContent: "No scorecards match." }),
            el("button", {
              className: "link",
              textContent: "Clear search",
              onclick: function () {
                cardSearch.value = "";
                drawCards();
              },
            }),
          );
          return;
        }
        var nStd = shown.filter(function (r) {
          return !r.compare;
        }).length;
        var items = [];
        shown.forEach(function (r, i) {
          if (i === 0 && !r.compare) items.push(el("div", { className: "rank-head", textContent: plural(nStd, "standard") }));
          if (r.compare && (i === 0 || !shown[i - 1].compare)) items.push(el("div", { className: "rank-head", textContent: plural(shown.length - nStd, "comparison factor") + ", also on the Compare tab" }));
          var c = r.compare ? (D.comparisons || {})[r.name] || {} : null;
          var verdict = c ? (c.verdict || {})[(c.peers || [])[0]] || "unknown" : null;
          var debt = c ? (verdict === "unknown" ? "not measured" : verdict) : r.none > 0 ? r.none + " at none" : r.bronze > 0 ? r.bronze + " at bronze" : "✓ clear";
          var a = link("scorecard/" + enc(r.name), "", [el("span", {}, title(D, r.name), judgedMark(D, r.name) ? " " : null, judgedMark(D, r.name)), el("span", { className: "debt " + (c ? (verdict === "behind" ? "bad" : verdict === "ahead" ? "ok" : "") : r.none > 0 ? "bad" : r.bronze > 0 ? "" : "ok"), textContent: debt })]);
          a.title = r.name + " · " + r.sc.kind;
          if (r.name === active) a.setAttribute("aria-current", "true");
          items.push(a);
        });
        cardNav.replaceChildren.apply(cardNav, items);
      };
      cardSearch.oninput = function () {
        drawCards();
        edges();
      };
      drawCards();
      var edges = function () {
        cardNav.classList.toggle("more", cardNav.scrollTop + cardNav.clientHeight < cardNav.scrollHeight - 4);
        cardNav.classList.toggle("less", cardNav.scrollTop > 4);
      };
      cardNav.onscroll = edges;
      requestAnimationFrame(function () {
        var cur = cardNav.querySelector("[aria-current]");
        if (cur && cardNav.scrollHeight > cardNav.clientHeight) cardNav.scrollTop = cur.offsetTop - cardNav.clientHeight / 3;
        edges();
      });

      var levelFilter = query.level && (order.indexOf(query.level) >= 0 || query.level === "skipped") ? query.level : "all";
      var outcomeFilter = query.outcome === "failing" || query.outcome === "passing" ? query.outcome : "all";
      var ruleFilter = query.rule && (sc.rules || {})[query.rule] ? query.rule : null;
      var search = el("input", { type: "search", placeholder: "Filter by entity or rule…", ariaLabel: "Filter entities", value: query.q || "" });
      var keepInUrl = function () {
        var qs = new URLSearchParams();
        if (levelFilter !== "all") qs.set("level", levelFilter);
        if (outcomeFilter !== "all") qs.set("outcome", outcomeFilter);
        if (ruleFilter) qs.set("rule", ruleFilter);
        if (search.value) qs.set("q", search.value);
        var tail = qs.toString();
        if (!tail && location.hash.indexOf("?") < 0) return;
        var want = "#scorecard/" + enc(active) + (tail ? "?" + tail : "");
        if (location.hash !== want && history.replaceState) history.replaceState(history.state, "", want);
      };
      var filters = el("div", { className: "filters" });
      var list = el("div");
      var limit = 12;
      var seg = function (label, options, current, set) {
        var g = el(
          "div",
          { className: "seg" },
          ...options.map(function (o) {
            return el("button", {
              textContent: o.label,
              ariaPressed: String(current === o.value),
              onclick: function () {
                set(o.value);
                redrawFilters();
                draw();
              },
            });
          }),
        );
        g.setAttribute("role", "group");
        g.setAttribute("aria-label", label);
        return g;
      };
      var clearFilters = function () {
        levelFilter = "all";
        outcomeFilter = "all";
        ruleFilter = null;
        search.value = "";
        redrawFilters();
        draw();
      };
      var redrawFilters = function () {
        var present = order.filter(function (l) {
          return parts.some(function (p) {
            return p.level === l;
          });
        });
        filters.replaceChildren(
          seg(
            "Level",
            [{ label: "All levels", value: "all" }].concat(
              present.map(function (l) {
                return { label: l === "skipped" ? "not in scope" : l, value: l };
              }),
            ),
            levelFilter,
            function (v) {
              levelFilter = v;
            },
          ),
          seg(
            "Outcome",
            [
              { label: "All outcomes", value: "all" },
              { label: "Failing", value: "failing" },
              { label: "Passing", value: "passing" },
            ],
            outcomeFilter,
            function (v) {
              outcomeFilter = v;
            },
          ),
          ruleFilter
            ? el(
                "button",
                {
                  className: "chip-filter",
                  ariaLabel: "Remove filter: not passing " + ruleFilter,
                  onclick: function () {
                    ruleFilter = null;
                    redrawFilters();
                    draw();
                  },
                },
                "Not passing ",
                el("b", { textContent: ruleFilter }),
                el("span", { ariaHidden: "true", textContent: " ×" }),
              )
            : "",
        );
      };
      var draw = function () {
        keepInUrl();
        var q = search.value.toLowerCase();
        var shown = rows
          .filter(function (pair) {
            return pair[1].level !== "skipped" || levelFilter === "skipped";
          })
          .filter(function (pair) {
            return levelFilter === "all" || pair[1].level === levelFilter;
          })
          .filter(function (pair) {
            if (!q) return true;
            if (pair[0].toLowerCase().indexOf(q) >= 0) return true;
            return Object.keys(pair[1].rules || {}).some(function (rn) {
              return rn.toLowerCase().indexOf(q) >= 0;
            });
          })
          .filter(function (pair) {
            if (!ruleFilter) return true;
            var o = (pair[1].rules || {})[ruleFilter];
            return isFailing(o) || o === "unknown";
          })
          .filter(function (pair) {
            if (outcomeFilter === "all") return true;
            var outs = Object.values(pair[1].rules || {});
            return outcomeFilter === "failing"
              ? outs.some(isFailing)
              : outs.every(function (o) {
                  return isPassing(o) || o === "skipped";
                });
          })
          .sort(function (a, b) {
            return order.indexOf(a[1].level) - order.indexOf(b[1].level) || a[0].localeCompare(b[0]);
          });
        if (!shown.length) {
          list.replaceChildren(
            el("div", { className: "empty", textContent: "No entities match these filters." }),
            el("button", { className: "link", textContent: "Clear filters", onclick: clearFilters }),
          );
          return;
        }
        var outOfScope = rows.filter(function (pair) {
          return pair[1].level === "skipped";
        }).length;
        list.replaceChildren(el("div", { className: "sub", style: "margin:6px 0", textContent: plural(shown.length, "entity", "entities") + (outOfScope && levelFilter === "all" && outcomeFilter === "all" && !ruleFilter && !q ? " in scope, of " + kindCount(D, sc.kind, rows.length) : "") + ", lowest level first" }));
        shown.slice(0, limit).forEach(function (pair) {
          var id = pair[0];
          var r = pair[1];
          var outs = Object.entries(r.rules || {});
          var failing = outs.filter(function (ro) {
            return isFailing(ro[1]);
          });
          var passing = outs.filter(function (ro) {
            return isPassing(ro[1]);
          });
          var unrun = outs.filter(function (ro) {
            return ro[1] === "unknown";
          }).length;
          var short = shortId(id);
          var next = nextLevel(sc, r.level);
          var need = next ? blockers(sc, r, next) : [];
          var later = failing.length - need.length;
          var d = el(
            "details",
            { className: "ent" },
            el(
              "summary",
              {},
              r.level === "skipped" ? badge(r.level) : levelPill(r.level, unassessedIn(D, active, id), unrunIn(sc, r), !unassessedIn(D, active, id)),
              el("span", { className: "name" }, short.replace(/[^\s\-/]+$/, ""), el("span", { className: "tail" }, (/[^\s\-/]+$/.exec(short) || [""])[0], el("span", { className: "chev", textContent: "›" })), short !== id ? el("small", { textContent: id, title: id }) : null),
              el(
                "span",
                { className: "tally" },
                need.length
                  ? el(
                      "b",
                      { className: r.level === "none" ? "bad" : "" },
                      next.charAt(0).toUpperCase() + next.slice(1) + " needs: ",
                      ...need.slice(0, 2).map(function (n, i) {
                        return el("span", { className: "nr" }, (i ? ", " : "") + n + " ", basisBadge(basisOf((sc.rules || {})[n]), true));
                      }),
                      need.length > 2 ? " +" + (need.length - 2) : "",
                    )
                  : failing.length
                    ? ""
                    : "✓ all pass",
                later > 0 ? (need.length ? " · " : "") + later + " more for higher levels" : "",
                " · " + passing.length + " of " + outs.length + " pass",
                unrun ? el("span", { className: "provisional" }, " · " + unrun + " not run") : "",
              ),
            ),
          );
          var bodyBuilt = false;
          d.addEventListener("toggle", function () {
            if (!d.open || bodyBuilt) return;
            bodyBuilt = true;
            d.append(
              el(
                "div",
                { className: "ent-body" },
                self.nextStep(sc, r),
                self.rulesFor(sc, r, active, unassessedIn(D, active, id)),
                el("p", { style: "margin:12px 0 0" }, link("entity/" + enc(id), "", "Open " + short + " across every scorecard →")),
              ),
            );
          });
          list.append(d);
        });
        if (shown.length > limit) {
          list.append(
            el(
              "div",
              { className: "more" },
              el("button", {
                className: "link",
                textContent: "Show all " + fmt(shown.length) + " (" + fmt(shown.length - limit) + " more)",
                onclick: function () {
                  limit = Infinity;
                  draw();
                },
              }),
            ),
          );
        }
      };
      search.oninput = draw;
      redrawFilters();
      draw();

      var scored = rows.filter(function (pair) {
        return pair[1].level !== "skipped";
      });
      var failCounts = {};
      var unrunCounts = {};
      var provisionalN = 0;
      scored.forEach(function (pair) {
        Object.entries(pair[1].rules || {}).forEach(function (ro) {
          if (isFailing(ro[1])) failCounts[ro[0]] = (failCounts[ro[0]] || 0) + 1;
          else if (ro[1] === "unknown") unrunCounts[ro[0]] = (unrunCounts[ro[0]] || 0) + 1;
        });
        if (unrunIn(sc, pair[1]).length) provisionalN++;
      });
      var unrunNames = Object.keys(unrunCounts).sort();
      var unrunNote = unrunNames.length
        ? el(
            "p",
            { className: "sub unrun-note" },
            el("b", { className: "provisional", textContent: "Not run in this build: " }),
            ...unrunNames.map(function (n, i) {
              return el("span", { className: "nr" }, i ? ", " : "", ruleLink(active, n), " ", basisBadge(basisOf((sc.rules || {})[n]), true), " (" + unrunCounts[n] + " of " + scored.length + ")");
            }),
            ". A check that was not run is neither a pass nor a failure, so it is left out of the bars above and of the biggest lever. Levels that need one are provisional: they hold only if it passes.",
          )
        : null;
      var provisionalNote = provisionalN
        ? el("p", { className: "sub", style: "margin:6px 0 0" }, el("b", { className: "provisional", textContent: "Provisional: " }), plural(provisionalN, "entity's level rests", "entities' levels rest") + " on checks not run in this build (drawn dotted below). ", el("span", { textContent: "They hold only if those checks pass." }))
        : null;
      // The rule that holds the most entities at the lowest occupied level below the top.
      var lever = null;
      var leverRule = null;
      var low = order.find(function (l) {
        return l !== "skipped" && nextLevel(sc, l) && parts.some(function (p) {
          return p.level === l;
        });
      });
      if (low) {
        var held = {};
        var up = nextLevel(sc, low);
        scored.forEach(function (pair) {
          if (pair[1].level !== low) return;
          blockers(sc, pair[1], up).forEach(function (n) {
            held[n] = (held[n] || 0) + 1;
          });
        });
        var best = Object.entries(held).sort(function (a, b) {
          return b[1] - a[1] || a[0].localeCompare(b[0]);
        })[0];
        if (best) leverRule = best[0];
        var close = scored.filter(function (pair) {
          var nx = pair[1].level !== "skipped" && nextLevel(sc, pair[1].level);
          return nx && blockers(sc, pair[1], nx).length === 1;
        }).length;
        if (best)
          lever = el(
            "div",
            { className: "next", style: "margin:0 0 10px" },
            el("b", { textContent: "Biggest lever" }),
            el("span", { className: "sub", textContent: " (the rule that lifts the most entities a level)" }),
            el("br"),
            ruleLink(active, best[0]),
            " ",
            basisBadge(basisOf((sc.rules || {})[best[0]])),
            " keeps " + plural(best[1], "entity", "entities") + " at " + low + "; they need it for " + up + ". ",
            ...rich(((sc.rules || {})[best[0]] || {}).fix),
            close ? el("span", { className: "close-note" }, el("br"), el("b", { textContent: "Within reach: " }), plural(close, "entity is", "entities are") + " one rule from " + (close === 1 ? "its" : "their") + " next level.") : null,
          );
      }
      var topFails = Object.entries(failCounts)
        .sort(function (a, b) {
          return b[1] - a[1] || a[0].localeCompare(b[0]);
        })
        .slice(0, 5);
      var failChart = topFails.length
        ? el(
            "div",
            { className: "fails" },
            ...topFails.map(function (f) {
              var rl = ((sc.rules || {})[f[0]] || {}).level;
              var share = Math.round((f[1] / scored.length) * 100);
              var fillSpan = el("span", { style: "flex-grow:0;background:" + (rl ? fill(rl) : "var(--faint)") });
              grow([fillSpan], [share]);
              var b = el(
                "button",
                {
                  className: "fail-row",
                  title: ((sc.rules || {})[f[0]] || {}).fix || "",
                  onclick: function () {
                    search.value = "";
                    levelFilter = "all";
                    outcomeFilter = "all";
                    ruleFilter = f[0];
                    limit = 25;
                    redrawFilters();
                    draw();
                    filters.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
                    var chip = filters.querySelector(".chip-filter");
                    if (chip) chip.focus({ preventScroll: true });
                  },
                },
                el("span", { className: "rn" }, el("span", { textContent: f[0] }), basisBadge(basisOf((sc.rules || {})[f[0]]), false, true), rl ? badge(rl) : null, lever && f[0] === leverRule ? el("span", { className: "pill", title: "The biggest lever: fixing it lifts the most entities a level", textContent: "biggest lever" }) : null),
                el("span", { className: "bar xs" }, fillSpan, el("span", { style: "flex-grow:" + (100 - share) + ";background:transparent" })),
                el("span", { className: "fc", textContent: f[1] + " of " + scored.length }),
              );
              b.setAttribute("aria-label", f[0] + ": fails on " + f[1] + " of " + scored.length + ". Show them.");
              return b;
            }),
          )
        : el("p", { className: "empty good", textContent: unrunNames.length ? "✓ No rule fails on any entity; some checks were not run." : "✓ No rule fails on any entity." });

      var needs = el("div", { className: "levels-need" });
      (sc.levels || []).forEach(function (l) {
        var names = Object.keys(sc.rules || {})
          .filter(function (n) {
            return sc.rules[n].level === l;
          })
          .sort();
        var bases = names.map(function (n) {
          return basisOf(sc.rules[n]);
        });
        var shared = bases.length && bases.every(function (b) {
          return b === bases[0];
        })
          ? bases[0]
          : null;
        needs.append(
          el(
            "div",
            { className: "need" },
            el("div", { className: "need-head" }, badge(l), shared ? el("span", { className: "need-basis" }, names.length > 1 ? "All " : "", basisBadge(shared, true)) : null),
            names.length
              ? el(
                  "ul",
                  {},
                  ...names.map(function (n) {
                    var rd = sc.rules[n];
                return el("li", {}, el("span", { style: "display:inline-flex;flex-wrap:wrap;gap:4px 6px;align-items:center" }, el("b", {}, ruleLink(active, n)), shared ? null : basisBadge(basisOf(rd), true)), rd.rationale || rd.fix ? el("span", { className: "why" }, ...rich(rd.rationale || rd.fix)) : null, refs(rd.links));
                  }),
                )
              : el("div", { className: "sub", textContent: "No rules at this level." }),
          ),
        );
      });
      // Reasons are clamped to three lines so the levels read side by side; the full text stays in the
      // accessibility tree and one toggle opens them all.
      needs.classList.add("clamped");
      var needsToggle = el("button", {
        className: "link need-toggle",
        textContent: "Show the full reasons",
        ariaExpanded: "false",
        onclick: function () {
          var open = needs.classList.toggle("clamped") === false;
          needsToggle.textContent = open ? "Show shorter reasons" : "Show the full reasons";
          needsToggle.setAttribute("aria-expanded", String(open));
        },
      });
      var picker = el(
        "select",
        {
          ariaLabel: "Scorecard",
          onchange: function () {
            location.hash = "scorecard/" + enc(picker.value);
          },
        },
        ...[false, true]
          .map(function (cmp) {
            var group = ranked.filter(function (r) {
              return !!r.compare === cmp;
            });
            if (!group.length) return null;
            return el(
              "optgroup",
              { label: cmp ? plural(group.length, "comparison factor") : plural(group.length, "standard") },
              ...group.map(function (r) {
                var c = cmp ? (D.comparisons || {})[r.name] || {} : null;
                var debt = c ? (c.verdict || {})[(c.peers || [])[0]] || "not measured" : r.none > 0 ? r.none + " at none" : r.bronze > 0 ? r.bronze + " at bronze" : "clear";
                return el("option", { value: r.name, selected: r.name === active, textContent: title(D, r.name) + " (" + debt + ")" });
              }),
            );
          })
          .filter(Boolean),
      );
      var owner = sc.owner ? sc.owner.replace(/^group:/, "") : "";
      var hStand = el("h3", { textContent: "Where entities stand", tabIndex: -1 });
      var hFail = el("h3", { textContent: "Most common failures, at any level", tabIndex: -1 });
      var hNeeds = el("h3", { textContent: "What each level needs (each level also needs the ones below)", tabIndex: -1 });
      var hList = el("h3", { textContent: "Entities", tabIndex: -1 });
      var jump = function (label, target) {
        return el("button", {
          className: "link",
          textContent: label,
          onclick: function () {
            target.scrollIntoView({ block: "start", behavior: matchMedia("(prefers-reduced-motion: reduce)").matches ? "auto" : "smooth" });
            target.focus({ preventScroll: true });
          },
        });
      };
      var jumpBar = el("nav", { className: "jumpbar", ariaLabel: "On this page" }, el("span", { className: "sub", textContent: "On this page:" }), jump("Where entities stand", hStand), jump("Failures", hFail), jump("What each level needs", hNeeds), jump("Entities (" + fmt(scored.length) + ")", hList));
      return el(
        "div",
        { className: "split" },
        el(
          "section",
          {},
          el("div", { className: "head" }, el("h2", { textContent: "Scorecards" })),
          el("p", { className: "sub", style: "margin:0", textContent: "Most entities at none first" }),
          cardSearch,
          cardNav,
        ),
        el(
          "section",
          {},
          el("label", { className: "picker" }, el("span", { className: "sub", textContent: "Scorecard" }), picker),
          el("div", { className: "head" }, el("h2", { textContent: title(D, active) }), el("code", { className: "sub", textContent: active })),
          el("p", { className: "lede" }, ...richCards(D, sc.description, active)),
          sc.rationale || (sc.links || []).length ? el("div", { className: "rationale" }, el("b", { textContent: "Why this standard" }), sc.rationale ? el("div", {}, ...richCards(D, sc.rationale, active)) : null, refs(sc.links)) : null,
          el(
            "div",
            { className: "sub" },
            "Rates every " + kindTitle(D, sc.kind) + (owner ? " · owned by " + owner : ""),
            sc.track ? el("span", {}, " · track ", link("track/" + enc(sc.track), "", trackTitle(D, sc.track))) : null,
            basisInline(basisCounts(sc.rules)),
          ),
          isComparative(D, active) ? el("div", { className: "callout" }, el("span", {}, "This scorecard compares products rather than levelling them. ", link("compare/" + enc(active), "", "See how they stand →"))) : null,
          basisInline(basisCounts(sc.rules)) ? null : basisKey(basisCounts(sc.rules)),
          jumpBar,
          hStand,
          barEl(parts, "lg"),
          legendEl(parts),
          unassessedNote(D, active, scored.length),
          provisionalNote,
          lever ? el("div", { style: "margin-top:12px" }, lever) : null,
          hFail,
          failChart ? el("p", { className: "sub", style: "margin:0 0 6px", textContent: "Each bar is the share of entities failing the rule, coloured by the level the rule is needed for." }) : null,
          failChart,
          unrunNote,
          hNeeds,
          needsToggle,
          needs,
          hList,
          filters,
          search,
          list,
        ),
      );
    }
  }

  customElements.define("toolbox-report", ToolboxReport);
})();
