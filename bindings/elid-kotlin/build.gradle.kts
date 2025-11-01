plugins {
    kotlin("jvm") version "2.0.21"
    id("maven-publish")
    id("signing")
}

group = "com.elid"
version = "0.1.0"

repositories {
    mavenCentral()
}

dependencies {
    // JNA for native library loading
    implementation("net.java.dev.jna:jna:5.14.0")

    // Kotlin standard library
    implementation(kotlin("stdlib"))

    // Testing dependencies
    testImplementation(kotlin("test"))
    testImplementation("org.junit.jupiter:junit-jupiter:5.10.1")
    testImplementation("com.google.code.gson:gson:2.10.1")
}

kotlin {
    jvmToolchain(21)
}

tasks.test {
    useJUnitPlatform()

    // Set library path for native library loading
    // Use the workspace target directory (shared by all crates)
    systemProperty("java.library.path", "${projectDir}/../../target/release")

    // Enable test logging
    testLogging {
        events("passed", "skipped", "failed")
        showStandardStreams = true
    }
}

tasks.withType<org.jetbrains.kotlin.gradle.tasks.KotlinCompile> {
    kotlinOptions {
        jvmTarget = "21"
        freeCompilerArgs = listOf("-Xjsr305=strict")
    }
}

// Task to copy native libraries for distribution
tasks.register<Copy>("copyNativeLibs") {
    description = "Copy native libraries from workspace target directory"
    group = "build"

    from("../../target/release") {
        include("libelid_ffi.so")
        include("libelid_ffi.dylib")
        include("elid_ffi.dll")
    }
    into("${buildDir}/libs/native")
}

// Task to build the Rust library first
tasks.register<Exec>("buildRustLib") {
    description = "Build the Rust elid-ffi library"
    group = "build"

    workingDir = file("../..")  // Workspace root
    commandLine = listOf("cargo", "build", "--release", "-p", "elid-ffi")
}

// Make jar depend on Rust build
tasks.named("jar") {
    dependsOn("buildRustLib", "copyNativeLibs")
}

publishing {
    publications {
        create<MavenPublication>("maven") {
            from(components["java"])

            pom {
                name.set("ELID Kotlin")
                description.set("Kotlin bindings for ELID - Embedding Locality-preserving IDentifiers")
                url.set("https://github.com/your-org/elid")

                licenses {
                    license {
                        name.set("MIT License")
                        url.set("https://opensource.org/licenses/MIT")
                    }
                    license {
                        name.set("Apache License 2.0")
                        url.set("https://www.apache.org/licenses/LICENSE-2.0")
                    }
                }

                developers {
                    developer {
                        id.set("elid-team")
                        name.set("ELID Team")
                    }
                }

                scm {
                    connection.set("scm:git:git://github.com/your-org/elid.git")
                    developerConnection.set("scm:git:ssh://github.com/your-org/elid.git")
                    url.set("https://github.com/your-org/elid")
                }
            }
        }
    }

    repositories {
        maven {
            name = "OSSRH"
            val releasesRepoUrl = uri("https://s01.oss.sonatype.org/service/local/staging/deploy/maven2/")
            val snapshotsRepoUrl = uri("https://s01.oss.sonatype.org/content/repositories/snapshots/")
            url = if (version.toString().endsWith("SNAPSHOT")) snapshotsRepoUrl else releasesRepoUrl

            credentials {
                username = project.findProperty("ossrhUsername") as String? ?: System.getenv("OSSRH_USERNAME")
                password = project.findProperty("ossrhPassword") as String? ?: System.getenv("OSSRH_PASSWORD")
            }
        }
    }
}

signing {
    sign(publishing.publications["maven"])
}
