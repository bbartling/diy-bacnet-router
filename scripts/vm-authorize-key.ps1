#Requires -Version 5.1
<#
.SYNOPSIS
  One-time: authorize the Windows SSH public key on ubuntu2.

.DESCRIPTION
  Prompts for the VM password once, then future agent/ssh sessions use keys.
#>
[CmdletBinding()]
param(
    [int]$Port = 2222,
    [string]$User = 'ben',
    [string]$HostName = '127.0.0.1'
)

$ErrorActionPreference = 'Stop'
$target = "${User}@${HostName}"
$pub = "$env:USERPROFILE\.ssh\id_rsa.pub"
if (-not (Test-Path $pub)) { $pub = "$env:USERPROFILE\.ssh\id_ed25519.pub" }
if (-not (Test-Path $pub)) { throw "No SSH public key found under $env:USERPROFILE\.ssh" }

Write-Host "Authorizing $(Split-Path $pub -Leaf) on $target (port $Port)" -ForegroundColor Cyan
Write-Host "Enter the VM password when prompted." -ForegroundColor Yellow

Get-Content $pub | ssh -p $Port -o StrictHostKeyChecking=accept-new $target `
    "umask 077; mkdir -p ~/.ssh; cat >> ~/.ssh/authorized_keys; chmod 700 ~/.ssh; chmod 600 ~/.ssh/authorized_keys; echo KEY_INSTALLED"

if ($LASTEXITCODE -ne 0) { throw "Key authorization failed" }

ssh -o BatchMode=yes -p $Port $target "echo SSH_KEY_OK"
if ($LASTEXITCODE -ne 0) { throw "BatchMode verification failed after key install" }

Write-Host "Done. Run: .\scripts\vm-ensure.ps1 -RunSetup" -ForegroundColor Green
