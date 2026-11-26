//! Loading a portal for local commands: validated manifest + fixtures → derived state.

use seikika_derive::{Derived, DerivedState, FullRecompute};
use seikika_model::Manifest;
use std::path::{Path, PathBuf};

pub struct Local {
    pub root: PathBuf,
    pub manifest: Manifest,
    pub derived: Derived,
}

/// Validate the manifest at `path`, load its fixtures and recompute. Errors are printed as text.
pub fn load(path: &Path) -> Result<Local, String> {
    let report = seikika_model::validate_portal(path).map_err(|e| e.to_string())?;
    if !report.ok() {
        let lines: Vec<String> = report.errors.iter().map(|d| format!("ERROR {d}")).collect();
        return Err(format!(
            "{}\nthe portal is invalid; fix the errors above (seikika validate)",
            lines.join("\n")
        ));
    }
    let src = seikika_model::load::load(path).map_err(|e| e.to_string())?;
    let manifest = manifest_from(&src)?;
    let store = seikika_store::load_fixtures(&src.root).map_err(|e| e.to_string())?;
    let derived = FullRecompute.recompute(&manifest, &store.snapshot());
    Ok(Local {
        root: src.root,
        manifest,
        derived,
    })
}

/// The typed manifest of a loaded source, or its diagnostics (each naming the file it is in).
pub fn manifest_from(src: &seikika_model::Source) -> Result<Manifest, String> {
    let (m, errors) = seikika_model::validate_source(src);
    match m {
        Some(m) if errors.is_empty() => Ok(m),
        _ => Err(errors
            .iter()
            .map(|d| format!("ERROR {d}"))
            .collect::<Vec<_>>()
            .join("\n")),
    }
}
