# Connecting a vault on Android: the failure and its fix (2026-11-23)

Owner report: *"Connecting a vault doesn't work on the tablet, find out why."*

## What the devices showed

- **Pad 3** (Android 16, `v.26.11.23-001`). It was asleep and locked when this lane started, so nothing was driven on
  it. Its log buffer still held the owner's own four attempts (`pad_logcat_excerpt.txt`, 22:44:02, 22:44:32,
  22:44:45 and 22:47:10). Each attempt shows the picker opening (`FilePicker: Error cannot set initial directory`,
  then `OnPause`) and the app resuming. After that the app process logged **nothing**: no error, no exception.
  `dumpsys activity permissions` shows a persisted grant on one tree, created at 22:47:21, which is the moment the
  last attempt returned. The folder name is redacted here.
- **6T** (Android 15, the same build). Driven with `adb shell input`: MENU ▸ Records ▸ Markdown vault… ▸ CONNECT
  VAULT…. In the picker, a new folder `CartalithScratchVault` was created at the top of shared storage, then USE
  THIS FOLDER, then ALLOW. DocumentsUI returned the tree URI (`PickerActionHandler: onFinished()`), and the grant
  was persisted (`persisted=0x3`). The window still read "No Markdown vault connected", and the app logged
  nothing (`t6_logcat_excerpt.txt`). **The failure is the same on both devices, so it is Android-wide.** It does
  not depend on the tablet form.
- `FilePicker: Error cannot set initial directory` appears on every attempt on both devices, and the picker opens
  anyway. It comes from passing an empty start directory and is harmless.

## Where it fails

The persisted grant proves the app's own code ran as far as `vault_saf_android.gd::persist_grant`. Godot 4.7.2's
`FilePicker.handleActivityResult` does not take the grant; it was read from the shipped
`godot-lib.template_release.aar` with `javap`. It also hands GDScript the URI's `toString()` unchanged. So
`prepare()` and `persist_grant()` held, and the refusal comes later: `connect_with_driver` →
`vault_connect_saf` → `VaultSession::connect_provider` → `provider.available()` → the handler's `"available"` →
`driver.available()` → `root_id()` → `stat(root)` → `_resolver.query(uri, cols, null, null, null)`.

## Cause (read at the source; a device connect is still owed)

The source is Godot 4.7.2-stable `platform/android/java_class_wrapper.cpp`, `JavaClass::_call_method`. A `String`
or `String[]` parameter accepts only a string (`is_string()`) and never `null`. An object parameter does accept
`null`. The 5-argument `ContentResolver.query(Uri, String[], String, String[], String)` with three `null`s
therefore matches no overload, and the wrapper answers `null`. A Java exception is also silent: the wrapper stores
it for `JavaClassWrapper.get_exception()` and prints nothing. A release build does not report a failed call either.
So `stat` answered "the document could not be read", `available()` answered false, and Rust refused with
`"<tree uri>" is not reachable`. That text went only to `app.set_status("hint", ...)`. On the 6T no status line is
drawn while the vault window is open, and the line was not seen afterwards. **On the Pad the owner's own screenshot
(world seed 51203) shows it:** the status bar reads `Vault: "content://com.android.externalstorage.documents/tree/<redacted>…`,
cut off at the bar's edge. That is the opening of exactly this refusal, `Vault: "<tree uri>" is not reachable`
(`VaultSession::connect_provider`, raised when `provider.available()` is false). It is not a stored binding: a
refused connect stores nothing. So the Pad's failure point is the one read above. Every `children()` listing and every `create()` name check uses the same call and would have failed the
same way.

A second defect of the same family was read at `JavaObject::callp`, which tries Godot's own methods before Java's.
`grant_is_held()` called `perms.get(i)`. `Object.get` is a Godot method, and a release build does not check its
argument type, so the call reads a Godot property and answers null. Restoring a saved vault at launch would then
always have failed silently.

## Fix

- `vault_saf_android.gd::_query` calls the 4-argument overload `query(Uri, String[], Bundle, CancellationSignal)`
  (API 26), whose `null`s are objects. All four former call sites (`children`, `stat` and `create`'s name check)
  go through it.
- `grant_is_held` walks the list with `iterator()`, `hasNext()` and `next()`, none of which `Object` has.
- `vault_saf_connect.gd::connect_android` prints one line to logcat on every attempt: `[vault-saf] connect ok`,
  or `[vault-saf] connect refused: <step>`, where `<step>` comes from `diagnose()`: root_id, stat(root) or
  children(root), plus the Java exception's class. Neither the folder address nor the exception message is
  printed, because both can carry the owner's folder name.

## Verified off-device

- `_vaultpadsaf_probe.gd` (headless) runs the real driver against fakes that apply the wrapper's argument rule,
  then the real handler and Rust provider: **10 checks, 0 failed**. A1 and A2 are positive controls; they show the
  fake refuses what Godot refuses.
- `mutants.py`: `five_arg_query` is KILLED (B1-B6 and B8 go red, and B8 then reads `stat(root) failed`);
  `get_by_index` is KILLED (B7).
- Regression: `_vesaf_probe.gd` 77 checks, 0 failed; `_vaultsaf_probe.gd` PASS.

## What the next APK must be checked for (Pad, then the 6T)

1. MENU ▸ Records ▸ Markdown vault… ▸ CONNECT VAULT…, then a new empty scratch folder in the picker, then USE THIS
   FOLDER and ALLOW. The window must show the connection header (`<name> · 0 notes`, "Change vault…").
   `adb logcat | grep vault-saf` must show `[vault-saf] connect ok`. If it shows `connect refused: <step>`, that
   step is the next fault.
2. Create a note into the scratch vault from the window. It must appear in the file tree, and a second open must
   read it back.
3. Close the app fully, then reopen it. The scratch vault must still be connected; that is `restore` and
   `grant_is_held`.
4. Then the owner's own vault, by the owner.

## Left on the devices

The 6T still holds the empty folder `CartalithScratchVault` at the top of shared storage, with the app's persisted
grant on it. The device rules allow only input, screencap, logcat and reads, so there was no clean way to delete
it. It is also the folder to use for step 1 above. Delete it with Files when it is no longer needed. Screenshots
are outside the repository (they show the owner's home screen and folders).
