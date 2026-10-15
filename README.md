# SUB-IDE (Pro Edition)

**SUB-IDE** is the next-generation, high-performance intelligent IDE built in pure Rust, designed as part of the **SUB Ecosystem** (`SUB-OS`, `SUB-CPU`, `SUB-AI`, `sub-lang`, `SUB-IDE`). 

It is a clean-room architectural reconstruction of **JetBrains PyCharm Professional Edition**, reverse-engineered from PyCharm binaries and plugins using the **REA (Reverse Engineer Anything)** toolset and **Ghidra**.

---

## 🏛️ Architecture & Subsystems Mapping

### 1. Core Platform & VFS
| Subsystem | PyCharm Source / Binary | SUB-IDE Rust Module | Capabilities |
| :--- | :--- | :--- | :--- |
| **File Watcher** | `bin/fsnotifier` (C/inotify) | [`src/vfs.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/vfs.rs) | In-Memory VFS with `fsnotifier` IPC protocol parser (`ROOTS\n`, `<EVENT>\n<PATH>\n`) and tree sync. |
| **Editor Buffer** | `fleet.andel.jar` | [`src/buffer.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/buffer.rs) | Line-indexed text buffer with multi-cursor support, line/column coordinate conversions, and transactional Undo/Redo. |
| **Indenting Lexer**| `intellij.python.parser.jar` | [`src/lexer.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/lexer.rs) | Tracks indentation stack and bracket nesting (`()`, `[]`, `{}`), emitting synthetic `Indent` and `Dedent` tokens. |
| **PSI / AST** | `intellij.python.psi.jar` | [`src/parser.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/parser.rs) | Recursive-descent AST parser extracting classes, methods, decorators, type annotations, docstrings, and imports. |
| **Stub Index** | `intellij.python.psi.impl.jar` | [`src/stub_index.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/stub_index.rs) | Compact Stub Indexing engine for sub-millisecond "Go to Class" (Ctrl+N), "Go to Symbol", and "Search Everywhere". |
| **Python & SUB SDKs** | `intellij.python.sdk.jar` | [`src/sdk.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/sdk.rs) | Auto-detects Virtualenvs (`.venv`), UV (`uv.lock`), Conda, Poetry, System Python, and `sub-lang` toolchains. |
| **Visual Workbench** | IntelliJ Platform GUI | [`src/workbench.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/workbench.rs) | Visual IDE layout renderer: menu bar, breadcrumbs, project explorer, editor with breakpoints, database panel, tool window dock, and status bar. |

### 2. Professional Edition Features
| Pro Feature | PyCharm Plugin / Jar | SUB-IDE Rust Module | Capabilities |
| :--- | :--- | :--- | :--- |
| **Django & Web Frameworks** | `plugins/python-django` | [`src/django.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/django.rs) | URL routing parser, view target linking, Django ORM model introspection (tables, fields, foreign keys), and FastAPI/Flask route discovery. |
| **Database Tools & SQL** | `plugins/DatabaseTools` | [`src/database.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/database.rs) | Table schema introspection, query executor, tabular result formatting, and visual ER Diagram generation (`DbDiagramProvider`). |
| **Jupyter Notebooks** | `plugins/jupyter-plugin` | [`src/notebook.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/notebook.rs) | Complete `.ipynb` v4 JSON document model, code/markdown cell parsing, execution pipeline, and stream/error output capture. |
| **REST & HTTP Client** | `plugins/restClient` | [`src/http_client.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/http_client.rs) | Parses `.http` / `.rest` scripts (methods, headers, multi-line bodies) and executes HTTP requests. |
| **Docker Environments** | `plugins/clouds-docker-impl` | [`src/docker.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/docker.rs) | Auto-detects `Dockerfile`, `docker-compose.yml`, checks Docker daemon availability, and lists active containers. |
| **Python Debugger (DAP)** | `plugins/python-dap` | [`src/dap.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/dap.rs) | Debug Adapter Protocol client, breakpoint manager, stack frame inspector, and stepping controller (`PythonDapClient`). |
| **Git Version Control** | `plugins/vcs-git` | [`src/vcs_git.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/vcs_git.rs) | Working tree status, commit log parser, branch switcher, and visual ASCII DAG graph. |
| **Data Wrangler** | `plugins/dataWrangler-plugin` | [`src/data_wrangler.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/data_wrangler.rs) | DataFrame profiling, column statistics headers (min/max/mean/median/std/histograms), filters, and reproducible Pandas code generator. |
| **Search Everywhere** | `plugins/searchEverywhereMl` | [`src/search_everywhere.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/search_everywhere.rs) | Double-Shift search popup across Classes, Files, Symbols, Actions, and Text with CamelCase acronym matching. |
| **Built-in MCP Server** | `plugins/mcpserver` | [`src/mcp.rs`](file:///home/subhobhai943/Github/SUB-IDE/src/mcp.rs) | Model Context Protocol JSON-RPC server exposing VFS, symbol search, Django routes, and SQL execution to AI agents. |

---

## 🚀 Running & Testing

### Run the SUB-IDE Interactive Workbench:
```bash
cd /home/subhobhai943/Github/SUB-IDE
cargo run
```

### Run All Unit Tests:
```bash
cargo test
```
