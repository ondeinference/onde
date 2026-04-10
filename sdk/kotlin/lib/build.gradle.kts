import com.vanniktech.maven.publish.AndroidSingleVariantLibrary
import com.vanniktech.maven.publish.SonatypeHost

plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    id("com.vanniktech.maven.publish")
}

android {
    namespace = "com.ondeinference.onde"
    compileSdk = 35

    defaultConfig {
        minSdk = 26 // Os.setenv needs API 26+
        targetSdk = 35
        consumerProguardFiles("consumer-rules.pro")
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }

    kotlinOptions {
        jvmTarget = "17"
    }

    sourceSets {
        getByName("main") {
            jniLibs.srcDirs("src/main/jniLibs")
            // UniFFI-generated onde.kt lands in src/generated/kotlin/ after
            // running scripts/generate-bindings.sh — gitignored, regenerate after Rust API changes.
            kotlin.srcDirs("src/main/kotlin", "src/generated/kotlin")
        }
    }

    // No android { publishing { singleVariant(...) } } block here.
    // Vanniktech's AndroidSingleVariantLibrary (below) registers it — adding
    // it twice causes "singleVariant publishing DSL multiple times" error.
}

dependencies {
    // JNA — the UniFFI-generated onde.kt uses com.sun.jna.* for its FFI bridge.
    // The @aar suffix pulls the Android-specific build with native .so files included.
    implementation("net.java.dev.jna:jna:5.14.0@aar")

    // Coroutines — needed by the suspend funs and Flow streaming in OndeInference.kt
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.8.1")
}

// Required secrets (in ~/.gradle/gradle.properties or CI env vars):
//   ORG_GRADLE_PROJECT_mavenCentralUsername
//   ORG_GRADLE_PROJECT_mavenCentralPassword
//   ORG_GRADLE_PROJECT_signingKeyId
//   ORG_GRADLE_PROJECT_signingKey        (ASCII-armored PGP private key)
//   ORG_GRADLE_PROJECT_signingPassword
mavenPublishing {
    // AndroidSingleVariantLibrary is the correct way to configure variant + jars
    // for an Android library with Vanniktech 0.25+. It calls singleVariant("release")
    // internally, so the android { publishing { } } block must NOT be used alongside it.
    configure(
        AndroidSingleVariantLibrary(
            variant = "release",
            sourcesJar = true,
            publishJavadocJar = true,
        )
    )

    publishToMavenCentral(SonatypeHost.CENTRAL_PORTAL)
    signAllPublications()

    coordinates(
        groupId    = "com.ondeinference",
        artifactId = "onde-inference",
        version    = project.property("VERSION_NAME") as String,
    )

    pom {
        name.set("Onde Inference")
        description.set(
            "On-device LLM inference for Android. " +
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
