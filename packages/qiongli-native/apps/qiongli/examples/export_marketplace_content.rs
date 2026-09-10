//! Export the CLI's verified embedded profile for registry Plugin packaging.
#![allow(clippy::disallowed_methods)]

use std::{env, error::Error, fs, path::PathBuf};

use qiongli_content::project_profile;
use serde_json::json;

fn main() -> Result<(), Box<dyn Error>> {
    let out = PathBuf::from(env::args_os().nth(1).ok_or("output directory required")?);
    let commit = env::var("QIONGLI_NATIVE_SOURCE_COMMIT")?;
    if commit.len() != 40
        || !commit
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
    {
        return Err("exact release source commit required".into());
    }
    let content = qiongli::embedded_content()?;
    let pack = content.pack();
    let resources = project_profile(pack, "marketplace-lite", None)?;
    fs::create_dir(&out)?;
    let mut entries = Vec::new();
    for resource in resources {
        let path = out.join(resource.path());
        fs::create_dir_all(path.parent().ok_or("resource parent required")?)?;
        fs::write(path, resource.bytes())?;
        entries.push(json!({
            "path": resource.path(), "size_bytes": resource.size_bytes(),
            "sha256": resource.canonical_sha256(),
        }));
    }
    fs::write(
        out.join(".qiongli-marketplace-export.json"),
        serde_json::to_vec_pretty(&json!({
            "schema_version": 1, "version": env!("CARGO_PKG_VERSION"),
            "source_commit": commit, "content_source_commit": pack.manifest().source_commit,
            "pack_sha256": pack.pack_sha256(),
            "pack_manifest_json": String::from_utf8(serde_json_canonicalizer::to_vec(pack.manifest())?)?,
            "content_root_sha256": pack.manifest().content_root_sha256,
            "entries": entries,
        }))?,
    )?;
    Ok(())
}
