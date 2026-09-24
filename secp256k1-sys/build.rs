// SPDX-License-Identifier: CC0-1.0

//! # Build script

// Coding conventions
#![deny(non_upper_case_globals)]
#![deny(non_camel_case_types)]
#![deny(non_snake_case)]
#![deny(unused_mut)]
#![warn(missing_docs)]

extern crate cc;

use std::{collections::BTreeSet, env, fs, path::Path};

const SYMBOL_PREFIX: &str = "svarog_secp256k1_v0_1";

// Namespace the vendored C identifiers at preprocessing time. The checked-in
// sources keep their upstream names, so upstream updates remain reviewable.
fn collect_symbols(dir: &Path, symbols: &mut BTreeSet<String>, macros: &mut BTreeSet<String>) {
    for entry in fs::read_dir(dir).expect("read vendored source directory") {
        let path = entry.expect("read vendored source entry").path();
        if path.is_dir() {
            collect_symbols(&path, symbols, macros);
        } else if matches!(path.extension().and_then(|s| s.to_str()), Some("c" | "h")) {
            let source = fs::read_to_string(&path).expect("read vendored C source");
            for line in source.lines() {
                if let Some(definition) = line
                    .trim_start()
                    .strip_prefix('#')
                    .and_then(|s| s.trim_start().strip_prefix("define"))
                {
                    if let Some(name) = definition
                        .trim_start()
                        .split(|c: char| !c.is_ascii_alphanumeric() && c != '_')
                        .next()
                    {
                        macros.insert(name.to_owned());
                    }
                }
            }
            for ident in source.split(|c: char| !c.is_ascii_alphanumeric() && c != '_') {
                if ident.starts_with("rustsecp256k1_v0_13_") || ident.starts_with("svarog_") {
                    symbols.insert(ident.to_owned());
                }
            }
        }
    }
}

fn namespaced_sources(out: &Path) -> Vec<std::path::PathBuf> {
    let mut symbols = BTreeSet::new();
    let mut macros = BTreeSet::new();
    collect_symbols(Path::new("depend/secp256k1"), &mut symbols, &mut macros);
    let mut header = String::new();
    for symbol in symbols.difference(&macros) {
        let suffix = symbol
            .strip_prefix("rustsecp256k1_v0_13_")
            .unwrap_or(symbol);
        header.push_str(&format!("#define {symbol} {SYMBOL_PREFIX}_{suffix}\n"));
    }
    fs::write(out.join("svarog_namespace.h"), header).expect("write C namespace header");
    [
        "contrib/lax_der_parsing.c",
        "src/precomputed_ecmult_gen.c",
        "src/precomputed_ecmult.c",
        "src/secp256k1.c",
    ]
    .iter()
    .enumerate()
    .map(|(index, source)| {
        let wrapper = out.join(format!("svarog_unit_{index}.c"));
        fs::write(
            &wrapper,
            format!("#include \"svarog_namespace.h\"\n#include \"depend/secp256k1/{source}\"\n"),
        )
        .expect("write namespaced C translation unit");
        wrapper
    })
    .collect()
}

fn main() {
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed=depend");
    println!("cargo:rerun-if-changed=wasm");
    let out = std::path::PathBuf::from(env::var_os("OUT_DIR").expect("OUT_DIR"));
    let sources = namespaced_sources(&out);
    // Actual build
    let mut base_config = cc::Build::new();
    base_config
        .include(env::var_os("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR"))
        .include(&out)
        .include("depend/secp256k1/")
        .include("depend/secp256k1/include")
        .include("depend/secp256k1/src")
        .flag_if_supported("-Wno-unused-function") // some ecmult stuff is defined but not used upstream
        .flag_if_supported("-Wno-unused-parameter") // patching out printf causes this warning
        .define("SECP256K1_API", Some(""))
        .define("ENABLE_MODULE_ECDH", Some("1"))
        .define("ENABLE_MODULE_SCHNORRSIG", Some("1"))
        .define("ENABLE_MODULE_EXTRAKEYS", Some("1"))
        .define("ENABLE_MODULE_ELLSWIFT", Some("1"))
        .define("ENABLE_MODULE_MUSIG", Some("1"))
        // upstream sometimes introduces calls to printf, which we cannot compile
        // with WASM due to its lack of libc. printf is never necessary and we can
        // just #define it away.
        .define("printf(...)", Some(""));

    if cfg!(feature = "lowmemory") {
        base_config.define("ECMULT_WINDOW_SIZE", Some("4")); // A low-enough value to consume negligible memory
        base_config.define("COMB_BLOCKS", Some("2"));
        base_config.define("COMB_TEETH", Some("5"));
    } else {
        base_config.define("ECMULT_WINDOW_SIZE", Some("15")); // This is the default in the configure file (`auto`)
        base_config.define("COMB_BLOCKS", Some("43"));
        base_config.define("COMB_TEETH", Some("6"));
    }
    base_config.define("USE_EXTERNAL_DEFAULT_CALLBACKS", Some("1"));
    #[cfg(feature = "recovery")]
    base_config.define("ENABLE_MODULE_RECOVERY", Some("1"));

    // WASM headers and size/align defines.
    if env::var("CARGO_CFG_TARGET_ARCH").unwrap() == "wasm32" {
        base_config.include("wasm/wasm-sysroot").file("wasm/wasm.c");
    }

    // secp256k1
    base_config.files(sources);

    if base_config.try_compile(SYMBOL_PREFIX).is_err() {
        // Some embedded platforms may not have, eg, string.h available, so if the build fails
        // simply try again with the wasm sysroot (but without the wasm type sizes) in the hopes
        // that it works.
        base_config.include("wasm/wasm-sysroot");
        base_config.compile(SYMBOL_PREFIX);
    }
}
