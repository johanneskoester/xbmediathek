use std::collections::HashSet;
use std::path::PathBuf;

fn main() {
    generate_providers();

    #[cfg(windows)]
    {
        windows::core::build!(Microsoft::UI::Xaml::Controls::{WebView2, XamlControlsResources});

        if std::env::var("TARGET").as_deref() == Ok("x86_64-uwp-windows-msvc") {
            let metadata = cargo_metadata::MetadataCommand::new()
                .other_options(vec!["--locked".into()])
                .exec()
                .expect("could not find Windows import library metadata");
            let package = metadata
                .packages
                .iter()
                .find(|package| {
                    package.name == "windows_x86_64_msvc" && package.version.to_string() == "0.28.0"
                })
                .expect("windows_x86_64_msvc 0.28.0 is missing");
            let library_dir = package.manifest_path.parent().unwrap().join("lib");
            println!("cargo:rustc-link-search=native={library_dir}");
        }
    }
}

fn generate_providers() {
    println!("cargo:rerun-if-changed=Cargo.toml");
    let manifest: toml::Value = include_str!("Cargo.toml")
        .parse()
        .expect("invalid Cargo.toml");
    let providers = manifest["package"]["metadata"]["providers"]
        .as_table()
        .expect("package.metadata.providers must be a table");
    assert!(!providers.is_empty(), "at least one provider is required");

    let mut variants = HashSet::new();
    let mut entries = Vec::new();
    for (key, value) in providers {
        let mut variant = String::new();
        for part in key.split(['-', '_']) {
            assert!(
                !part.is_empty()
                    && part.as_bytes()[0].is_ascii_lowercase()
                    && part
                        .bytes()
                        .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit()),
                "invalid provider key {key:?}: use lowercase words separated by '-' or '_'"
            );
            variant.push(part.as_bytes()[0].to_ascii_uppercase() as char);
            variant.push_str(&part[1..]);
        }
        assert!(
            variant != "Self",
            "provider key {key:?} generates the reserved identifier Self"
        );
        assert!(
            variants.insert(variant.clone()),
            "duplicate provider variant {variant}"
        );
        let field = |name| {
            value[name]
                .as_str()
                .unwrap_or_else(|| panic!("provider {key:?} requires a string {name:?}"))
        };
        entries.push((variant, field("name"), field("url"), field("logo")));
    }

    let mut output = String::from("define_providers! {\n");
    for (variant, name, url, logo) in &entries {
        output.push_str(&format!(
            "    {variant} {{ name: {name:?}, url: {url:?}, logo: {logo:?} }},\n"
        ));
    }
    output.push_str("}\n");
    let destination = PathBuf::from(std::env::var_os("OUT_DIR").unwrap()).join("providers.rs");
    std::fs::write(destination, output).expect("could not write generated providers");
}
