<#
.SYNOPSIS
    Automates integration of the shared errors crate across workspace.
#>
[CmdletBinding()]
param()

# Crate folders to update
$crates = @('shared', 'compiler', 'tlang')

# 1) Add errors dependency to Cargo.toml
foreach ($crate in $crates) {
    $tomlPath = Join-Path $PSScriptRoot "$crate\Cargo.toml"
    $content = Get-Content -Raw $tomlPath
    if ($content -match 'errors = \{ path = "\.\./errors" \}') {
        Write-Host "[SKIP] errors dependency exists in $crate"
    } else {
        $updated = $content -replace '(?m)^\[dependencies\]', "`$&`nerrors = { path = '../errors' }"
        Set-Content -Path $tomlPath -Value $updated
        Write-Host "[ADD] errors dependency to $crate"
    }
}

# 2) Replace old imports in .rs files
get-childitem -Path compiler,shared,tlang -Recurse -Filter *.rs | ForEach-Object {
    $file = $_.FullName
    $text = Get-Content -Raw $file
    $orig = $text
    # Replace qualified import
    $text = $text -replace 'use crate::errors::CompileError;', 'use errors::CompileError;'
    # Remove mod error lines
    $text = $text -replace 'mod error;\s*', ''
    # Prepend import if missing
    if ($text -notmatch 'use errors::CompileError;') {
        $text = "use errors::CompileError;`r`n`r`n" + $text
    }
    if ($text -ne $orig) {
        Set-Content -Path $file -Value $text
        Write-Host "[UPDATE] imports in $file"
    }
}

# 3) Final build check
Write-Host "Running cargo build --workspace..."
cargo build --workspace
Write-Host "Integration complete."