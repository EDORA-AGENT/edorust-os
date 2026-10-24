.PHONY: build run

build:
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/build.ps1

run:
powershell -NoProfile -ExecutionPolicy Bypass -File scripts/run-qemu.ps1

# ============================================================
# EDORA OS Rust v1.0
# Build / Check / Image / QEMU workflow
# Host: Windows
# ============================================================

.DEFAULT_GOAL := help
SHELL := cmd.exe
.SHELLFLAGS := /C

PROJECT      := D:/edorust-os
CARGO        := cargo
PYTHON       := python
QEMU         := qemu/qemu-system-x86_64.exe

KERNEL       := target/x86_64-unknown-none/release/edora-nova
BASE_IMAGE   := build/edora-nova-fixed.img
WORK_IMAGE   := build/edora-nova-memory-map.img

# Set this to the path of your verified FAT32 image updater.
# Example:
# make image IMAGE_SCRIPT=tools/update_image.py
IMAGE_SCRIPT ?=

.PHONY: help check fmt build release kernel image run debug clean verify

help:
	@echo.
	@echo EDORA OS Rust v1.0 - Available targets
	@echo.
	@echo make check    - Check Rust code
	@echo make fmt      - Format Rust source
	@echo make build    - Build debug kernel
	@echo make release  - Build release kernel
	@echo make kernel   - Build release kernel and verify output
	@echo make image    - Update working disk image using verified updater
	@echo make run      - Boot the working image in QEMU
	@echo make debug    - Boot QEMU with a debug console
	@echo make verify   - Verify required project files
	@echo make clean    - Remove Cargo build artifacts
	@echo.

check:
	$(CARGO) check --target x86_64-unknown-none

fmt:
	$(CARGO) fmt --all -- --check

build:
	$(CARGO) build

release:
	$(CARGO) build --release

kernel: release
	@if not exist "$(KERNEL)" (echo ERROR: Kernel not found: $(KERNEL) & exit /b 1)
	@echo Kernel build verified: $(KERNEL)

verify:
	@if not exist "Cargo.toml" (echo ERROR: Cargo.toml missing & exit /b 1)
	@if not exist "src/main.rs" (echo ERROR: src/main.rs missing & exit /b 1)
	@if not exist "$(QEMU)" (echo ERROR: QEMU executable missing & exit /b 1)
	@if not exist "build" (echo ERROR: build directory missing & exit /b 1)
	@echo Required project files found.

image: kernel
	@if "$(IMAGE_SCRIPT)"=="" (echo ERROR: Set IMAGE_SCRIPT to your verified FAT32 updater. & echo No image was modified. & exit /b 1)
	@if not exist "$(IMAGE_SCRIPT)" (echo ERROR: Updater script not found: $(IMAGE_SCRIPT) & exit /b 1)
	@if not exist "$(BASE_IMAGE)" (echo ERROR: Base image missing: $(BASE_IMAGE) & exit /b 1)
	$(PYTHON) "$(IMAGE_SCRIPT)"

run:
	@if not exist "$(QEMU)" (echo ERROR: QEMU not found & exit /b 1)
	@if not exist "$(WORK_IMAGE)" (echo ERROR: Working image missing. Run make image first. & exit /b 1)
	"$(QEMU)" -drive "file=$(WORK_IMAGE),format=raw" -m 256M

debug:
	@if not exist "$(WORK_IMAGE)" (echo ERROR: Working image missing. Run make image first. & exit /b 1)
	"$(QEMU)" -drive "file=$(WORK_IMAGE),format=raw" -m 256M -no-reboot -no-shutdown

clean:
	$(CARGO) clean
