$CursorSha256 = ""
$cursorData = if ($env:TIGHTSHIP_DATA) { $env:TIGHTSHIP_DATA } else { "{{path:data}}" }
$cursorPlugin = "{{path:plugins}}/tightship"
$cursorSettings = "{{path:config}}/cli-config.json"

function Test-Cursor {
    if (-not $FromDir -and -not $ReleaseTag) {
        throw "error: this install.ps1 belongs to no release; run $Repository/releases/latest/download/install.ps1, or pass -FromDir <folder>"
    }
}

function Read-CursorOptions {
    $optionsFile = Join-Path $cursorData "options.json"
    if (-not (Test-Path $optionsFile)) {
        return $null
    }
    try {
        $parsed = Get-Content -Raw $optionsFile | ConvertFrom-Json
    } catch {
        throw "error: $optionsFile is not valid JSON"
    }
    $options = [ordered]@{ contract = $null; memory = $null; status_line = $null; auto_update = $null; vault_folder = $null }
    foreach ($name in "contract", "memory", "status_line", "auto_update") {
        if ($null -ne $parsed.$name) { $options[$name] = ([string]$parsed.$name).ToLowerInvariant() }
    }
    if ($null -ne $parsed.vault_folder) { $options.vault_folder = [string]$parsed.vault_folder }
    $options
}

function Invoke-CursorTightship {
    if ($env:OS -eq "Windows_NT") {
        & (Join-Path $cursorPlugin "bin\tightship.cmd") @args
    } else {
        & sh (Join-Path $cursorPlugin "bin/tightship") @args
    }
}

function Write-CursorOptions {
    if (-not (Test-Path $cursorData)) {
        New-Item -ItemType Directory -Force -Path $cursorData | Out-Null
    }
    $table = [ordered]@{
        contract    = ($finalContract -eq "true")
        memory      = ($finalMemory -eq "true")
        status_line = ($finalStatusLine -eq "true")
        auto_update = ($finalAutoUpdate -eq "true")
    }
    if ($finalVaultFolder) {
        $table["vault_folder"] = $finalVaultFolder
    }
    $json = ($table | ConvertTo-Json -Depth 3) + "`n"
    Set-Content -Path (Join-Path $cursorData "options.json") -Value $json -Encoding utf8
}

function Install-CursorCompanion([string]$name, [string]$repository) {
    $work = Join-Path $workDir "companion-$name"
    New-Item -ItemType Directory -Force -Path $work | Out-Null
    $zipDest = Join-Path $work "$name.zip"
    $url = "https://github.com/$repository/archive/refs/heads/main.zip"
    Invoke-Step "Installing the $name companion" {
        try {
            Invoke-RestMethod -Uri $url -OutFile $zipDest
        } catch {
            $url = "https://github.com/$repository/archive/refs/heads/master.zip"
            Invoke-RestMethod -Uri $url -OutFile $zipDest
        }
        $extracted = Join-Path $work "extracted"
        Expand-Archive -Path $zipDest -DestinationPath $extracted -Force
        $root = Get-ChildItem -Path $extracted -Directory |
            Where-Object { $_.Name -notlike '__MACOSX*' } |
            Select-Object -First 1
        $dest = Join-Path "{{path:plugins}}" $name
        if (Test-Path $dest) {
            Remove-Item -Recurse -Force $dest
        }
        New-Item -ItemType Directory -Force -Path $dest | Out-Null
        Copy-Item -Path (Join-Path $root.FullName "*") -Destination $dest -Recurse -Force
    }
}

function Install-Cursor {
    {{install:companions}}

    $sourceDir = $null
    $tempZipFolder = $null
    if ($FromDir) {
        $sourceDir = $FromDir
    } else {
        $tempZipFolder = Join-Path $workDir "cursor"
        New-Item -ItemType Directory -Force -Path $tempZipFolder | Out-Null
        $zipDest = Join-Path $tempZipFolder "tightship-cursor.zip"
        $downloadUrl = "$Repository/releases/download/$ReleaseTag/tightship-cursor.zip"
        Invoke-Step "Downloading tightship-cursor.zip $ReleaseTag" {
            try {
                Invoke-RestMethod -Uri $downloadUrl -OutFile $zipDest
            } catch {
                throw "error: failed to download tightship-cursor.zip from $downloadUrl`nhint: if you are running in a local repository clone, run cargo xtask build and pass -FromDir dist\cursor"
            }
        }
        $zipSha256 = (Get-FileHash -Algorithm SHA256 -Path $zipDest).Hash.ToLowerInvariant()
        Invoke-Step "Checking the sha256 of tightship-cursor.zip" {
            if ($zipSha256 -ne $CursorSha256) {
                throw "error: tightship-cursor.zip has sha256 $zipSha256; $ReleaseTag published $CursorSha256"
            }
        }
        $extracted = Join-Path $tempZipFolder "extracted"
        Expand-Archive -Path $zipDest -DestinationPath $extracted -Force
        $sourceDir = $extracted
    }

    Invoke-Step "Installing tightship for {{install:label}}" {
        if (Test-Path $cursorPlugin) {
            Remove-Item -Recurse -Force $cursorPlugin
        }
        New-Item -ItemType Directory -Force -Path $cursorPlugin | Out-Null
        Copy-Item -Path (Join-Path $sourceDir "*") -Destination $cursorPlugin -Recurse -Force
    }

    Write-CursorOptions

    $env:TIGHTSHIP_DATA = $cursorData
    $env:TIGHTSHIP_SETTINGS = $cursorSettings
    $env:TIGHTSHIP_ROOT = $cursorPlugin
    Invoke-Step "Applying the status line option" { Invoke-CursorTightship hook status-line }

    if ($finalInitVault -and $finalVaultFolder) {
        Invoke-Step "Creating the vault at $expandedVault" { Invoke-CursorTightship vault init $expandedVault }
    }
    $installedIn["{{install:label}}"] = [System.IO.Path]::GetFullPath($cursorPlugin)
}
