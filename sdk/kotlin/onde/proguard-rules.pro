# ──────────────────────────────────────────────────────────────────────────────
# ProGuard / R8 rules for the Onde Kotlin library (UniFFI + JNA)
# ──────────────────────────────────────────────────────────────────────────────

# ── JNA ───────────────────────────────────────────────────────────────────────
# JNA uses reflection extensively to map Java/Kotlin interfaces to native
# function pointers.  Stripping or renaming any of these classes breaks the
# native call bridge at runtime.
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-dontwarn com.sun.jna.**

# ── UniFFI-generated bindings ─────────────────────────────────────────────────
# The generated Kotlin code in com.ondeinference.onde contains:
#   - JNA Library interface (extends com.sun.jna.Library)
#   - JNA Structure subclasses (RustBuffer, ForeignBytes, etc.)
#   - Callback interfaces mapped to Rust trait objects
#   - Companion objects with @JvmStatic native method declarations
# All of these are resolved by name or via reflection — none may be renamed
# or removed.
-keep class com.ondeinference.onde.** { *; }
-keepclassmembers class com.ondeinference.onde.** { *; }

# Keep the JNA callback interfaces (UniFFI registers them by concrete class
# name at runtime via JNA's CallbackReference).
-keep class * implements com.sun.jna.Callback { *; }

# Keep JNA Structure fields — JNA maps struct fields by name and order.
-keepclassmembers class * extends com.sun.jna.Structure {
    <fields>;
}

# ── Kotlin coroutines ─────────────────────────────────────────────────────────
# UniFFI async exports generate suspend functions that depend on coroutine
# internals.  The default kotlin-stdlib rules handle most of this, but we
# explicitly keep the continuation classes to be safe.
-keepclassmembers class kotlin.coroutines.** { *; }
-dontwarn kotlinx.coroutines.**

# ── General safety ────────────────────────────────────────────────────────────
# Suppress warnings for annotations that are compile-time only.
-dontwarn javax.annotation.**
-dontwarn kotlin.annotations.jvm.**
