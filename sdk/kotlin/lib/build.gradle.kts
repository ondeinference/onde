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

    sourceSets {
        // Both androidMain and jvmMain share the code in src/shared/kotlin/
        // and the UniFFI-generated bindings in src/generated/kotlin/.
        // This avoids the complexity of intermediate source sets while keeping
        // the shared engine wrapper code DRY.

        androidMain.dependencies {
            // JNA — UniFFI-generated onde.kt uses com.sun.jna.* for its FFI bridge.
            // The @aar suffix pulls the Android-specific build with native .so files.
            implementation("net.java.dev.jna:jna:5.14.0@aar")
            // Coroutines — suspend funs and Flow streaming in OndeInference
            implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
        }

        jvmMain.dependencies {
            // Desktop JNA with native libs for macOS/Linux/Windows
            implementation("net.java.dev.jna:jna:5.14.0")
            // Coroutines
            implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.8.1")
        }

        // Add shared source directories to both JVM-based targets.
        // src/shared/kotlin/ — hand-written OndeInference wrapper + convenience objects
        // src/generated/kotlin/ — UniFFI-generated onde.kt (gitignored, regenerated)
        androidMain.get().kotlin.srcDir("src/shared/kotlin")
        androidMain.get().kotlin.srcDir("src/generated/kotlin")
        jvmMain.get().kotlin.srcDir("src/shared/kotlin")
        jvmMain.get().kotlin.srcDir("src/generated/kotlin")
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
            "On-device LLM inference for Android and JVM (macOS Apple Silicon). " +
            "Run Qwen 2.5 models locally — no cloud, no API key, no data leaving the device."
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
