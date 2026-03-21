// tvos_chkstk.s — stub for ___chkstk_darwin on tvOS
//
// On macOS and iOS, ___chkstk_darwin is provided by libSystem as a stack
// probing function.  On arm64 Apple platforms the kernel grows the stack
// automatically via guard pages, so the probe is effectively a no-op that
// just returns.  tvOS does NOT export this symbol, but aws-lc-sys assembly
// references it unconditionally.
//
// This file provides the missing symbol so the linker succeeds.
// It is only compiled for tvOS targets (see build.rs).
//
// Symbol naming:
//   Mach-O adds one leading underscore to C symbols automatically.
//   The linker error says: Undefined symbol "___chkstk_darwin"
//   That means the C-level name is "__chkstk_darwin" (two underscores).
//   In assembly we write the raw Mach-O symbol = "___chkstk_darwin" (three underscores).

.text
.globl ___chkstk_darwin
.p2align 2

___chkstk_darwin:
    ret
