#Requires -Version 5.1
<#
.SYNOPSIS
  Load config/vm.env into the current PowerShell session.
#>
param(
    [string]$EnvFile = (Join-Path (Split-Path $PSScriptRoot -Parent) 'config\vm.env')
)

if (-not (Test-Path $EnvFile)) {
    Write-Warning "Missing $EnvFile - copy config/vm.env.example to config/vm.env and set VM_SSH_PASSWORD"
    return
}

Get-Content $EnvFile | ForEach-Object {
    $line = $_.Trim()
    if (-not $line -or $line.StartsWith('#')) { return }
    $eq = $line.IndexOf('=')
    if ($eq -lt 1) { return }
    $name = $line.Substring(0, $eq).Trim()
    $value = $line.Substring($eq + 1).Trim()
    Set-Item -Path "Env:$name" -Value $value
}

Write-Verbose "Loaded VM env from $EnvFile"
