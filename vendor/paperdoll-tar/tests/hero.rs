use paperdoll_tar::{paperdoll::Manifest, FILE_NAME_MANIFEST};
use std::{
    collections::BTreeMap,
    io::{Cursor, Read},
};

const HERO: &[u8] = include_bytes!("../../../assets/paperdoll/hero.ppd");

#[test]
fn real_hero_pixels_and_dimensions_match_archive_images() {
    let factory =
        paperdoll_tar::read(Cursor::new(HERO)).expect("read actual hero asset from bytes");
    let loaded = factory.to_manifest();
    let mut files = BTreeMap::new();
    for entry in tar::Archive::new(Cursor::new(HERO)).entries().unwrap() {
        let mut entry = entry.unwrap();
        if !entry.header().entry_type().is_file() {
            continue;
        }
        let name = entry.path().unwrap().to_string_lossy().to_string();
        let mut bytes = Vec::new();
        entry.read_to_end(&mut bytes).unwrap();
        files.insert(name, bytes);
    }
    let original: Manifest = serde_yaml::from_slice(&files[FILE_NAME_MANIFEST]).unwrap();
    assert_eq!(loaded.meta.name, original.meta.name);
    assert_eq!(loaded.dolls.len(), original.dolls.len());
    assert_eq!(loaded.fragments.len(), original.fragments.len());
    assert!(loaded.dolls.len() >= 3 && loaded.fragments.len() >= 8);
    for doll in &loaded.dolls {
        let decoded = image::load_from_memory(&files[&doll.path])
            .unwrap()
            .into_rgba8();
        assert_eq!((doll.image.width, doll.image.height), decoded.dimensions());
        assert_eq!(doll.image.pixels, decoded.into_vec());
    }
    for fragment in &loaded.fragments {
        let decoded = image::load_from_memory(&files[&fragment.path])
            .unwrap()
            .into_rgba8();
        assert_eq!(
            (fragment.image.width, fragment.image.height),
            decoded.dimensions()
        );
        assert_eq!(fragment.image.pixels, decoded.into_vec());
    }
}

#[test]
#[cfg(not(target_arch = "wasm32"))]
fn real_hero_load_needs_no_usable_temp_directory() {
    const CHILD: &str = "PAPERDOLL_MEMORY_READ_CHILD";
    if std::env::var_os(CHILD).is_some() {
        paperdoll_tar::read(Cursor::new(HERO))
            .expect("memory read must not touch temp directories");
        return;
    }
    // Isolate environment changes. A regular file is an unusable temp-dir path;
    // the upstream TempDir::new reader fails here, while the memory reader succeeds.
    let invalid_temp =
        std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../assets/paperdoll/hero.ppd");
    assert!(invalid_temp.is_file());
    let result = std::process::Command::new(std::env::current_exe().unwrap())
        .args([
            "--exact",
            "real_hero_load_needs_no_usable_temp_directory",
            "--nocapture",
        ])
        .env(CHILD, "1")
        .env("TMPDIR", invalid_temp)
        .status()
        .unwrap();
    assert!(result.success());
}

#[test]
fn upstream_dot_prefix_archives_and_missing_images_are_handled_in_memory() {
    let mut output = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut output);
        for entry in tar::Archive::new(Cursor::new(HERO)).entries().unwrap() {
            let mut entry = entry.unwrap();
            if !entry.header().entry_type().is_file() {
                continue;
            }
            let name = format!("./{}", entry.path().unwrap().display());
            let mut header = entry.header().clone();
            builder.append_data(&mut header, name, &mut entry).unwrap();
        }
        builder.finish().unwrap();
    }
    assert!(paperdoll_tar::read(Cursor::new(output)).is_ok());
    let mut output = Vec::new();
    {
        let mut builder = tar::Builder::new(&mut output);
        for entry in tar::Archive::new(Cursor::new(HERO)).entries().unwrap() {
            let mut entry = entry.unwrap();
            if entry.path().unwrap().as_ref() == std::path::Path::new(FILE_NAME_MANIFEST) {
                let mut header = entry.header().clone();
                builder
                    .append_data(&mut header, FILE_NAME_MANIFEST, &mut entry)
                    .unwrap();
            }
        }
        builder.finish().unwrap();
    }
    let error = paperdoll_tar::read(Cursor::new(output))
        .err()
        .expect("archive without images must fail");
    assert!(error.to_string().contains("missing image"));
}
