#Requires -Version 5.1
<#
.SYNOPSIS
  Start the ubuntu2 Buildroot VM and verify SSH for the agent.

.DESCRIPTION
  - Starts VirtualBox VM "ubuntu2" headless if not running
  - Waits for localhost:2222
  - Tests key-based SSH (BatchMode)
  - If keys are missing, prints the one-time authorize command

.EXAMPLE
  .\scripts\vm-ensure.ps1
  .\scripts\vm-ensure.ps1 -RunSetup
  .\scripts\vm-ensure.ps1 -AcceptRunId 33642454599
#>
[CmdletBinding()]
param(
    [switch]$RunSetup,
    [switch]$RunBuild,
    [string]$AcceptRunId = '',
    [int]$BootWaitSeconds = 20
)

$ErrorActionPreference = 'Stop'
$VBox = 'C:\Program Files\Oracle\VirtualBox\VBoxManage.exe'
$VmName = 'ubuntu2'
$SshHost = '127.0.0.1'
$SshPort = 2222
$SshUser = 'ben'
$SshTarget = "${SshUser}@${SshHost}"

function Write-Step([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

if (-not (Test-Path $VBox)) {
    throw "VirtualBox not found at $VBox"
}

$running = & $VBox list runningvms 2>$null | Select-String -SimpleMatch "`"$VmName`""
if (-not $running) {
    Write-Step "Starting VM '$VmName' headless"
    & $VBox startvm $VmName --type headless | Out-Null
    Write-Step "Waiting ${BootWaitSeconds}s for boot"
    Start-Sleep -Seconds $BootWaitSeconds
} else {
    Write-Step "VM '$VmName' already running"
}

Write-Step "Checking TCP ${SshHost}:${SshPort}"
$tcp = Test-NetConnection -ComputerName $SshHost -Port $SshPort -WarningAction SilentlyContinue
if (-not $tcp.TcpTestSucceeded) {
    throw "SSH port ${SshHost}:${SshPort} is not open. Wait longer or inspect VM network/NAT forwarding."
}

Write-Step "Testing key-based SSH"
$probe = $null
$sshExit = 0
$prevEap = $ErrorActionPreference
$ErrorActionPreference = 'Continue'
try {
    $probe = & ssh -o BatchMode=yes -o ConnectTimeout=10 -o StrictHostKeyChecking=accept-new `
        -p $SshPort $SshTarget "echo SSH_KEY_OK && uname -sr && nproc" 2>&1
    $sshExit = $LASTEXITCODE
} finally {
    $ErrorActionPreference = $prevEap
}
if ($sshExit -ne 0) {
    Write-Host ""
    Write-Host "SSH is up but your Windows key is not authorized on the VM yet." -ForegroundColor Yellow
    Write-Host "Run this ONCE in PowerShell (enter VM password when prompted):" -ForegroundColor Yellow
    Write-Host ""
    Write-Host "  .\scripts\vm-authorize-key.ps1" -ForegroundColor White
    Write-Host ""
    Write-Host "Then re-run: .\scripts\vm-ensure.ps1$(if ($RunSetup) { ' -RunSetup' })$(if ($RunBuild) { ' -RunBuild' })" -ForegroundColor Yellow
    exit 2
}

Write-Host $probe
Write-Step "SSH key auth OK"

if ($RunSetup) {
    Write-Step "Running vm-setup.sh on VM"
    Get-Content (Join-Path $PSScriptRoot 'vm-setup.sh') -Raw | & ssh -p $SshPort $SshTarget "bash -s"
    if ($LASTEXITCODE -ne 0) { throw "vm-setup.sh failed" }
}

if ($RunBuild) {
    Write-Step "Running vm-build-x86.sh on VM (long-running)"
    & ssh -p $SshPort $SshTarget 'cd ~/src/diy-bacnet-router && git pull --ff-only && bash scripts/vm-build-x86.sh'
    if ($LASTEXITCODE -ne 0) { throw "vm-build-x86.sh failed" }
}

if ($AcceptRunId) {
    Write-Step "Running vm-accept-artifact.sh for run $AcceptRunId"
    Get-Content (Join-Path $PSScriptRoot 'vm-accept-artifact.sh') -Raw |
        & ssh -p $SshPort $SshTarget "bash -s -- $AcceptRunId"
    if ($LASTEXITCODE -ne 0) { throw "vm-accept-artifact.sh failed" }
}

Write-Step "VM ready"
