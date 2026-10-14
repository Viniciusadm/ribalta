plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

val nativeLibs = layout.buildDirectory.dir("generated/jniLibs")
val languageDir = rootProject.layout.projectDirectory.dir("..")

val buildNative by tasks.registering(Exec::class) {
    description = "Builds the Ribalta compiler for Android with the NDK"
    commandLine(rootProject.layout.projectDirectory.file("scripts/build-native.sh").asFile.absolutePath, nativeLibs.get().asFile.absolutePath)
    inputs.dir(languageDir.dir("crates/ribalta/src"))
    inputs.dir(languageDir.dir("crates/ribalta-jni/src"))
    outputs.dir(nativeLibs)
}

val buildHostNative by tasks.registering(Exec::class) {
    description = "Builds the Ribalta compiler for the host, used by unit tests"
    commandLine("cargo", "build", "-p", "ribalta-jni", "--manifest-path", languageDir.file("Cargo.toml").asFile.absolutePath)
    inputs.dir(languageDir.dir("crates/ribalta/src"))
    inputs.dir(languageDir.dir("crates/ribalta-jni/src"))
    outputs.file(languageDir.file("target/debug/libribalta_jni.so"))
}

android {
    namespace = "com.archieapps.ribaltareader"
    compileSdk {
        version = release(37) {
            minorApiLevel = 1
        }
    }

    defaultConfig {
        applicationId = "com.archieapps.ribaltareader"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "1.0"

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
    }

    buildTypes {
        release {
            optimization {
                enable = false
            }
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_11
        targetCompatibility = JavaVersion.VERSION_11
    }
    buildFeatures {
        compose = true
    }
    sourceSets {
        getByName("main").jniLibs.directories.add(nativeLibs.get().asFile.absolutePath)
        getByName("test").resources.directories.add(languageDir.dir("crates/ribalta/conformance").asFile.absolutePath)
    }
    testOptions {
        unitTests.isIncludeAndroidResources = true
        unitTests.all {
            it.dependsOn(buildHostNative)
            it.systemProperty("java.library.path", languageDir.dir("target/debug").asFile.absolutePath)
            it.systemProperty("ribalta.screenshots", layout.buildDirectory.dir("screenshots").get().asFile.absolutePath)
            System.getenv("RIBALTA_DOCUMENTS")?.let { documents -> it.systemProperty("ribalta.documents", documents) }
            it.systemProperty("ribalta.docs", languageDir.dir("docs").asFile.absolutePath)
        }
    }
}

tasks.named("preBuild") {
    dependsOn(buildNative)
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.graphics)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.kotlinx.serialization.json)
    testImplementation(libs.junit)
    testImplementation(libs.robolectric)
    testImplementation(platform(libs.androidx.compose.bom))
    testImplementation(libs.androidx.compose.ui.test.junit4)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.compose.ui.test.junit4)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(libs.androidx.junit)
    debugImplementation(libs.androidx.compose.ui.test.manifest)
    debugImplementation(libs.androidx.compose.ui.tooling)
}