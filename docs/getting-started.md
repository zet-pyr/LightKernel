# Getting Started with LightKernel

Welcome to LightKernel! This guide will help you get up and running with kernel development, whether you're a complete beginner or an experienced developer new to this project.

## 📋 Table of Contents

- [System Requirements](#-system-requirements)
- [Development Environment Setup](#-development-environment-setup)
- [Building Your First Kernel](#-building-your-first-kernel)
- [Running and Testing](#-running-and-testing)
- [Understanding the Codebase](#-understanding-the-codebase)
- [Making Your First Changes](#-making-your-first-changes)
- [Debugging and Troubleshooting](#-debugging-and-troubleshooting)
- [Next Steps](#-next-steps)

## 🖥️ System Requirements

### Supported Host Systems
- **Linux** (Ubuntu 20.04+, Debian 11+, Fedora 35+, Arch Linux)
- **macOS** (10.15+ with Homebrew)
- **Windows** (WSL2/Ubuntu recommended)

### Hardware Requirements
- **RAM**: Minimum 4GB (8GB+ recommended)
- **Storage**: At least 2GB free space for tools and source
- **CPU**: x64 processor (ARM64 support coming soon)

## 🛠️ Development Environment Setup

### Step 1: Install Dependencies

#### Ubuntu/Debian
```bash
# Update package list
sudo apt update

# Install essential build tools
sudo apt install -y \
    build-essential \
    nasm \
    qemu-system-x86 \
    qemu-system-arm \
    git \
    gdb \
    hexdump \
    tree

# Install additional development tools (optional but recommended)
sudo apt install -y \
    vim \
    emacs \
    code \
    clang \
    lldb
```

#### Fedora/RHEL/CentOS
```bash
# Install development tools
sudo dnf groupinstall "Development Tools"
sudo dnf install -y \
    nasm \
    qemu-system-x86 \
    qemu-system-arm \
    git \
    gdb \
    hexdump \
    tree
```

#### macOS
```bash
# Install Homebrew if not already installed
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# Install dependencies
brew install \
    nasm \
    qemu \
    git \
    gdb \
    tree \
    llvm
```

#### Windows (WSL2)
```bash
# Install WSL2 with Ubuntu, then follow Ubuntu instructions above
# Also install Windows Terminal for better experience
```

### Step 2: Verify Installation

```bash
# Check GCC version (should be 7.0+)
gcc --version

# Check NASM version
nasm -version

# Check QEMU installation
qemu-system-x86_64 --version

# Check Git
git --version
```

## 🚀 Building Your First Kernel

### Step 1: Clone the Repository

```bash
# Clone LightKernel
git clone https://github.com/zet-pyr/LightKernel.git
cd LightKernel

# Check out the stable branch (recommended for beginners)
git checkout main
```

### Step 2: Explore the Project Structure

```bash
# Get an overview of the project
tree -L 2

# Expected structure:
# ├── src/           # Kernel source code
# ├── include/       # Header files
# ├── scripts/       # Build and utility scripts
# ├── docs/          # Documentation
# ├── tests/         # Test suite
# ├── tools/         # Development tools
# ├── boot/          # Boot loader code
# └── Makefile       # Main build configuration
```

### Step 3: Build the Kernel

```bash
# Make sure you're in the project root
cd LightKernel

# Build the kernel (this may take 2-5 minutes on first run)
make clean && make all

# Alternative: Use the build script if available
./scripts/build.sh

# Check if build was successful
ls -la build/
# You should see: lightkernel.bin, lightkernel.iso, or similar files
```

### Build Configuration Options

```bash
# Debug build (with symbols and debugging info)
make DEBUG=1

# Release build (optimized)
make RELEASE=1

# Build for specific architecture
make ARCH=x86_64    # Default
make ARCH=arm64     # ARM 64-bit
make ARCH=riscv64   # RISC-V 64-bit

# Verbose build (show all commands)
make VERBOSE=1
```

## 🔧 Running and Testing

### Step 1: Run in QEMU

```bash
# Run the kernel in QEMU
make run

# Alternative: Use the run script
./scripts/run.sh

# Run with specific options
make run QEMU_OPTS="-m 512M -smp 2"  # 512MB RAM, 2 CPUs
```

### Step 2: QEMU Controls

When LightKernel boots in QEMU, you can use these controls:
- **Ctrl+Alt+G**: Release mouse from QEMU window
- **Ctrl+Alt+2**: Switch to QEMU monitor console
- **Ctrl+Alt+1**: Switch back to kernel console
- **Ctrl+A, X**: Quit QEMU (in text mode)

### Step 3: Expected Output

A successful boot should show something like:
```
LightKernel v0.1.0 - Community Edition
Initializing memory management...
Setting up interrupts...
Starting scheduler...
Welcome to LightKernel!
lightkernel> _
```

### Testing with Different Configurations

```bash
# Test on different architectures
make run ARCH=x86_64
make run ARCH=arm64

# Test with debugging enabled
make run DEBUG=1

# Run automated tests
make test
./scripts/run_tests.sh
```

## 📚 Understanding the Codebase

### Key Directories and Files

```
src/
├── kernel/         # Core kernel functionality
│   ├── main.c      # Kernel entry point
│   ├── memory.c    # Memory management
│   ├── scheduler.c # Process scheduling
│   └── interrupt.c # Interrupt handling
├── drivers/        # Device drivers
│   ├── keyboard.c  # Keyboard driver
│   ├── serial.c    # Serial port driver
│   └── timer.c     # Timer driver
├── lib/           # Kernel library functions
│   ├── string.c   # String operations
│   ├── printf.c   # Formatted output
│   └── math.c     # Math utilities
└── boot/          # Boot sequence code
    ├── boot.s     # Assembly boot code
    └── loader.c   # C boot loader
```

### Code Style and Conventions

LightKernel follows these conventions:
- **Indentation**: 4 spaces (no tabs)
- **Naming**: `snake_case` for functions and variables
- **Comments**: Detailed comments for complex logic
- **Headers**: All functions documented in header files

### Start Reading Here

1. **`src/kernel/main.c`** - The kernel's main entry point
2. **`include/kernel.h`** - Main kernel header with key definitions
3. **`docs/architecture.md`** - High-level system overview
4. **`src/boot/boot.s`** - Assembly code that starts everything

## ✏️ Making Your First Changes

### Beginner Track
1. **Complete the basic challenges** above
2. **Read the architecture documentation** in `docs/`
3. **Explore existing drivers** in `src/drivers/`
4. **Join the community discussions** on GitHub

### Intermediate Track
1. **Implement a new system call**
2. **Write a simple device driver**
3. **Add support for a new file system**
4. **Optimize memory allocation**

### Advanced Track
1. **Port to a new architecture** (ARM64, RISC-V)
2. **Implement SMP (multi-core) support**
3. **Add networking capabilities**
4. **Design a new scheduling algorithm**

### Learning Resources

- **Books**:
  - "Operating System Concepts" by Silberschatz
  - "Modern Operating Systems" by Tanenbaum
  - "The Design and Implementation of the FreeBSD Operating System"

- **Online Resources**:
  - [OSDev Wiki](https://wiki.osdev.org/) - Comprehensive OS development resource
  - [Intel Software Developer Manuals](https://software.intel.com/content/www/us/en/develop/articles/intel-sdm.html)
  - [LightKernel Documentation](docs/) - Project-specific guides

- **Community**:
  - [GitHub Discussions](https://github.com/zet-pyr/LightKernel/discussions)
  - [Issues and Bug Reports](https://github.com/zet-pyr/LightKernel/issues)
  - [Contributing Guidelines](../CONTRIBUTING.md)

## 🎉 You're Ready!

Congratulations! You now have LightKernel running on your system and understand the basics of the development workflow. The kernel world is vast and exciting – start small, experiment fearlessly, and don't hesitate to ask questions in our community.

Remember: Every expert was once a beginner. Welcome to the LightKernel family! 🚀

---

**Need help?** 
- 📖 Check the [FAQ](docs/faq.md)
- 💬 Join our [discussions](https://github.com/zet-pyr/LightKernel/discussions)
- 🐛 Report issues on [GitHub](https://github.com/zet-pyr/LightKernel/issues)