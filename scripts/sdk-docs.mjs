// Inspect the SDK docs that ship to Rust and npm consumers. Emit Seikika NDJSON records.
import { execFileSync } from "node:child_process";
import { existsSync, globSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { join, relative } from "node:path";
import { fileURLToPath } from "node:url";

const root = fileURLToPath(new URL("../", import.meta.url));
const read = path => readFileSync(path, "utf8");
const run = (program, args) => execFileSync(program, args, {
  cwd: root, encoding: "utf8", maxBuffer: 32 * 1024 * 1024,
});

function report(name, path, entryPoints, undocumented, referenceGaps) {
  console.log(JSON.stringify({
    type: "record", kind: "sdk-docs", externalId: `sdk-docs:${name}`,
    data: {
      name, path: relative(root, path), entryPoints: entryPoints.sort(),
      undocumented: [...new Set(undocumented)].sort(), referenceGaps: [...new Set(referenceGaps)].sort(),
    },
  }));
}

function rustDocs() {
  const name = "seikika-connect";
  const manifest = join(root, "crates", name, "Cargo.toml");
  const diagnostics = run("cargo", [
    "rustdoc", "--quiet", "--locked", "--package", name, "--lib", "--all-features",
    "--message-format=json", "--", "-W", "missing_docs", "-W", "rustdoc::all",
  ]).trim().split("\n").map(line => JSON.parse(line));
  const undocumented = [];
  const referenceGaps = [];
  for (const diagnostic of diagnostics) {
    if (diagnostic.reason !== "compiler-message" || diagnostic.manifest_path !== manifest) continue;
    const { message, code, spans } = diagnostic.message;
    if (!code) continue;
    const span = spans.find(candidate => candidate.is_primary);
    const text = span?.text.map(line => line.text.trim()).join(" ");
    const location = span ? `${span.file_name}:${span.line_start}` : name;
    const finding = `${location}: ${message}${text ? ` (${text})` : ""}`;
    if (code.code === "missing_docs") undocumented.push(finding);
    if (code.code.startsWith("rustdoc::")) referenceGaps.push(finding);
  }
  if (!read(manifest).includes(`documentation = "https://docs.rs/${name}"`)) {
    referenceGaps.push("Cargo.toml: link the crate reference on docs.rs");
  }
  report(name, manifest, ["seikika_connect"], undocumented, referenceGaps);
}

function npmDocs() {
  const requireFromUi = createRequire(new URL("../ui-kit/package.json", import.meta.url));
  const ts = requireFromUi("typescript");
  const { parse } = requireFromUi("vue/compiler-sfc");
  const workspaces = JSON.parse(run("pnpm", ["-r", "list", "--depth", "-1", "--json"]));

  for (const workspace of workspaces.filter(pkg => !pkg.private)) {
    const manifest = join(workspace.path, "package.json");
    const pkg = JSON.parse(read(manifest));
    const readmePath = join(workspace.path, "README.md");
    const readme = existsSync(readmePath) ? read(readmePath) : "";
    const points = [];
    const undocumented = [];
    const referenceGaps = [];

    function describe(node, label) {
      if (!node.jsDoc?.some(doc => String(doc.comment ?? "").trim())) undocumented.push(label);
    }

    function members(nodes, label) {
      for (const member of nodes) describe(member, `${label}.${member.name?.getText() ?? "signature"}`);
    }

    function checkSource(target, name, section) {
      const text = read(target);
      const slots = new Set();
      if (target.endsWith(".vue")) {
        const { descriptor, errors } = parse(text, { filename: target });
        if (errors.length) throw new Error(`${name}: ${errors.join("; ")}`);
        const source = ts.createSourceFile("props.ts", descriptor.scriptSetup?.content ?? "", ts.ScriptTarget.Latest, true);
        const inspect = node => {
          if (ts.isCallExpression(node) && node.expression.getText() === "defineProps") {
            const type = node.typeArguments?.[0];
            const props = type && (ts.isTypeLiteralNode(type) ? type : source.statements.find(
              statement => ts.isInterfaceDeclaration(statement) && statement.name.text === type.getText(),
            ));
            if (!props) throw new Error(`${name}: expected inline props or a local props interface`);
            describe(props, name);
            members(props.members, `${name} props`);
          }
          ts.forEachChild(node, inspect);
        };
        inspect(source);
        const visit = node => {
          if (node.tag === "slot") {
            if (node.props.some(prop => prop.name === "bind" && (!prop.arg || prop.arg.content === "name"))) {
              throw new Error(`${name}: document dynamic slots and extend the SDK check`);
            }
            slots.add(node.props.find(prop => prop.name === "name")?.value?.content ?? "default");
          }
          for (const child of node.children ?? []) visit(child);
        };
        if (descriptor.template?.ast) visit(descriptor.template.ast);
      } else if (target.endsWith(".astro")) {
        const frontmatter = text.match(/^---\r?\n([\s\S]*?)\r?\n---/);
        const source = ts.createSourceFile("props.ts", frontmatter?.[1] ?? "", ts.ScriptTarget.Latest, true);
        const props = source.statements.find(node => ts.isInterfaceDeclaration(node) && node.name.text === "Props");
        if (props) members(props.members, `${name} props`);
        else if (frontmatter?.[1].includes("Astro.props")) undocumented.push(`${name}: declare typed props`);
        for (const slot of text.matchAll(/<slot(?:\s+name="([\w-]+)")?\s*\/?\s*>/g)) slots.add(slot[1] ?? "default");
      } else if (target.endsWith(".css")) {
        for (const block of text.matchAll(/^\s*\.seikika-ui\s*\{([^}]*)\}/gm)) {
          for (const [, token] of block[1].matchAll(/(--[\w-]+)\s*:/g)) {
            if (!section.includes(`\`${token}\``)) referenceGaps.push(`${name}: ${token}`);
          }
        }
      } else {
        const program = ts.createProgram([target], { allowJs: true, noResolve: true, types: [] });
        const source = program.getSourceFile(target);
        const checker = program.getTypeChecker();
        const module = source && checker.getSymbolAtLocation(source);
        if (!module) throw new Error(`${name}: no module declarations in ${target}`);
        for (const symbol of checker.getExportsOfModule(module)) {
          for (const declaration of symbol.declarations ?? []) {
            const label = `${name} ${symbol.name}`;
            describe(ts.isVariableDeclaration(declaration) ? declaration.parent.parent : declaration, label);
          }
        }
        // Public module interfaces include options carried by a function signature.
        for (const node of source.statements.filter(ts.isInterfaceDeclaration)) {
          const label = `${name} ${node.name.text}`;
          describe(node, label);
          members(node.members, label);
        }
      }
      for (const slot of slots) {
        if (!section.includes(`| \`${slot}\` |`)) referenceGaps.push(`${name}: ${slot} slot`);
      }
    }

    for (const [key, entry] of Object.entries(pkg.exports ?? {})) {
      const name = pkg.name + (key === "." ? "" : key.slice(1));
      const pattern = typeof entry === "string" ? entry : entry.types ?? entry.default;
      if (!pattern?.startsWith("./")) throw new Error(`${name}: expected a package-relative export target`);
      const paths = globSync(pattern.slice(2), { cwd: workspace.path });
      if (!paths.length) referenceGaps.push(`${name}: export target ${pattern} is missing`);
      const [prefix, suffix] = pattern.slice(2).split("*");
      for (const path of paths) {
        const capture = suffix === undefined ? "" : path.slice(prefix.length, path.length - suffix.length);
        const subpath = name.replace("*", capture);
        points.push(subpath);
        if (entry.default && !existsSync(join(workspace.path, entry.default.replace("*", capture)))) {
          referenceGaps.push(`${subpath}: runtime export target ${entry.default} is missing`);
        }
        const section = readme.split(`\n### \`${subpath}\`\n`)[1]?.split(/^#{1,3} /m)[0] ?? "";
        if (!section.trim()) referenceGaps.push(`${subpath}: add a README reference section`);
        checkSource(join(workspace.path, path), subpath, section);
      }
    }
    report(pkg.name, manifest, points, undocumented, referenceGaps);
  }
}

const targets = process.argv.slice(2);
if (targets.some(target => target !== "--rust" && target !== "--npm")) {
  throw new Error("Usage: node scripts/sdk-docs.mjs [--rust] [--npm]");
}
if (!targets.length || targets.includes("--rust")) rustDocs();
if (!targets.length || targets.includes("--npm")) npmDocs();
