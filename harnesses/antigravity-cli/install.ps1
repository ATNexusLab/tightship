$AntigravityCliSha256 = ""
$antigravityCliData = if ($env:TIGHTSHIP_DATA) { $env:TIGHTSHIP_DATA } else { "{{path:data}}" }
$antigravityCliPlugin = "{{path:plugins}}/tightship"

function Test-AntigravityCli {
    if (-not $FromDir -and -not $ReleaseTag) {
        Write-Stderr "error: this install.ps1 belongs to no release; run $Repository/releases/latest/download/install.ps1, or pass -FromDir <folder>"
        exit 1
    }
}

function Resolve-AntigravityCliOptions {
    Join-Path $antigravityCliData "options.toml"
}

function Invoke-AntigravityCliTightship {
    if ($env:OS -eq "Windows_NT") {
        & (Join-Path $antigravityCliPlugin "bin\tightship.cmd") @args
    } else {
        & sh (Join-Path $antigravityCliPlugin "bin/tightship") @args
    }
}

function Install-Skill([string]$name, [string]$repository) {
    $skillDir = "{{path:skills}}/$name"
    if (-not (Test-Path $skillDir)) {
        New-Item -ItemType Directory -Force -Path $skillDir | Out-Null
    }
    $skillFile = Join-Path $skillDir "SKILL.md"
    Invoke-Step "Downloading the $name skill" { Invoke-RestMethod -Uri "https://raw.githubusercontent.com/$repository/main/SKILL.md" -OutFile $skillFile }
}

function Install-AntigravityCli {
    {{install:companions}}

    $sourceDir = $null
    $tempZipFolder = $null
    if ($FromDir) {
        $sourceDir = $FromDir
    } else {
        $tempZipFolder = Join-Path ([System.IO.Path]::GetTempPath()) ("tightship-" + [System.Guid]::NewGuid().ToString())
        New-Item -ItemType Directory -Force -Path $tempZipFolder | Out-Null
        $zipDest = Join-Path $tempZipFolder "tightship-antigravity-cli.zip"
        $downloadUrl = "$Repository/releases/download/$ReleaseTag/tightship-antigravity-cli.zip"
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

    Invoke-Step "Installing tightship for {{install:label}}" { & agy plugin install $sourceDir }

    if (-not (Test-Path $antigravityCliData)) {
        New-Item -ItemType Directory -Force -Path $antigravityCliData | Out-Null
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
    Set-Content -Path (Resolve-AntigravityCliOptions) -Value ($tomlContent + "`n") -Encoding utf8

    $env:TIGHTSHIP_DATA = $antigravityCliData
    Invoke-Step "Applying the status line option" { Invoke-AntigravityCliTightship hook status-line }

    if ($finalInitVault -and $finalVaultFolder) {
        Invoke-Step "Creating the vault at $finalVaultFolder" { Invoke-AntigravityCliTightship vault init $finalVaultFolder }
    }

    if ($tempZipFolder -and (Test-Path $tempZipFolder)) {
        Remove-Item -Recurse -Force $tempZipFolder -ErrorAction SilentlyContinue
    }
}
