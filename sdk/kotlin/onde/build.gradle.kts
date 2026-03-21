plugins {
    id("com.android.library")
    id("org.jetbrains.kotlin.android")
    `maven-publish`
    signing
}

// ── Coordinates ──────────────────────────────────────────────────────────────
// Read from gradle.properties or fall back to defaults.
val libGroupId: String = findProperty("onde.groupId") as? String ?: "com.ondeinference"
val libArtifactId: String = findProperty("onde.artifactId") as? String ?: "onde"
val libVersion: String = findProperty("onde.version") as? String ?: "0.1.0"

group = libGroupId
version = libVersion

android {
    namespace = "com.ondeinference.onde"
    compileSdk = 36

    defaultConfig {
        minSdk = 24

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")

        aarMetadata {
            minCompileSdk = 24
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
    }

    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_1_8
        targetCompatibility = JavaVersion.VERSION_1_8
    }

    kotlinOptions {
        jvmTarget = "1.8"
    }

    // The generated Kotlin source from UniFFI lives alongside hand-written
    // Kotlin code.  Point the source set at the generated directory so
    // Gradle picks it up automatically after `build-kotlin.sh` runs.
    sourceSets {
        getByName("main") {
            java.srcDir("src/main/kotlin")
        }
    }

    publishing {
        singleVariant("release") {
            withSourcesJar()
        }
    }
}

dependencies {
    // JNA is required by UniFFI-generated Kotlin code to call into the
    // native shared library (libonde.so) via JNI.
    implementation("net.java.dev.jna:jna:5.14.0@aar")

    // Kotlin coroutines — the generated async bindings use
    // kotlinx.coroutines suspend functions under the hood.
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-core:1.9.0")
    implementation("org.jetbrains.kotlinx:kotlinx-coroutines-android:1.9.0")

    // AndroidX core (transitive, but explicit for clarity)
    implementation("androidx.annotation:annotation:1.9.1")

    // Test dependencies
    testImplementation("junit:junit:4.13.2")
    androidTestImplementation("androidx.test.ext:junit:1.2.1")
    androidTestImplementation("androidx.test.espresso:espresso-core:3.6.1")
}

// ── Maven Publishing ─────────────────────────────────────────────────────────

afterEvaluate {
    publishing {
        publications {
            create<MavenPublication>("release") {
                from(components["release"])

                groupId = libGroupId
                artifactId = libArtifactId
                version = libVersion

                pom {
                    name.set("Onde")
                    description.set(
                        "On-device inference for Android — run LLMs, diffusion models, " +
                        "and speech-to-text locally with automatic model management."
                    )
                    url.set("https://github.com/ondeinference/onde")
                    inceptionYear.set("2025")

                    licenses {
                        license {
                            name.set("MIT License")
                            url.set("https://opensource.org/licenses/MIT")
                            distribution.set("repo")
                        }
                        license {
                            name.set("Apache License 2.0")
                            url.set("https://www.apache.org/licenses/LICENSE-2.0")
                            distribution.set("repo")
                        }
                    }

                    developers {
                        developer {
                            id.set("ondeinference")
                            name.set("Onde Inference")
                            url.set("https://ondeinference.com")
                        }
                    }

                    scm {
                        url.set("https://github.com/ondeinference/onde")
                        connection.set("scm:git:git://github.com/ondeinference/onde.git")
                        developerConnection.set("scm:git:ssh://git@github.com/ondeinference/onde.git")
                    }

                    issueManagement {
                        system.set("GitHub Issues")
                        url.set("https://github.com/ondeinference/onde/issues")
                    }
                }
            }
        }

        repositories {
            // ── Local Maven (~/.m2/repository) ───────────────────────────────
            // Publish: ./gradlew :onde:publishReleasePublicationToMavenLocal
            mavenLocal()

            // ── Maven Central (Sonatype OSSRH) ──────────────────────────────
            // Publish: ./gradlew :onde:publishReleasePublicationToSonatypeRepository
            //
            // Required properties (in ~/.gradle/gradle.properties or passed via -P):
            //   sonatypeUsername
            //   sonatypePassword
            maven {
                name = "Sonatype"
                val releasesUrl = uri("https://s01.oss.sonatype.org/service/local/staging/deploy/maven2/")
                val snapshotsUrl = uri("https://s01.oss.sonatype.org/content/repositories/snapshots/")
                url = if (libVersion.endsWith("-SNAPSHOT")) snapshotsUrl else releasesUrl

                credentials {
                    username = findProperty("sonatypeUsername") as? String ?: System.getenv("SONATYPE_USERNAME") ?: ""
                    password = findProperty("sonatypePassword") as? String ?: System.getenv("SONATYPE_PASSWORD") ?: ""
                }
            }

            // ── GitHub Packages ──────────────────────────────────────────────
            // Publish: ./gradlew :onde:publishReleasePublicationToGitHubPackagesRepository
            //
            // Required properties:
            //   gpr.user   — GitHub username
            //   gpr.token  — GitHub personal access token (write:packages)
            maven {
                name = "GitHubPackages"
                url = uri("https://maven.pkg.github.com/ondeinference/onde")

                credentials {
                    username = findProperty("gpr.user") as? String ?: System.getenv("GITHUB_ACTOR") ?: ""
                    password = findProperty("gpr.token") as? String ?: System.getenv("GITHUB_TOKEN") ?: ""
                }
            }
        }
    }

    // ── GPG Signing ──────────────────────────────────────────────────────────
    // Required for Maven Central.  Skipped entirely when signing credentials
    // are absent (e.g. local development, CI dry-runs, mavenLocal publishes).
    //
    // Configure in ~/.gradle/gradle.properties:
    //   signing.keyId=AABBCCDD            (last 8 chars of your GPG key ID)
    //   signing.password=<passphrase>
    //   signing.secretKeyRingFile=/path/to/secring.gpg
    //
    // Or use in-memory key (CI-friendly):
    //   ORG_GRADLE_PROJECT_signingKey=<ascii-armored-key>
    //   ORG_GRADLE_PROJECT_signingPassword=<passphrase>
    val signingKey: String? = findProperty("signingKey") as? String ?: System.getenv("GPG_SIGNING_KEY")
    val signingPassword: String? = findProperty("signingPassword") as? String ?: System.getenv("GPG_SIGNING_PASSWORD")
    val hasKeyRingFile = findProperty("signing.secretKeyRingFile") != null
    val hasSigningCredentials = (signingKey != null && signingPassword != null) || hasKeyRingFile

    if (hasSigningCredentials) {
        signing {
            if (signingKey != null && signingPassword != null) {
                useInMemoryPgpKeys(signingKey, signingPassword)
            }
            sign(publishing.publications["release"])
        }
    }
}
