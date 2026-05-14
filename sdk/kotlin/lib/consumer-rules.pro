# Onde Inference — ProGuard consumer rules
#
# These rules are applied to any app that depends on the Onde Inference AAR.
# They preserve the UniFFI JNI bridge symbols that the Rust native library
# resolves at runtime via System.loadLibrary("onde").

# Keep all UniFFI-generated classes and their native method declarations
-keep class uniffi.onde.** { *; }

# Keep the OndeInference public API and companion objects
-keep class com.ondeinference.onde.** { *; }

# Keep the StreamChunkListener callback interface (called from Rust)
-keep interface uniffi.onde.StreamChunkListener { *; }

# Prevent stripping of native method registrations
-keepclasseswithmembernames class * {
    native <methods>;
}

# Keep Kotlin coroutine internal classes referenced by the Flow streaming API
-keepnames class kotlinx.coroutines.internal.MainDispatcherFactory {}
-keepnames class kotlinx.coroutines.CoroutineExceptionHandler {}

# Prevent R8 from removing the load-library static initializer in the
# UniFFI-generated onde.kt (it calls System.loadLibrary in a companion object)
-keepclassmembers class uniffi.onde.* {
    static <clinit>();
}
