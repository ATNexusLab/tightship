param(
    [Alias("h")]
    [string]$Harness,

    [string]$Contract,

    [string]$Memory,

    [string]$StatusLine,

    [string]$VaultFolder,

    [string]$InitVault,

    [string]$FromDir,

    [Alias("y")]
    [switch]$Yes,

    [switch]$Help
)

$ErrorActionPreference = "Stop"

$ReleaseTag = ""
$AntigravityCliSha256 = ""

if ($Help) {
    Write-Output @"
Usage: install.ps1 [OPTIONS]

Install the tightship plugin for Claude Code, Antigravity CLI, or both.

Options:
  -Harness <name>       Target harness: claude-code, antigravity-cli, or all
  -Contract <bool>      Enable the working contract: true or false (default: true)
  -Memory <bool>        Enable memory layer: true or false (default: true)
  -StatusLine <bool>    Enable status line: true or false (default: true)
  -VaultFolder <path>   Path to memory vault (e.g. ~/ai-memory)
  -InitVault <bool>     Initialize missing vault folder with tightship vault init (default: false)
  -FromDir <path>       Install from a local directory instead of downloading this installer's release
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

$canPrompt = -not [Console]::IsInputRedirected
if (-not $Yes -and -not $canPrompt) {
    Write-Stderr "error: standard input is not a terminal; pass --yes or provide required flags"
    exit 1
}

function ConvertTo-Bool([string]$source, [string]$value) {
    if ($value -in @("true", "yes", "y", "on", "1")) { return "true" }
    if ($value -in @("false", "no", "n", "off", "0")) { return "false" }
    Write-Stderr "error: $source expects true or false; got `"$value`""
    exit 1
}

function Invoke-Step([string]$description, [scriptblock]$command) {
    Write-Host "> $description"
    $global:LASTEXITCODE = 0
    & $command
    if ($LASTEXITCODE -ne 0) {
        Write-Stderr "error: $description failed"
        exit 1
    }
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

if ($Harness -ne "claude-code" -and -not $FromDir -and -not $ReleaseTag) {
    Write-Stderr "error: this install.ps1 belongs to no release; run https://github.com/ATNexusLab/tightship/releases/latest/download/install.ps1, or pass -FromDir <folder>"
    exit 1
}

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
        if ($_ -match '^\s*vault_folder\s*=\s*"((?:[^"\\]|\\.)*)"') { $existingVaultFolder = $Matches[1] -replace '\\(.)', '$1' }
    }
}

if ($Contract) {
    $finalContract = ConvertTo-Bool "-Contract" $Contract
} elseif ($Yes) {
    $finalContract = if ($existingContract) { $existingContract } else { "true" }
} else {
    $def = if ($existingContract -eq "false") { "n" } else { "Y" }
    $ans = Prompt-User "Enable the working contract in every session? [Y/n]: " $def
    $finalContract = ConvertTo-Bool "the answer" $ans
}

if ($Memory) {
    $finalMemory = ConvertTo-Bool "-Memory" $Memory
} elseif ($Yes) {
    $finalMemory = if ($existingMemory) { $existingMemory } else { "true" }
} else {
    $def = if ($existingMemory -eq "false") { "n" } else { "Y" }
    $ans = Prompt-User "Enable the memory layer (curated vault notes and handoffs)? [Y/n]: " $def
    $finalMemory = ConvertTo-Bool "the answer" $ans
}

if ($VaultFolder) {
    $finalVaultFolder = $VaultFolder
} elseif ($Yes) {
    $finalVaultFolder = if ($existingVaultFolder) { $existingVaultFolder } else { "" }
} else {
    if ($finalMemory -eq "true") {
        if ($existingVaultFolder) {
            $finalVaultFolder = Prompt-User "Vault folder path [$existingVaultFolder]: " $existingVaultFolder
        } else {
            $finalVaultFolder = Prompt-User "Vault folder path (for example ~/ai-memory, or leave empty to set later): " ""
        }
    } else {
        $finalVaultFolder = if ($existingVaultFolder) { $existingVaultFolder } else { "" }
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
    } elseif ($Yes) {
        $finalInitVault = $false
    } else {
        $ans = Prompt-User "Vault folder $finalVaultFolder does not exist. Create it now with tightship vault init? [Y/n]: " "Y"
        $finalInitVault = (ConvertTo-Bool "the answer" $ans) -eq "true"
    }
}

if ($StatusLine) {
    $finalStatusLine = ConvertTo-Bool "-StatusLine" $StatusLine
} elseif ($Yes) {
    $finalStatusLine = if ($existingStatusLine) { $existingStatusLine } else { "true" }
} else {
    $def = if ($existingStatusLine -eq "false") { "n" } else { "Y" }
    $ans = Prompt-User "Enable tightship's status line (replaces your current statusLine and restores it when off)? [Y/n]: " $def
    $finalStatusLine = ConvertTo-Bool "the answer" $ans
}

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
    Invoke-Step "Installing tightship for Claude Code" { & claude @claudeArgs }
}

if ($Harness -eq "antigravity-cli" -or $Harness -eq "all") {
    Invoke-Step "Installing the ponytail companion" { & agy plugin install https://github.com/DietrichGebert/ponytail }
    Invoke-Step "Installing the i-have-adhd companion" { & agy plugin install https://github.com/ayghri/i-have-adhd }

    $humanizerDir = Join-Path $userProfile ".gemini\config\skills\humanizer"
    if (-not (Test-Path $humanizerDir)) {
        New-Item -ItemType Directory -Force -Path $humanizerDir | Out-Null
    }
    $humanizerSkill = Join-Path $humanizerDir "SKILL.md"
    Invoke-Step "Downloading the humanizer skill" { Invoke-RestMethod -Uri "https://raw.githubusercontent.com/blader/humanizer/main/SKILL.md" -OutFile $humanizerSkill }

    $sourceDir = $null
    $tempZipFolder = $null
    if ($FromDir) {
        $sourceDir = $FromDir
    } else {
        $tempZipFolder = Join-Path ([System.IO.Path]::GetTempPath()) ("tightship-" + [System.Guid]::NewGuid().ToString())
        New-Item -ItemType Directory -Force -Path $tempZipFolder | Out-Null
        $zipDest = Join-Path $tempZipFolder "tightship-antigravity-cli.zip"
        $downloadUrl = "https://github.com/ATNexusLab/tightship/releases/download/$ReleaseTag/tightship-antigravity-cli.zip"
        Write-Host "> Downloading tightship-antigravity-cli.zip $ReleaseTag"
        try {
            Invoke-RestMethod -Uri $downloadUrl -OutFile $zipDest
        } catch {
            Write-Stderr "error: failed to download tightship-antigravity-cli.zip from $downloadUrl"
            Write-Stderr "hint: if you are running in a local repository clone, run cargo xtask build and pass -FromDir dist\antigravity-cli"
            exit 1
        }
        Write-Host "> Checking the sha256 of tightship-antigravity-cli.zip"
        $zipSha256 = (Get-FileHash -Algorithm SHA256 -Path $zipDest).Hash.ToLowerInvariant()
        if ($zipSha256 -ne $AntigravityCliSha256) {
            Write-Stderr "error: tightship-antigravity-cli.zip has sha256 $zipSha256; $ReleaseTag published $AntigravityCliSha256"
            exit 1
        }
        $extracted = Join-Path $tempZipFolder "extracted"
        Expand-Archive -Path $zipDest -DestinationPath $extracted -Force
        $sourceDir = $extracted
    }

    Invoke-Step "Installing tightship for Antigravity CLI" { & agy plugin install $sourceDir }
    $pluginTarget = Join-Path $userProfile ".gemini\config\plugins\tightship"

    if (-not (Test-Path $tightshipData)) {
        New-Item -ItemType Directory -Force -Path $tightshipData | Out-Null
    }
    $tomlContent = @"
contract = $finalContract
memory = $finalMemory
status_line = $finalStatusLine
"@
    if ($finalVaultFolder) {
        $escapedVaultFolder = $finalVaultFolder.Replace('\', '\\').Replace('"', '\"')
        $tomlContent += "`nvault_folder = `"$escapedVaultFolder`""
    }
    Set-Content -Path $optionsFile -Value ($tomlContent + "`n") -Encoding utf8

    $env:TIGHTSHIP_DATA = $tightshipData
    function Invoke-Tightship {
        if ($env:OS -eq "Windows_NT") {
            & (Join-Path $pluginTarget "bin\tightship.cmd") @args
        } else {
            & sh (Join-Path $pluginTarget "bin/tightship") @args
        }
    }
    Invoke-Step "Applying the status line option" { Invoke-Tightship hook status-line }

    if ($finalInitVault -and $finalVaultFolder) {
        Invoke-Step "Creating the vault at $finalVaultFolder" { Invoke-Tightship vault init $finalVaultFolder }
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
