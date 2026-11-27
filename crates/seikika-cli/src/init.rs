//! Interactive, offline scaffolding for the first-party connectors and standards packages.

use serde::{Deserialize, Serialize};
use serde_json::{Map, Value, json};
use std::collections::BTreeSet;
use std::fs::{self, OpenOptions};
use std::io::{self, BufRead, ErrorKind, StdinLock, StdoutLock, Write};
use std::path::Path;

const DEFAULT_NAME: &str = "portal";
const DEFAULT_OWNER: &str = "group:maintainers";
const DEFAULT_CONNECTOR: &str = "workspace";

#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Catalog {
    package_revision: String,
    connectors: Vec<Connector>,
    packages: Vec<Package>,
}

#[derive(Deserialize)]
struct Choice {
    name: String,
    description: String,
    #[serde(default)]
    inputs: Vec<Input>,
}

#[derive(Deserialize)]
struct Connector {
    #[serde(flatten)]
    choice: Choice,
    #[serde(default = "in_image")]
    runtime: String,
    #[serde(default)]
    config: Map<String, Value>,
    records: Vec<String>,
    kinds: Vec<Value>,
    #[serde(default)]
    scorecards: Vec<Value>,
}

fn in_image() -> String {
    "in-image".into()
}

#[derive(Deserialize)]
struct Package {
    #[serde(flatten)]
    choice: Choice,
    #[serde(default)]
    uses: Vec<String>,
    workspace: Option<Workspace>,
}

#[derive(Deserialize)]
struct Workspace {
    config: Map<String, Value>,
    records: Vec<String>,
}

#[derive(Deserialize)]
struct Input {
    key: String,
    label: String,
    default: Option<String>,
    #[serde(default, rename = "type")]
    kind: InputType,
}

#[derive(Default, Deserialize)]
#[serde(rename_all = "kebab-case")]
enum InputType {
    #[default]
    Text,
    Name,
    KindName,
    Owner,
    PositiveInteger,
    Regex,
    StringList,
}

impl InputType {
    fn parse(&self, text: &str) -> Result<Value, &'static str> {
        match self {
            Self::Text if text.is_empty() => Err("Enter a value."),
            Self::Name => matching(text, "^[a-z][a-z0-9-]*$", "Enter a name like my-portal."),
            Self::KindName => {
                matching(text, "^[A-Z][A-Za-z0-9]*$", "Enter a kind like PythonFile.")
            }
            Self::Owner => matching(
                text,
                "^[a-z][a-z0-9]*:[a-z0-9][a-z0-9._/-]*$",
                "Enter an owner like group:platform.",
            ),
            Self::PositiveInteger => text
                .parse::<i64>()
                .ok()
                .filter(|n| *n > 0)
                .map(Value::from)
                .ok_or("Enter a positive whole number."),
            Self::Regex => regex::Regex::new(text)
                .ok()
                .filter(|_| !text.is_empty())
                .map(|_| Value::from(text))
                .ok_or("Enter a valid Rust regular expression."),
            Self::StringList => serde_json::from_str::<Vec<String>>(text)
                .ok()
                .filter(|items| !items.is_empty() && items.iter().all(|s| !s.trim().is_empty()))
                .map(Value::from)
                .ok_or("Enter a JSON array of strings, for example [\"src/**/*.rs\"]."),
            Self::Text => Ok(Value::from(text)),
        }
    }
}

fn matching(text: &str, pattern: &str, hint: &'static str) -> Result<Value, &'static str> {
    regex::Regex::new(pattern)
        .expect("valid prompt pattern")
        .is_match(text)
        .then(|| Value::from(text))
        .ok_or(hint)
}

struct Prompt {
    input: StdinLock<'static>,
    output: StdoutLock<'static>,
}

impl Prompt {
    fn ask(&mut self, label: &str, default: Option<&str>) -> io::Result<String> {
        write!(self.output, "{label}")?;
        if let Some(default) = default {
            write!(self.output, " [{default}]")?;
        }
        write!(self.output, ": ")?;
        self.output.flush()?;
        let mut line = String::new();
        if self.input.read_line(&mut line)? == 0 {
            return Err(io::Error::new(
                ErrorKind::UnexpectedEof,
                "setup canceled at end of input; run `seikika init --yes` to use the defaults",
            ));
        }
        let text = line.trim();
        Ok(if text.is_empty() {
            default.unwrap_or_default().to_owned()
        } else {
            text.to_owned()
        })
    }

    fn value(&mut self, label: &str, default: Option<&str>, kind: &InputType) -> io::Result<Value> {
        loop {
            match kind.parse(&self.ask(label, default)?) {
                Ok(value) => return Ok(value),
                Err(hint) => writeln!(self.output, "{hint}")?,
            }
        }
    }

    fn choose<T>(
        &mut self,
        title: &str,
        entries: &[T],
        choice: impl Fn(&T) -> &Choice,
        default: &str,
    ) -> io::Result<Vec<usize>> {
        writeln!(self.output, "\n{title}")?;
        for (i, entry) in entries.iter().enumerate() {
            let entry = choice(entry);
            writeln!(
                self.output,
                "  {}. {} — {}",
                i + 1,
                entry.name,
                entry.description
            )?;
        }
        loop {
            let text = self.ask(
                "Choose numbers or names, separated by commas, or type none",
                Some(default),
            )?;
            if text.eq_ignore_ascii_case("none") {
                return Ok(Vec::new());
            }
            let selected: Option<Vec<_>> = text
                .split(',')
                .map(|token| {
                    let token = token.trim();
                    entries.iter().enumerate().position(|(i, entry)| {
                        token.eq_ignore_ascii_case(&choice(entry).name)
                            || token.parse::<usize>() == Ok(i + 1)
                    })
                })
                .collect();
            if let Some(mut selected) = selected {
                selected.sort_unstable();
                selected.dedup();
                return Ok(selected);
            }
            writeln!(self.output, "Choose from the options above, or type none.")?;
        }
    }

    fn settings(&mut self, choice: &Choice) -> io::Result<Map<String, Value>> {
        if !choice.inputs.is_empty() {
            writeln!(self.output, "\nConfigure {}", choice.name)?;
        }
        choice
            .inputs
            .iter()
            .map(|input| {
                self.value(&input.label, input.default.as_deref(), &input.kind)
                    .map(|value| (input.key.clone(), value))
            })
            .collect()
    }

    fn choices(&mut self, catalog: &Catalog) -> io::Result<(Draft, Vec<usize>, Vec<usize>)> {
        let name = self.value("Portal name", Some(DEFAULT_NAME), &InputType::Name)?;
        let owner = self.value("Portal owner", Some(DEFAULT_OWNER), &InputType::Owner)?;
        let connectors = self.choose(
            "Connectors collect data",
            &catalog.connectors,
            |c| &c.choice,
            DEFAULT_CONNECTOR,
        )?;
        let packages = self.choose(
            "Standards packages add reusable checks",
            &catalog.packages,
            |p| &p.choice,
            "none",
        )?;
        Ok((Draft::new(name, owner), connectors, packages))
    }
}

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Draft {
    api_version: &'static str,
    portal: Value,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    connectors: Vec<Value>,
    kinds: Vec<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    scorecards: Vec<Value>,
    #[serde(skip_serializing_if = "Vec::is_empty")]
    include: Vec<Value>,
}

impl Draft {
    fn new(name: Value, owner: Value) -> Self {
        Self {
            api_version: "seikika.com/v1alpha1",
            portal: json!({"name": name, "owner": owner, "seikika": concat!(env!("CARGO_PKG_VERSION_MAJOR"), ".", env!("CARGO_PKG_VERSION_MINOR"))}),
            connectors: Vec::new(),
            kinds: Vec::new(),
            scorecards: Vec::new(),
            include: Vec::new(),
        }
    }

    fn add_connector(&mut self, connector: &Connector, settings: Map<String, Value>) {
        let name = &connector.choice.name;
        let mut config = Value::Object(connector.config.clone());
        for (key, value) in settings {
            *config.pointer_mut(&key).expect("catalog config path") = value;
        }
        let mut declaration = json!({
            "name": name, "package": format!("builtin:{name}"), "runtime": connector.runtime,
            "records": connector.records,
        });
        if !connector.config.is_empty() {
            declaration["config"] = config;
        }
        self.connectors.push(declaration);
        self.kinds.extend(connector.kinds.iter().cloned());
        self.scorecards
            .extend(connector.scorecards.iter().cloned().map(|mut card| {
                card["owner"] = self.portal["owner"].clone();
                card
            }));
    }

    fn render(&self, manifest: &Path) -> Result<String, String> {
        // Package fragments are resolved by `install` and validated when the portal is loaded.
        let mut doc = serde_json::to_value(self).map_err(|e| e.to_string())?;
        doc.as_object_mut()
            .expect("manifest object")
            .remove("include");
        let (_, errors) = seikika_model::validate_manifest(&doc, &manifest.display().to_string());
        if !errors.is_empty() {
            return Err(errors
                .iter()
                .map(ToString::to_string)
                .collect::<Vec<_>>()
                .join("\n"));
        }
        serde_norway::to_string(self).map_err(|e| e.to_string())
    }
}

fn write_new(path: &Path, manifest: &Path, text: &str) -> io::Result<()> {
    fs::create_dir_all(path)?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(manifest)?;
    let result = file.write_all(text.as_bytes());
    drop(file);
    if result.is_err() {
        let _ = fs::remove_file(manifest);
    }
    result
}

fn check_destination(path: &Path) -> Result<(), String> {
    for name in [
        "seikika.yaml",
        "seikika.yml",
        "seikika.json",
        "seikika.config.ts",
    ] {
        let existing = path.join(name);
        match fs::symlink_metadata(&existing) {
            Ok(_) => {
                return Err(format!(
                    "{} already exists; choose a new portal directory",
                    existing.display()
                ));
            }
            Err(e) if e.kind() == ErrorKind::NotFound => (),
            Err(e) => return Err(format!("{}: {e}", existing.display())),
        }
    }
    Ok(())
}

/// Write a new portal manifest from the wizard or from the Workspace defaults.
pub fn run(path: &Path, yes: bool) -> Result<(), Box<dyn std::error::Error>> {
    check_destination(path)?;
    let mut prompt = Prompt {
        input: io::stdin().lock(),
        output: io::stdout().lock(),
    };
    let catalog: Catalog = serde_norway::from_str(include_str!("init-catalog.yaml"))
        .map_err(|e| format!("could not read the starter catalog: {e}"))?;
    let workspace = catalog
        .connectors
        .iter()
        .position(|c| c.choice.name == DEFAULT_CONNECTOR)
        .expect("Workspace in catalog");
    let (mut draft, mut selected, packages) = if yes {
        (
            Draft::new(json!(DEFAULT_NAME), json!(DEFAULT_OWNER)),
            vec![workspace],
            Vec::new(),
        )
    } else {
        prompt.choices(&catalog)?
    };

    let mut connectors = catalog.connectors;
    let mut binaries = BTreeSet::new();
    let mut package_names = Vec::new();
    for i in packages {
        let package = &catalog.packages[i];
        let name = &package.choice.name;
        if let Some(additions) = &package.workspace {
            if !selected.contains(&workspace) {
                writeln!(prompt.output, "Adding Workspace for {name}.")?;
                selected.push(workspace);
            }
            connectors[workspace]
                .config
                .extend(additions.config.clone());
            connectors[workspace]
                .records
                .extend(additions.records.iter().cloned());
        }
        let settings = prompt.settings(&package.choice)?;
        let reference = format!(
            "github:seikika/seikika/packages/{name}@{}",
            catalog.package_revision
        );
        let mut include = json!({"package": reference});
        if !settings.is_empty() {
            include["with"] = Value::Object(settings);
        }
        draft.include.push(include);
        binaries.extend(package.uses.iter().map(String::as_str));
        package_names.push(name.as_str());
    }
    for i in selected {
        let connector = &connectors[i];
        draft.add_connector(connector, prompt.settings(&connector.choice)?);
        binaries.insert(connector.choice.name.as_str());
    }
    if draft.kinds.is_empty() {
        draft.kinds.push(json!({
            "name": "Service", "identityFrom": ["native"],
            "fields": {"title": {"type": "string"}}, "sources": {"default": ["native"]},
        }));
    }

    let manifest = path.join("seikika.yaml");
    let text = draft.render(&manifest)?;
    check_destination(path)?;
    write_new(path, &manifest, &text).map_err(|e| format!("{}: {e}", manifest.display()))?;
    writeln!(prompt.output, "\nCreated {}", manifest.display())?;
    next_steps(&mut prompt.output, path, &draft, &binaries, &package_names)?;
    Ok(())
}

fn next_steps(
    output: &mut impl Write,
    path: &Path,
    draft: &Draft,
    binaries: &BTreeSet<&str>,
    packages: &[&str],
) -> io::Result<()> {
    if !binaries.is_empty() {
        writeln!(
            output,
            "Install the matching connector binaries and follow their setup guides."
        )?;
        for name in binaries {
            writeln!(
                output,
                "  seikika-connector-{name}  https://seikika.com/connectors/{name}/"
            )?;
        }
    }
    let arg = shell_path(path);
    if !packages.is_empty() {
        writeln!(
            output,
            "Fetch the selected packages with `seikika install{arg}`."
        )?;
        for name in packages {
            writeln!(output, "  https://seikika.com/packages/{name}/")?;
        }
    }
    writeln!(output, "Check the manifest with `seikika validate{arg}`.")?;
    if !draft.connectors.is_empty() || !draft.include.is_empty() {
        writeln!(output, "Check dependencies with `seikika doctor{arg}`.")?;
    }
    if !draft.scorecards.is_empty() || !draft.include.is_empty() {
        writeln!(output, "Score the checkout with `seikika report{arg}`.")?;
    }
    writeln!(output, "Start the portal with `seikika serve{arg}`.")
}

fn shell_path(path: &Path) -> String {
    let text = path.to_string_lossy();
    if path == Path::new(".") {
        String::new()
    } else if text
        .bytes()
        .all(|b| b.is_ascii_alphanumeric() || b"._/-".contains(&b))
        && !text.starts_with('-')
    {
        format!(" {text}")
    } else {
        format!(" -- '{}'", text.replace('\'', "'\\''"))
    }
}
