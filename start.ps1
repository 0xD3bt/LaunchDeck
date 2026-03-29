$ErrorActionPreference = "Stop"

$projectRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
$engineManifestPath = (Join-Path $projectRoot "rust\launchdeck-engine\Cargo.toml").ToLowerInvariant()
$launchDeckLogDir = Join-Path $projectRoot ".local\launchdeck"

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

function Stop-OldLaunchDeckRuntime {
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

  return @{
    EnginePort = $enginePort
    FollowDaemonPort = $followDaemonPort
  }
}

function Wait-ForHealthEndpoint {
  param(
    [Parameter(Mandatory = $true)]
    [string]$Url,
    [Parameter(Mandatory = $true)]
    [string]$Name,
    [int]$MaxAttempts = 20,
    [int]$DelayMilliseconds = 500
  )

  for ($attempt = 0; $attempt -lt $MaxAttempts; $attempt++) {
    Start-Sleep -Milliseconds $DelayMilliseconds
    try {
      $response = Invoke-RestMethod -UseBasicParsing $Url -TimeoutSec 2
      if (
        ($null -ne $response.ok -and $response.ok -eq $true) -or
        ($null -ne $response.running -and $response.running -eq $true)
      ) {
        return $true
      }
    } catch {
      # Service may still be starting.
    }
  }

  Write-Warning "$Name did not report healthy startup at $Url."
  return $false
}

function Start-LaunchDeckProcesses {
  $ports = Stop-OldLaunchDeckRuntime
  New-Item -ItemType Directory -Path $launchDeckLogDir -Force | Out-Null

  $daemonStdoutPath = Join-Path $launchDeckLogDir "follow-daemon.log"
  $daemonStderrPath = Join-Path $launchDeckLogDir "follow-daemon-error.log"
  Start-Process `
    -FilePath "cargo" `
    -ArgumentList @("run", "--manifest-path", "rust/launchdeck-engine/Cargo.toml", "--bin", "launchdeck-follow-daemon") `
    -WorkingDirectory $projectRoot `
    -WindowStyle Hidden `
    -RedirectStandardOutput $daemonStdoutPath `
    -RedirectStandardError $daemonStderrPath | Out-Null

  $daemonHealthy = Wait-ForHealthEndpoint `
    -Url "http://127.0.0.1:$($ports.FollowDaemonPort)/health" `
    -Name "LaunchDeck follow daemon" `
    -MaxAttempts 40 `
    -DelayMilliseconds 500

  $stdoutPath = Join-Path $launchDeckLogDir "engine.log"
  $stderrPath = Join-Path $launchDeckLogDir "engine-error.log"

  Start-Process `
    -FilePath "cargo" `
    -ArgumentList @("run", "--manifest-path", "rust/launchdeck-engine/Cargo.toml", "--bin", "launchdeck-engine") `
    -WorkingDirectory $projectRoot `
    -WindowStyle Hidden `
    -RedirectStandardOutput $stdoutPath `
    -RedirectStandardError $stderrPath | Out-Null

  $engineHealthy = Wait-ForHealthEndpoint `
    -Url "http://127.0.0.1:$($ports.EnginePort)/health" `
    -Name "LaunchDeck Rust host" `
    -MaxAttempts 20 `
    -DelayMilliseconds 500

  if ($daemonHealthy) {
    Write-Host "LaunchDeck follow daemon ready on port $($ports.FollowDaemonPort)."
  } else {
    Write-Warning "Check .local\launchdeck\follow-daemon-error.log if the follow daemon failed to start."
  }

  if ($engineHealthy) {
    Write-Host "LaunchDeck Rust host ready on port $($ports.EnginePort)."
    Start-Process "http://127.0.0.1:$($ports.EnginePort)" | Out-Null
  } else {
    Write-Warning "Check .local\launchdeck\engine-error.log if the Rust host failed to start."
  }
}

Set-Location $projectRoot
Start-LaunchDeckProcesses
