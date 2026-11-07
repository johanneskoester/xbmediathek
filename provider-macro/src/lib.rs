use std::collections::HashSet;
use std::path::PathBuf;

use proc_macro::TokenStream;
use quote::{format_ident, quote};

#[proc_macro]
pub fn define_providers(_input: TokenStream) -> TokenStream {
    match expand_providers() {
        Ok(tokens) => tokens.into(),
        Err(message) => quote! { compile_error!(#message); }.into(),
    }
}

fn expand_providers() -> Result<proc_macro2::TokenStream, String> {
    let manifest_dir = std::env::var_os("CARGO_MANIFEST_DIR").ok_or_else(|| {
        "CARGO_MANIFEST_DIR is not set while expanding define_providers!".to_owned()
    })?;
    let manifest_path = PathBuf::from(manifest_dir).join("Cargo.toml");
    let manifest_text = std::fs::read_to_string(&manifest_path)
        .map_err(|error| format!("could not read {}: {error}", manifest_path.display()))?;
    let manifest: toml::Value = manifest_text
        .parse()
        .map_err(|error| format!("could not parse {}: {error}", manifest_path.display()))?;
    let providers = manifest
        .get("package")
        .and_then(|package| package.get("metadata"))
        .and_then(|metadata| metadata.get("providers"))
        .and_then(toml::Value::as_table)
        .ok_or_else(|| "Cargo.toml must define [package.metadata.providers]".to_owned())?;

    if providers.is_empty() {
        return Err("[package.metadata.providers] must contain at least one provider".to_owned());
    }

    let mut variants = HashSet::new();
    let mut entries = Vec::new();
    for (key, value) in providers {
        let variant_name = provider_variant(key)?;
        if variant_name == "Self" {
            return Err(format!(
                "provider key {key:?} generates reserved identifier Self"
            ));
        }
        if !variants.insert(variant_name.clone()) {
            return Err(format!(
                "provider key {key:?} generates a duplicate enum variant"
            ));
        }
        let variant = format_ident!("{variant_name}");

        let field = |name| {
            value
                .get(name)
                .and_then(toml::Value::as_str)
                .ok_or_else(|| format!("provider {key:?} requires a string {name:?}"))
        };
        entries.push((
            variant,
            field("name")?.to_owned(),
            field("url")?.to_owned(),
            field("logo")?.to_owned(),
        ));
    }

    let count = entries.len();
    let enum_variants = entries.iter().map(|(variant, _, _, _)| variant);
    let all_variants = entries.iter().map(|(variant, _, _, _)| variant);
    let name_arms = entries
        .iter()
        .map(|(variant, name, _, _)| quote! { Self::#variant => #name });
    let url_arms = entries
        .iter()
        .map(|(variant, _, url, _)| quote! { Self::#variant => #url });
    let logo_arms = entries
        .iter()
        .map(|(variant, _, _, logo)| quote! { Self::#variant => #logo });

    Ok(quote! {
        #[derive(Clone, Copy, Debug, PartialEq, Eq)]
        pub enum Provider {
            #( #enum_variants, )*
        }

        impl Provider {
            pub const ALL: [Provider; #count] = [
                #( Provider::#all_variants, )*
            ];

            pub fn name(self) -> &'static str {
                match self { #( #name_arms, )* }
            }

            pub fn url(self) -> &'static str {
                match self { #( #url_arms, )* }
            }

            pub fn logo(self) -> &'static str {
                match self { #( #logo_arms, )* }
            }
        }
    })
}

fn provider_variant(key: &str) -> Result<String, String> {
    let mut variant = String::new();
    for part in key.split(['-', '_']) {
        if part.is_empty()
            || !part.as_bytes()[0].is_ascii_lowercase()
            || !part
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit())
        {
            return Err(format!(
                "invalid provider key {key:?}: use lowercase words separated by '-' or '_'"
            ));
        }
        variant.push(part.as_bytes()[0].to_ascii_uppercase() as char);
        variant.push_str(&part[1..]);
    }
    Ok(variant)
}
