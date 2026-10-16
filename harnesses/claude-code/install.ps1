function Install-ClaudeCode {
    try { & claude plugin marketplace add DietrichGebert/ponytail } catch {}
    try { & claude plugin marketplace add ayghri/i-have-adhd } catch {}
    try { & claude plugin marketplace add blader/humanizer } catch {}
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
