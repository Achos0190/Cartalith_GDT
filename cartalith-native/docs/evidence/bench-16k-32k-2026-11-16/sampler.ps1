# Sidecar sampler for PERFORMANCE_BENCHMARKS.md section 13 (2026-11-16).
# Usage: sampler.ps1 -ProcName <exe name without .exe> -Out <csv> [-IntervalMs 1000]
# Waits for the process, then once per interval writes: unix_ms, process working set,
# peak working set, private bytes (commit), peak page-file bytes, page faults/sec
# (Win32_PerfFormattedData_PerfProc_Process), and the system's paging-file %usage and
# peak, pages-input/sec (hard faults), available MB. Stops when the process exits.
param([string]$ProcName,[string]$Out,[int]$IntervalMs=1000)
$ErrorActionPreference='SilentlyContinue'
"unix_ms,ws_mb,ws_peak_mb,private_mb,pagefile_peak_mb,faults_per_s,pf_used_pct,pf_peak_pct,pages_in_per_s,avail_mb,sys_commit_mb,pf_used_mb,pf_alloc_mb,proc_cpu_pct,sys_cpu_pct" | Out-File $Out -Encoding ascii
$proc=$null
while(-not $proc){ $proc=Get-Process -Name $ProcName | Select -First 1; Start-Sleep -Milliseconds 100 }
$procid=$proc.Id
while($true){
  $p=Get-CimInstance Win32_PerfFormattedData_PerfProc_Process -Filter "IDProcess=$procid"
  if(-not $p){ break }
  $pf=Get-CimInstance Win32_PerfFormattedData_PerfOS_PagingFile -Filter "Name='_Total'"
  $m=Get-CimInstance Win32_PerfFormattedData_PerfOS_Memory
  $pu=Get-CimInstance Win32_PageFileUsage
  $sc=Get-CimInstance Win32_PerfFormattedData_PerfOS_Processor -Filter "Name='_Total'"
  $t=[DateTimeOffset]::UtcNow.ToUnixTimeMilliseconds()
  "{0},{1:F0},{2:F0},{3:F0},{4:F0},{5},{6},{7},{8},{9},{10:F0},{11},{12},{13},{14}" -f $t,($p.WorkingSet/1MB),($p.WorkingSetPeak/1MB),($p.WorkingSetPrivate/1MB),($p.PageFileBytesPeak/1MB),$p.PageFaultsPersec,$pf.PercentUsage,$pf.PercentUsagePeak,$m.PagesInputPersec,$m.AvailableMBytes,($m.CommittedBytes/1MB),$pu.CurrentUsage,$pu.AllocatedBaseSize,$p.PercentProcessorTime,$sc.PercentProcessorTime | Add-Content $Out -Encoding ascii
  Start-Sleep -Milliseconds $IntervalMs
}
