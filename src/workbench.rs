//! SUB-IDE Visual Workbench Renderer.
//! Replicating the full PyCharm Pro IDE GUI/TUI interface:
//! Menu bar, breadcrumbs, project explorer, code editor with syntax highlighting,
//! breakpoint markers, database & structure pane, dockable bottom tool windows
//! (Run, DAP Debugger, Git VCS DAG, Data Wrangler, MCP Server), and status bar.


/// Active bottom dock tool tab
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToolWindowTab {
    Run,
    Debug,
    GitLog,
    DataWrangler,
    Database,
    SearchEverywhere,
}

pub struct WorkbenchRenderer;

impl WorkbenchRenderer {
    /// Renders the complete visual IDE screen
    pub fn render_screen(active_tab: ToolWindowTab) -> String {
        let mut screen = String::new();

        let term_width = 100;
        let line_bar = "─".repeat(term_width);
        let double_line = "═".repeat(term_width);

        // 1. Menu Bar
        screen.push_str(&format!("╔{}╗\n", double_line));
        screen.push_str("║  SUB-IDE Pro 2026.2 (Rust Engine)   File  Edit  View  Navigate  Code  Refactor  Run  Tools  Git  Help  🔍 Double-Shift  ║\n");
        screen.push_str(&format!("╠{}╣\n", double_line));

        // 2. Navigation Bar (Breadcrumbs)
        screen.push_str("║  📁 SUB-IDE > 📂 src > 🐍 models.py > 🏛️ class UserModel > ⚡ def get_display_name()                               ║\n");
        screen.push_str(&format!("╠{}╣\n", line_bar));

        // 3. Main Workspace: 3 Columns [Project Explorer (24) | Editor Buffer (48) | Database & Structure (28)]
        screen.push_str("║ 📂 PROJECT EXPLORER       │ 📄 models.py [UTF-8]                  │ 🗄️ DATABASE & STRUCTURE       ║\n");
        screen.push_str("║ ─────────────────────────┼────────────────────────────────────────┼───────────────────────────────║\n");
        screen.push_str("║ ▼ 📁 SUB-IDE              │ 1   import datetime                    │ ▼ 🏛️ blog_user               ║\n");
        screen.push_str("║   ▼ 📂 src                │ 2                                      │   * 🔑 id (INTEGER PK)        ║\n");
        screen.push_str("║     📄 buffer.rs          │ 3   class UserModel:                   │   * 📧 email (VARCHAR)        ║\n");
        screen.push_str("║     📄 dap.rs             │ 4       \"\"\"Core User entity model\"\"\"   │   * 👤 username (VARCHAR)     ║\n");
        screen.push_str("║     📄 data_wrangler.rs   │ 5       def __init__(self, username):  │ ▼ 🏛️ blog_post               ║\n");
        screen.push_str("║     📄 database.rs        │ 6           self.username = username   │   * 🔑 id (INTEGER PK)        ║\n");
        screen.push_str("║     📄 django.rs          │ 7           self.created_at = None     │   * 🔗 user_id -> blog_user   ║\n");
        screen.push_str("║     📄 lexer.rs           │ 8                                      │   * 📝 title (VARCHAR)        ║\n");
        screen.push_str("║     📄 main.rs            │ 9       def get_display_name(self):    │                               ║\n");
        screen.push_str("║   ▶ 📂 tests              │10🔴         name = self.username.strip()│ ── Foreign Key Relations ──── ║\n");
        screen.push_str("║     📄 Cargo.toml         │11           return f\"User({name})\"     │ blog_post.user_id             ║\n");
        screen.push_str("║     📄 README.md          │12                                      │   └─> blog_user.id            ║\n");
        screen.push_str("║                           │13   def calculate_tax(amount: float):  │                               ║\n");
        screen.push_str("║ [Stub: 14 classes indexed]│14       return amount * 0.18           │ 🟢 DB: sqlite://:memory:      ║\n");

        // 4. Tool Window Splitter
        screen.push_str(&format!("╠{}╣\n", line_bar));

        // 5. Tool Window Header Tabs
        let t_run = if active_tab == ToolWindowTab::Run { "[>_ 1: Run]" } else { " >_ 1: Run " };
        let t_dbg = if active_tab == ToolWindowTab::Debug { "[🐛 2: Debug (DAP)]" } else { " 🐛 2: Debug (DAP) " };
        let t_git = if active_tab == ToolWindowTab::GitLog { "[🌿 3: Git Log]" } else { " 🌿 3: Git Log " };
        let t_wrg = if active_tab == ToolWindowTab::DataWrangler { "[📊 4: Data Wrangler]" } else { " 📊 4: Data Wrangler " };
        let t_sch = if active_tab == ToolWindowTab::SearchEverywhere { "[🔍 5: Search Everywhere]" } else { " 🔍 5: Search Everywhere " };

        screen.push_str(&format!("║ {} {} {} {} {} ║\n", t_run, t_dbg, t_git, t_wrg, t_sch));
        screen.push_str(&format!("╠{}╣\n", line_bar));

        // 6. Tool Window Content Panel based on active_tab
        match active_tab {
            ToolWindowTab::Debug => {
                screen.push_str("║ 🔴 PAUSED at models.py:10 in get_display_name() [Thread #1: MainThread]                          ║\n");
                screen.push_str("║ ── Variables ──────────────────────────┬── Frames ───────────────────────────────────────────────║\n");
                screen.push_str("║ * self: <UserModel object at 0x7fa2>   │ [0] get_display_name() at models.py:10                   ║\n");
                screen.push_str("║ * self.username: 'subho_admin' (str)   │ [1] render_profile_view() at views.py:42                 ║\n");
                screen.push_str("║ * name: 'subho_admin' (str)            │ [2] dispatch_request() at router.py:88                   ║\n");
                screen.push_str("║ * rate: 3.14159 (float)                │ [3] main() at main.py:15                                 ║\n");
                screen.push_str("║                                        │ Controls: [F8: Step Over] [F7: Step In] [F9: Resume]    ║\n");
            }
            ToolWindowTab::DataWrangler => {
                screen.push_str("║ DataFrame Profiler: [5 rows x 4 columns] | Source: customer_metrics.csv                          ║\n");
                screen.push_str("║ [age]      total: 5, missing: 20%, min: 24, max: 45, mean: 32.5  hist: [24-29:2 29-34:1 40-45:1] ║\n");
                screen.push_str("║ [purchase] total: 5, missing: 0%,  min: 45, max: 320, mean: 159.1 hist: [45-114:2 182-320:2]     ║\n");
                screen.push_str("║ ── Data Rows ────────────────────────────────────────────────────────────────────────────────────║\n");
                screen.push_str("║ 1: Alice   | age: 32 | purchase: $150.50 | VIP: true                                            ║\n");
                screen.push_str("║ 2: Bob     | age: 24 | purchase:  $45.00 | VIP: false                                           ║\n");
                screen.push_str("║ 3: Charlie | age: 45 | purchase: $320.00 | VIP: true                                            ║\n");
            }
            ToolWindowTab::GitLog => {
                screen.push_str("║ Git Log & Branch History (HEAD -> master)                                                        ║\n");
                screen.push_str("║ * (HEAD) [130c008] Initial pure Rust implementation of PyCharm Pro - subhobhai943 (2026-10-15)    ║\n");
                screen.push_str("║ |                                                                                                ║\n");
                screen.push_str("║ *        [6b00bc3] Implement Rust launcher, DAP, and fsnotifier - JetBrains Team (2026-10-15)    ║\n");
                screen.push_str("║ |                                                                                                ║\n");
                screen.push_str("║ *        [25ef9e3] Initial project structure & AST lexer - JetBrains Dev (2026-10-14)            ║\n");
            }
            ToolWindowTab::SearchEverywhere => {
                screen.push_str("║ Search Everywhere [All Tabs] Query: \"user\"                                                      ║\n");
                screen.push_str("║ 1. [Class ] UserModel             src/models.py:3                                                ║\n");
                screen.push_str("║ 2. [Func  ] get_user_by_id        src/views.py:18                                                ║\n");
                screen.push_str("║ 3. [Action] User Settings...      Preferences                                (Ctrl+Alt+S)        ║\n");
                screen.push_str("║ 4. [File  ] user_guide.md         docs/user_guide.md                                             ║\n");
            }
            ToolWindowTab::Run => {
                screen.push_str("║ /home/subhobhai943/Github/SUB-IDE/.venv/bin/python3 main.py                                      ║\n");
                screen.push_str("║ >> [SUB-IDE Server] Listening on 127.0.0.1:8000 (Reload active)                                 ║\n");
                screen.push_str("║ >> [Database] Connected to sqlite://:memory: (2 tables synchronized)                             ║\n");
                screen.push_str("║ >> [MCP Server] Ready: 5 tools available for AI Assistant pair programming                       ║\n");
                screen.push_str("║ Process finished with exit code 0                                                                ║\n");
            }
            ToolWindowTab::Database => {
                screen.push_str("║ Database Console: sqlite://:memory:                                                              ║\n");
                screen.push_str("║ SELECT * FROM blog_user;                                                                         ║\n");
                screen.push_str("║ id | username     | email                                                                        ║\n");
                screen.push_str("║ 1  | subho_admin  | subho@sub.org                                                                ║\n");
                screen.push_str("║ 2  | test_user    | test@sub.org                                                                 ║\n");
            }
        }

        // 7. Status Bar
        screen.push_str(&format!("╠{}╣\n", line_bar));
        screen.push_str("║  🌿 master  │ 🐍 Python 3.12 (UV / venv) │ Ln 10, Col 8 │ 4 spaces │ UTF-8 │ 🟢 MCP Agent: Active  ║\n");
        screen.push_str(&format!("╚{}╝\n", double_line));

        screen
    }
}
