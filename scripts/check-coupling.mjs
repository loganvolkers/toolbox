import { execFile } from "node:child_process";
import { existsSync, globSync, readFileSync, realpathSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, isAbsolute, join, relative, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";
import { promisify } from "node:util";

const checkout = fileURLToPath(new URL("../", import.meta.url));
const rootArg = process.argv.find(arg => arg.startsWith("--root="));
const root = resolve(rootArg ? rootArg.slice("--root=".length) : checkout);
const requireFromUi = createRequire(new URL("../npm/ui/package.json", import.meta.url));
const ts = requireFromUi("typescript");
const { parse: parseVue } = requireFromUi("vue/compiler-sfc");
const read = path => readFileSync(resolve(root, path), "utf8");
const execute = promisify(execFile);
const command = async (name, args) => JSON.parse((await execute(name, args, { cwd: checkout, encoding: "utf8" })).stdout);
const inside = (path, directory) => path === directory || path.startsWith(directory + sep);
const deployment = path => [".seikika", "examples", "experiments"].some(dir => inside(path, resolve(root, dir)));
const corePackage = name => name !== "seikika-site";
const categories = ["cycles", "boundaries", "entrypoints", "deploymentData"];

function module(name, path) {
  return { name, path, edges: new Set(), findings: Object.fromEntries(categories.map(kind => [kind, new Set()])) };
}

function findCycles(modules) {
  // For each outgoing edge, find a shortest return path. Count a cyclic edge once
  // even if several paths close it, so the measure stays stable as cycles overlap.
  for (const source of modules.values()) {
    for (const target of [...source.edges].sort()) {
      const queue = [[target]];
      const seen = new Set(queue[0]);
      for (const path of queue) {
        const last = path.at(-1);
        if (last === source.name) {
          source.findings.cycles.add([source.name, ...path].join(" -> "));
          break;
        }
        for (const next of [...modules.get(last).edges].sort()) {
          if (!seen.has(next)) {
            seen.add(next);
            queue.push([...path, next]);
          }
        }
      }
    }
  }
}

const cargo = await command("cargo", ["metadata", "--locked", "--offline", "--format-version", "1", "--no-deps", "--manifest-path", join(root, "Cargo.toml")]);
const crates = cargo.packages.filter(pkg => cargo.workspace_members.includes(pkg.id));
const rust = new Map(crates.map(pkg => [pkg.name, module(pkg.name, relative(root, pkg.manifest_path))]));
const cratePaths = new Map(crates.map(pkg => [dirname(pkg.manifest_path), pkg.name]));
for (const pkg of crates) {
  const source = rust.get(pkg.name);
  if (!inside(dirname(pkg.manifest_path), resolve(root, "crates"))) {
    source.findings.boundaries.add("Rust workspace members must live under crates/");
  }
  for (const dep of pkg.dependencies) {
    if (!dep.path) continue;
    const target = cratePaths.get(dep.path);
    if (target) source.edges.add(target);
    if (!target || !inside(dep.path, resolve(root, "crates"))) source.findings.boundaries.add(`${dep.kind ?? "normal"} dependency ${dep.name} points outside the Rust workspace: ${relative(root, dep.path)}`);
  }
  const dir = dirname(pkg.manifest_path);
  const files = [...globSync("src/**/*.rs", { cwd: dir }), "build.rs"].filter(file => existsSync(join(dir, file)));
  for (const file of files) {
    // Look at literal source inclusions. Arbitrary runtime file I/O and
    // computed macro paths need review and behavior tests.
    for (const match of read(join(dir, file)).matchAll(/(?:\binclude(?:_str|_bytes)?!\s*\(\s*|#\s*\[\s*path\s*=\s*)(?:r#*)?"([^"]+)"/g)) {
      const path = resolve(dir, dirname(file), match[1]);
      if (deployment(path)) source.findings.boundaries.add(`${relative(root, join(dir, file))} includes ${relative(root, path)}`);
    }
  }
}

const packages = globSync("npm/*/package.json", { cwd: root }).sort().map(path => ({
  path, dir: resolve(root, dirname(path)), manifest: JSON.parse(read(path)),
}));
const npm = new Map(packages.map(({ path, manifest }) => [manifest.name, module(manifest.name, path)]));
if (npm.size !== packages.length || !npm.size || !rust.size) throw new Error("Expected unique, nonempty npm and Rust workspaces");

function imports(file, text) {
  const script = code => ts.preProcessFile(code, true, true).importedFiles.map(ref => ref.fileName);
  const styles = code => [...code.matchAll(/@import\s+(?:url\(\s*)?["']([^"']+)["']/g)].map(match => match[1]);
  if (file.endsWith(".css")) return styles(text);
  if (file.endsWith(".vue")) {
    const { descriptor, errors } = parseVue(text, { filename: file });
    if (errors.length) throw new Error(`Cannot parse ${file}: ${errors.join(", ")}`);
    return [descriptor.script, descriptor.scriptSetup, ...descriptor.styles].filter(Boolean).flatMap(block =>
      [...(block.src ? [block.src] : []), ...(descriptor.styles.includes(block) ? styles(block.content) : script(block.content))]);
  }
  if (file.endsWith(".astro")) {
    const frontmatter = text.match(/^---\r?\n([\s\S]*?)^---\s*$/m)?.[1] ?? "";
    return [...script(frontmatter), ...[...text.matchAll(/<script\b([^>]*)>([\s\S]*?)<\/script>/g)].flatMap(([, attrs, code]) => [
      ...script(code), ...[...attrs.matchAll(/\bsrc=["']([^"']+)["']/g)].map(match => match[1]),
    ])];
  }
  if (file.endsWith(".mdx")) {
    return script(text.replace(/^```[^\n]*\n[\s\S]*?^```\s*$/gm, ""));
  }
  return script(text);
}

function hasExport(exports, entry) {
  return typeof exports === "string" ? entry === "." : Object.keys(exports ?? {}).some(key => {
    if (!key.includes("*")) return key === entry;
    const [prefix, suffix] = key.split("*");
    return entry.startsWith(prefix) && entry.endsWith(suffix) && entry.length > prefix.length + suffix.length;
  });
}

// Astro resolves tsconfig path aliases in its build. Ask TypeScript for the same
// mapping, including inherited configs, before applying the filesystem rules.
const aliasOptions = new Map();
const formatDiagnostic = diagnostic => ts.flattenDiagnosticMessageText(diagnostic.messageText, "\n");
function resolveAlias(spec, file) {
  const config = ts.findConfigFile(dirname(file), ts.sys.fileExists);
  if (!config) return;
  if (!aliasOptions.has(config)) {
    const source = ts.readConfigFile(config, ts.sys.readFile);
    if (source.error) throw new Error(`${relative(root, config)}: ${formatDiagnostic(source.error)}`);
    const parsed = ts.parseJsonConfigFileContent(source.config, ts.sys, dirname(config), undefined, config);
    const errors = parsed.errors.filter(error => error.code !== 18003); // Source can be all Astro or Vue.
    if (errors.length) throw new Error(`${relative(root, config)}: ${errors.map(formatDiagnostic).join("; ")}`);
    aliasOptions.set(config, parsed.options);
  }
  const options = aliasOptions.get(config);
  const explicit = Object.keys(options.paths ?? {}).some(pattern => {
    if (!pattern.includes("*")) return pattern === spec;
    const [start, end] = pattern.split("*");
    return spec.length >= start.length + end.length && spec.startsWith(start) && spec.endsWith(end);
  });
  if (!explicit && !options.baseUrl) return;
  // TypeScript probes declarations for arbitrary extensions. Use actual source
  // files for Vue, Astro, MDX and styles when there is no separate declaration.
  const sourcePath = file => file.replace(/\.d\.(vue|astro|mdx|css)\.ts$/, ".$1");
  const host = { ...ts.sys, fileExists: file => ts.sys.fileExists(file) || ts.sys.fileExists(sourcePath(file)) };
  const { resolvedModule } = ts.resolveModuleName(spec, file, {
    ...options, moduleResolution: ts.ModuleResolutionKind.Bundler, allowArbitraryExtensions: true,
  }, host);
  if (!resolvedModule || resolvedModule.isExternalLibraryImport) return explicit ? null : undefined;
  const resolved = resolvedModule.resolvedFileName;
  return existsSync(resolved) ? resolved : sourcePath(resolved);
}

// Custom aliases can override Astro's tsconfig resolution. Reject inline Vite
// alias maps and require application aliases to use the checked mechanism.
function hasViteAliases(file) {
  const code = ts.createSourceFile(file, read(file), ts.ScriptTarget.Latest, true);
  const key = node => ts.isIdentifier(node) || ts.isStringLiteral(node) ? node.text : undefined;
  function allowedRendererAliases(prop) {
    if (!ts.isPropertyAssignment(prop) || !ts.isObjectLiteralExpression(prop.initializer)) return false;
    // The report integration pins Astro's renderer exports to its own installed
    // Vue integration. Each alias must resolve that exact same package export.
    return prop.initializer.properties.every(entry => {
      if (!ts.isPropertyAssignment(entry)) return false;
      const name = key(entry.name);
      return /^@astrojs\/vue\/(client|server)\.js$/.test(name ?? "")
        && entry.initializer.getText(code).replace(/\s+/g, "").replaceAll("'", '"')
          === `fileURLToPath(import.meta.resolve(${JSON.stringify(name)}))`;
    });
  }
  function visit(node) {
    if (ts.isPropertyAssignment(node) && key(node.name) === "resolve" && ts.isObjectLiteralExpression(node.initializer)) {
      if (node.initializer.properties.some(prop => prop.name && key(prop.name) === "alias" && !allowedRendererAliases(prop))) {
        return true;
      }
    }
    return ts.forEachChild(node, visit);
  }
  return visit(code);
}

for (const pkg of packages) {
  const source = npm.get(pkg.manifest.name);
  for (const config of globSync("{astro.config,vite.config,integration}.{js,ts,mjs,mts,cjs,cts}", { cwd: pkg.dir })) {
    const path = join(pkg.dir, config);
    if (hasViteAliases(path)) {
      source.findings.entrypoints.add(`${relative(root, path)} defines Vite resolve.alias. Declare application aliases in tsconfig paths.`);
    }
  }
  const declared = new Set();
  const runtime = new Set();
  for (const section of ["dependencies", "optionalDependencies", "peerDependencies", "devDependencies"]) {
    for (const [name, spec] of Object.entries(pkg.manifest[section] ?? {})) {
      if (section !== "devDependencies") runtime.add(name);
      if (/^(?:file:|link:|workspace:\.{1,2}\/)/.test(spec)) {
        source.findings.boundaries.add(`${section} ${name}: use a named workspace dependency or a registry dependency`);
      }
      const alias = spec.match(/^(?:npm:|workspace:)((?:@[^/]+\/)?[^@/]+)@/)?.[1];
      const target = alias ?? name;
      if (!npm.has(target)) continue;
      declared.add(name);
      source.edges.add(target);
      if (corePackage(source.name) && !corePackage(target)) source.findings.boundaries.add(`${section} ${name} points at the documentation host`);
      if (!spec.startsWith("workspace:")) source.findings.entrypoints.add(`${section} ${name} must use the workspace: protocol`);
    }
  }

  const files = globSync(["src/**/*.{ts,tsx,mts,cts,js,jsx,mjs,cjs,vue,astro,mdx,css}", "integration.{mjs,mts}"], { cwd: pkg.dir });
  for (const file of files.sort()) {
    const path = join(pkg.dir, file);
    const label = relative(root, path);
    const text = read(path);
    for (const spec of imports(file, text)) {
      const bare = spec.split("?")[0];
      const to = bare.startsWith(".") || isAbsolute(bare) ? resolve(dirname(path), bare) : resolveAlias(bare, path);
      if (to === null) {
        source.findings.entrypoints.add(`${label} uses an unresolved path alias: ${spec}`);
        continue;
      }
      if (to) {
        const real = existsSync(to) ? realpathSync(to) : to;
        if (corePackage(source.name) && (deployment(to) || deployment(real))) {
          source.findings.boundaries.add(`${label} imports deployment data: ${spec}`);
        }
        const other = packages.find(candidate => candidate !== pkg && inside(real, candidate.dir));
        if (other) {
          if (corePackage(source.name) && !corePackage(other.manifest.name)) source.findings.boundaries.add(`${label} imports the documentation host: ${spec}`);
          source.edges.add(other.manifest.name);
          source.findings.entrypoints.add(`${label} bypasses the package exports: ${spec}`);
        }
        continue;
      }
      const target = packages.find(({ manifest }) => spec === manifest.name || spec.startsWith(manifest.name + "/"));
      if (!target || target === pkg) continue;
      source.edges.add(target.manifest.name);
      const subpath = spec.slice(target.manifest.name.length);
      if (!declared.has(target.manifest.name) || !runtime.has(target.manifest.name)) {
        source.findings.entrypoints.add(`${label} imports ${spec} without a runtime workspace declaration`);
      }
      if (!hasExport(target.manifest.exports, subpath ? "." + subpath : ".")) {
        source.findings.entrypoints.add(`${label} imports an unexported entry point: ${spec}`);
      }
    }
    if (corePackage(source.name)) {
      for (const match of text.matchAll(/https?:\/\/(?:demo\.seikika\.com(?=[/:?#\s"'<>\x60]|$)|seikika\.com\/demo(?=[/?#\s"'<>\x60]|$))[^\s"'<>\x60]*|#scorecard\/[a-z][\w-]+/g)) {
        source.findings.deploymentData.add(`${label} pins a deployment link: ${match[0]}`);
      }
    }
  }
}

findCycles(rust);
findCycles(npm);
const report = [...rust.values(), ...npm.values()].sort((a, b) => a.path.localeCompare(b.path)).map(item => ({
  name: item.name, path: item.path, dependencies: [...item.edges].sort(),
  ...Object.fromEntries(categories.map(kind => [kind, [...item.findings[kind]].sort()])),
}));
if (process.argv.includes("--ndjson")) {
  for (const data of report) {
    console.log(JSON.stringify({ type: "record", kind: "module-coupling", externalId: `module-coupling:${data.path}`, data }));
  }
} else {
  let failures = 0;
  for (const item of report) for (const kind of categories) for (const finding of item[kind]) {
    console.error(`${item.path} [${kind}] ${finding}`);
    failures++;
  }
  console.log(`${report.length} modules, ${report.reduce((sum, item) => sum + item.dependencies.length, 0)} internal edges, ${failures} findings`);
  if (failures) process.exitCode = 1;
}
