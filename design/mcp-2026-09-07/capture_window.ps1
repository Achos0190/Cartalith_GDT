# Capture ONE window by its title, not the whole screen.
#
# Why by-window and not the primary display: a full-screen grab picks up
# whatever else the owner has open. This project's captures are attached to
# reports and committed alongside them, so the capture must contain only the
# thing under test.
param(
  [Parameter(Mandatory=$true)][string]$TitleLike,
  [Parameter(Mandatory=$true)][string]$OutPath
)

Add-Type -AssemblyName System.Windows.Forms, System.Drawing
Add-Type @"
using System;
using System.Runtime.InteropServices;
public class Win {
  [DllImport("user32.dll")] public static extern bool SetForegroundWindow(IntPtr hWnd);
  [DllImport("user32.dll")] public static extern bool ShowWindow(IntPtr hWnd, int nCmdShow);
  [StructLayout(LayoutKind.Sequential)] public struct RECT { public int L, T, R, B; }
  [DllImport("dwmapi.dll")] public static extern int DwmGetWindowAttribute(IntPtr hwnd, int attr, out RECT r, int size);
  [DllImport("user32.dll")] public static extern bool GetWindowRect(IntPtr hWnd, out RECT r);
}
"@

$proc = Get-Process | Where-Object { $_.MainWindowTitle -like "*$TitleLike*" } | Select-Object -First 1
if (-not $proc) { Write-Output "NOWINDOW: no window matching '$TitleLike'"; exit 2 }

$h = $proc.MainWindowHandle
[void][Win]::ShowWindow($h, 9)        # SR_RESTORE
[void][Win]::SetForegroundWindow($h)
Start-Sleep -Milliseconds 900

# DWM bounds exclude the drop shadow that GetWindowRect includes.
$r = New-Object Win+RECT
$ok = [Win]::DwmGetWindowAttribute($h, 9, [ref]$r, 16)   # DWMWA_EXTENDED_FRAME_BOUNDS
if ($ok -ne 0) { [void][Win]::GetWindowRect($h, [ref]$r) }

$w = $r.R - $r.L
$hgt = $r.B - $r.T
if ($w -le 0 -or $hgt -le 0) { Write-Output "BADRECT: ${w}x${hgt}"; exit 3 }

$bmp = New-Object System.Drawing.Bitmap $w, $hgt
$g = [System.Drawing.Graphics]::FromImage($bmp)
$g.CopyFromScreen((New-Object System.Drawing.Point $r.L, $r.T), [System.Drawing.Point]::Empty, (New-Object System.Drawing.Size $w, $hgt))
$bmp.Save($OutPath, [System.Drawing.Imaging.ImageFormat]::Png)
$g.Dispose(); $bmp.Dispose()
Write-Output "OK ${w}x${hgt} -> $OutPath"
