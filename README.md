# LightKernel

<p align="center">
  <img src="assets/logo/logo.png" alt="LightKernel Logo" width="200"/>
</p>

<p align="center">
  <strong>When minds unite, even the smallest light can guide the world.</strong>
</p>

<p align="center">
  <a href="#-features">Features</a> •
  <a href="#-quick-start">Quick Start</a> •
  <a href="#-documentation">Documentation</a> •
  <a href="#-contributing">Contributing</a> •
  <a href="#-community">Community</a>
</p>

---

## 🌟 What is LightKernel?

LightKernel is a **community-driven, educational kernel project** designed to democratize operating system development. We believe that kernel programming shouldn't be an exclusive domain – it should be accessible to anyone curious about how computers work at their core.

Our mission is to create a **simple, well-documented, and extensible** operating system kernel that serves as both a learning platform and a foundation for innovation.

## ✨ Features

### 🏗️ Architecture
- **Modular design** - Clean separation of concerns with pluggable components
- **Multi-platform support** - x86_64, ARM64, RISC-V (more platforms coming!)
- **Microkernel approach** - Minimal kernel with userspace services

### 📚 Developer Experience
- **Crystal-clear documentation** - Every function, every concept explained
- **Interactive tutorials** - Step-by-step guides for kernel development
- **Extensive testing suite** - Automated testing for reliability
- **Modern toolchain** - Support for latest compilers and debugging tools

### 🎯 Learning Focus
- **Educational by design** - Code optimized for understanding, not just performance
- **Progressive complexity** - Start simple, gradually explore advanced topics
- **Real-world examples** - Practical implementations of OS concepts

## 🚀 Quick Start

### Prerequisites
- GCC or Clang compiler
- NASM assembler
- QEMU (for testing)
- Git

### Installation

```bash
# Clone the repository
git clone https://github.com/zet-pyr/LightKernel.git
cd LightKernel

# Install dependencies (Ubuntu/Debian)
sudo apt update
sudo apt install build-essential nasm qemu-system-x86 git

# Build the kernel
./scripts/build.sh

# Run in QEMU
./scripts/run.sh
```

### Your First Boot
After running the commands above, you should see LightKernel boot in QEMU! 🎉

## 📖 Documentation

| Resource | Description |
|----------|-------------|
| [**Getting Started Guide**](docs/getting-started.md) | Your first steps with LightKernel |
| [**Architecture Overview**](docs/architecture.md) | High-level system design |
| [**API Reference**](docs/api/) | Complete function documentation |
| [**Tutorials**](docs/tutorials/) | Step-by-step learning modules |
| [**Development Guide**](docs/development.md) | Advanced development topics |

## 🤝 Contributing

We welcome contributors of **all skill levels**! Whether you're a seasoned kernel developer or just starting your journey in systems programming, there's a place for you here.

### How to Contribute

1. **🔍 Explore** - Browse our [good first issues](https://github.com/zet-pyr/LightKernel/labels/good%20first%20issue)
2. **📋 Plan** - Read our [Contributing Guidelines](CONTRIBUTING.md)
3. **💻 Code** - Make your changes and test thoroughly
4. **📤 Submit** - Create a pull request with clear description
5. **🏆 Celebrate** - Add yourself to [CREDITS/USERS/](CREDITS/USERS/) following the [template](CREDITS/model.md)

### Contribution Areas

- **🔧 Core Development** - Kernel features, drivers, system calls
- **📝 Documentation** - Tutorials, guides, code comments
- **🧪 Testing** - Unit tests, integration tests, bug reports
- **🎨 Tooling** - Build scripts, debugging tools, utilities
- **🌍 Outreach** - Blog posts, presentations, community building

## 🏆 Project Stats

![GitHub stars](https://img.shields.io/github/stars/zet-pyr/LightKernel?style=social)
![GitHub forks](https://img.shields.io/github/forks/zet-pyr/LightKernel?style=social)
![GitHub issues](https://img.shields.io/github/issues/zet-pyr/LightKernel)
![GitHub pull requests](https://img.shields.io/github/issues-pr/zet-pyr/LightKernel)

## 🌐 Community

Join our vibrant community of kernel enthusiasts!

- **💬 Discussions** - [GitHub Discussions](https://github.com/zet-pyr/LightKernel/discussions)
- **🐛 Issues** - [Report bugs](https://github.com/zet-pyr/LightKernel/issues/new?template=bug_report.md)
- **💡 Feature Requests** - [Suggest features](https://github.com/zet-pyr/LightKernel/issues/new?template=feature_request.md)
- **📧 Mailing List** - [kernel-dev@lightkernel.org](mailto:kernel-dev@lightkernel.org)

## 🎯 Roadmap

### Phase 1: Foundation (Current)
- [x] Basic boot process
- [x] Memory management
- [ ] Process management
- [ ] Basic I/O system

### Phase 2: Expansion
- [ ] Networking stack
- [ ] Filesystem support
- [ ] GUI subsystem
- [ ] Package manager

### Phase 3: Innovation
- [ ] Container support
- [ ] WebAssembly runtime
- [ ] AI acceleration
- [ ] Edge computing features

## 🙏 Acknowledgements

LightKernel exists because of our amazing community. Every contributor, from first-time committers to veteran developers, plays a vital role in our success.

**Special thanks to all our contributors:**
- [View all contributors](https://github.com/zet-pyr/LightKernel/graphs/contributors)
- [Community credits](CREDITS/USERS/)

## 📜 License

This project is licensed under the **Community Open Source License**. See the [LICENSE](LICENSE) file for complete details.

---

<p align="center">
  <strong>🚀 Ready to dive into kernel development? <a href="docs/getting-started.md">Start your journey here!</a></strong>
</p>

<p align="center">
  <em>LightKernel is powered by the community. Join us and help light the way! ✨</em>
</p>