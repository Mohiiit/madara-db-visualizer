use std::env;
use std::path::PathBuf;

fn main() {
    // Only enforce `dist/` presence when embedding the UI.
    if env::var_os("CARGO_FEATURE_EMBEDDED_UI").is_none() {
        return;
    }

    // The toolchain embeds the pre-built Trunk `dist/` directory.
    // Keep this explicit (CI/dev must run `./scripts/build_dist.sh` before building).
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let repo_root = manifest_dir
        .parent()
        .and_then(|p| p.parent())
        .expect("api crate should live at <repo>/crates/api");

    let dist = env::var("MAKIMONO_DIST_DIR")
        .ok()
        .map(PathBuf::from)
        .unwrap_or_else(|| repo_root.join("dist"));

    println!("cargo:rerun-if-env-changed=MAKIMONO_DIST_DIR");
    println!("cargo:rerun-if-changed={}", dist.display());

    if !dist.join("index.html").is_file() {
        eprintln!(
            "makimono-viz requires a Trunk dist directory at {} (missing index.html).\n\
Run: ./scripts/build_dist.sh",
            dist.display()
        );
        std::process::exit(1);
    }
}
