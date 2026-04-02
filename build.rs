fn main() {
    // ── UniFFI scaffolding ───────────────────────────────────────────────
    // Not strictly required when using `uniffi::setup_scaffolding!()` in
    // lib.rs (the proc-macro approach), but having it here ensures the
    // build-dep on `uniffi` with `features = ["build"]` is exercised and
    // keeps compatibility if we ever switch to UDL-based generation.

    // ── tvOS: provide missing ___chkstk_darwin symbol ────────────────────
    //
    // On macOS and iOS, `___chkstk_darwin` is exported by libSystem as a
    // stack probing function.  On arm64, the kernel grows the stack via
    // guard pages so the probe is effectively a no-op.  tvOS does NOT
    // export this symbol, but `aws-lc-sys` assembly (pulled in transitively
    // via reqwest → rustls → aws-lc-rs) references it unconditionally.
    //
    // We compile a tiny assembly stub that provides the symbol as a `ret`
    // instruction — only when targeting tvOS.
    let target_os = std::env::var("CARGO_CFG_TARGET_OS").unwrap_or_default();

    if target_os == "tvos" {
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
