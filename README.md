# 📖 ePub Reader

A **simple, lightweight, and cross-platform e-reader** for `.epub` files. Built with **Tauri**, **Rust**, and **React**, this project’s mission is to provide a clean, secure, and distraction-free reading experience.

- 🦀 **Rust backend** ensures efficiency and memory safety  
- ⚛️ **React frontend** delivers a modern, responsive UI  
- 🔒 Designed with privacy and performance in mind  

---

## ✨ Table of Contents

1. [📥 Installation](#-installation)  
2. [🛠️ Building from Source](#️-building-from-source)  
   - [Prerequisites](#prerequisites)  
   - [Compiling](#compiling)  
3. [🚀 Features](#-features)  
4. [🤝 Contributing](#-contributing)  
5. [📜 License](#-license)

---

## 📥 Installation

The easiest way to get started is to download a pre-built release for your operating system:

1. Visit the [Releases Page](https://github.com/kirankumarthakur/epub-reader/releases).
2. Find the **latest release** marked as “Latest”.
3. Download the appropriate installer under the **Assets** section:

- **Windows**: `.msi` installer  
- **macOS**: `.dmg` image (universal binary for Intel and Apple Silicon)  
- **Linux**: Use either the `.AppImage` (portable) or `.deb` file (for Debian/Ubuntu)

---

## 🛠️ Building from Source

If you're a developer or prefer to build from scratch:

### Prerequisites

Set up the full **Tauri development environment**. Follow the [official Tauri guide](https://tauri.app/v1/guides/getting-started/prerequisites) for platform-specific setup.

You’ll need:

- **Node.js (LTS)**: [Download here](https://nodejs.org/)  
- **Rust** (with Cargo): [Install via rustup](https://rustup.rs/)  
- Required system libraries (WebKit, C compilers, etc. — see Tauri docs)

### Compiling

```bash
# 1. Clone the repository
git clone https://github.com/kirankumarthakur/epub-reader.git

# 2. Enter the project directory
cd epub-reader

# 3. Install frontend dependencies
npm install

# 4. Run the app in development mode (with hot reload)
npm run tauri dev

# 5. Build the production app
npm run tauri build
```

## 🚀 Features

### ✅ Core Functionality

- ✅ Open and render EPUB files (supports EPUB 2)
- ✅ Table of Contents
- ✅ Chapter-based pagination
- ✅ Responsive layout for all screen sizes
- ✅ Remember last position per book

---

## 🧪 In Development / Planned

### 📚 User Experience

- ⬜ Library View: Organize your entire book collection
- ⬜ Light / Dark Mode
- ⬜ n-Book Search: Quickly find text or characters

### ⚙️ Customization

- ⬜ Adjustable font size
- ⬜ Selectable font family
- ⬜ Custom margins and line spacing

---

## 🤝 Contributing

Contributions are welcome! Here's how to get started:

1. Open an Issue to propose a feature or report a bug  
2. Fork the repo, create a new branch, and submit a Pull Request

Please ensure:
- Your code follows the existing style  
- You add appropriate comments/documentation  
