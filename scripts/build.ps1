$ErrorActionPreference = "Stop"
Set-Location (Split-Path -Parent $PSScriptRoot)

Write-Host "[1/4] Building EDORA NOVA kernel..." -ForegroundColor Cyan
cargo build --release

$kernel = "target\x86_64-unknown-none\release\edora-nova"
if (-not (Test-Path $kernel)) {
    throw "Kernel ELF not found: $kernel"
}

New-Item -ItemType Directory -Force "iso_root\boot\limine" | Out-Null
New-Item -ItemType Directory -Force "iso_root\EFI\BOOT" | Out-Null
New-Item -ItemType Directory -Force "build" | Out-Null

Copy-Item $kernel "iso_root\boot\edora-nova.elf" -Force
Copy-Item "limine.conf" "iso_root\boot\limine\limine.conf" -Force

Write-Host "[2/4] Looking for Limine boot files..." -ForegroundColor Cyan
$required = @(
    "limine-bios.sys",
    "limine-bios-cd.bin",
    "limine-uefi-cd.bin",
    "BOOTX64.EFI"
)

$found = @{}
foreach ($name in $required) {
    $file = Get-ChildItem "limine-12.9.4" -Recurse -File |
        Where-Object { $_.Name -ieq $name } |
        Select-Object -First 1

    if ($file) {
        $found[$name] = $file.FullName
    }
}

$missing = @($required | Where-Object { -not $found.ContainsKey($_) })
if ($missing.Count -gt 0) {
    Write-Host "Kernel built successfully, but ISO packaging needs these Limine binary files:" -ForegroundColor Yellow
    $missing | ForEach-Object { Write-Host "  - $_" }
    Write-Host "Download the Limine binary release and extract its files into limine-12.9.4."
    Write-Host "Then run this script again."
    exit 2
}

Copy-Item $found["limine-bios.sys"] "iso_root\boot\limine\limine-bios.sys" -Force
Copy-Item $found["limine-bios-cd.bin"] "iso_root\boot\limine\limine-bios-cd.bin" -Force
Copy-Item $found["limine-uefi-cd.bin"] "iso_root\boot\limine\limine-uefi-cd.bin" -Force
Copy-Item $found["BOOTX64.EFI"] "iso_root\EFI\BOOT\BOOTX64.EFI" -Force

Write-Host "[3/4] Checking xorriso..." -ForegroundColor Cyan
$xorriso = Get-Command xorriso -ErrorAction SilentlyContinue
if (-not $xorriso) {
    Write-Host "Kernel built and ISO staging directory prepared." -ForegroundColor Yellow
    Write-Host "Install xorriso, add it to PATH, then rerun this script to create the ISO."
    exit 3
}

Write-Host "[4/4] Creating ISO..." -ForegroundColor Cyan
& xorriso -as mkisofs `
    -R -r -J `
    -b boot/limine/limine-bios-cd.bin `
    -no-emul-boot -boot-load-size 4 -boot-info-table `
    --efi-boot boot/limine/limine-uefi-cd.bin `
    -efi-boot-part --efi-boot-image --protective-msdos-label `
    iso_root -o build/edora-nova.iso

if ($LASTEXITCODE -ne 0) {
    throw "xorriso failed with exit code $LASTEXITCODE"
}

Write-Host "ISO created: build\edora-nova.iso" -ForegroundColor Green
