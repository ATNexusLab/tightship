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
$Repository = "{{repository}}"

if ($Help) {
    Write-Output @"
Usage: install.ps1 [OPTIONS]

Install the tightship plugin for one harness or all of them.

Options:
  -Harness <name>       Target harness: {{harness-choices}}
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

$userProfile = if ($env:USERPROFILE) { $env:USERPROFILE } else { $env:HOME }

{{harnesses}}

function Assert-Version([string]$name) {
    $command = $Harnesses[$name].Command
    $minimum = $Harnesses[$name].MinVersion
    if (-not (Get-Command $command -ErrorAction SilentlyContinue)) {
        Write-Stderr "error: $command $minimum or later is required; found not installed"
        exit 1
    }
    $raw = & $command --version 2>&1 | Out-String
    $ver = Extract-Version $raw
    if ((Compare-Semver $ver $minimum) -lt 0) {
        Write-Stderr "error: $command $minimum or later is required; found $ver"
        exit 1
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

{{fragments}}
$names = @($Harnesses.Keys)
if ($Harness) {
    $value = $Harness.ToLowerInvariant()
    if ($value -in "all", "both") {
        $selected = $names
    } else {
        $found = Find-Harness $value
        if (-not $found) {
            Write-Stderr "error: unknown harness $Harness; expected {{harness-choices}}"
            exit 1
        }
        $selected = @($found)
    }
} else {
    $detected = @($names | Where-Object { Get-Command $Harnesses[$_].Command -ErrorAction SilentlyContinue })
    $defaultSelection = if ($detected.Count -gt 0) { $detected } else { $names }

    if ($Yes) {
        $selected = $defaultSelection
    } else {
        $menu = (1..$names.Count | ForEach-Object { "[$_] $($Harnesses[$names[$_ - 1]].Label)" }) -join " "
        $allNumber = "$($names.Count + 1)"
        $choice = (Prompt-User "Install tightship for: $menu [$allNumber] All (default: $(Join-Labels $defaultSelection)): " "").ToLowerInvariant()
        $number = 0
        $named = Find-Harness $choice
        if (-not $choice) {
            $selected = $defaultSelection
        } elseif ($choice -in "all", "both", $allNumber) {
            $selected = $names
        } elseif ([int]::TryParse($choice, [ref]$number) -and $number -ge 1 -and $number -le $names.Count) {
            $selected = @($names[$number - 1])
        } elseif ($named) {
            $selected = @($named)
        } else {
            $selected = $defaultSelection
        }
    }
}
$selected = @($selected)

foreach ($name in $selected) {
    Assert-Version $name
    Invoke-Hook "Test-{0}" $name
}

$existingContract = $null
$existingMemory = $null
$existingStatusLine = $null
$existingVaultFolder = $null

foreach ($name in $selected) {
    $optionsFile = Invoke-Hook "Resolve-{0}Options" $name
    if (-not $optionsFile -or -not (Test-Path $optionsFile)) {
        continue
    }
    $raw = Get-Content -Raw $optionsFile
    if ($raw -match '\{') {
        try {
            $parsed = $raw | ConvertFrom-Json
            if ($null -ne $parsed.contract) { $existingContract = [string]$parsed.contract }
            if ($null -ne $parsed.memory) { $existingMemory = [string]$parsed.memory }
            if ($null -ne $parsed.status_line) { $existingStatusLine = [string]$parsed.status_line }
            if ($null -ne $parsed.vault_folder) { $existingVaultFolder = [string]$parsed.vault_folder }
        } catch {
            continue
        }
    } else {
        Get-Content $optionsFile | ForEach-Object {
            if ($_ -match '^\s*contract\s*=\s*(.+)$') { $existingContract = $Matches[1].Trim(" '`"") }
            if ($_ -match '^\s*memory\s*=\s*(.+)$') { $existingMemory = $Matches[1].Trim(" '`"") }
            if ($_ -match '^\s*status_line\s*=\s*(.+)$') { $existingStatusLine = $Matches[1].Trim(" '`"") }
            if ($_ -match '^\s*vault_folder\s*=\s*"((?:[^"\\]|\\.)*)"') { $existingVaultFolder = $Matches[1] -replace '\\(.)', '$1' }
        }
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

foreach ($name in $selected) {
    & "Install-$($Harnesses[$name].Id)"
}

Write-Output "Installed tightship for $(Join-Labels $selected)."
