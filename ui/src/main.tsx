import { StrictMode, useCallback, useEffect, useState } from "react";
import { createRoot } from "react-dom/client";
import { api, devUser, type ApiResult, type Json } from "./api";
import "./app.css";

// ----------------------------------------------------------------------------------- routing
function usePath(): string {
  const [path, setPath] = useState(location.pathname + location.search);
  useEffect(() => {
    const on = () => setPath(location.pathname + location.search);
    addEventListener("popstate", on);
    return () => removeEventListener("popstate", on);
  }, []);
  return path;
}

function go(to: string) {
  history.pushState(null, "", to);
  dispatchEvent(new PopStateEvent("popstate"));
}

function Link({ to, children }: { to: string; children: React.ReactNode }) {
  return (
    <a href={to} onClick={(e) => { if (!e.metaKey && !e.ctrlKey) { e.preventDefault(); go(to); } }}>
      {children}
    </a>
  );
}

const enc = encodeURIComponent;
const entityLink = (id: string) => <Link to={`/e/${enc(id)}`}>{id}</Link>;

// ----------------------------------------------------------------------------------- data
function useApi(path: string | null, deps: unknown[] = []): [ApiResult | null, () => void] {
  const [res, setRes] = useState<ApiResult | null>(null);
  const [n, setN] = useState(0);
  const [shownPath, setShownPath] = useState<string | null>(null);
  useEffect(() => {
    if (!path) return;
    let live = true;
    // Clear only when navigating; a reload keeps the current page (and its controls) on screen.
    if (path !== shownPath) setRes(null);
    api(path).then((r) => { if (live) { setRes(r); setShownPath(path); } });
    return () => { live = false; };
  }, [path, n, ...deps]);
  return [res, () => setN((x) => x + 1)];
}

// -------------------------------------------------------------------------- shared pieces
function Value({ v }: { v: Json }) {
  if (v && typeof v === "object" && v.redacted) {
    return (
      <span className="redacted" title={`Needs ${v.requires}`}>
        🔒 Visible to {v.visible_to}
      </span>
    );
  }
  if (v === null || v === undefined) return <span className="muted">—</span>;
  if (typeof v === "string") return v.includes(":") && !v.includes(" ") ? entityLink(v) : <>{v}</>;
  if (typeof v === "boolean") return <>{v ? "yes" : "no"}</>;
  if (Array.isArray(v)) return <>{v.length ? v.map((x, i) => <span key={i}>{i > 0 && ", "}<Value v={x} /></span>) : <span className="muted">none</span>}</>;
  return <code>{JSON.stringify(v)}</code>;
}

function Level({ level }: { level: Json }) {
  if (level && typeof level === "object") return <Value v={level} />;
  return <span className={`level level-${level}`}>{String(level)}</span>;
}

function Problem({ res }: { res: ApiResult }) {
  return (
    <div className="problem" role="alert">
      <strong>{res.body?.title ?? `HTTP ${res.status}`}</strong> {res.body?.detail}
      {res.status === 401 && <p>Sign in through your identity-aware proxy, or on a dev server add <code>?as=&lt;user&gt;</code> to the URL.</p>}
    </div>
  );
}

function Loading() {
  return <div className="skeleton" aria-busy="true" aria-label="Loading" />;
}

function AsOf({ snapshot }: { snapshot: string | null }) {
  return snapshot ? <span className="asof">as of snapshot <code>{snapshot}</code></span> : null;
}

// ----------------------------------------------------------------------------------- pages
function Home() {
  const [res] = useApi("/v1/views/home");
  if (!res) return <Loading />;
  if (res.status !== 200) return <Problem res={res} />;
  const h = res.body;
  return (
    <>
      <h1>{h.portal?.title ?? "Toolbox"}</h1>
      <p className="muted">Signed in as {h.viewer.user}{h.viewer.admin ? " (portal admin)" : ""}. <AsOf snapshot={res.snapshot} /></p>
      <section>
        <h2>Your teams' entities</h2>
        {h.teams.length === 0 && <p className="muted">You're not in a team that owns anything yet.</p>}
        {h.owned.length > 0 && (
          <table>
            <thead><tr><th>Entity</th><th>Levels</th><th>Failing rules</th></tr></thead>
            <tbody>
              {h.owned.map((e: Json) => (
                <tr key={e.id}>
                  <td>{entityLink(e.id)}</td>
                  <td>{Object.entries(e.levels).map(([k, v]) => <span key={k} className="chip">{k}: <Level level={v} /></span>)}</td>
                  <td>{e.failing.length ? e.failing.join(", ") : <span className="muted">none</span>}</td>
                </tr>
              ))}
            </tbody>
          </table>
        )}
      </section>
      <section className="grid">
        <div>
          <h2>Browse</h2>
          <ul>{Object.entries(h.kinds).map(([k, n]) => <li key={k}><Link to={`/c/${k}`}>{k}</Link> <span className="muted">({String(n)})</span></li>)}</ul>
        </div>
        <div>
          <h2>Scorecards</h2>
          <ul>{h.scorecards.map((s: Json) => <li key={s.name}><Link to={`/s/${s.name}`}>{s.name}</Link> <span className="muted">on {s.kind}</span></li>)}</ul>
          {(h.campaigns?.length ?? 0) > 0 && (
            <>
              <h2>Campaigns</h2>
              <ul>{h.campaigns.map((c: Json) => <li key={c.name}><Link to={`/cgn/${c.name}`}>{c.title ?? c.name}</Link> <span className="muted">{c.percent}% / {c.targetPercent}%</span></li>)}</ul>
            </>
          )}
        </div>
      </section>
    </>
  );
}

function EntityList({ path, empty }: { path: string; empty: string }) {
  const [res] = useApi(path);
  if (!res) return <Loading />;
  if (res.status !== 200) return <Problem res={res} />;
  const items = res.body.items as Json[];
  if (!items.length) return <p className="muted">{empty}</p>;
  return (
    <table>
      <thead><tr><th>Entity</th><th>Title</th><th>Owner</th></tr></thead>
      <tbody>
        {items.map((e) => (
          <tr key={e.id}>
            <td>{entityLink(e.id)}</td>
            <td><Value v={e.fields.title ?? e.fields.displayName} /></td>
            <td>{e.fields.ownership ? <>{e.fields.ownership.owner ? <Value v={e.fields.ownership.owner} /> : <span className="muted">none</span>} <span className={`status status-${e.fields.ownership.status}`}>{e.fields.ownership.status}</span></> : <span className="muted">—</span>}</td>
          </tr>
        ))}
      </tbody>
    </table>
  );
}

function Search({ q }: { q: string }) {
  return (
    <>
      <h1>Search: {q}</h1>
      <EntityList path={`/v1/entities?q=${enc(q)}&limit=50`} empty="Nothing matches." />
    </>
  );
}

function Catalog({ kind, filter }: { kind: string; filter: string }) {
  const [draft, setDraft] = useState(filter);
  return (
    <>
      <h1>{kind}</h1>
      <form className="filter" onSubmit={(e) => { e.preventDefault(); go(`/c/${kind}${draft ? `?filter=${enc(draft)}` : ""}`); }}>
        <label htmlFor="filter">CEL filter over <code>entity</code></label>
        <input id="filter" value={draft} placeholder="entity.ownership.status != 'verified'" onChange={(e) => setDraft(e.target.value)} />
        <button type="submit">Apply</button>
      </form>
      <EntityList path={`/v1/entities?kind=${enc(kind)}&limit=200${filter ? `&filter=${enc(filter)}` : ""}`} empty="No entities match." />
    </>
  );
}

function Refresh({ id, onDone }: { id: string; onDone: (snapshot: string | null) => void }) {
  const [state, setState] = useState<"idle" | "running" | "done" | "error">("idle");
  const [msg, setMsg] = useState("");
  const run = useCallback(async () => {
    setState("running");
    setMsg("Asking the connectors to re-read this entity…");
    const r = await api(`/v1/entities/${enc(id)}/refresh`, { method: "POST" });
    if (r.status !== 202) {
      setState("error");
      setMsg(r.body?.detail ?? `HTTP ${r.status}`);
      return;
    }
    let snapshot: string | null = null;
    for (const c of r.body.commands as string[]) {
      const s = await api(`/v1/commands/${c}?wait=30`);
      if (s.body?.status !== "succeeded") {
        setState("error");
        setMsg(`Refresh ${s.body?.status ?? "failed"}`);
        return;
      }
      snapshot = s.body.snapshot ?? snapshot;
    }
    setState("done");
    setMsg(`Refreshed at snapshot ${snapshot}`);
    onDone(snapshot);
  }, [id, onDone]);
  return (
    <span className="refresh">
      <button onClick={run} disabled={state === "running"} aria-busy={state === "running"}>Refresh</button>
      <span role="status" className={`muted ${state}`}>{msg}</span>
    </span>
  );
}

function Entity({ id }: { id: string }) {
  const [res, reload] = useApi(`/v1/views/entity/${enc(id)}`);
  const [owner] = useApi(`/v1/entities/${enc(id)}/explain?field=ownership`);
  if (!res) return <Loading />;
  if (res.status !== 200) return <Problem res={res} />;
  const v = res.body;
  const e = v.entity;
  const auth = owner?.status === 200 ? owner.body.authority : null;
  return (
    <>
      <header className="entity-head">
        <div>
          <h1>{e.fields.title ?? e.fields.displayName ?? id}</h1>
          <p className="muted"><code>{id}</code> · {e.kind} · <AsOf snapshot={v.snapshot} /></p>
        </div>
        <Refresh id={id} onDone={() => reload()} />
      </header>
      {auth && (
        <section className={`callout status-${auth.status}`}>
          <strong>Ownership: {auth.status}</strong> — {auth.explanation}.{auth.fix && <> <em>Fix:</em> {auth.fix}.</>}
        </section>
      )}
      <section className="grid">
        <div>
          <h2>Fields</h2>
          <dl className="fields">
            {Object.entries(e.fields).filter(([k]) => k !== "ownership").map(([k, val]) => (
              <div key={k}><dt title={`from ${e.provenance[k] ?? "?"}`}>{k}</dt><dd><Value v={val} /> <span className="source">{e.provenance[k]}</span></dd></div>
            ))}
          </dl>
        </div>
        <div>
          <h2>Scorecards</h2>
          {Object.entries(v.scorecards).map(([name, sc]: [string, Json]) => (
            <div key={name} className="card">
              <h3><Link to={`/s/${name}`}>{name}</Link> <Level level={sc.level} /></h3>
              {sc.rules?.redacted ? <Value v={sc.rules} /> : sc.rules && (
                <ul className="rules">
                  {Object.entries(sc.rules).map(([r, o]: [string, Json]) => (
                    <li key={r} className={`rule-${o.outcome}`}><span className="outcome">{o.outcome}</span> {r}{o.fix && <div className="fix">{o.fix}</div>}</li>
                  ))}
                </ul>
              )}
            </div>
          ))}
          {Object.keys(v.scorecards).length === 0 && <p className="muted">No scorecards apply.</p>}
        </div>
      </section>
      <section className="grid">
        <div>
          <h2>Relations</h2>
          <ul>
            {v.relations.out.map((r: Json, i: number) => <li key={`o${i}`}>{r.relation} → {entityLink(r.to)}</li>)}
            {v.relations.in.map((r: Json, i: number) => <li key={`i${i}`}>{entityLink(r.from)} → {r.relation}</li>)}
            {!v.relations.out.length && !v.relations.in.length && <li className="muted">None visible.</li>}
          </ul>
        </div>
        <div>
          <h2>Actions</h2>
          <ul>
            {v.actions.map((a: Json) => <li key={a.name}><strong>{a.name}</strong> — {a.description} {a.allowed ? <span className="ok">available</span> : <span className="muted">(needs {a.permission})</span>}</li>)}
            {!v.actions.length && <li className="muted">No actions.</li>}
          </ul>
          <h2>Freshness</h2>
          <ul>{v.freshness.map((f: Json) => <li key={f.connector + f.kind}>{f.connector}/{f.kind}: {f.last_read ?? <span className="muted">never read by a connector</span>}</li>)}</ul>
        </div>
      </section>
    </>
  );
}

function Scorecard({ name }: { name: string }) {
  const [res] = useApi(`/v1/views/scorecard/${enc(name)}`);
  if (!res) return <Loading />;
  if (res.status !== 200) return <Problem res={res} />;
  const v = res.body;
  return (
    <>
      <h1>{name}</h1>
      <p className="muted">On {v.scorecard.kind}, owned by <Value v={v.scorecard.owner} />. <AsOf snapshot={res.snapshot} /></p>
      <section className="grid">
        <div>
          <h2>By level</h2>
          {Object.entries(v.by_level).map(([lvl, ids]: [string, Json]) => (
            <div key={lvl}><h3><Level level={lvl} /> <span className="muted">({ids.length})</span></h3><ul>{ids.map((i: string) => <li key={i}>{entityLink(i)}</li>)}</ul></div>
          ))}
        </div>
        <div>
          <h2>Failing rules</h2>
          <ul>{Object.entries(v.failing_rules).map(([r, n]) => <li key={r}>{r}: {String(n)}</li>)}</ul>
          {Object.keys(v.failing_rules).length === 0 && <p className="muted">None you can see.</p>}
        </div>
      </section>
    </>
  );
}

function Campaign({ name }: { name: string }) {
  const [res] = useApi(`/v1/views/campaign/${enc(name)}`);
  if (!res) return <Loading />;
  if (res.status !== 200) return <Problem res={res} />;
  const c = res.body.campaign ?? res.body;
  const gaps = (c.entities as Json[]).filter((e) => !e.meeting);
  return (
    <>
      <h1>{c.title ?? c.name}</h1>
      <p className="muted">{c.status} · {c.percent}% of {c.targetPercent}% target ({c.meeting}/{c.inScope}). <AsOf snapshot={res.snapshot} /></p>
      {c.description && <p>{c.description}</p>}
      <h2>Not meeting ({gaps.length})</h2>
      <ul>
        {gaps.map((e: Json) => (
          <li key={e.id}>{entityLink(e.id)} <span className="muted">{(e.gaps as string[]).join("; ")}</span></li>
        ))}
        {!gaps.length && <li className="muted">Every visible in-scope entity meets the goal.</li>}
      </ul>
    </>
  );
}

function Ops() {
  const [res] = useApi("/v1/views/ops");
  if (!res) return <Loading />;
  if (res.status !== 200) return <Problem res={res} />;
  const v = res.body;
  return (
    <>
      <h1>Operations</h1>
      <p>Snapshot <code>{v.snapshot}</code>, input version {v.input_version}, last recompute {v.recompute_ms} ms.</p>
      <h2>Connectors</h2>
      <table>
        <thead><tr><th>Connector</th><th>Runtime</th><th>Registered</th><th>Last full read</th></tr></thead>
        <tbody>{v.connectors.map((c: Json) => (
          <tr key={c.name}><td>{c.name}</td><td>{c.runtime}</td><td>{c.registered ?? <span className="muted">no</span>}</td>
            <td>{Object.entries(c.last_read).map(([k, t]) => <div key={k}>{k}: {t ? String(t) : <span className="muted">never</span>}</div>)}</td></tr>
        ))}</tbody>
      </table>
      <h2>Pending commands ({v.pending_commands.length})</h2>
      <ul>{v.pending_commands.map((c: Json) => <li key={c.id}>{c.id}: {c.command} {c.kind ?? c.kinds?.join(",")} via {c.connector} ({c.status})</li>)}</ul>
      <h2>Diagnostics</h2>
      <ul>
        {v.unverified_ownership.map((i: string) => <li key={i}>Unverified ownership claim: {entityLink(i)}</li>)}
        {v.orphans.map((i: string) => <li key={i}>Orphan (no identity source): <code>{i}</code></li>)}
        {v.diagnostics.map((d: Json, i: number) => <li key={i}>{d.code}: {d.location} — {d.message}</li>)}
      </ul>
    </>
  );
}

// ----------------------------------------------------------------------------------- shell
function NewerData({ current }: { current: string | null }) {
  const [latest, setLatest] = useState<string | null>(null);
  useEffect(() => {
    const t = setInterval(async () => setLatest((await api("/v1/snapshot")).snapshot), 10_000);
    return () => clearInterval(t);
  }, []);
  if (!latest || !current || latest === current) return null;
  return <button className="newer" onClick={() => location.reload()}>Newer data — update</button>;
}

function App() {
  const path = usePath();
  const [q, setQ] = useState("");
  const url = new URL(path, location.origin);
  const parts = url.pathname.split("/").filter(Boolean).map(decodeURIComponent);
  let page: React.ReactNode;
  if (parts.length === 0) page = <Home />;
  else if (parts[0] === "search") page = <Search q={url.searchParams.get("q") ?? ""} />;
  else if (parts[0] === "c" && parts[1]) page = <Catalog kind={parts[1]} filter={url.searchParams.get("filter") ?? ""} />;
  else if (parts[0] === "e" && parts[1]) page = <Entity key={parts[1]} id={parts[1]} />;
  else if (parts[0] === "s" && parts[1]) page = <Scorecard name={parts[1]} />;
  else if (parts[0] === "cgn" && parts[1]) page = <Campaign name={parts[1]} />;
  else if (parts[0] === "ops") page = <Ops />;
  else page = <p>Not found. <Link to="/">Home</Link></p>;
  const user = devUser();
  return (
    <>
      <nav className="top">
        <Link to="/"><strong>Toolbox</strong></Link>
        <form role="search" onSubmit={(e) => { e.preventDefault(); if (q) go(`/search?q=${enc(q)}`); }}>
          <input aria-label="Search" placeholder="Search entities" value={q} onChange={(e) => setQ(e.target.value)} />
        </form>
        <Link to="/ops">Operations</Link>
        <NewerData current={null} />
        {user && <span className="muted">dev: {user}</span>}
      </nav>
      <main>{page}</main>
    </>
  );
}

createRoot(document.getElementById("root")!).render(<StrictMode><App /></StrictMode>);
