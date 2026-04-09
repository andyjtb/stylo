/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at https://mozilla.org/MPL/2.0/. */

use std::env;
use std::path::{Path, PathBuf};
use std::process::{exit, Command};
use std::sync::LazyLock;
use walkdir::WalkDir;

mod build_cpp_codegen;

#[cfg(feature = "gecko")]
mod build_gecko;

#[cfg(not(feature = "gecko"))]
mod build_gecko {
    pub fn generate() {}
}

pub static PYTHON: LazyLock<String> = LazyLock::new(|| {
    env::var("PYTHON3").ok().unwrap_or_else(|| {
        let candidates = if cfg!(windows) {
            ["python.exe"]
        } else {
            ["python3"]
        };
        for &name in &candidates {
            if Command::new(name)
                .arg("--version")
                .output()
                .ok()
                .map_or(false, |out| out.status.success())
            {
                return name.to_owned();
            }
        }
        panic!(
            "Can't find python (tried {})! Try fixing PATH or setting the PYTHON3 env var",
            candidates.join(", ")
        )
    })
});

fn generate_properties(engine: &str) {
    for entry in WalkDir::new("properties") {
        let entry = entry.unwrap();
        match entry.path().extension().and_then(|e| e.to_str()) {
            Some("mako") | Some("rs") | Some("py") | Some("zip") | Some("toml") => {
                println!("cargo:rerun-if-changed={}", entry.path().display());
            },
            _ => {},
        }
    }

    let script = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .join("properties")
        .join("build.py");

    let status = Command::new(&*PYTHON)
        // `cargo publish` isn't happy with the `__pycache__` files that are created
        // when we run the property generator.
        //
        // TODO(mrobinson): Is this happening because of how we run this script? It
        // would be better to ensure are just placed in the output directory.
        .env("PYTHONDONTWRITEBYTECODE", "1")
        .arg(&script)
        .arg(engine)
        .arg("style-crate")
        .status()
        .unwrap();
    if !status.success() {
        exit(1)
    }
}

fn main() {
    let gecko = cfg!(feature = "gecko");
    let servo = cfg!(feature = "servo");
    let engine = match (gecko, servo) {
        (true, false) => "gecko",
        (false, true) => "servo",
        _ => panic!(
            "\n\n\
             The style crate requires enabling one of its 'servo' or 'gecko' feature flags. \
             \n\n"
        ),
    };
    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:out_dir={}", env::var("OUT_DIR").unwrap());

    // Rerun if local_names.txt changes (affects CSS property generation)
    let local_names_path = Path::new(&env::var_os("CARGO_MANIFEST_DIR").unwrap())
        .parent()
        .unwrap()
        .join("html5ever/web_atoms/local_names.txt");
    println!("cargo:rerun-if-changed={}", local_names_path.display());

    generate_properties(engine);
    build_gecko::generate();

    // Generate C++ headers exposing web_atoms::LocalName static-atom u64 values.
    // Honors STYLO_CPP_OUTPUT_DIR for CMake integration; otherwise lands in OUT_DIR.
    let cpp_out = env::var("STYLO_CPP_OUTPUT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(env::var("OUT_DIR").unwrap()));
    std::fs::create_dir_all(&cpp_out).ok();
    println!("cargo:rerun-if-env-changed=STYLO_CPP_OUTPUT_DIR");
    build_cpp_codegen::generate_attribute_header(&local_names_path, &cpp_out);

    // ui_css_properties.h is only emitted for the servo engine by build.py;
    // ui_changed_properties.h depends on it for the PropertyId type and for
    // the longhand count, so gate it on the same condition.
    if servo {
        let css_props_header = cpp_out.join("ui_css_properties.h");
        let longhand_count = build_cpp_codegen::read_longhand_count(&css_props_header);
        build_cpp_codegen::generate_changed_properties_header(longhand_count, &cpp_out);
    }
}
