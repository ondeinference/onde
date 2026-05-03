fn main() {
    // SDK credentials
    // Load .env from the onde crate root so pulse/client.rs can read these
    // through option_env!(). Consumer apps do not set them. They belong to
    // Onde Inference's own infrastructure.
    println!("cargo:rerun-if-changed=.env");
    let _ = dotenvy::from_filename(
        std::path::Path::new(&std::env::var("CARGO_MANIFEST_DIR").unwrap()).join(".env"),
    );
    for var in [
        "GRESIQ_API_KEY_DEV",
        "GRESIQ_API_SECRET_DEV",
        "GRESIQ_API_KEY_PRODUCTION",
        "GRESIQ_API_SECRET_PRODUCTION",
        "HF_TOKEN",
    ] {
        println!("cargo:rerun-if-env-changed={var}");
        if let Ok(val) = std::env::var(var) {
            if !val.is_empty() {
                println!("cargo:rustc-env={var}={val}");
            }
        }
    }

    // UniFFI scaffolding
    // `uniffi::setup_scaffolding!()` in lib.rs already covers the proc-macro
    // path, but keeping this here still exercises the build dependency and
    // leaves us room to switch back to UDL-based generation later.

    // Apple targets: provide a missing ___chkstk_darwin symbol
    //
    // On macOS, `___chkstk_darwin` is exported by libSystem as a stack
    // probing function.  On arm64 the kernel grows the stack via guard
    // pages so the probe is effectively a no-op `ret`.
    //
    // tvOS, visionOS, and watchOS do NOT export this symbol at all.
    //
    // iOS does export it in libSystem, but deployment-target mismatches
    // between dependencies (for example onig_sys built for iOS 26.0 linked
    // against a min-target of iOS 10.0) can still make the linker miss it.
    // Shipping the stub on iOS is harmless. If libSystem already provides the
    // symbol, the linker uses that. If not, our stub covers the reference.
    //
    // `aws-lc-sys` assembly (transitive via reqwest -> rustls -> aws-lc-rs)
    // always references ___chkstk_darwin, so we compile a tiny `ret` stub for
    // all non-macOS Apple targets.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "ios"
        || target_os == "tvos"
        || target_os == "visionos"
        || target_os == "watchos"
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let asm_path = std::path::Path::new(&manifest_dir).join("scripts/tvos_chkstk.s");

        if asm_path.exists() {
            // Use `cc` to assemble the stub into a static library the linker
            // can pick up. `cc` is already in the build graph through crates
            // like aws-lc-sys and ring, so we just use it directly.
            cc::Build::new().file(&asm_path).compile("tvos_chkstk");

            println!("cargo:rerun-if-changed=scripts/tvos_chkstk.s");
        } else {
            println!(
                "cargo:warning=tvos_chkstk.s not found at {}. \
                 ___chkstk_darwin will stay unresolved on tvOS",
                asm_path.display()
            );
        }
    }
}
