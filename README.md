# EDORust-OS 🦀

<p align="center">
  <strong>A Rust-powered Operating System Project by EDORA STUDIO</strong>
  <br>
  Building an operating system from the ground up, one component at a time.
</p>

---

## 🖥️ About

**EDORust-OS** is an experimental operating system development project written primarily in **Rust**, created by **EDORA STUDIO**.

The project explores low-level system programming, kernel development, memory management, hardware interaction, and graphical output without relying on a conventional operating system kernel.

EDORust-OS is a learning-driven project focused on understanding how operating systems work internally — from the moment the kernel starts executing to the initialization of memory and hardware interfaces.

Rather than attempting to build a complete operating system immediately, EDORust-OS is developed incrementally, with each component serving as a foundation for future improvements.

## ✨ Features

- 🦀 **Rust-based kernel** — Core operating system components are implemented in Rust.
- 🚀 **Limine boot protocol** — Uses Limine to load and initialize the kernel.
- 🖼️ **Framebuffer graphics** — Supports graphical output through the framebuffer.
- 🧠 **Memory map support** — Retrieves memory map information for low-level memory management.
- 🖱️ **PS/2 mouse support** — Implements mouse input handling for basic interaction.
- 🎨 **Custom splash screen** — Displays EDORust-OS branding during startup.
- 🖥️ **Experimental graphical desktop** — Includes early desktop drawing and graphical interface components.
- 🔧 **Modular architecture** — Organizes kernel functionality into separate modules, including panic handling and memory management.

## 🏗️ Project Architecture

The current project is organized around the following components:

| Component | Purpose |
|---|---|
| Kernel entry point | Initializes the operating system |
| Limine integration | Provides bootloader and boot protocol support |
| Framebuffer | Handles graphical output |
| Memory map | Provides information about available system memory |
| Panic handler | Handles kernel panic situations |
| Memory module | Establishes the foundation for memory management |
| PS/2 mouse driver | Processes basic mouse input |
| Graphical interface | Draws the startup screen and early desktop elements |

*The architecture is experimental and will evolve as development progresses.*

## 🛠️ Technology Stack

- **Programming language:** Rust
- **Bootloader:** Limine
- **Graphics:** Framebuffer
- **Input:** PS/2 mouse interface
- **Development focus:** Bare-metal and low-level systems programming
- **Project identity:** EDORA STUDIO

## 🚧 Development Status

**EDORust-OS V1.0 — Experimental Development**

The project is currently in its early development stage.

Current work focuses on establishing the kernel foundation, displaying graphics, retrieving memory map information, and implementing basic input handling.

### Development roadmap

- [x] Establish the initial Rust kernel
- [x] Integrate the Limine boot protocol
- [x] Implement framebuffer-based graphical output
- [x] Add the EDORust-OS startup screen
- [x] Implement initial PS/2 mouse handling
- [ ] Improve memory management
- [ ] Develop a more complete interrupt handling system
- [ ] Expand keyboard and mouse support
- [ ] Improve the graphical desktop
- [ ] Develop a command-line shell
- [ ] Add filesystem support
- [ ] Explore multitasking and process management
- [ ] Improve hardware compatibility

*Roadmap items reflect development goals and do not imply that unfinished features are already implemented.*

## 🎯 Project Goals

EDORust-OS aims to:

1. Explore operating system development using Rust.
2. Understand kernel initialization and hardware interfaces.
3. Build a modular foundation for future operating system components.
4. Experiment with graphical interfaces in a bare-metal environment.
5. Develop practical systems programming skills through hands-on implementation.

## ⚠️ Disclaimer

EDORust-OS is an experimental operating system project intended for educational and development purposes.

It is **not a replacement for Windows, Linux, or other production operating systems**. Features, hardware support, and system stability are limited by the current development stage.

Run experimental builds in a virtual machine whenever possible.

## 👨‍💻 Author

**EDORA STUDIO — Xiao Louie**

An independent operating system development project built through experimentation, persistence, and a passion for low-level programming.

> Every operating system starts somewhere. EDORust-OS starts with Rust.

---

## 👨‍💻 About the Developer

EDORust-OS is independently developed by a **13-year-old programmer** under the name **EDORA STUDIO**.

Driven by a passion for programming, cybersecurity, and low-level systems development, the developer is exploring the fundamentals of operating system engineering using Rust.

From kernel initialization to framebuffer graphics and hardware interaction, EDORust-OS represents an ongoing journey into the world of systems programming.

**Age:** 13  
**Role:** Independent Developer  
**Primary Language:** Rust  
**Project:** EDORust-OS

> Age is just a number. Curiosity builds what comes next. 🦀

---
<p align="center">
  <strong>EDORust-OS</strong>
  <br>
  <sub>Built with Rust. Driven by curiosity. Developed by EDORA STUDIO.</sub>
</p>
