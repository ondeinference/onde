# ──────────────────────────────────────────────────────────────────────────────
# Consumer ProGuard / R8 rules for the Onde AAR
#
# These rules are automatically applied to any app that depends on this library
# via the AAR's consumer-rules mechanism.  They ensure that JNA, the UniFFI
# generated bindings, and the native bridge survive minification and name
# mangling in the consuming app's release build.
# ──────────────────────────────────────────────────────────────────────────────

# ── JNA ───────────────────────────────────────────────────────────────────────
# JNA resolves native function pointers and struct layouts via reflection.
# Any renaming or stripping breaks the JNI bridge at runtime.
-keep class com.sun.jna.** { *; }
-keep class * implements com.sun.jna.** { *; }
-keep class * implements com.sun.jna.Callback { *; }
-keepclassmembers class * extends com.sun.jna.Structure {
    <fields>;
}
-dontwarn com.sun.jna.**

# ── UniFFI-generated bindings ─────────────────────────────────────────────────
# The generated Kotlin code in com.ondeinference.onde contains JNA Library
# interfaces, Structure subclasses, callback interfaces, and companion objects
# with @JvmStatic native method declarations — all resolved by name or via
# reflection at runtime.
-keep class com.ondeinference.onde.** { *; }
-keepclassmembers class com.ondeinference.onde.** { *; }

# ── Kotlin coroutines ─────────────────────────────────────────────────────────
# UniFFI async exports generate suspend functions that rely on coroutine
# continuation internals.
-keepclassmembers class kotlin.coroutines.** { *; }
-dontwarn kotlinx.coroutines.**

# ── General safety ────────────────────────────────────────────────────────────
-dontwarn javax.annotation.**
-dontwarn kotlin.annotations.jvm.**
