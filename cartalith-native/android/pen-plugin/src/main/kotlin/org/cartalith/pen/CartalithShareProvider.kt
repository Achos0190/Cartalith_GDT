package org.cartalith.pen

import androidx.core.content.FileProvider

/**
 * The FileProvider behind File > Share project, as a distinct subclass.
 *
 * Why a subclass: Godot's Android template already declares `androidx.core.content.FileProvider` (authority
 * `<applicationId>.fileprovider`, paths `godot_provider_paths`). A second provider of the SAME class makes Gradle's
 * manifest merger fail on `authorities` and on the `android.support.FILE_PROVIDER_PATHS` meta-data, so the Godot
 * export errors out in processStandardReleaseMainManifest even though this plugin's own Gradle build passes. A
 * subclass has its own component name, so both providers coexist.
 *
 * It adds no behaviour. It must never be exported; read access is granted per intent only (Ruling BK annotation).
 */
class CartalithShareProvider : FileProvider()
