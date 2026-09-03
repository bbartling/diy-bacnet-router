#Requires -Version 5.1
<#
.SYNOPSIS
  Ensure the Ubuntu Buildroot lab guest is reachable over SSH, then run optional tasks.

.DESCRIPTION
  Supported lab is a VMware Ubuntu guest (SSH ben@127.0.0.1:2222). VirtualBox
  remains an optional start path when VBoxManage and VM "ubuntu2" are present.

  -Hypervisor auto|vmware|virtualbox|none
  -SkipVmStart  — never start a hypervisor VM; only probe SSH and run scripts

.EXAMPLE
  .\scripts\vm-ensure.ps1 -Hypervisor vmware
  .\scripts\vm-ensure.ps1 -SkipVmStart -AcceptRunId 33671378385
  .\scripts\vm-ensure.ps1 -Hypervisor virtualbox -RunSetup
#>
[CmdletBinding()]
param(
    [ValidateSet('auto', 'vmware', 'virtualbox', 'none')]
    [string]$Hypervisor = 'auto',
    [switch]$SkipVmStart,
    [switch]$RunSetup,
    [switch]$RunBuild,
    [switch]$DebugBuild,
    [string]$AcceptRunId = '',
    [int]$BootWaitSeconds = 20
)

$ErrorActionPreference = 'Stop'
. (Join-Path $PSScriptRoot 'vm-load-env.ps1')

$VBox = 'C:\Program Files\Oracle\VirtualBox\VBoxManage.exe'
$VmName = 'ubuntu2'
$SshHost = if ($env:VM_SSH_HOST) { $env:VM_SSH_HOST } else { '127.0.0.1' }
$SshPort = if ($env:VM_SSH_PORT) { [int]$env:VM_SSH_PORT } else { 2222 }
$SshUser = if ($env:VM_SSH_USER) { $env:VM_SSH_USER } else { 'ben' }
$SshTarget = "${SshUser}@${SshHost}"

function Write-Step([string]$Message) {
    Write-Host "==> $Message" -ForegroundColor Cyan
}

function Test-SshPortOpen {
    $tcp = Test-NetConnection -ComputerName $SshHost -Port $SshPort -WarningAction SilentlyContinue
    return [bool]$tcp.TcpTestSucceeded
}

function Invoke-VmBashScript {
    param(
        [Parameter(Mandatory)][string]$ScriptPath,
        [string]$RemoteCommand = 'bash -s'
    )
    if (-not (Test-Path $ScriptPath)) { throw "Missing script: $ScriptPath" }
    $wrapper = $RemoteCommand
    if ($env:VM_SSH_PASSWORD) {
        $b64 = [Convert]::ToBase64String([Text.Encoding]::UTF8.GetBytes($env:VM_SSH_PASSWORD))
        $wrapper = "export VM_SSH_PASSWORD=`$(echo '$b64' | base64 -d); $RemoteCommand"
    }
    $content = (Get-Content $ScriptPath -Raw) -replace "`r`n", "`n"
    $content | & ssh -p $SshPort $SshTarget $wrapper
}

function Start-VirtualBoxGuest {
    if (-not (Test-Path $VBox)) {
        throw "VirtualBox not found at $VBox. Use -Hypervisor vmware|none or start the VMware guest manually."
    }
    $running = & $VBox list runningvms 2>$null | Select-String -SimpleMatch "`"$VmName`""
    if (-not $running) {
        Write-Step "Starting VirtualBox VM '$VmName' headless"
        & $VBox startvm $VmName --type headless | Out-Null
        Write-Step "Waiting ${BootWaitSeconds}s for boot"
        Start-Sleep -Seconds $BootWaitSeconds
    } else {
        Write-Step "VirtualBox VM '$VmName' already running"
    }
}

# Resolve how (or whether) to start a guest.
$mode = $Hypervisor
if ($SkipVmStart) {
    $mode = 'none'
    Write-Step "SkipVmStart set; not starting any hypervisor VM"
} elseif ($mode -eq 'auto') {
    if (Test-SshPortOpen) {
        $mode = 'none'
        Write-Step "SSH already reachable on ${SshHost}:${SshPort}; skipping hypervisor start"
    } elseif (Test-Path $VBox) {
        $mode = 'virtualbox'
        Write-Step "auto: VirtualBox present and SSH closed; will try VirtualBox start"
    } else {
        $mode = 'vmware'
        Write-Step "auto: no VirtualBox; expecting a manually started VMware guest"
    }
}

switch ($mode) {
    'virtualbox' {
        Start-VirtualBoxGuest
    }
    'vmware' {
        Write-Step "VMware mode: start the Ubuntu guest in VMware if needed (no GUI automation)"
        Write-Host "    Expected SSH: ${SshUser}@${SshHost}:${SshPort} (alias ubuntu2-buildroot)" -ForegroundColor DarkGray
    }
    'none' {
        Write-Step "none mode: hypervisor start skipped"
    }
    default {
        throw "Unknown hypervisor mode: $mode"
    }
}

Write-Step "Checking TCP ${SshHost}:${SshPort}"
if (-not (Test-SshPortOpen)) {
    throw @"
SSH port ${SshHost}:${SshPort} is not open.
Start the Ubuntu guest in VMware (NAT port forward host 2222 -> guest 22), then re-run:
  .\scripts\vm-ensure.ps1 -Hypervisor vmware
Or with an already-running guest:
  .\scripts\vm-ensure.ps1 -SkipVmStart
"@
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
    Write-Host "1. Copy config/vm.env.example to config/vm.env and set VM_SSH_PASSWORD" -ForegroundColor Yellow
    Write-Host "2. Run: .\scripts\vm-authorize-key.ps1" -ForegroundColor White
    Write-Host ""
    Write-Host "Then re-run: .\scripts\vm-ensure.ps1 -Hypervisor $Hypervisor$(if ($SkipVmStart) { ' -SkipVmStart' })$(if ($RunSetup) { ' -RunSetup' })$(if ($RunBuild) { ' -RunBuild' })" -ForegroundColor Yellow
    exit 2
}

Write-Host $probe
Write-Step "SSH key auth OK"

if ($RunSetup) {
    Write-Step "Running vm-setup.sh on VM"
    Invoke-VmBashScript -ScriptPath (Join-Path $PSScriptRoot 'vm-setup.sh')
    if ($LASTEXITCODE -ne 0) { throw "vm-setup.sh failed" }
}

if ($RunBuild) {
    Write-Step "Running vm-build-x86.sh on VM (long-running)"
    & ssh -p $SshPort $SshTarget 'cd ~/src/diy-bacnet-router && git pull --ff-only && bash scripts/vm-build-x86.sh'
    if ($LASTEXITCODE -ne 0) { throw "vm-build-x86.sh failed" }
}

if ($DebugBuild) {
    Write-Step "Running vm-debug-build.sh on VM (long-running)"
    Invoke-VmBashScript -ScriptPath (Join-Path $PSScriptRoot 'vm-debug-build.sh')
    if ($LASTEXITCODE -ne 0) { throw "vm-debug-build.sh failed" }
}

if ($AcceptRunId) {
    Write-Step "Running vm-accept-artifact.sh for run $AcceptRunId"
    Get-Content (Join-Path $PSScriptRoot 'vm-accept-artifact.sh') -Raw |
        & ssh -p $SshPort $SshTarget "bash -s -- $AcceptRunId"
    if ($LASTEXITCODE -ne 0) { throw "vm-accept-artifact.sh failed" }
}

Write-Step "VM ready"
