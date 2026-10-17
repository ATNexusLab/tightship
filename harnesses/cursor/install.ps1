$CursorSha256 = ""
$cursorData = if ($env:TIGHTSHIP_DATA) { $env:TIGHTSHIP_DATA } else { "{{path:data}}" }
$cursorPlugin = "{{path:plugins}}/tightship"
$cursorSettings = "{{path:config}}/cli-config.json"

function Test-Cursor {
    if (-not $FromDir -and -not $ReleaseTag) {
        Write-Stderr "error: this install.ps1 belongs to no release; run $Repository/releases/latest/download/install.ps1, or pass -FromDir <folder>"
        exit 1
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
        Write-Stderr "error: $optionsFile is not valid JSON"
        exit 1
    }
    $options = [ordered]@{ contract = $null; memory = $null; status_line = $null; vault_folder = $null }
    foreach ($name in "contract", "memory", "status_line") {
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
    }
    if ($finalVaultFolder) {
        $table["vault_folder"] = $finalVaultFolder
    }
    $json = ($table | ConvertTo-Json -Depth 3) + "`n"
    Set-Content -Path (Join-Path $cursorData "options.json") -Value $json -Encoding utf8
}

function Install-CursorCompanion([string]$name, [string]$repository) {
    $work = Join-Path ([System.IO.Path]::GetTempPath()) ("tightship-$name-" + [System.Guid]::NewGuid().ToString())
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
    if (Test-Path $work) {
        Remove-Item -Recurse -Force $work -ErrorAction SilentlyContinue
    }
}

function Install-Cursor {
    {{install:companions}}

    $sourceDir = $null
    $tempZipFolder = $null
    if ($FromDir) {
        $sourceDir = $FromDir
    } else {
        $tempZipFolder = Join-Path ([System.IO.Path]::GetTempPath()) ("tightship-" + [System.Guid]::NewGuid().ToString())
        New-Item -ItemType Directory -Force -Path $tempZipFolder | Out-Null
        $zipDest = Join-Path $tempZipFolder "tightship-cursor.zip"
        $downloadUrl = "$Repository/releases/download/$ReleaseTag/tightship-cursor.zip"
        Write-Host "> Downloading tightship-cursor.zip $ReleaseTag"
        try {
            Invoke-RestMethod -Uri $downloadUrl -OutFile $zipDest
        } catch {
            Write-Stderr "error: failed to download tightship-cursor.zip from $downloadUrl"
            Write-Stderr "hint: if you are running in a local repository clone, run cargo xtask build and pass -FromDir dist\cursor"
            exit 1
        }
        Write-Host "> Checking the sha256 of tightship-cursor.zip"
        $zipSha256 = (Get-FileHash -Algorithm SHA256 -Path $zipDest).Hash.ToLowerInvariant()
        if ($zipSha256 -ne $CursorSha256) {
            Write-Stderr "error: tightship-cursor.zip has sha256 $zipSha256; $ReleaseTag published $CursorSha256"
            exit 1
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
    $configDir = Split-Path $cursorSettings -Parent
    if (-not (Test-Path $configDir)) {
        New-Item -ItemType Directory -Force -Path $configDir | Out-Null
    }
    Invoke-Step "Applying the status line option" { Invoke-CursorTightship hook status-line }

    if ($finalInitVault -and $finalVaultFolder) {
        Invoke-Step "Creating the vault at $finalVaultFolder" { Invoke-CursorTightship vault init $finalVaultFolder }
    }

    if ($tempZipFolder -and (Test-Path $tempZipFolder)) {
        Remove-Item -Recurse -Force $tempZipFolder -ErrorAction SilentlyContinue
    }
}
