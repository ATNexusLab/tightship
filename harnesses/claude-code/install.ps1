function Install-ClaudeCode {
    {{install:companions}}
    if ($FromDir) {
        try { & claude plugin marketplace add $FromDir } catch {}
    } else {
        try { & claude plugin marketplace add ($Repository -replace '^https://github.com/', '') } catch {}
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
    Invoke-Step "Installing tightship for {{install:label}}" { & claude @claudeArgs }
}
