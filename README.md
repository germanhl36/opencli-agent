# OpenCLI Agent

An open-source, cross-platform desktop AI coding assistant built with Tauri 2, Rust, and React/TypeScript.

Chat with local or hosted LLMs, get file edits proposed with a diff preview, approve or reject every action, and keep a full undo history — all in a native desktop app.

---

## Features

- **Multi-provider LLM support**: Ollama (local), OpenRouter, HuggingFace, or any OpenAI-compatible endpoint
- **Approval gate**: every file write, delete, or shell command requires explicit approval before execution
- **Diff viewer**: unified diff shown before any file change is applied
- **Undo stack**: reverse any file change made during a session
- **Plugin system**: skills (prompt templates) and agents (multi-step workflows) defined in YAML
- **Audit log**: append-only NDJSON log of every approved/rejected action
- **API keys in OS keychain**: keys never stored in config files or sent to the frontend

---

## Prerequisites

### System tools

| Tool | Version | Install |
|------|---------|---------|
| **Node.js** | 18+ | [nodejs.org](https://nodejs.org) |
| **npm** | 9+ | Bundled with Node.js |
| **Rust + Cargo** | 1.77+ | [rustup.rs](https://rustup.rs) |
| **Xcode Command Line Tools** *(macOS)* | latest | `xcode-select --install` |
| **WebKit2GTK + build tools** *(Linux)* | see below | see below |
| **WebView2** *(Windows)* | latest | Bundled in installer; [manual install](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) if needed |

### macOS

```bash
xcode-select --install
```

### Linux (Debian/Ubuntu)

```bash
sudo apt update
sudo apt install -y \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libssl-dev \
  libgtk-3-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev
```

### Windows

Install [Visual Studio Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) with the **Desktop development with C++** workload, then install Rust via [rustup.rs](https://rustup.rs).

### Rust PATH (if `cargo` is not on your PATH)

After installing Rust, add Cargo to your shell profile:

```bash
# ~/.zshrc or ~/.bashrc
export PATH="$HOME/.cargo/bin:$PATH"
```

Then reload: `source ~/.zshrc`

---

## AI Provider Setup

At least one provider is required to use the chat features.

### Ollama (recommended — free, local, no API key)

1. Install from [ollama.com](https://ollama.com)
2. Pull a model: `ollama pull llama3.2`
3. Start the server: `ollama serve`

Ollama is selected by default. No further configuration needed.

### OpenRouter

1. Sign up at [openrouter.ai](https://openrouter.ai) and get an API key
2. Launch the app → open **Settings** → enter your OpenRouter API key
3. Select **OpenRouter** as the provider and pick a model

### HuggingFace Inference API

1. Get an API key from [huggingface.co/settings/tokens](https://huggingface.co/settings/tokens)
2. Launch the app → open **Settings** → enter your HuggingFace API key
3. Select **HuggingFace** as the provider

### Custom OpenAI-compatible endpoint

Enter your base URL and API key in Settings. Works with any server that supports the `/chat/completions` endpoint (e.g. LM Studio, vLLM, LocalAI).

---

## Getting Started

```bash
# 1. Clone the repo
git clone https://github.com/your-org/opencli-agent.git
cd opencli-agent

# 2. Install frontend dependencies
npm install

# 3. Start in development mode
npm run tauri dev
```

The app window will open. On first launch it uses Ollama with `llama3.2` by default.

---

## Building a Distributable

```bash
npm run tauri build
```

Output locations:
- **macOS**: `src-tauri/target/release/bundle/dmg/*.dmg`
- **Linux**: `src-tauri/target/release/bundle/deb/*.deb` and `appimage/*.AppImage`
- **Windows**: `src-tauri/target/release/bundle/msi/*.msi` and `nsis/*.exe`

---

## Project Structure

```
opencli-agent/
├── src/                        # React/TypeScript frontend
│   ├── components/             # ChatPanel, DiffViewer, ApprovalDialog, ...
│   ├── hooks/                  # useSession, useStream, useApproval
│   └── lib/                    # types.ts, tauri.ts (typed IPC wrappers)
├── src-tauri/                  # Rust backend
│   └── src/
│       ├── commands/           # Tauri IPC command handlers
│       ├── config/             # Config loading, keychain
│       ├── core/               # Session, context builder, parser, approval gate
│       ├── llm/                # Provider trait + Ollama/OpenRouter/HF/Custom drivers
│       ├── runtime/            # Diff engine, FS executor, shell, undo, audit
│       └── plugins/            # Skills, agents, custom commands
├── skills/                     # Built-in skill YAML files
└── agents/                     # Built-in agent YAML files
```

---

## IDE Setup

- [VS Code](https://code.visualstudio.com/) with:
  - [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode)
  - [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)
