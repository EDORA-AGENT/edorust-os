$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

$iso = "build\edora-nova.iso"
if (-not (Test-Path $iso)) {
    throw "ISO not found. Run .\scripts\build.ps1 first."
}

$qemu = Get-Command qemu-system-x86_64 -ErrorAction SilentlyContinue
if (-not $qemu) {
    throw "QEMU was not found in PATH. Install QEMU for Windows and add its bin folder to PATH."
}

& $qemu.Source -m 256M -cdrom $iso -boot d
