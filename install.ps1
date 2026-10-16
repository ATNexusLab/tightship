<#
.SYNOPSIS
    Install the tightship plugin for Claude Code, Antigravity CLI, or both.
#>

param(
    [Alias("h")]
    [string]$Harness,

    [string]$Contract,

    [string]$Memory,

    [string]$StatusLine,

    [string]$VaultFolder,

    [string]$InitVault,

    [string]$Version = "latest",

    [string]$FromDir,

    [Alias("y")]
    [switch]$Yes,

    [switch]$Help
)

$ErrorActionPreference = "Stop"

if ($Help) {
    Write-Output @"
Usage: install.ps1 [OPTIONS]

Install the tightship plugin for Claude Code, Antigravity CLI, or both.

Options:
  -Harness <name>       Target harness: claude-code, antigravity-cli, or all
  -Contract <mode>      Contract mode: strict, lenient, off, true, or false (default: strict)
  -Memory <bool>        Enable memory layer: true or false (default: true)
  -StatusLine <bool>    Enable status line: true or false (default: true)
  -VaultFolder <path>   Path to memory vault (e.g. ~/ai-memory)
  -InitVault <bool>     Initialize missing vault folder with tightship vault init (default: false)
  -Version <tag>        Release version to install (default: latest)
  -FromDir <path>       Install from local directory instead of downloading release
  -Yes, -y              Accept defaults without interactive prompts
  -Help                 Show this help message
"@
    exit 0
}

function Write-Stderr([string]$Message) {
    [Console]::Error.WriteLine($Message)
}

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

$CLAUDE_MIN = "2.1.295"
$AGY_MIN = "1.4.0"

# Check if input can be read interactively
$canPrompt = -not [Console]::IsInputRedirected
if (-not $Yes -and -not $canPrompt) {
    Write-Stderr "error: standard input is not a terminal; pass --yes or provide required flags"
    exit 1
}

function Prompt-User([string]$promptText, [string]$defaultVal) {
    if (-not $canPrompt) {
        return $defaultVal
    }
    [Console]::Error.Write($promptText)
    $reply = [Console]::ReadLine()
    if ([string]::IsNullOrWhiteSpace($reply)) {
        return $defaultVal
    }
    return $reply.Trim()
}

# Normalize or detect Harness
$hasClaude = [bool](Get-Command claude -ErrorAction SilentlyContinue)
$hasAgy = [bool](Get-Command agy -ErrorAction SilentlyContinue)

if ($Harness) {
    switch ($Harness.ToLowerInvariant()) {
        { $_ -in "claude", "claude-code" } { $Harness = "claude-code" }
        { $_ -in "antigravity-cli", "agy" } { $Harness = "antigravity-cli" }
        { $_ -in "all", "both" } { $Harness = "all" }
        default {
            Write-Stderr "error: unknown harness $Harness; expected claude-code, antigravity-cli, or all"
            exit 1
        }
    }
} else {
    $defaultHarness = "all"
    $defaultLabel = "Both"
    if ($hasClaude -and -not $hasAgy) {
        $defaultHarness = "claude-code"
        $defaultLabel = "Claude Code"
    } elseif (-not $hasClaude -and $hasAgy) {
        $defaultHarness = "antigravity-cli"
        $defaultLabel = "Antigravity CLI"
    }

    if ($Yes) {
        $Harness = $defaultHarness
    } else {
        $choice = Prompt-User "Install tightship for: [1] Claude Code [2] Antigravity CLI [3] Both (default: $defaultLabel): " $defaultHarness
        switch ($choice.ToLowerInvariant()) {
            { $_ -in "1", "claude", "claude-code" } { $Harness = "claude-code" }
            { $_ -in "2", "antigravity-cli", "agy" } { $Harness = "antigravity-cli" }
            { $_ -in "3", "all", "both" } { $Harness = "all" }
            default { $Harness = $defaultHarness }
        }
    }
}

# Verify CLI requirements
function Test-Claude() {
    if (-not (Get-Command claude -ErrorAction SilentlyContinue)) {
        Write-Stderr "error: claude $CLAUDE_MIN or later is required; found not installed"
        exit 1
    }
    $raw = & claude --version 2>&1 | Out-String
    $ver = Extract-Version $raw
    if ((Compare-Semver $ver $CLAUDE_MIN) -lt 0) {
        Write-Stderr "error: claude $CLAUDE_MIN or later is required; found $ver"
        exit 1
    }
}

function Test-Agy() {
    if (-not (Get-Command agy -ErrorAction SilentlyContinue)) {
        Write-Stderr "error: agy $AGY_MIN or later is required; found not installed"
        exit 1
    }
    $raw = & agy --version 2>&1 | Out-String
    $ver = Extract-Version $raw
    if ((Compare-Semver $ver $AGY_MIN) -lt 0) {
        Write-Stderr "error: agy $AGY_MIN or later is required; found $ver"
        exit 1
    }
}

if ($Harness -eq "claude-code" -or $Harness -eq "all") {
    Test-Claude
}
if ($Harness -eq "antigravity-cli" -or $Harness -eq "all") {
    Test-Agy
}

# Resolve options and existing options.toml
$userProfile = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }
$tightshipData = if ($env:TIGHTSHIP_DATA) { $env:TIGHTSHIP_DATA } else { Join-Path $userProfile ".gemini\config\plugin_data\tightship" }
$optionsFile = Join-Path $tightshipData "options.toml"

$existingContract = $null
$existingMemory = $null
$existingStatusLine = $null
$existingVaultFolder = $null

if (Test-Path $optionsFile) {
    Get-Content $optionsFile | ForEach-Object {
        if ($_ -match '^\s*contract\s*=\s*(.+)$') { $existingContract = $Matches[1].Trim(" '`"") }
        if ($_ -match '^\s*memory\s*=\s*(.+)$') { $existingMemory = $Matches[1].Trim(" '`"") }
        if ($_ -match '^\s*status_line\s*=\s*(.+)$') { $existingStatusLine = $Matches[1].Trim(" '`"") }
        if ($_ -match '^\s*vault_folder\s*=\s*"([^"]*)"') { $existingVaultFolder = $Matches[1] }
    }
}

# Contract
if ($Contract) {
    if ($Contract -in @("off", "false", "0")) {
        $finalContract = "false"
    } else {
        $finalContract = "true"
    }
} elseif ($Yes) {
    $finalContract = if ($existingContract) { $existingContract } else { "true" }
} else {
    $def = if ($existingContract -eq "false") { "n" } else { "Y" }
    $ans = Prompt-User "Enable the working contract in every session? [Y/n]: " $def
    if ($ans -in @("n", "N", "no", "false", "0")) {
        $finalContract = "false"
    } else {
        $finalContract = "true"
    }
}

# Memory
if ($Memory) {
    if ($Memory -in @("off", "false", "0")) {
        $finalMemory = "false"
    } else {
        $finalMemory = "true"
    }
} elseif ($Yes) {
    $finalMemory = if ($existingMemory) { $existingMemory } else { "true" }
} else {
    $def = if ($existingMemory -eq "false") { "n" } else { "Y" }
    $ans = Prompt-User "Enable the memory layer (curated vault notes and handoffs)? [Y/n]: " $def
    if ($ans -in @("n", "N", "no", "false", "0")) {
        $finalMemory = "false"
    } else {
        $finalMemory = "true"
    }
}

# Vault folder
if ($VaultFolder) {
    $finalVaultFolder = $VaultFolder
} elseif ($Yes) {
    $finalVaultFolder = if ($existingVaultFolder) { $existingVaultFolder } else { "" }
} else {
    if ($finalMemory -eq "true") {
        $finalVaultFolder = Prompt-User "Vault folder path (for example ~/ai-memory, or leave empty to set later): " ""
    } else {
        $finalVaultFolder = if ($existingVaultFolder) { $existingVaultFolder } else { "" }
    }
}

# Init vault
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
        $finalInitVault = ($InitVault -in @("true", "1", "yes", "y"))
    } elseif ($Yes) {
        $finalInitVault = $false
    } else {
        $ans = Prompt-User "Vault folder $finalVaultFolder does not exist. Create it now with tightship vault init? [Y/n]: " "Y"
        $finalInitVault = ($ans -notin @("n", "N", "no", "false", "0"))
    }
}

# Status line
if ($StatusLine) {
    if ($StatusLine -in @("off", "false", "0")) {
        $finalStatusLine = "false"
    } else {
        $finalStatusLine = "true"
    }
} elseif ($Yes) {
    $finalStatusLine = if ($existingStatusLine) { $existingStatusLine } else { "true" }
} else {
    $def = if ($existingStatusLine -eq "false") { "n" } else { "Y" }
    $ans = Prompt-User "Enable tightship's status line (replaces your current statusLine and restores it when off)? [Y/n]: " $def
    if ($ans -in @("n", "N", "no", "false", "0")) {
        $finalStatusLine = "false"
    } else {
        $finalStatusLine = "true"
    }
}

# Install: Claude Code
if ($Harness -eq "claude-code" -or $Harness -eq "all") {
    try { & claude plugin marketplace add DietrichGebert/ponytail } catch {}
    try { & claude plugin marketplace add ayghri/i-have-adhd } catch {}
    try { & claude plugin marketplace add blader/humanizer } catch {}

    if ($FromDir) {
        try { & claude plugin marketplace add $FromDir } catch {}
    } else {
        try { & claude plugin marketplace add ATNexusLab/tightship } catch {}
    }

    $claudeArgs = @(
        "plugin", "install", "tightship@tightship",
        "--config", "contract=$finalContract",
        "--config", "memory=$finalMemory",
        "--config", "status_line=$finalStatusLine"
    )
    if ($finalVaultFolder) {
        $claudeArgs += @("--config", "vault_folder=$finalVaultFolder")
    }
    & claude @claudeArgs
}

# Install: Antigravity CLI
if ($Harness -eq "antigravity-cli" -or $Harness -eq "all") {
    try { & agy plugin install https://github.com/DietrichGebert/ponytail } catch {}
    try { & agy plugin install https://github.com/ayghri/i-have-adhd } catch {}

    $humanizerDir = Join-Path $userProfile ".gemini\config\skills\humanizer"
    if (-not (Test-Path $humanizerDir)) {
        New-Item -ItemType Directory -Force -Path $humanizerDir | Out-Null
    }
    $humanizerSkill = Join-Path $humanizerDir "SKILL.md"
    Invoke-RestMethod -Uri "https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md" -OutFile $humanizerSkill -ErrorAction SilentlyContinue

    $sourceDir = $null
    $tempZipFolder = $null
    if ($FromDir) {
        $sourceDir = $FromDir
    } else {
        $tempZipFolder = Join-Path ([System.IO.Path]::GetTempPath()) ("tightship-" + [System.Guid]::NewGuid().ToString())
        New-Item -ItemType Directory -Force -Path $tempZipFolder | Out-Null
        $zipDest = Join-Path $tempZipFolder "tightship-antigravity-cli.zip"
        $downloadUrl = if ($Version -eq "latest") {
            "https://github.com/ATNexusLab/tightship/releases/latest/download/tightship-antigravity-cli.zip"
        } else {
            "https://github.com/ATNexusLab/tightship/releases/download/$Version/tightship-antigravity-cli.zip"
        }
        Invoke-RestMethod -Uri $downloadUrl -OutFile $zipDest
        $extracted = Join-Path $tempZipFolder "extracted"
        Expand-Archive -Path $zipDest -DestinationPath $extracted -Force
        $sourceDir = $extracted
    }

    try { & agy plugin install $sourceDir } catch {}

    $pluginTarget = Join-Path $userProfile ".gemini\config\plugins\tightship"
    if (-not (Test-Path $pluginTarget)) {
        New-Item -ItemType Directory -Force -Path $pluginTarget | Out-Null
    }
    if ($sourceDir -and (Test-Path $sourceDir)) {
        Copy-Item -Path "$sourceDir\*" -Destination $pluginTarget -Recurse -Force
    }

    # Write options.toml
    if (-not (Test-Path $tightshipData)) {
        New-Item -ItemType Directory -Force -Path $tightshipData | Out-Null
    }
    $tomlContent = @"
contract = $finalContract
memory = $finalMemory
status_line = $finalStatusLine
"@
    if ($finalVaultFolder) {
        $tomlContent += "`nvault_folder = `"$finalVaultFolder`""
    }
    Set-Content -Path $optionsFile -Value ($tomlContent + "`n") -Encoding utf8

    # Invoke status-line hook
    $env:TIGHTSHIP_DATA = $tightshipData
    $cmdLauncher = Join-Path $pluginTarget "bin\tightship.cmd"
    $shLauncher = Join-Path $pluginTarget "bin\tightship"
    if (Test-Path $cmdLauncher) {
        try { & $cmdLauncher hook status-line } catch {}
    } elseif (Test-Path $shLauncher) {
        try { & sh $shLauncher hook status-line } catch {}
    }

    # Vault init
    if ($finalInitVault -and $finalVaultFolder) {
        if (Test-Path $cmdLauncher) {
            try { & $cmdLauncher vault init $finalVaultFolder } catch {}
        } elseif (Test-Path $shLauncher) {
            try { & sh $shLauncher vault init $finalVaultFolder } catch {}
        }
    }

    if ($tempZipFolder -and (Test-Path $tempZipFolder)) {
        Remove-Item -Recurse -Force $tempZipFolder -ErrorAction SilentlyContinue
    }
}

switch ($Harness) {
    "claude-code" { Write-Output "Installed tightship for Claude Code." }
    "antigravity-cli" { Write-Output "Installed tightship for Antigravity CLI." }
    "all" { Write-Output "Installed tightship for Claude Code and Antigravity CLI." }
}
