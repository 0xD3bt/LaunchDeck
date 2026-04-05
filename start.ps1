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
      $_.CommandLine.ToLowerInvariant().Contains("launchdeck-follow-daemon")
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
    [Parameter(Mandatory = $true)]
    [int]$ProcessId,
    [string]$LogPath = "",
    [int]$MaxAttempts = 20,
    [int]$DelayMilliseconds = 500,
    [int]$SlowNoticeAttempt = 10
  )

  $slowNoticeShown = $false

  for ($attempt = 0; $attempt -lt $MaxAttempts; $attempt++) {
    try {
      $response = Invoke-RestMethod -UseBasicParsing $Url -TimeoutSec 2
      if (
        ($null -ne $response.ok -and $response.ok -eq $true) -or
        ($null -ne $response.running -and $response.running -eq $true)
      ) {
        return @{
          State = "healthy"
          Url = $Url
        }
      }
    } catch {
      # Service may still be starting.
    }

    if (-not (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue)) {
      return @{
        State = "failed"
        Url = $Url
        LogPath = $LogPath
      }
    }

    if (-not $slowNoticeShown -and ($attempt + 1) -ge $SlowNoticeAttempt) {
      Write-Host "$Name is still compiling or starting..."
      $slowNoticeShown = $true
    }

    Start-Sleep -Milliseconds $DelayMilliseconds
  }

  if (Get-Process -Id $ProcessId -ErrorAction SilentlyContinue) {
    return @{
      State = "starting"
      Url = $Url
      LogPath = $LogPath
    }
  }

  return @{
    State = "failed"
    Url = $Url
    LogPath = $LogPath
  }
}

function Start-LaunchDeckProcesses {
  $ports = Stop-OldLaunchDeckRuntime
  New-Item -ItemType Directory -Path $launchDeckLogDir -Force | Out-Null

  $daemonStdoutPath = Join-Path $launchDeckLogDir "follow-daemon.log"
  $daemonStderrPath = Join-Path $launchDeckLogDir "follow-daemon-error.log"
  $daemonProcess = Start-Process `
    -FilePath "cargo" `
    -ArgumentList @("run", "--manifest-path", "rust/launchdeck-engine/Cargo.toml", "--bin", "launchdeck-follow-daemon") `
    -WorkingDirectory $projectRoot `
    -WindowStyle Hidden `
    -PassThru `
    -RedirectStandardOutput $daemonStdoutPath `
    -RedirectStandardError $daemonStderrPath

  $stdoutPath = Join-Path $launchDeckLogDir "engine.log"
  $stderrPath = Join-Path $launchDeckLogDir "engine-error.log"

  $engineProcess = Start-Process `
    -FilePath "cargo" `
    -ArgumentList @("run", "--manifest-path", "rust/launchdeck-engine/Cargo.toml", "--bin", "launchdeck-engine") `
    -WorkingDirectory $projectRoot `
    -WindowStyle Hidden `
    -PassThru `
    -RedirectStandardOutput $stdoutPath `
    -RedirectStandardError $stderrPath

  $daemonResult = Wait-ForHealthEndpoint `
    -Url "http://127.0.0.1:$($ports.FollowDaemonPort)/health" `
    -Name "LaunchDeck follow daemon" `
    -ProcessId $daemonProcess.Id `
    -LogPath $daemonStderrPath `
    -MaxAttempts 40 `
    -DelayMilliseconds 500

  $engineResult = Wait-ForHealthEndpoint `
    -Url "http://127.0.0.1:$($ports.EnginePort)/health" `
    -Name "LaunchDeck Rust host" `
    -ProcessId $engineProcess.Id `
    -LogPath $stderrPath `
    -MaxAttempts 60 `
    -DelayMilliseconds 500

  switch ($daemonResult.State) {
    "healthy" {
      Write-Host "LaunchDeck follow daemon ready on port $($ports.FollowDaemonPort)."
    }
    "starting" {
      Write-Host "LaunchDeck follow daemon is still starting in the background on port $($ports.FollowDaemonPort)."
      if ($daemonResult.LogPath) {
        Write-Host "Info: Check $($daemonResult.LogPath) only if it does not become healthy soon."
      }
    }
    default {
      Write-Error "LaunchDeck follow daemon exited before reporting healthy startup at $($daemonResult.Url)."
      if ($daemonResult.LogPath) {
        Write-Error "Check $($daemonResult.LogPath) for details."
      }
    }
  }

  switch ($engineResult.State) {
    "healthy" {
      Write-Host "LaunchDeck Rust host ready on port $($ports.EnginePort)."
      Start-Process "http://127.0.0.1:$($ports.EnginePort)" | Out-Null
    }
    "starting" {
      Write-Host "LaunchDeck Rust host is still starting in the background on port $($ports.EnginePort)."
      if ($engineResult.LogPath) {
        Write-Host "Info: Check $($engineResult.LogPath) only if it does not become healthy soon."
      }
    }
    default {
      Write-Error "LaunchDeck Rust host exited before reporting healthy startup at $($engineResult.Url)."
      if ($engineResult.LogPath) {
        Write-Error "Check $($engineResult.LogPath) for details."
      }
    }
  }
}

Set-Location $projectRoot
Start-LaunchDeckProcesses
