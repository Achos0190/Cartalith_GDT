# cartalith-pen: the read-only Stylo 2 plugin (Ruling EK)

Kotlin source of the Godot v2 Android plugin CartalithPen. Its GDScript half is godot-project/shell/pen_link.gd.
Scope, bars, risks: STYLO2_PEN_PROPOSAL.md (root). Pack it fixes: docs/evidence/stylus-2026-11-08/cartalith-pen/.

Build (JDK 21, Android SDK with platform 36, the Gradle 8.13 distribution Godot also uses):

    JAVA_HOME="C:/Program Files/Microsoft/jdk-21.0.13.21-hotspot" ANDROID_HOME="$LOCALAPPDATA/Android/Sdk" \n      <gradle-8.13>/bin/gradle assembleRelease test
    cp build/outputs/aar/pen-plugin-release.aar ../../godot-project/android/libs/cartalith_pen.aar

The app Gradle build lists that file as a local dependency, so no editor plugin and no project.godot change is needed.
See ANDROID_BUILD_SCOPE.md for the APK recipe.
