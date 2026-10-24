
param(
    [Parameter(Position = 0)]
    [ValidateSet("help", "check", "fmt", "build", "release", "kernel", "verify", "run", "debug", "clean")]
    [string]$Task = "help"
)

$ErrorActionPreference = "Stop"
Set-Location $PSScriptRoot

$Kernel = "target\x86_64-unknown-none\release\edora-nova"
$Qemu = "qemu\qemu-system-x86_64.exe"
$Image = "build\edora-nova-memory-map.img"

function Require-File([string]$Path) {
    if (!(Test-Path $Path -PathType Leaf)) {
        throw "Khong tim thay file: $Path"
    }
}

function Invoke-Cargo([string[]]$Arguments) {
    & cargo @Arguments
    if ($LASTEXITCODE -ne 0) {
        throw "Cargo that bai, exit code $LASTEXITCODE"
    }
}

try {
    switch ($Task) {
        "help" {
            @"
EDORA OS Rust v1.0 - Build tools

  .\build.ps1 help     Hien thi tro giup
  .\build.ps1 verify   Kiem tra cac file can thiet
  .\build.ps1 check    Kiem tra Rust code
  .\build.ps1 fmt      Kiem tra format
  .\build.ps1 build    Debug build
  .\build.ps1 release  Release build
  .\build.ps1 kernel   Build va kiem tra kernel
  .\build.ps1 run      Chay image bang QEMU
  .\build.ps1 debug    Chay QEMU debug
  .\build.ps1 clean    Xoa build artifacts

Luu y: run/debug chi chay image co san.
Script khong tu sua image FAT32.
"@
        }

        "verify" {
            Require-File "Cargo.toml"
            Require-File "src\main.rs"
            Require-File $Qemu
            Require-File "build\edora-nova-fixed.img"
            Write-Host "Project files: OK" -ForegroundColor Green
        }

        "check" {
            Invoke-Cargo @("check", "--target", "x86_64-unknown-none")
        }

        "fmt" {
            Invoke-Cargo @("fmt", "--all", "--", "--check")
        }

        "build" {
            Invoke-Cargo @("build")
        }

        "release" {
            Invoke-Cargo @("build", "--release")
        }

        "kernel" {
            Invoke-Cargo @("build", "--release")
            Require-File $Kernel
            Write-Host "Kernel: $Kernel" -ForegroundColor Green
            Write-Host "Size: $((Get-Item $Kernel).Length) bytes"
        }

        "run" {
            Require-File $Qemu
            Require-File $Image
            & $Qemu -drive "file=$Image,format=raw" -m 256M
            if ($LASTEXITCODE -ne 0) {
                throw "QEMU exited with code $LASTEXITCODE"
            }
        }

        "debug" {
            Require-File $Qemu
            Require-File $Image
            & $Qemu -drive "file=$Image,format=raw" -m 256M -no-reboot -no-shutdown
            if ($LASTEXITCODE -ne 0) {
                throw "QEMU exited with code $LASTEXITCODE"
            }
        }

        "clean" {
            Invoke-Cargo @("clean")
        }
    }

    if ($Task -eq "help" -or $Task -eq "verify") {
        exit 0
    }
}
catch {
    Write-Host "ERROR: $_" -ForegroundColor Red
    exit 1
}
