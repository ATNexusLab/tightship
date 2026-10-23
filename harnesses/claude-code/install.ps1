function Read-ClaudeCodeOptions {
    try {
        $parsed = & claude plugin configure tightship@tightship --json 2>$null | Out-String | ConvertFrom-Json
    } catch {
        return $null
    }
    if (-not $parsed.inputs) {
        return $null
    }
    $options = [ordered]@{ contract = $null; memory = $null; status_line = $null; auto_update = $null; vault_folder = $null }
    foreach ($name in "contract", "memory", "status_line", "auto_update") {
        if ($null -ne $parsed.inputs.$name) { $options[$name] = ([string]$parsed.inputs.$name).ToLowerInvariant() }
    }
    if ($null -ne $parsed.inputs.vault_folder) { $options.vault_folder = [string]$parsed.inputs.vault_folder }
    $options
}

function Install-ClaudeCode {
    {{install:companions}}
    if ($FromDir) {
        try { & claude plugin marketplace add $FromDir *> $null } catch {}
    } else {
        try { & claude plugin marketplace add ($Repository -replace '^https://github.com/', '') *> $null } catch {}
    }

    $listed = try { & claude plugin list --json 2>$null | Out-String } catch { "" }
    if ($listed -match '"tightship@tightship"') {
        try { & claude plugin marketplace update tightship *> $null } catch {}
        Invoke-Step "Updating tightship for {{install:label}}" { & claude plugin update tightship@tightship }
        $values = [ordered]@{
            contract    = $finalContract
            memory      = $finalMemory
            status_line = $finalStatusLine
            auto_update = $finalAutoUpdate
        }
        if ($finalVaultFolder) {
            $values["vault_folder"] = $finalVaultFolder
        }
        $json = $values | ConvertTo-Json -Compress
        Invoke-Step "Applying the options to {{install:label}}" { $json | & claude plugin configure tightship@tightship --values-stdin }
    } else {
        $claudeArgs = @(
            "plugin", "install", "tightship@tightship",
            "--config", "contract=$finalContract",
            "--config", "memory=$finalMemory",
            "--config", "status_line=$finalStatusLine",
            "--config", "auto_update=$finalAutoUpdate"
        )
        if ($finalVaultFolder) {
            $claudeArgs += @("--config", "vault_folder=$finalVaultFolder")
        }
        Invoke-Step "Installing tightship for {{install:label}}" { & claude @claudeArgs }
    }
    $installedIn["{{install:label}}"] = "tightship@tightship, listed by claude plugin list"
}
