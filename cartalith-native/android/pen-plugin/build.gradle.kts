// Ruling EK: one Godot v2 Android plugin, read-only, off by default. The Godot library is compileOnly: the app's
// own build supplies it at run time (a second copy in the .aar would clash).
plugins {
    id("com.android.library") version "8.6.1"
    id("org.jetbrains.kotlin.android") version "2.1.21"
}

android {
    namespace = "org.cartalith.pen"
    compileSdk = 36
    defaultConfig {
        // 24 equals Godot 4.7.2's template minSdk, so the manifest merge accepts it. The client's API-33 GATT
        // calls and API-31 permission are guarded at run time instead (PenSupport), never by raising the app's minSdk.
        minSdk = 24
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions { jvmTarget = "17" }
}

dependencies {
    compileOnly("org.godotengine:godot:4.7.2.stable")
    testImplementation("junit:junit:4.13.2")
}
