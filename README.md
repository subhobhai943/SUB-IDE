# PyCharm-RS: Pure Rust IDE with Full Pro Capabilities

This project is a high-performance, modular re-implementation of the complete architecture and features of **JetBrains PyCharm Professional Edition**, reverse-engineered from PyCharm binaries and JAR packages using the **REA (Reverse Engineer Anything)** toolset and **Ghidra**.

---

## 🏛️ Architecture & Pro Subsystems Mapping

### 1. Core Platform & VFS
| Subsystem | PyCharm Source / Binary | PyCharm-RS Rust Module | Capabilities |
| :--- | :--- | :--- | :--- |
| **File Watcher** | `bin/fsnotifier` (C/inotify) | [`src/vfs.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/vfs.rs) | In-Memory VFS with `fsnotifier` IPC protocol parser (`ROOTS\n`, `<EVENT>\n<PATH>\n`) and tree sync. |
| **Editor Buffer** | `fleet.andel.jar` | [`src/buffer.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/buffer.rs) | Line-indexed text buffer with multi-cursor support, line/column coordinate conversions, and transactional Undo/Redo. |
| **Indenting Lexer**| `intellij.python.parser.jar` | [`src/lexer.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/lexer.rs) | Tracks indentation stack and bracket nesting (`()`, `[]`, `{}`), emitting synthetic `Indent` and `Dedent` tokens. |
| **PSI / AST** | `intellij.python.psi.jar` | [`src/parser.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/parser.rs) | Recursive-descent AST parser extracting classes, methods, decorators, type annotations, docstrings, and imports. |
| **Stub Index** | `intellij.python.psi.impl.jar` | [`src/stub_index.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/stub_index.rs) | Compact Stub Indexing engine for sub-millisecond "Go to Class" (Ctrl+N), "Go to Symbol", and "Search Everywhere". |
| **Python SDKs** | `intellij.python.sdk.jar` | [`src/sdk.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/sdk.rs) | Auto-detects Virtualenvs (`.venv`), UV (`uv.lock`), Conda, Poetry, and System Python. |

### 2. PyCharm Professional Edition Features
| Pro Feature | PyCharm Plugin / Jar | PyCharm-RS Rust Module | Capabilities |
| :--- | :--- | :--- | :--- |
| **Django & Web Frameworks** | `plugins/python-django` | [`src/django.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/django.rs) | URL routing parser, view target linking, Django ORM model introspection (tables, fields, foreign keys), and FastAPI/Flask route discovery. |
| **Database Tools & SQL** | `plugins/DatabaseTools` | [`src/database.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/database.rs) | Table schema introspection, query executor, tabular result formatting, and visual ER Diagram generation (`DbDiagramProvider`). |
| **Jupyter Notebooks** | `plugins/jupyter-plugin` | [`src/notebook.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/notebook.rs) | Complete `.ipynb` v4 JSON document model, code/markdown cell parsing, execution pipeline, and stream/error output capture. |
| **REST & HTTP Client** | `plugins/restClient` | [`src/http_client.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/http_client.rs) | Parses `.http` / `.rest` scripts (methods, headers, multi-line bodies) and executes HTTP requests. |
| **Docker Environments** | `plugins/clouds-docker-impl` | [`src/docker.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/docker.rs) | Auto-detects `Dockerfile`, `docker-compose.yml`, checks Docker daemon availability, and lists active containers. |
| **Built-in MCP Server** | `plugins/mcpserver` | [`src/mcp.rs`](file:///home/subhobhai943/Github/pycharm-rs/src/mcp.rs) | Model Context Protocol JSON-RPC server exposing VFS, symbol search, Django routes, and SQL execution to AI agents. |

---

## 🚀 Running & Testing

### Run the IDE Engine Demonstration:
```bash
cd /home/subhobhai943/Github/pycharm-rs
cargo run
```

### Run All Unit Tests:
```bash
cargo test
```
