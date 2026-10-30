use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use std::process::Command;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SdkFlavor {
    Uv,
    VirtualEnv,
    Conda,
    Poetry,
    System,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PythonSdk {
    pub name: String,
    pub flavor: SdkFlavor,
    pub binary_path: PathBuf,
    pub version: String,
    pub sys_path: Vec<PathBuf>,
}

pub struct SdkDetector;

impl SdkDetector {
    /// Detect available Python SDKs for a project directory
    pub fn detect_sdks<P: AsRef<Path>>(project_root: P) -> Vec<PythonSdk> {
        let root = project_root.as_ref();
        let mut sdks = Vec::new();

        // 1. Check for UV or standard virtualenv in .venv or venv
        for candidate_name in &[".venv", "venv", ".env"] {
            let venv_dir = root.join(candidate_name);
            let python_bin = venv_dir.join("bin").join("python");
            if python_bin.exists() {
                let is_uv = root.join("uv.lock").exists() || venv_dir.join("uv.lock").exists();
                let flavor = if is_uv {
                    SdkFlavor::Uv
                } else {
                    SdkFlavor::VirtualEnv
                };

                if let Some(sdk) = Self::probe_python(
                    &format!("{}: {}", if is_uv { "uv" } else { "venv" }, candidate_name),
                    flavor,
                    &python_bin,
                ) {
                    sdks.push(sdk);
                }
            }
        }

        // 2. Check for Conda environment
        if let Ok(conda_prefix) = std::env::var("CONDA_PREFIX") {
            let conda_bin = PathBuf::from(conda_prefix).join("bin").join("python");
            if conda_bin.exists() {
                if let Some(sdk) = Self::probe_python("conda: active", SdkFlavor::Conda, &conda_bin)
                {
                    sdks.push(sdk);
                }
            }
        }

        // 3. Check for System Python
        for sys_cmd in &["python3", "python"] {
            let lookup = if cfg!(target_os = "windows") {
                "where"
            } else {
                "which"
            };
            if let Ok(output) = Command::new(lookup).arg(sys_cmd).output() {
                if output.status.success() {
                    let path_str = String::from_utf8_lossy(&output.stdout)
                        .lines()
                        .next()
                        .unwrap_or("")
                        .trim()
                        .to_string();
                    let bin_path = PathBuf::from(path_str);
                    if bin_path.exists() && !sdks.iter().any(|s| s.binary_path == bin_path) {
                        if let Some(sdk) = Self::probe_python(
                            &format!("system: {}", sys_cmd),
                            SdkFlavor::System,
                            &bin_path,
                        ) {
                            sdks.push(sdk);
                        }
                    }
                }
            }
        }

        sdks
    }

    /// Run Python to probe its version and sys.path
    fn probe_python(name: &str, flavor: SdkFlavor, binary_path: &Path) -> Option<PythonSdk> {
        let version_output = Command::new(binary_path).arg("--version").output().ok()?;
        let version_str = String::from_utf8_lossy(&version_output.stdout);
        let mut version = version_str.trim().to_string();
        if version.is_empty() {
            let stderr_str = String::from_utf8_lossy(&version_output.stderr);
            if !stderr_str.trim().is_empty() {
                version = stderr_str.trim().to_string();
            }
        }

        // Probe sys.path
        let probe_script = "import sys, json; print(json.dumps([p for p in sys.path if p]))";
        let path_output = Command::new(binary_path)
            .args(["-c", probe_script])
            .output()
            .ok()?;

        let mut sys_path = Vec::new();
        if path_output.status.success() {
            let out_str = String::from_utf8_lossy(&path_output.stdout);
            if let Ok(paths) = serde_json::from_str::<Vec<String>>(out_str.trim()) {
                sys_path = paths.into_iter().map(PathBuf::from).collect();
            }
        }

        Some(PythonSdk {
            name: name.to_string(),
            flavor,
            binary_path: binary_path.to_path_buf(),
            version,
            sys_path,
        })
    }
}
