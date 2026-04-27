import com.vanniktech.maven.publish.KotlinMultiplatform
import com.vanniktech.maven.publish.JavadocJar

plugins {
    id("org.jetbrains.kotlin.multiplatform")
    id("com.android.library")
    id("com.vanniktech.maven.publish")
    id("signing")
}

kotlin {
    androidTarget {
        publishLibraryVariants("release")
        compilations.all {
            kotlinOptions { jvmTarget = "17" }
        }
    }

    jvm {
        compilations.all {
            kotlinOptions { jvmTarget = "17" }
        }
    }

    // ── iOS targets ────────────────────────────────────────────────────────
    //
    // Kotlin/Native targets for iOS device and simulator.
    // These use cinterop to call the Rust C API (ffi/c_api.rs) directly,
    // bypassing the JNA-based UniFFI bindings used on Android/JVM.
    //
    // Prerequisites:
    //   1. Run scripts/build-ios.sh to compile libonde.a for each target
    //      and generate the C header (onde_c_api.h).
    //   2. The static libraries must be placed at:
    //        src/iosArm64Main/libs/libonde.a
    //        src/iosSimulatorArm64Main/libs/libonde.a

    iosArm64 {
        compilations.getByName("main") {
            cinterops {
                val onde by creating {
                    defFile("src/nativeInterop/cinterop/onde.def")
                    includeDirs("src/nativeInterop/cinterop")
                }
            }
        }
        binaries.all {
            linkerOpts("-L${projectDir}/src/iosArm64Main/libs", "-londe")
            // System frameworks required by the Rust static library
            linkerOpts("-framework", "Metal")
            linkerOpts("-framework", "MetalPerformanceShaders")
            linkerOpts("-framework", "Foundation")
            linkerOpts("-framework", "Security")
            linkerOpts("-framework", "SystemConfiguration")
            linkerOpts("-lz", "-lc++")
        }
    }

    iosSimulatorArm64 {
        compilations.getByName("main") {
            cinterops {
                val onde by creating {
                    defFile("src/nativeInterop/cinterop/onde.def")
                    includeDirs("src/nativeInterop/cinterop")
                }
            }
        }
        binaries.all {
            linkerOpts("-L${projectDir}/src/iosSimulatorArm64Main/libs", "-londe")
            linkerOpts("-framework", "Metal")
            linkerOpts("-framework", "MetalPerformanceShaders")
            linkerOpts("-framework", "Foundation")
            linkerOpts("-framework", "Security")
            linkerOpts("-framework", "SystemConfiguration")
            linkerOpts("-lz", "-lc++")
        }
    }

    // ── Source set hierarchy ────────────────────────────────────────────────
    //
    //   commonMain
    //   ├── jvmBasedMain          (intermediate: Android + JVM shared code)
    //   │   ├── androidMain
    //   │   └── jvmMain
    //   └── iosMain               (intermediate: all iOS targets)
    //       ├── iosArm64Main
    //       └── iosSimulatorArm64Main

    sourceSets {
        // ── commonMain ─────────────────────────────────────────────────────
        // Pure Kotlin types (Types.kt), expect class OndeInference,
        // expect objects OndeSampling / OndeModels / OndeMessage.
        // No JVM or Native platform dependencies here.
        commonMain.dependencies {
            implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1")
        }

        // ── jvmBasedMain ───────────────────────────────────────────────────
        // Intermediate source set shared by Android and JVM.
        // Contains actual class OndeInference (JNA/UniFFI-based),
        // actual convenience objects, PlatformSupport interface,
        // and the UniFFI-generated bindings.
        val jvmBasedMain by creating {
            dependsOn(commonMain.get())
        }

        androidMain {
            dependsOn(jvmBasedMain)
            dependencies {
                // JNA: UniFFI-generated onde.kt uses com.sun.jna.* for its FFI bridge.
                // The @aar suffix pulls the Android-specific build with native .so files.
                implementation("net.java.dev.jna:jna:5.14.0@aar")
                // Coroutines: suspend funs and Flow streaming in OndeInference
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
            }
        }

        jvmMain {
            dependsOn(jvmBasedMain)
            dependencies {
                // Desktop JNA with native libs for macOS/Linux/Windows
                implementation("net.java.dev.jna:jna:5.14.0")
                // Coroutines
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1")
            }
        }

        // Add shared source directories to jvmBasedMain.
        // src/generated/kotlin/ — UniFFI-generated onde.kt (gitignored, regenerated)
        jvmBasedMain.kotlin.srcDir("src/generated/kotlin")

        // ── iosMain ────────────────────────────────────────────────────────
        // Intermediate source set shared by all iOS targets.
        // Contains actual class OndeInference (cinterop/C API-based),
        // actual convenience objects, and the iOS factory function.
        val iosMain by creating {
            dependsOn(commonMain.get())
            dependencies {
                implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1")
            }
        }

        val iosArm64Main by getting {
            dependsOn(iosMain)
        }

        val iosSimulatorArm64Main by getting {
            dependsOn(iosMain)
        }
    }
}

android {
    namespace = "com.ondeinference.onde"
    compileSdk = 35

    defaultConfig {
        minSdk = 26 // Os.setenv needs API 26+
        consumerProguardFiles("consumer-rules.pro")
    }

    lint {
        // The UniFFI-generated onde.kt uses java.lang.ref.Cleaner (API 33+)
        // with an internal fallback for older API levels. We don't control
        // the generated code, so suppress the lint error.
        disable += "NewApi"
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    sourceSets {
        getByName("main") {
            // Android JNI shared libraries live under androidMain/jniLibs/
            jniLibs.srcDirs("src/androidMain/jniLibs")
        }
    }
}

// ── Maven Central publishing (KMP) ─────────────────────────────────────────
//
// Required secrets (in ~/.gradle/gradle.properties or CI env vars):
//   ORG_GRADLE_PROJECT_mavenCentralUsername
//   ORG_GRADLE_PROJECT_mavenCentralPassword
//   ORG_GRADLE_PROJECT_signingKeyId
//   ORG_GRADLE_PROJECT_signingKey        (ASCII-armored PGP private key)
//   ORG_GRADLE_PROJECT_signingPassword
// ── Signing ────────────────────────────────────────────────────────────────
// Vanniktech 0.28 looks for signingInMemoryKey / signingInMemoryKeyId /
// signingInMemoryKeyPassword. Local gradle.properties (and CI secrets) use
// the shorter names signingKey / signingKeyId / signingPassword.
// Bridge them here so both naming conventions work.
val signingKeyId    = findProperty("signingInMemoryKeyId")       as String?
    ?: findProperty("signingKeyId")                              as String?
val signingKey      = findProperty("signingInMemoryKey")         as String?
    ?: findProperty("signingKey")                                as String?
val signingPassword = findProperty("signingInMemoryKeyPassword") as String?
    ?: findProperty("signingPassword")                           as String?

signing {
    if (signingKey != null) {
        useInMemoryPgpKeys(signingKeyId, signingKey, signingPassword)
    }
}


mavenPublishing {
    configure(
        KotlinMultiplatform(
            javadocJar = JavadocJar.Empty(),
            sourcesJar = true,
        )
    )

    publishToMavenCentral()
    signAllPublications()

    // Coordinates are read automatically from gradle.properties:
    //   GROUP          → groupId
    //   POM_ARTIFACT_ID → artifactId
    //   VERSION_NAME   → version
    // Do NOT call coordinates() explicitly — it conflicts with the
    // properties-based approach and causes "value is final" errors.

    pom {
        name.set("Onde Inference")
        description.set(
            "On-device LLM inference for Android, JVM (macOS Apple Silicon), and iOS. " +
            "Run Qwen 2.5 models locally. No cloud, no API key, no data leaves the device."
        )
        url.set("https://ondeinference.com")
        inceptionYear.set("2025")

        licenses {
            license {
                name.set("MIT OR Apache-2.0")
                url.set("https://github.com/ondeinference/onde/blob/main/LICENSE")
                distribution.set("repo")
            }
        }

        developers {
            developer {
                id.set("ondeinference")
                name.set("Onde Inference")
                email.set("hello@ondeinference.com")
                url.set("https://ondeinference.com")
                organization.set("Splitfire AB")
                organizationUrl.set("https://splitfire.se")
            }
        }

        scm {
            url.set("https://github.com/ondeinference/onde")
            connection.set("scm:git:github.com/ondeinference/onde.git")
            developerConnection.set("scm:git:ssh://github.com/ondeinference/onde.git")
        }
    }
}
