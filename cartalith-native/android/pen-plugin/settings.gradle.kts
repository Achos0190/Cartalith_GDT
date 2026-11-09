// The Stylo 2 pen plugin's own Gradle project (Ruling EK; STYLO2_PEN_PROPOSAL.md section 6). Versions match what
// Godot 4.7.2's Android template pins in its config.gradle (AGP 8.6.1, Kotlin 2.1.21) so the .aar merges cleanly
// into the app's Gradle build. Build: see README.md in this directory.
pluginManagement {
    repositories {
        google()
        mavenCentral()
        gradlePluginPortal()
    }
}
dependencyResolutionManagement {
    repositories {
        google()
        mavenCentral()
    }
}
rootProject.name = "cartalith-pen"
