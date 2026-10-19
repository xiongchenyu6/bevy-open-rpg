//! [Tar](https://en.wikipedia.org/wiki/Tar_%28computing%29) archive container format for [paperdoll](https://github.com/fralonra/paperdoll).
//!
//! # Examples
//!
//! ```no_run
//! use paperdoll_tar::paperdoll;
//!
//! let factory = paperdoll::PaperdollFactory::default();
//!
//! paperdoll_tar::save(&mut factory.to_manifest(), "/path/to/save/your.ppd");
//!
//! let factory = paperdoll_tar::load("/path/to/save/your.ppd").unwrap();
//! ```

use anyhow::{Context, Result};
pub use paperdoll;
use paperdoll::{ImageData, Manifest, PaperdollFactory};
#[cfg(not(target_arch = "wasm32"))]
use std::fs::write;
use std::{
    collections::BTreeMap,
    fs::File,
    io::Read,
    path::{Component, Path, PathBuf},
};
use tar::Archive;
#[cfg(not(target_arch = "wasm32"))]
use tar::Builder;

/// The file extension.
pub const EXTENSION_NAME: &'static str = "ppd";

/// The file name of the manifest file saved in the `ppd` file.
pub const FILE_NAME_MANIFEST: &'static str = "manifest.yml";

#[cfg(not(target_arch = "wasm32"))]
const FILE_NAME_TEMP_DIR: &'static str = "paperdoll_ppd_";

/// Loads a paperdoll project from the path of a `ppd` file.
pub fn load<P>(path: P) -> Result<PaperdollFactory>
where
    P: AsRef<Path>,
{
    read(File::open(&path)?)
}

/// Reads a paperdoll project from a reader containing the bytes of a `ppd` file.
pub fn read<R>(r: R) -> Result<PaperdollFactory>
where
    R: Read,
{
    // Keep the archive entirely in memory: browser targets have no temp directory.
    let mut files = BTreeMap::new();
    for entry in Archive::new(r).entries()? {
        let mut entry = entry?;
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let path = archive_path(&entry.path()?);
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes)?;
        files.insert(path, bytes);
    }
    let manifest_bytes = files
        .get(Path::new(FILE_NAME_MANIFEST))
        .context("paperdoll archive is missing manifest.yml")?;
    let mut manifest: Manifest = serde_yaml::from_slice(manifest_bytes)?;
    for doll in &mut manifest.dolls {
        fill_image(&doll.path, &mut doll.image, &files)?;
    }
    for fragment in &mut manifest.fragments {
        fill_image(&fragment.path, &mut fragment.image, &files)?;
    }
    PaperdollFactory::from_manifest(manifest)
}

fn archive_path(path: &Path) -> PathBuf {
    path.components()
        .filter(|part| !matches!(part, Component::CurDir))
        .collect()
}

fn fill_image(path: &str, image: &mut ImageData, files: &BTreeMap<PathBuf, Vec<u8>>) -> Result<()> {
    if path.is_empty() {
        return Ok(());
    }
    let bytes = files
        .get(&archive_path(Path::new(path)))
        .with_context(|| format!("paperdoll archive is missing image {path}"))?;
    let decoded = image::load_from_memory(bytes)?.into_rgba8();
    image.width = decoded.width();
    image.height = decoded.height();
    image.pixels = decoded.into_vec();
    Ok(())
}

/// Saves a `ppd` file using the given manifest to the path.
#[cfg(not(target_arch = "wasm32"))]
pub fn save<P>(manifest: &mut Manifest, path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    let temp_dir = tempfile::Builder::new()
        .prefix(FILE_NAME_TEMP_DIR)
        .tempdir()?;

    for doll in &mut manifest.dolls {
        if doll.image.is_empty() {
            continue;
        }

        let extension = (!doll.path.is_empty())
            .then(|| {
                PathBuf::from(&doll.path)
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_string())
            })
            .flatten()
            .map(|ext| {
                // Saving WebP is not supported
                if &ext == "webp" {
                    "png".to_owned()
                } else {
                    ext
                }
            })
            .unwrap_or("png".to_owned());

        let filename = format!("doll_{}.{}", doll.id(), extension);

        let img_path = temp_dir.path().clone().join(&filename);

        doll.path = filename;

        image::save_buffer(
            img_path,
            &doll.image.pixels,
            doll.image.width,
            doll.image.height,
            image::ColorType::Rgba8,
        )?;
    }

    for fragment in &mut manifest.fragments {
        if fragment.image.is_empty() {
            continue;
        }

        let extension = (!fragment.path.is_empty())
            .then(|| {
                PathBuf::from(&fragment.path)
                    .extension()
                    .map(|ext| ext.to_string_lossy().to_string())
            })
            .flatten()
            .map(|ext| {
                // Saving WebP is not supported
                if &ext == "webp" {
                    "png".to_owned()
                } else {
                    ext
                }
            })
            .unwrap_or("png".to_owned());

        let filename = format!("fragment_{}.{}", fragment.id(), extension);

        let img_path = temp_dir.path().clone().join(&filename);

        fragment.path = filename;

        image::save_buffer(
            img_path,
            &fragment.image.pixels,
            fragment.image.width,
            fragment.image.height,
            image::ColorType::Rgba8,
        )?;
    }

    let manifest_str = serde_yaml::to_string(manifest)?;

    let manifest_path = temp_dir.path().clone().join(FILE_NAME_MANIFEST);
    write(manifest_path, manifest_str)?;

    let output = File::create(path)?;

    let mut archive = Builder::new(output);

    archive.append_dir_all(".", temp_dir.path())?;

    archive.finish()?;

    temp_dir.close()?;

    Ok(())
}

/// Filesystem saving is unavailable in a browser; in-memory reading remains supported.
#[cfg(target_arch = "wasm32")]
pub fn save<P>(_manifest: &mut Manifest, _path: P) -> Result<()>
where
    P: AsRef<Path>,
{
    Err(std::io::Error::new(
        std::io::ErrorKind::Unsupported,
        "paperdoll filesystem saving is unavailable on wasm32",
    )
    .into())
}
