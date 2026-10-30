use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::Path;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum CellType {
    Code,
    Markdown,
    Raw,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "output_type", rename_all = "snake_case")]
pub enum CellOutput {
    Stream {
        name: String, // "stdout" or "stderr"
        text: Vec<String>,
    },
    ExecuteResult {
        execution_count: Option<u64>,
        data: HashMap<String, serde_json::Value>,
    },
    DisplayData {
        data: HashMap<String, serde_json::Value>,
    },
    Error {
        ename: String,
        evalue: String,
        traceback: Vec<String>,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookCell {
    pub id: Option<String>,
    pub cell_type: CellType,
    pub execution_count: Option<u64>,
    pub source: Vec<String>,
    #[serde(default)]
    pub outputs: Vec<CellOutput>,
    #[serde(default)]
    pub metadata: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KernelSpec {
    pub name: String,
    pub display_name: String,
    pub language: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NotebookMetadata {
    pub kernelspec: Option<KernelSpec>,
    #[serde(default)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JupyterNotebook {
    pub nbformat: u32,
    pub nbformat_minor: u32,
    pub metadata: NotebookMetadata,
    pub cells: Vec<NotebookCell>,
}

pub struct NotebookManager;

impl NotebookManager {
    /// Load and deserialize a .ipynb notebook file
    pub fn load_notebook<P: AsRef<Path>>(path: P) -> Result<JupyterNotebook, String> {
        let content = std::fs::read_to_string(path.as_ref())
            .map_err(|e| format!("Failed to read notebook file: {}", e))?;

        serde_json::from_str::<JupyterNotebook>(&content)
            .map_err(|e| format!("Invalid notebook JSON format: {}", e))
    }

    /// Save notebook back to disk
    pub fn save_notebook<P: AsRef<Path>>(nb: &JupyterNotebook, path: P) -> Result<(), String> {
        let json_str = serde_json::to_string_pretty(nb)
            .map_err(|e| format!("Failed to serialize notebook: {}", e))?;

        std::fs::write(path.as_ref(), json_str)
            .map_err(|e| format!("Failed to write notebook file: {}", e))
    }

    /// Execute a code cell using a local Python process and capture outputs
    pub fn execute_code_cell(
        python_bin: &Path,
        code: &str,
        count: u64,
    ) -> (Option<u64>, Vec<CellOutput>) {
        let mut outputs = Vec::new();

        let res = std::process::Command::new(python_bin)
            .args(["-c", code])
            .output();

        match res {
            Ok(output) => {
                if !output.stdout.is_empty() {
                    let out_str = String::from_utf8_lossy(&output.stdout);
                    outputs.push(CellOutput::Stream {
                        name: "stdout".to_string(),
                        text: out_str.lines().map(|l| format!("{}\n", l)).collect(),
                    });
                }
                if !output.stderr.is_empty() {
                    let err_str = String::from_utf8_lossy(&output.stderr);
                    outputs.push(CellOutput::Stream {
                        name: "stderr".to_string(),
                        text: err_str.lines().map(|l| format!("{}\n", l)).collect(),
                    });
                }
                (Some(count), outputs)
            }
            Err(e) => {
                outputs.push(CellOutput::Error {
                    ename: "ExecutionError".to_string(),
                    evalue: e.to_string(),
                    traceback: vec![format!("Failed to execute Python process: {}", e)],
                });
                (None, outputs)
            }
        }
    }
}
