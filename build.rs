fn main() {
    // ── UniFFI scaffolding ───────────────────────────────────────────────
    // Not strictly required when using `uniffi::setup_scaffolding!()` in
    // lib.rs (the proc-macro approach), but having it here ensures the
    // build-dep on `uniffi` with `features = ["build"]` is exercised and
    // keeps compatibility if we ever switch to UDL-based generation.

    // ── Apple: provide missing ___chkstk_darwin symbol ───────────────────
    //
    // On macOS, `___chkstk_darwin` is exported by libSystem as a stack
    // probing function.  On arm64 the kernel grows the stack via guard
    // pages so the probe is effectively a no-op `ret`.
    //
    // tvOS, visionOS, and watchOS do NOT export this symbol at all.
    //
    // iOS exports it in libSystem, but deployment-target mismatches
    // between dependencies (e.g. onig_sys built for iOS 26.0 linked
    // against a min-target of iOS 10.0) can cause the linker to fail
    // to resolve it.  Providing the stub on iOS is harmless — if
    // libSystem already has the symbol the linker prefers it; if not,
    // our stub satisfies the reference.
    //
    // `aws-lc-sys` assembly (transitive via reqwest → rustls → aws-lc-rs)
    // references ___chkstk_darwin unconditionally, so we compile a tiny
    // `ret` stub for all non-macOS Apple targets.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "ios"
        || target_os == "tvos"
        || target_os == "visionos"
        || target_os == "watchos"
    {
        let manifest_dir = std::env::var("CARGO_MANIFEST_DIR").expect("CARGO_MANIFEST_DIR not set");
        let asm_path = std::path::Path::new(&manifest_dir).join("scripts/tvos_chkstk.s");

        if asm_path.exists() {
            // Use the `cc` crate to assemble the stub into a static library
            // that the linker will pick up.  If `cc` is not available as a
            // build-dependency, fall back to printing a cargo link directive
            // that tells the linker to include the object directly.
            //
            // Since `cc` is already a transitive build-dep (pulled in by
            // aws-lc-sys, ring, etc.), we use it directly.
            cc::Build::new().file(&asm_path).compile("tvos_chkstk");

            println!("cargo:rerun-if-changed=scripts/tvos_chkstk.s");
        } else {
            println!(
                "cargo:warning=tvos_chkstk.s not found at {} — \
                 ___chkstk_darwin will be unresolved on tvOS",
                asm_path.display()
            );
        }
    }
}
