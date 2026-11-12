# cartalith-pen: the read-only Stylo 2 plugin (Ruling EK)

Kotlin source of the Godot v2 Android plugin CartalithPen. Its GDScript half is godot-project/shell/pen_link.gd.
Scope, bars, risks: STYLO2_PEN_PROPOSAL.md (root). Pack it fixes: docs/evidence/stylus-2026-11-08/cartalith-pen/.

Build (JDK 21, Android SDK with platform 36, the Gradle 8.13 distribution Godot also uses):

    JAVA_HOME="C:/Program Files/Microsoft/jdk-21.0.13.21-hotspot" ANDROID_HOME="$LOCALAPPDATA/Android/Sdk" \
      <gradle-8.13>/bin/gradle assembleRelease test
    cp build/outputs/aar/cartalith-pen-release.aar ../../godot-project/addons/cartalith_pen.aar

The output is named after the Gradle project (`rootProject.name = "cartalith-pen"` in settings.gradle.kts), so it is
`cartalith-pen-release.aar`. The copy must sit at the root of `godot-project/addons/`: Godot's Android template picks up
`addons/*.aar` with a non-recursive file tree, so a subfolder (or `android/libs/`) is silently ignored and the plugin is
missing from the APK. No editor plugin and no project.godot change is needed.
See ANDROID_BUILD_SCOPE.md section 3.14 for the APK recipe.
