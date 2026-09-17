extern crate vergen;
use anyhow::Result;
use vergen::*;
use std::path::Path;

fn main() -> Result<()> {
    if !Path::new(".git").exists() {
        println!("cargo:warning=Git metadata unavailable; using fallback build metadata");
        println!("cargo:rustc-env=VERGEN_GIT_SHA=unknown");
        println!("cargo:rustc-env=VERGEN_GIT_SEMVER=unknown");
        println!("cargo:rustc-env=VERGEN_GIT_COMMIT_MESSAGE=unknown");
        println!("cargo:rustc-env=VERGEN_SYSINFO_CPU_BRAND=unknown");
        println!("cargo:rustc-env=VERGEN_CARGO_PROFILE={}", std::env::var("PROFILE").unwrap_or_else(|_| "unknown".into()));
        println!("cargo:rustc-env=VERGEN_RUSTC_SEMVER=unknown");
        return Ok(());
    }

    let mut config = Config::default();

    *config.git_mut().sha_kind_mut() = ShaKind::Normal;

    *config.git_mut().semver_kind_mut() = SemverKind::Normal;

    *config.git_mut().semver_mut() = true;

    *config.git_mut().semver_dirty_mut() = Some("-dirty");

    vergen(config)
}
