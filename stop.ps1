$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$engineManifestPath = (Join-Path $projectRoot "rust\launchdeck-engine\Cargo.toml").ToLowerInvariant()

function Get-ConfiguredNumericSetting {
  param(
    [Parameter(Mandatory = $true)]
    [string[]]$VariableNames,
    [Parameter(Mandatory = $true)]
    [int]$DefaultValue
  )

  foreach ($fileName in @(".env", ".env.local", ".env.example")) {
    $filePath = Join-Path $projectRoot $fileName
    if (-not (Test-Path $filePath)) {
      continue
    }
    foreach ($variableName in $VariableNames) {
      $pattern = "^\s*$([regex]::Escape($variableName))\s*=\s*(\d+)\s*$"
      $match = Select-String -Path $filePath -Pattern $pattern | Select-Object -First 1
      if ($match) {
        return [int]$match.Matches[0].Groups[1].Value
      }
    }
  }
  return $DefaultValue
}

function Get-ConfiguredEnginePort {
  return Get-ConfiguredNumericSetting -VariableNames @("LAUNCHDECK_PORT") -DefaultValue 8789
}

function Get-ConfiguredFollowDaemonPort {
  return Get-ConfiguredNumericSetting -VariableNames @("LAUNCHDECK_FOLLOW_DAEMON_PORT") -DefaultValue 8790
}

function Stop-LaunchDeckProcess {
  param(
    [Parameter(Mandatory = $true)]
    [int]$ProcessId,
    [Parameter(Mandatory = $true)]
    [string]$Reason
  )

  if ($ProcessId -eq $PID) {
    return
  }

  try {
    Stop-Process -Id $ProcessId -Force -ErrorAction Stop
    Write-Host "Stopped process $ProcessId ($Reason)."
  } catch {
    Write-Warning "Failed to stop process $ProcessId ($Reason): $($_.Exception.Message)"
  }
}

function Stop-ProcessesListeningOnPort {
  param(
    [Parameter(Mandatory = $true)]
    [int]$Port,
    [Parameter(Mandatory = $true)]
    [AllowEmptyCollection()]
    [System.Collections.Generic.HashSet[int]]$KnownPids,
    [Parameter(Mandatory = $true)]
    [string]$Label
  )

  $netstatOutput = netstat -ano -p tcp | Select-String -Pattern "127\.0\.0\.1:$Port\s+.*LISTENING\s+(\d+)$"
  foreach ($line in $netstatOutput) {
    if ($line.Matches.Count -eq 0) {
      continue
    }
    $matchedPid = [int]$line.Matches[0].Groups[1].Value
    if ($KnownPids.Add($matchedPid)) {
      Stop-LaunchDeckProcess -ProcessId $matchedPid -Reason "$Label listener on port $Port"
    }
  }
}

function Stop-LaunchDeckRuntime {
  $knownPids = New-Object System.Collections.Generic.HashSet[int]

  $processes = Get-CimInstance Win32_Process | Where-Object {
    $_.ProcessId -ne $PID -and
    $_.CommandLine -and
    (
      $_.CommandLine.ToLowerInvariant().Contains($engineManifestPath) -or
      $_.CommandLine.ToLowerInvariant().Contains("launchdeck-engine") -or
      $_.CommandLine.ToLowerInvariant().Contains("launchdeck-follow-daemon") -or
      $_.CommandLine.ToLowerInvariant().Contains("ui-server.js")
    )
  }

  foreach ($process in $processes) {
    if ($knownPids.Add([int]$process.ProcessId)) {
      Stop-LaunchDeckProcess -ProcessId ([int]$process.ProcessId) -Reason "existing LaunchDeck runtime"
    }
  }

  $enginePort = Get-ConfiguredEnginePort
  $followDaemonPort = Get-ConfiguredFollowDaemonPort
  Stop-ProcessesListeningOnPort -Port $enginePort -KnownPids $knownPids -Label "LaunchDeck engine"
  Stop-ProcessesListeningOnPort -Port $followDaemonPort -KnownPids $knownPids -Label "LaunchDeck follow daemon"

  if ($knownPids.Count -eq 0) {
    Write-Host "No running LaunchDeck engine or follow-daemon processes were found."
  } else {
    Write-Host "LaunchDeck runtime stopped."
  }
}

Set-Location $projectRoot
Stop-LaunchDeckRuntime
