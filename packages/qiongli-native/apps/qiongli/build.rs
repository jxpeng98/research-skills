use std::env;
use std::error::Error;
use std::fs;
use std::io::{self, Read as _};
use std::path::PathBuf;

use qiongli_content::{
    QIONGLI_CORE_RESOURCE_PACK_LOCK_V1, ResourcePackLockV1, build_resource_pack,
    collect_canonical_sources,
};
use qiongli_platform::{
    MAX_NATIVE_RELEASE_AUTHORITY_BYTES, NativeReleaseAuthority,
    ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE, ZOTERO_COMPANION_PACKAGED_XPI_FILE,
    ZOTERO_COMPANION_SOURCE_PATHS, ZoteroCompanionSourceEntry, compose_zotero_companion_artifact,
};

const PACK_FILE: &str = "qiongli-core.qlpack";
const DIGEST_FILE: &str = "qiongli-core.qlpack.sha256";
const RELEASE_AUTHORITY_FILE: &str = "qiongli-native-release-authority.json";
const RELEASE_AUTHORITY_ENV: &str = "QIONGLI_NATIVE_RELEASE_AUTHORITY_FILE";
const SOURCE_COMMIT_FILE: &str = "qiongli-native-source-commit.txt";
const SOURCE_COMMIT_ENV: &str = "QIONGLI_NATIVE_SOURCE_COMMIT";
const MACOS_TEAM_ID_FILE: &str = "qiongli-macos-team-id.txt";
const MACOS_TEAM_ID_ENV: &str = "QIONGLI_MACOS_EXPECTED_TEAM_ID";

fn main() {
    if let Err(error) = build_embedded_assets() {
        panic!("failed to build verified Qiongli embedded assets: {error}");
    }
    #[cfg(feature = "desktop")]
    build_desktop_resources();
}

#[cfg(feature = "desktop")]
fn build_desktop_resources() {
    let mut attributes = tauri_build::Attributes::new();
    if env::var("CARGO_CFG_TARGET_OS").as_deref() == Ok("windows")
        && env::var("CARGO_CFG_TARGET_ENV").as_deref() == Ok("msvc")
    {
        // Tauri mock IPC also links Common Controls v6. Embed its manifest in
        // every target, including lib tests, instead of only the application.
        // https://github.com/tauri-apps/tauri/issues/13419
        let manifest = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("windows-app-manifest.xml");
        println!("cargo:rerun-if-changed={}", manifest.display());
        println!("cargo:rustc-link-arg=/MANIFEST:EMBED");
        println!("cargo:rustc-link-arg=/MANIFESTINPUT:{}", manifest.display());
        attributes = attributes
            .windows_attributes(tauri_build::WindowsAttributes::new_without_app_manifest());
    }
    tauri_build::try_build(attributes).expect("failed to build Qiongli Tauri resources");
}

fn build_embedded_assets() -> Result<(), Box<dyn Error>> {
    build_embedded_pack()?;
    build_embedded_release_authority()?;
    build_embedded_source_commit()?;
    build_embedded_macos_team_id()?;
    build_embedded_zotero_companion()?;
    Ok(())
}

fn build_embedded_zotero_companion() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let companion_root = manifest_dir.join("../../../qiongli-zotero-companion");
    let mut sources = Vec::with_capacity(ZOTERO_COMPANION_SOURCE_PATHS.len());
    for relative in ZOTERO_COMPANION_SOURCE_PATHS {
        let path = companion_root.join(relative);
        println!("cargo:rerun-if-changed={}", path.display());
        let metadata = fs::symlink_metadata(&path)?;
        if metadata.file_type().is_symlink() || !metadata.is_file() || metadata.len() == 0 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "Zotero Companion source entry is not a regular file",
            )
            .into());
        }
        sources.push((relative, fs::read(path)?));
    }
    let entries = sources
        .iter()
        .map(|(path, bytes)| ZoteroCompanionSourceEntry { path, bytes })
        .collect::<Vec<_>>();
    let artifact = compose_zotero_companion_artifact(&entries)?;
    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo OUT_DIR is unavailable"))?;
    fs::write(
        out_dir.join(ZOTERO_COMPANION_PACKAGED_XPI_FILE),
        artifact.xpi_bytes(),
    )?;
    fs::write(
        out_dir.join(ZOTERO_COMPANION_ARTIFACT_MANIFEST_FILE),
        artifact.manifest_bytes(),
    )?;
    Ok(())
}

fn build_embedded_macos_team_id() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed={MACOS_TEAM_ID_ENV}");
    let team_id = match env::var_os(MACOS_TEAM_ID_ENV) {
        None => String::new(),
        Some(value) => {
            let value = value.to_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "macOS Team ID is not valid UTF-8",
                )
            })?;
            if value.len() != 10
                || !value
                    .bytes()
                    .all(|byte| byte.is_ascii_uppercase() || byte.is_ascii_digit())
            {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "macOS Team ID must be 10 uppercase ASCII letters or digits",
                )
                .into());
            }
            value.to_string()
        }
    };
    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo OUT_DIR is unavailable"))?;
    fs::write(out_dir.join(MACOS_TEAM_ID_FILE), team_id.as_bytes())?;
    Ok(())
}

fn build_embedded_source_commit() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed={SOURCE_COMMIT_ENV}");
    let source_commit = match env::var_os(SOURCE_COMMIT_ENV) {
        None => String::new(),
        Some(value) => {
            let value = value.to_str().ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "native source commit is not valid UTF-8",
                )
            })?;
            if !valid_source_commit(value) {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "native source commit must be 40 or 64 lowercase hexadecimal characters",
                )
                .into());
            }
            value.to_string()
        }
    };
    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo OUT_DIR is unavailable"))?;
    fs::write(out_dir.join(SOURCE_COMMIT_FILE), source_commit.as_bytes())?;
    Ok(())
}

fn valid_source_commit(value: &str) -> bool {
    matches!(value.len(), 40 | 64)
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn build_embedded_pack() -> Result<(), Box<dyn Error>> {
    let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let content_root = manifest_dir.join("../../../../content");
    let lock_path =
        manifest_dir.join("../../crates/qiongli-content/resources/qiongli-core.lock.json");
    println!("cargo:rerun-if-changed={}", content_root.display());
    println!("cargo:rerun-if-changed={}", lock_path.display());

    let lock = ResourcePackLockV1::from_json(QIONGLI_CORE_RESOURCE_PACK_LOCK_V1)?;
    if lock.to_canonical_json()?.as_slice() != QIONGLI_CORE_RESOURCE_PACK_LOCK_V1.as_bytes() {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "qiongli-core resource-pack lock must use canonical JSON",
        )
        .into());
    }
    let resources = collect_canonical_sources(&content_root)?;
    let built = build_resource_pack(&lock.metadata()?, &resources)?;
    lock.verify(&built)?;

    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo OUT_DIR is unavailable"))?;
    fs::write(out_dir.join(PACK_FILE), built.core_bytes())?;
    fs::write(out_dir.join(DIGEST_FILE), lock.pack_sha256.as_bytes())?;
    Ok(())
}

fn build_embedded_release_authority() -> Result<(), Box<dyn Error>> {
    println!("cargo:rerun-if-env-changed={RELEASE_AUTHORITY_ENV}");
    let bytes = match env::var_os(RELEASE_AUTHORITY_ENV) {
        None => Vec::new(),
        Some(value) if value.is_empty() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "native release authority path is empty",
            )
            .into());
        }
        Some(value) => {
            let path = PathBuf::from(value);
            println!("cargo:rerun-if-changed={}", path.display());
            let file = fs::File::open(path)?;
            let limit = u64::try_from(MAX_NATIVE_RELEASE_AUTHORITY_BYTES)
                .map_err(|_| io::Error::other("native release authority limit is invalid"))?;
            let mut bounded = file.take(limit.saturating_add(1));
            let mut input = Vec::new();
            bounded.read_to_end(&mut input)?;
            if input.len() > MAX_NATIVE_RELEASE_AUTHORITY_BYTES {
                return Err(io::Error::new(
                    io::ErrorKind::InvalidData,
                    "native release authority is too large",
                )
                .into());
            }
            if input.last() == Some(&b'\n') {
                input.pop();
                if input.last() == Some(&b'\r') {
                    input.pop();
                }
            }
            let authority = NativeReleaseAuthority::from_json(&input)?;
            authority.validate_product_version(env!("CARGO_PKG_VERSION"))?;
            authority.to_canonical_json()?
        }
    };
    let out_dir = env::var_os("OUT_DIR")
        .map(PathBuf::from)
        .ok_or_else(|| io::Error::new(io::ErrorKind::NotFound, "Cargo OUT_DIR is unavailable"))?;
    fs::write(out_dir.join(RELEASE_AUTHORITY_FILE), bytes)?;
    Ok(())
}
