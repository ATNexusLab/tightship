param(
    [string]$Harness,

    [string]$Contract,

    [string]$Memory,

    [string]$StatusLine,

    [string]$AutoUpdate,

    [string]$VaultFolder,

    [string]$InitVault,

    [string]$FromDir,

    [Alias("y")]
    [switch]$Yes,

    [Alias("h")]
    [switch]$Help
)

$ReleaseTag = ""
$Repository = "{{repository}}"

& {
$callerEnvironment = @{}
foreach ($variable in "TIGHTSHIP_DATA", "TIGHTSHIP_SETTINGS", "TIGHTSHIP_ROOT") {
    $callerEnvironment[$variable] = [Environment]::GetEnvironmentVariable($variable)
}
$callerEncoding = [Console]::OutputEncoding
$workDir = Join-Path ([System.IO.Path]::GetTempPath()) ("tightship-" + [System.Guid]::NewGuid().ToString())
$colorOut = -not $env:NO_COLOR -and $env:TERM -ne "dumb" -and -not [Console]::IsOutputRedirected
$colorErr = -not $env:NO_COLOR -and $env:TERM -ne "dumb" -and -not [Console]::IsErrorRedirected
$interactive = $false

function Paint([bool]$enabled, [string]$style, [string]$text) {
    if (-not $enabled) {
        return $text
    }
    return "$style$text$($PSStyle.Reset)"
}

try {
$ErrorActionPreference = "Stop"

if ($Help) {
    Write-Output @"
Usage: install.ps1 [OPTIONS]

Install the tightship plugin for one harness or all of them.

Options:
  -Harness <name>       Target harness: {{harness-choices}}
  -Contract <bool>      Enable the working contract: true or false (default: true)
  -Memory <bool>        Enable memory layer: true or false (default: true)
  -StatusLine <bool>    Enable status line: true or false (default: true)
  -AutoUpdate <bool>    Install each new release by itself: true or false (default: true)
  -VaultFolder <path>   Path to memory vault (e.g. ~/ai-memory)
  -InitVault <bool>     Initialize missing vault folder with tightship vault init (default: false)
  -FromDir <path>       Install from a local directory instead of downloading this installer's release
  -Yes, -y              Accept defaults without interactive prompts
  -Help, -h             Show this help message
"@
    return
}

[Console]::OutputEncoding = [System.Text.UTF8Encoding]::new($false)

function Compare-Semver([string]$v1, [string]$v2) {
    if ($v1 -eq $v2) { return 0 }
    $p1 = $v1.Split('.')
    $p2 = $v2.Split('.')

    $maj1 = if ($p1.Length -gt 0 -and $p1[0] -match '^\d+$') { [int]$p1[0] } else { 0 }
    $min1 = if ($p1.Length -gt 1 -and $p1[1] -match '^\d+$') { [int]$p1[1] } else { 0 }
    $pat1 = if ($p1.Length -gt 2 -and $p1[2] -match '^\d+$') { [int]$p1[2] } else { 0 }

    $maj2 = if ($p2.Length -gt 0 -and $p2[0] -match '^\d+$') { [int]$p2[0] } else { 0 }
    $min2 = if ($p2.Length -gt 1 -and $p2[1] -match '^\d+$') { [int]$p2[1] } else { 0 }
    $pat2 = if ($p2.Length -gt 2 -and $p2[2] -match '^\d+$') { [int]$p2[2] } else { 0 }

    if ($maj1 -gt $maj2) { return 1 }
    if ($maj1 -lt $maj2) { return -1 }
    if ($min1 -gt $min2) { return 1 }
    if ($min1 -lt $min2) { return -1 }
    if ($pat1 -gt $pat2) { return 1 }
    if ($pat1 -lt $pat2) { return -1 }
    return 0
}

function Extract-Version([string]$text) {
    if ($text -match '\b(\d+\.\d+(\.\d+)?)\b') {
        return $Matches[1]
    }
    return $text.Trim()
}

if (-not $Yes) {
    if ([Console]::IsInputRedirected) {
        throw "error: standard input is not a terminal; pass -Yes or provide required flags"
    }
    $interactive = $true
}

function ConvertTo-Bool([string]$source, [string]$value) {
    if ($value -in @("true", "yes", "y", "on", "1")) { return "true" }
    if ($value -in @("false", "no", "n", "off", "0")) { return "false" }
    throw "error: $source expects true or false; got `"$value`""
}

function Invoke-Step([string]$description, [scriptblock]$command) {
    $live = -not [Console]::IsOutputRedirected
    if ($live) {
        Write-Host -NoNewline "$(Paint $colorOut $PSStyle.Foreground.Cyan "◒")  $description"
    }
    $global:LASTEXITCODE = 0
    $output = ""
    $failure = $null
    try {
        $output = & $command *>&1 | Out-String
    } catch {
        $failure = $_
    }
    $failed = $failure -or $LASTEXITCODE -ne 0
    $mark = if ($failed) { Paint $colorOut $PSStyle.Foreground.Red "✖" } else { Paint $colorOut $PSStyle.Foreground.Green "✔" }
    if ($live) {
        Write-Host -NoNewline "`r`e[K"
    }
    Write-Host "$mark  $description"
    if (-not $failed) {
        return
    }
    foreach ($line in ($output -split "\r?\n" | Where-Object { $_ })) {
        [Console]::Error.WriteLine("   $line")
    }
    if ($failure) {
        throw $failure
    }
    throw "error: $description failed"
}

function Read-Key {
    $treatControlCAsInput = [Console]::TreatControlCAsInput
    [Console]::TreatControlCAsInput = $true
    try {
        $key = [Console]::ReadKey($true)
    } finally {
        [Console]::TreatControlCAsInput = $treatControlCAsInput
    }
    if ($key.Key -eq "C" -and ($key.Modifiers -band [ConsoleModifiers]::Control)) {
        throw [System.OperationCanceledException]::new()
    }
    switch ($key.Key) {
        "UpArrow" { return "up" }
        "DownArrow" { return "down" }
        "LeftArrow" { return "left" }
        "RightArrow" { return "right" }
        "Spacebar" { return "space" }
        "Enter" { return "enter" }
    }
    return "$($key.KeyChar)".ToLowerInvariant()
}

function Write-Prompt([string]$text) {
    [Console]::Error.Write($text)
}

function Clear-Lines([int]$count) {
    if ($count -gt 0) {
        Write-Prompt "`r`e[$($count)A`e[J"
    }
}

function Write-Answered([string]$title, [string]$value) {
    Write-Prompt "$(Paint $colorErr $PSStyle.Foreground.Green "◇")  $title`n│  $(Paint $colorErr $PSStyle.Dim $value)`n│`n"
}

function Write-Section([string]$title) {
    Write-Prompt "│`n│  $(Paint $colorErr "$($PSStyle.Bold)$($PSStyle.Dim)" $title)`n"
}

function Read-Harnesses([string[]]$names, [string[]]$detected) {
    $title = "Where should tightship be installed?"
    $checked = [System.Collections.Generic.List[string]]::new([string[]]$detected)
    $cursor = 0
    $notice = ""
    $drawn = 0
    Write-Prompt "`e[?25l"
    try {
        while ($true) {
            Clear-Lines $drawn
            $screen = "$(Paint $colorErr $PSStyle.Foreground.Cyan "◆")  $title`n"
            for ($index = 0; $index -lt $names.Count; $index++) {
                $name = $names[$index]
                $label = $Harnesses[$name].Label
                $active = $index -eq $cursor
                if ($name -notin $detected) {
                    $row = Paint $colorErr $PSStyle.Dim "◻ $label  not found"
                } else {
                    $box = if ($checked.Contains($name)) { Paint $colorErr $PSStyle.Foreground.Green "◼" } else { Paint $colorErr $PSStyle.Dim "◻" }
                    $text = if ($active) { Paint $colorErr "$($PSStyle.Bold)$($PSStyle.Foreground.Cyan)" $label } else { $label }
                    $row = "$box $text"
                }
                $pointer = if ($active) { Paint $colorErr $PSStyle.Foreground.Cyan "›" } else { " " }
                $screen += "│ $pointer $row`n"
            }
            $screen += "│  $(Paint $colorErr $PSStyle.Dim "↑/↓ move · space select · a all · enter confirm")`n"
            $screen += "└  $(Paint $colorErr $PSStyle.Foreground.Yellow $notice)`n"
            Write-Prompt $screen
            $drawn = $names.Count + 3
            $notice = ""
            $current = $names[$cursor]
            switch (Read-Key) {
                { $_ -in "up", "k" } { $cursor = ($cursor - 1 + $names.Count) % $names.Count }
                { $_ -in "down", "j" } { $cursor = ($cursor + 1) % $names.Count }
                "space" {
                    if ($current -notin $detected) {
                        $notice = "$($Harnesses[$current].Label) is not installed."
                    } elseif ($checked.Contains($current)) {
                        [void]$checked.Remove($current)
                    } else {
                        $checked.Add($current)
                    }
                }
                "a" {
                    $missing = @($detected | Where-Object { -not $checked.Contains($_) })
                    $checked.Clear()
                    if ($missing.Count -gt 0) {
                        $checked.AddRange([string[]]$detected)
                    }
                }
                "enter" {
                    if ($checked.Count -gt 0) {
                        Clear-Lines $drawn
                        $chosen = @($names | Where-Object { $checked.Contains($_) })
                        Write-Answered $title (Join-Labels $chosen)
                        return $chosen
                    }
                    $notice = "Select at least one harness."
                }
            }
        }
    } finally {
        Write-Prompt "`e[?25h"
    }
}

function Read-Confirm([string]$title, [string]$hint, [string]$default, [string]$yesLabel = "Yes", [string]$noLabel = "No") {
    $answer = $default -ne "false"
    $drawn = 0
    Write-Prompt "`e[?25l"
    try {
        while ($true) {
            Clear-Lines $drawn
            $choice = if ($answer) {
                "$(Paint $colorErr $PSStyle.Foreground.Green "●") $yesLabel $(Paint $colorErr $PSStyle.Dim "/ ○ $noLabel")"
            } else {
                "$(Paint $colorErr $PSStyle.Dim "○ $yesLabel /") $(Paint $colorErr $PSStyle.Foreground.Green "●") $noLabel"
            }
            $screen = "$(Paint $colorErr $PSStyle.Foreground.Cyan "◆")  $title`n"
            $drawn = 3
            if ($hint) {
                $screen += "│  $(Paint $colorErr $PSStyle.Dim $hint)`n"
                $drawn = 4
            }
            Write-Prompt "$screen│  $choice`n└`n"
            switch (Read-Key) {
                { $_ -in "up", "down", "left", "right", "space", "h", "j", "k", "l" } { $answer = -not $answer }
                "y" { $answer = $true }
                "n" { $answer = $false }
                "enter" {
                    Clear-Lines $drawn
                    Write-Answered $title $(if ($answer) { $yesLabel } else { $noLabel })
                    return $(if ($answer) { "true" } else { "false" })
                }
            }
        }
    } finally {
        Write-Prompt "`e[?25h"
    }
}

function Read-Text([string]$title, [string]$hint, [string]$default) {
    Write-Prompt "$(Paint $colorErr $PSStyle.Foreground.Cyan "◆")  $title`n│  $(Paint $colorErr $PSStyle.Dim $hint)`n│  "
    $reply = [Console]::ReadLine()
    $value = if ([string]::IsNullOrWhiteSpace($reply)) { $default } else { $reply.Trim() }
    Clear-Lines 3
    Write-Answered $title $(if ($value) { $value } else { "not set" })
    return $value
}

$userProfile = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }

{{harnesses}}

function Assert-Version([string]$name) {
    $command = $Harnesses[$name].Command
    $minimum = $Harnesses[$name].MinVersion
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        throw "error: $command $minimum or later is required; found not installed"
    }
    $raw = & $command --version 2>&1 | Out-String
    $ver = Extract-Version $raw
    if ((Compare-Semver $ver $minimum) -lt 0) {
        throw "error: $command $minimum or later is required; found $ver"
    }
}

function Invoke-Hook([string]$pattern, [string]$name) {
    $hook = $pattern -f $Harnesses[$name].Id
    if (Get-Command $hook -CommandType Function -ErrorAction SilentlyContinue) {
        & $hook
    }
}

function Find-Harness([string]$value) {
    foreach ($name in $Harnesses.Keys) {
        if ($value -eq $name -or $Harnesses[$name].Aliases -contains $value) {
            return $name
        }
    }
    return $null
}

function Join-Labels([string[]]$names) {
    $labels = @($names | ForEach-Object { $Harnesses[$_].Label })
    if ($labels.Count -le 1) {
        return $labels -join ""
    }
    return ($labels[0..($labels.Count - 2)] -join ", ") + " and " + $labels[-1]
}

$installedIn = [ordered]@{}

{{fragments}}
Write-Host "┌  $(Paint $colorOut "$($PSStyle.Bold)$($PSStyle.Foreground.Cyan)" "tightship $(if ($ReleaseTag) { $ReleaseTag } else { "dev" })")"

$names = @($Harnesses.Keys)
$detected = @($names | Where-Object { Get-Command $Harnesses[$_].Command -ErrorAction SilentlyContinue })
$skipped = @()
$value = "$Harness".ToLowerInvariant()
if (-not $value) {
    if ($detected.Count -eq 0) {
        throw "error: none of $(Join-Labels $names) is installed"
    }
    if ($interactive) {
        Write-Section "Harnesses"
        $selected = Read-Harnesses $names $detected
    } else {
        $selected = $detected
    }
} elseif ($value -in "all", "both") {
    if ($detected.Count -eq 0) {
        throw "error: none of $(Join-Labels $names) is installed"
    }
    $selected = $detected
    $skipped = @($names | Where-Object { $_ -notin $detected })
} else {
    $found = Find-Harness $value
    if (-not $found) {
        throw "error: unknown harness $Harness; expected {{harness-choices}}"
    }
    $selected = @($found)
}
$selected = @($selected)

foreach ($name in $selected) {
    Assert-Version $name
    Invoke-Hook "Test-{0}" $name
}

$existingContract = $null
$existingMemory = $null
$existingStatusLine = $null
$existingAutoUpdate = $null
$existingVaultFolder = $null

foreach ($name in $selected) {
    $existing = Invoke-Hook "Read-{0}Options" $name
    if (-not $existing) {
        continue
    }
    if ($existing.contract) { $existingContract = $existing.contract }
    if ($existing.memory) { $existingMemory = $existing.memory }
    if ($existing.status_line) { $existingStatusLine = $existing.status_line }
    if ($existing.auto_update) { $existingAutoUpdate = $existing.auto_update }
    if ($existing.vault_folder) { $existingVaultFolder = $existing.vault_folder }
}

function Get-DefaultHint([string]$existing) {
    $label = if ($existing -eq "false") { "No" } else { "Yes" }
    if ($existing) {
        return "Default: $label, from your last install."
    }
    return "Default: $label."
}

if ($interactive -and (-not $Contract -or -not $Memory -or -not $StatusLine -or -not $AutoUpdate)) {
    Write-Section "Options"
}

if ($Contract) {
    $finalContract = ConvertTo-Bool "-Contract" $Contract
} elseif (-not $interactive) {
    $finalContract = if ($existingContract) { $existingContract } else { "true" }
} else {
    $finalContract = Read-Confirm "Load the working contract in every session?" "The rules every session follows. $(Get-DefaultHint $existingContract)" $existingContract
}

if ($Memory) {
    $finalMemory = ConvertTo-Bool "-Memory" $Memory
} elseif (-not $interactive) {
    $finalMemory = if ($existingMemory) { $existingMemory } else { "true" }
} else {
    $finalMemory = Read-Confirm "Turn on the memory layer?" "Curated vault notes and handoffs. $(Get-DefaultHint $existingMemory)" $existingMemory
}

if ($StatusLine) {
    $finalStatusLine = ConvertTo-Bool "-StatusLine" $StatusLine
} elseif (-not $interactive) {
    $finalStatusLine = if ($existingStatusLine) { $existingStatusLine } else { "true" }
} else {
    $finalStatusLine = Read-Confirm "Show tightship's status line?" "Replaces your statusLine and restores it when off. $(Get-DefaultHint $existingStatusLine)" $existingStatusLine
}

if ($AutoUpdate) {
    $finalAutoUpdate = ConvertTo-Bool "-AutoUpdate" $AutoUpdate
} elseif (-not $interactive) {
    $finalAutoUpdate = if ($existingAutoUpdate) { $existingAutoUpdate } else { "true" }
} else {
    $finalAutoUpdate = Read-Confirm "Keep tightship updated automatically?" "Installs each new release by itself and says so at the next session. $(Get-DefaultHint $existingAutoUpdate)" $existingAutoUpdate
}

if ($VaultFolder) {
    $finalVaultFolder = $VaultFolder
} elseif (-not $interactive -or $finalMemory -ne "true") {
    $finalVaultFolder = if ($existingVaultFolder) { $existingVaultFolder } else { "" }
} else {
    Write-Section "Vault"
    if ($existingVaultFolder) {
        $finalVaultFolder = Read-Text "Where is your vault?" "Enter keeps $existingVaultFolder." $existingVaultFolder
    } else {
        $finalVaultFolder = Read-Text "Where is your vault?" "A folder such as ~/ai-memory. Enter leaves it for later." ""
    }
}

$finalInitVault = $false
$expandedVault = $null
if ($finalVaultFolder) {
    if ($finalVaultFolder.StartsWith("~")) {
        $expandedVault = Join-Path $userProfile $finalVaultFolder.Substring(1).TrimStart('\', '/')
    } else {
        $expandedVault = $finalVaultFolder
    }
}

if ($expandedVault -and -not (Test-Path $expandedVault)) {
    if ($InitVault) {
        $finalInitVault = (ConvertTo-Bool "-InitVault" $InitVault) -eq "true"
    } elseif ($interactive) {
        $finalInitVault = (Read-Confirm "$finalVaultFolder does not exist. Create it now?" "Runs tightship vault init. Default: Yes." "true") -eq "true"
    }
}

function Write-Row([string]$label, [string]$text) {
    Write-Host "│  $(Paint $colorOut $PSStyle.Dim $label.PadRight(15)) $text"
}

function Format-OnOff([string]$state) {
    if ($state -eq "true") { return "on" }
    return "off"
}

$vaultSummary = if ($finalVaultFolder) { $finalVaultFolder } else { "not set" }
if ($finalInitVault) {
    $vaultSummary += ", created now"
}

Write-Host "│"
Write-Host "$(Paint $colorOut $PSStyle.Foreground.Green "◇")  Summary"
Write-Row "Harnesses" (Join-Labels $selected)
if ($skipped.Count -gt 0) {
    Write-Row "Skipped" "$(Join-Labels $skipped), not installed"
}
Write-Row "Contract" (Format-OnOff $finalContract)
Write-Row "Memory" (Format-OnOff $finalMemory)
Write-Row "Vault" $vaultSummary
Write-Row "Status line" (Format-OnOff $finalStatusLine)
Write-Row "Auto-update" (Format-OnOff $finalAutoUpdate)
Write-Host "│"

if ($interactive -and (Read-Confirm "Install with these choices?" "" "true" "Install" "Cancel") -ne "true") {
    Write-Host "└  Cancelled; nothing was written."
    $global:LASTEXITCODE = 0
    return
}

New-Item -ItemType Directory -Force -Path $workDir | Out-Null
foreach ($name in $selected) {
    & "Install-$($Harnesses[$name].Id)"
}

Write-Host "│"
Write-Host "$(Paint $colorOut $PSStyle.Foreground.Green "◇")  Installed in"
foreach ($label in $installedIn.Keys) {
    Write-Row $label $installedIn[$label]
}
Write-Host "│"
Write-Host "└  $(Paint $colorOut $PSStyle.Foreground.Green "Installed tightship for $(Join-Labels $selected).")"
$global:LASTEXITCODE = 0
} catch [System.OperationCanceledException] {
    [Console]::Error.WriteLine("`n└  Cancelled.")
    $global:LASTEXITCODE = 130
    if ($PSCommandPath) {
        exit 130
    }
} catch {
    $lines = @($_.Exception.Message -split "\r?\n")
    [Console]::Error.WriteLine((Paint $colorErr $PSStyle.Foreground.Red "✖ $($lines[0])"))
    foreach ($line in $lines | Select-Object -Skip 1) {
        [Console]::Error.WriteLine((Paint $colorErr $PSStyle.Dim "  $line"))
    }
    $global:LASTEXITCODE = 1
    if ($PSCommandPath) {
        exit 1
    }
} finally {
    [Console]::OutputEncoding = $callerEncoding
    foreach ($variable in $callerEnvironment.Keys) {
        [Environment]::SetEnvironmentVariable($variable, $callerEnvironment[$variable])
    }
    if (Test-Path $workDir) {
        Remove-Item -Recurse -Force $workDir
    }
}
}
