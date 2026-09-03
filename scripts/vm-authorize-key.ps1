#Requires -Version 5.1
<#
.SYNOPSIS
  Authorize Windows SSH key on ubuntu2 using config/vm.env credentials.

.DESCRIPTION
  1. Copy config/vm.env.example to config/vm.env
  2. Set VM_SSH_PASSWORD (and user/host/port if needed)
  3. Run this script once - future sessions use key auth
#>
[CmdletBinding()]
param(
    [int]$Port = 0,
    [string]$User = '',
    [string]$HostName = ''
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'vm-load-env.ps1')

$Port = if ($Port) { $Port } else { [int]$env:VM_SSH_PORT }
if (-not $Port) { $Port = 2222 }
$User = if ($User) { $User } else { $env:VM_SSH_USER }
if (-not $User) { $User = 'ben' }
$HostName = if ($HostName) { $HostName } else { $env:VM_SSH_HOST }
if (-not $HostName) { $HostName = '127.0.0.1' }
$target = "${User}@${HostName}"

# Already authorized?
$prevEap = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
$probe = ssh -o BatchMode=yes -o ConnectTimeout=8 -p $Port $target "echo SSH_KEY_OK" 2>&1
$ErrorActionPreference = $prevEap
if ($LASTEXITCODE -eq 0) {
    Write-Host "SSH key already authorized: $probe" -ForegroundColor Green
    exit 0
}

Write-Host "Installing SSH key via config/vm.env ..." -ForegroundColor Cyan
python (Join-Path $PSScriptRoot 'vm-ssh-install-key.py')
if ($LASTEXITCODE -ne 0) { throw "Key install failed - check config/vm.env" }

ssh -o BatchMode=yes -p $Port $target 'echo SSH_KEY_OK; uname -sr; nproc'
if ($LASTEXITCODE -ne 0) { throw "BatchMode verification failed" }

Write-Host "Done. Next: .\scripts\vm-ensure.ps1 -Hypervisor vmware -RunSetup" -ForegroundColor Green
