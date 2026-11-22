; Cartalith Windows installer (Ruling HH, 2026-11-22). Inno Setup 6, per-user, unsigned.
; Build with tools/build_installer.sh, which passes both defines below:
;   ISCC /DExportDir=<Windows export folder> /DAppVer=<contents of version.txt> /O<out> cartalith.iss
; Unsigned on purpose (Ruling HH): SmartScreen shows "unknown publisher" until signing is revisited.

#ifndef ExportDir
  #error Pass /DExportDir=<folder holding Cartalith.exe>
#endif
#ifndef AppVer
  #error Pass /DAppVer=<version.txt contents>
#endif

[Setup]
; Fixed forever: Inno matches an existing install by AppId, so a newer setup upgrades in place.
AppId={{6C1F6B0E-3D5A-4B8F-9A47-C7A2E5D01B22}
AppName=Cartalith
AppVersion={#AppVer}
AppPublisher=Cartalith
; Per-user: no admin prompt, lands in {localappdata}\Programs\Cartalith.
PrivilegesRequired=lowest
DefaultDirName={autopf}\Cartalith
DefaultGroupName=Cartalith
DisableProgramGroupPage=yes
ArchitecturesAllowed=x64compatible
ArchitecturesInstallIn64BitMode=x64compatible
UninstallDisplayIcon={app}\Cartalith.exe
; An upgrade over a running copy: Restart Manager closes it so files are not locked.
CloseApplications=yes
RestartApplications=no
OutputBaseFilename=Cartalith-Setup-{#AppVer}
Compression=lzma2
SolidCompression=yes
WizardStyle=modern

[Tasks]
Name: desktopicon; Description: "Create a &desktop shortcut"; Flags: unchecked

[Files]
; Exactly what the export needs. The godotsteam / steam_api64 dlls are deliberately NOT
; listed: no script or crate uses Steam and the pck excludes addons/godotsteam, so they are
; unused at runtime. Only the export outputs below are installed.
Source: "{#ExportDir}\Cartalith.exe"; DestDir: "{app}"; Flags: ignoreversion
; The console wrapper exists only when the preset's console wrapper covers the export kind
; (debug/export_console_wrapper=1 is debug-only, so a --export-release has none): optional.
Source: "{#ExportDir}\Cartalith.console.exe"; DestDir: "{app}"; Flags: ignoreversion skipifsourcedoesntexist
Source: "{#ExportDir}\Cartalith.pck"; DestDir: "{app}"; Flags: ignoreversion
Source: "{#ExportDir}\cartalith_godot.dll"; DestDir: "{app}"; Flags: ignoreversion

; Settings and worlds live in %APPDATA%\Godot\app_userdata. There is deliberately no [Dirs],
; [Files] or [UninstallDelete] entry for it, so uninstalling never touches user data.

[Icons]
Name: "{group}\Cartalith"; Filename: "{app}\Cartalith.exe"
Name: "{group}\Uninstall Cartalith"; Filename: "{uninstallexe}"
Name: "{autodesktop}\Cartalith"; Filename: "{app}\Cartalith.exe"; Tasks: desktopicon

[Run]
Filename: "{app}\Cartalith.exe"; Description: "Launch Cartalith"; Flags: nowait postinstall skipifsilent
