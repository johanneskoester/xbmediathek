fn main() {
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
