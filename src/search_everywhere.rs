//! Search Everywhere engine (PyCharm Double-Shift).
//! Reverse engineered from JetBrains `searchEverywhereMl.jar` and IntelliJ Platform
//! (`SearchEverywhereContributor`, `ClassSearchEverywhereContributor`,
//! `ActionSearchEverywhereContributor`, `FileSearchEverywhereContributor`).
//!
//! Provides unified multi-contributor searching across Classes, Files, Symbols,
//! IDE Actions, and Text with CamelCase acronym and fuzzy ranking.

use serde::{Deserialize, Serialize};

/// Search tabs in the Double-Shift popup
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SearchTab {
    All,
    Classes,
    Files,
    Symbols,
    Actions,
    Text,
}

/// Item category for badges/icons
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ItemCategory {
    Class,
    Function,
    Method,
    File,
    Action,
    TextMatch,
}

/// A search item candidate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchItem {
    pub title: String,
    pub location: String,
    pub category: ItemCategory,
    pub shortcut: Option<String>,
}

/// Ranked search match
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResult {
    pub item: SearchItem,
    pub score: i32,
    pub match_tab: SearchTab,
}

/// Pure Rust Search Everywhere engine
pub struct SearchEverywhere {
    classes: Vec<SearchItem>,
    files: Vec<SearchItem>,
    symbols: Vec<SearchItem>,
    actions: Vec<SearchItem>,
    text_entries: Vec<SearchItem>,
}

impl SearchEverywhere {
    pub fn new() -> Self {
        let mut se = Self {
            classes: Vec::new(),
            files: Vec::new(),
            symbols: Vec::new(),
            actions: Vec::new(),
            text_entries: Vec::new(),
        };
        se.register_default_actions();
        se
    }

    /// Registers built-in IDE actions (like in PyCharm Actions tab)
    fn register_default_actions(&mut self) {
        let actions = vec![
            ("Run Application", "Run the active Python file", "Shift+F10"),
            (
                "Debug Application",
                "Start Python DAP debugger session",
                "Shift+F9",
            ),
            ("Git: Commit...", "Open Git Commit tool window", "Ctrl+K"),
            (
                "Git: Push...",
                "Push current branch to origin",
                "Ctrl+Shift+K",
            ),
            (
                "Git: Update Project",
                "Pull and rebase upstream commits",
                "Ctrl+T",
            ),
            (
                "Database: New Query Console",
                "Open SQL Console with ER diagram",
                "Ctrl+Shift+Q",
            ),
            (
                "Data Wrangler: Profile Table",
                "Inspect DataFrame statistics & histogram",
                "Alt+Shift+W",
            ),
            (
                "Reformat Code (Black/Ruff)",
                "Format code according to PEP 8",
                "Ctrl+Alt+L",
            ),
            (
                "Optimize Imports",
                "Sort and prune unused Python imports",
                "Ctrl+Alt+O",
            ),
            (
                "Terminal: Open New Session",
                "Open embedded interactive shell",
                "Alt+F12",
            ),
            ("Search Everywhere", "Double Shift popup", "Shift Shift"),
        ];

        for (title, loc, sc) in actions {
            self.actions.push(SearchItem {
                title: title.to_string(),
                location: loc.to_string(),
                category: ItemCategory::Action,
                shortcut: Some(sc.to_string()),
            });
        }
    }

    pub fn add_class(&mut self, name: &str, file: &str, line: usize) {
        self.classes.push(SearchItem {
            title: name.to_string(),
            location: format!("{}:{}", file, line),
            category: ItemCategory::Class,
            shortcut: None,
        });
    }

    pub fn add_file(&mut self, path: &str) {
        self.files.push(SearchItem {
            title: path.split('/').next_back().unwrap_or(path).to_string(),
            location: path.to_string(),
            category: ItemCategory::File,
            shortcut: None,
        });
    }

    pub fn add_symbol(&mut self, name: &str, file: &str, line: usize, is_method: bool) {
        self.symbols.push(SearchItem {
            title: name.to_string(),
            location: format!("{}:{}", file, line),
            category: if is_method {
                ItemCategory::Method
            } else {
                ItemCategory::Function
            },
            shortcut: None,
        });
    }

    pub fn add_text_match(&mut self, preview: &str, file: &str, line: usize) {
        self.text_entries.push(SearchItem {
            title: preview.to_string(),
            location: format!("{}:{}", file, line),
            category: ItemCategory::TextMatch,
            shortcut: None,
        });
    }

    /// Computes match score: Exact = 100, Prefix = 80, CamelCase acronym = 70, Substring = 50, Fuzzy = 20
    pub fn compute_score(query: &str, target: &str) -> Option<i32> {
        let q = query.trim().to_lowercase();
        let t = target.to_lowercase();

        if q.is_empty() {
            return Some(0);
        }

        // Exact match
        if t == q {
            return Some(100);
        }

        // Prefix match
        if t.starts_with(&q) {
            return Some(85);
        }

        // CamelCase acronym match (e.g. "UM" -> "UserModel", "cdwe" -> "CoreDataWranglerEngine")
        let acronym: String = target
            .chars()
            .filter(|c| c.is_uppercase() || *c == '_')
            .filter(|c| *c != '_')
            .collect::<String>()
            .to_lowercase();

        if !acronym.is_empty() && acronym.starts_with(&q) {
            return Some(75);
        }

        // Word boundary match (e.g. "data_wrangler" matching "wrangler")
        if target
            .split('_')
            .any(|part| part.to_lowercase().starts_with(&q))
        {
            return Some(65);
        }

        // Substring match
        if t.contains(&q) {
            return Some(50);
        }

        // Subsequence fuzzy match
        let mut q_chars = q.chars();
        let mut curr = q_chars.next();
        for ch in t.chars() {
            if let Some(target_ch) = curr {
                if ch == target_ch {
                    curr = q_chars.next();
                }
            }
        }
        if curr.is_none() {
            return Some(20);
        }

        None
    }

    /// Queries the search engine for a given tab
    pub fn search(&self, query: &str, tab: SearchTab, max_results: usize) -> Vec<SearchResult> {
        let mut candidates = Vec::new();

        let should_search_classes = tab == SearchTab::All || tab == SearchTab::Classes;
        let should_search_files = tab == SearchTab::All || tab == SearchTab::Files;
        let should_search_symbols = tab == SearchTab::All || tab == SearchTab::Symbols;
        let should_search_actions = tab == SearchTab::All || tab == SearchTab::Actions;
        let should_search_text = tab == SearchTab::All || tab == SearchTab::Text;

        if should_search_classes {
            for item in &self.classes {
                if let Some(score) = Self::compute_score(query, &item.title) {
                    candidates.push(SearchResult {
                        item: item.clone(),
                        score,
                        match_tab: SearchTab::Classes,
                    });
                }
            }
        }

        if should_search_files {
            for item in &self.files {
                let score_title = Self::compute_score(query, &item.title);
                let score_loc = Self::compute_score(query, &item.location);
                if let Some(score) = score_title.max(score_loc) {
                    candidates.push(SearchResult {
                        item: item.clone(),
                        score,
                        match_tab: SearchTab::Files,
                    });
                }
            }
        }

        if should_search_symbols {
            for item in &self.symbols {
                if let Some(score) = Self::compute_score(query, &item.title) {
                    candidates.push(SearchResult {
                        item: item.clone(),
                        score,
                        match_tab: SearchTab::Symbols,
                    });
                }
            }
        }

        if should_search_actions {
            for item in &self.actions {
                let score = Self::compute_score(query, &item.title);
                if let Some(s) = score {
                    candidates.push(SearchResult {
                        item: item.clone(),
                        score: s,
                        match_tab: SearchTab::Actions,
                    });
                }
            }
        }

        if should_search_text {
            for item in &self.text_entries {
                if let Some(score) = Self::compute_score(query, &item.title) {
                    candidates.push(SearchResult {
                        item: item.clone(),
                        score: score - 10, // Text matches get slight lower precedence than symbols
                        match_tab: SearchTab::Text,
                    });
                }
            }
        }

        // Sort descending by score
        candidates.sort_by_key(|r| std::cmp::Reverse(r.score));
        candidates.truncate(max_results);
        candidates
    }

    /// Renders visual popup (PyCharm Shift-Shift Search Everywhere)
    pub fn render_popup(&self, query: &str, tab: SearchTab, results: &[SearchResult]) -> String {
        let mut out = String::new();
        let tab_name = match tab {
            SearchTab::All => "[All]",
            SearchTab::Classes => "[Classes]",
            SearchTab::Files => "[Files]",
            SearchTab::Symbols => "[Symbols]",
            SearchTab::Actions => "[Actions]",
            SearchTab::Text => "[Text]",
        };
        out.push_str("┌─── Search Everywhere (Double-Shift) ────────────────────────────────┐\n");
        out.push_str(&format!("│ Tab: {:<12} Query: \"{}\"\n", tab_name, query));
        out.push_str("├─────────────────────────────────────────────────────────────────────┤\n");
        if results.is_empty() {
            out.push_str(
                "│ No results found.                                                   │\n",
            );
        } else {
            for (idx, r) in results.iter().enumerate() {
                let cat_badge = match r.item.category {
                    ItemCategory::Class => "Class ",
                    ItemCategory::Function => "Func  ",
                    ItemCategory::Method => "Method",
                    ItemCategory::File => "File  ",
                    ItemCategory::Action => "Action",
                    ItemCategory::TextMatch => "Text  ",
                };
                let shortcut_str = r
                    .item
                    .shortcut
                    .as_ref()
                    .map(|s| format!(" ({})", s))
                    .unwrap_or_default();
                out.push_str(&format!(
                    "│ {:>2}. [{}] {:<24} {:<24}{}\n",
                    idx + 1,
                    cat_badge,
                    r.item.title,
                    r.item.location,
                    shortcut_str
                ));
            }
        }
        out.push_str("└─────────────────────────────────────────────────────────────────────┘\n");
        out
    }
}

impl Default for SearchEverywhere {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_search_scoring_and_acronyms() {
        assert_eq!(
            SearchEverywhere::compute_score("UserModel", "UserModel"),
            Some(100)
        );
        assert_eq!(
            SearchEverywhere::compute_score("user", "UserModel"),
            Some(85)
        );
        assert_eq!(SearchEverywhere::compute_score("um", "UserModel"), Some(75));
        assert_eq!(
            SearchEverywhere::compute_score("model", "UserModel"),
            Some(50)
        );
    }

    #[test]
    fn test_search_everywhere_tabs() {
        let mut se = SearchEverywhere::new();
        se.add_class("UserProfile", "models.py", 14);
        se.add_symbol("get_user_profile", "views.py", 45, false);
        se.add_file("src/models.py");

        // Search "All"
        let all_results = se.search("user", SearchTab::All, 10);
        assert!(!all_results.is_empty());

        // Search "Classes" only
        let class_results = se.search("user", SearchTab::Classes, 10);
        assert_eq!(class_results.len(), 1);
        assert_eq!(class_results[0].item.title, "UserProfile");

        // Search "Actions"
        let action_results = se.search("debug", SearchTab::Actions, 5);
        assert!(!action_results.is_empty());
        assert_eq!(action_results[0].item.title, "Debug Application");
    }
}
