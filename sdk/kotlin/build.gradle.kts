// Root build.gradle.kts for the Onde Kotlin SDK project.
// Declares plugin versions shared by the Kotlin Multiplatform library,
// Android example app, and any legacy Android-only modules kept locally.

plugins {
    id("com.android.library") version "8.7.3" apply false
    id("com.android.application") version "8.7.3" apply false
    id("org.jetbrains.kotlin.android") version "2.0.21" apply false
    id("org.jetbrains.kotlin.multiplatform") version "2.0.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.0.21" apply false
    id("com.vanniktech.maven.publish") version "0.34.0" apply false
}
