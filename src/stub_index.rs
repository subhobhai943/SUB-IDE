use crate::parser::PyFile;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// Compact stub representing a Python class, matching PyCharm's `PyClassStub`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyClassStub {
    pub name: String,
    pub module_name: String,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub superclasses: Vec<String>,
    pub methods: Vec<String>,
    pub docstring: Option<String>,
}

/// Compact stub representing a Python function or method, matching PyCharm's `PyFunctionStub`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PyFunctionStub {
    pub name: String,
    pub module_name: String,
    pub file_path: PathBuf,
    pub line_number: usize,
    pub is_method: bool,
    pub containing_class: Option<String>,
    pub parameters: Vec<String>,
    pub return_type: Option<String>,
    pub docstring: Option<String>,
}

/// The Stub Indexing Engine:
/// Matches IntelliJ's `FileBasedIndex` and `StubIndex`.
/// Stores lightweight symbols on an inverted index for sub-millisecond lookup.
pub struct StubIndex {
    classes_by_name: HashMap<String, Vec<PyClassStub>>,
    functions_by_name: HashMap<String, Vec<PyFunctionStub>>,
    file_to_classes: HashMap<PathBuf, Vec<String>>,
    file_to_functions: HashMap<PathBuf, Vec<String>>,
}

impl StubIndex {
    pub fn new() -> Self {
        Self {
            classes_by_name: HashMap::new(),
            functions_by_name: HashMap::new(),
            file_to_classes: HashMap::new(),
            file_to_functions: HashMap::new(),
        }
    }

    /// Index a parsed PyFile and store its stubs
    pub fn index_py_file<P: AsRef<Path>>(&mut self, file_path: P, py_file: &PyFile) {
        let path = file_path.as_ref().to_path_buf();
        self.remove_file(&path);

        let mut class_names = Vec::new();
        let mut func_names = Vec::new();

        // Index Classes
        for cls in &py_file.classes {
            class_names.push(cls.name.clone());

            let method_names: Vec<String> = cls.methods.iter().map(|m| m.name.clone()).collect();
            let stub = PyClassStub {
                name: cls.name.clone(),
                module_name: py_file.module_name.clone(),
                file_path: path.clone(),
                line_number: cls.line_number,
                superclasses: cls.superclasses.clone(),
                methods: method_names,
                docstring: cls.docstring.clone(),
            };

            self.classes_by_name
                .entry(cls.name.clone())
                .or_default()
                .push(stub);

            // Index Methods
            for method in &cls.methods {
                func_names.push(method.name.clone());
                let m_stub = PyFunctionStub {
                    name: method.name.clone(),
                    module_name: py_file.module_name.clone(),
                    file_path: path.clone(),
                    line_number: method.line_number,
                    is_method: true,
                    containing_class: Some(cls.name.clone()),
                    parameters: method.parameters.iter().map(|p| p.name.clone()).collect(),
                    return_type: method.return_type.clone(),
                    docstring: method.docstring.clone(),
                };
                self.functions_by_name
                    .entry(method.name.clone())
                    .or_default()
                    .push(m_stub);
            }
        }

        // Index Top-level Functions
        for func in &py_file.functions {
            func_names.push(func.name.clone());
            let f_stub = PyFunctionStub {
                name: func.name.clone(),
                module_name: py_file.module_name.clone(),
                file_path: path.clone(),
                line_number: func.line_number,
                is_method: false,
                containing_class: None,
                parameters: func.parameters.iter().map(|p| p.name.clone()).collect(),
                return_type: func.return_type.clone(),
                docstring: func.docstring.clone(),
            };
            self.functions_by_name
                .entry(func.name.clone())
                .or_default()
                .push(f_stub);
        }

        self.file_to_classes.insert(path.clone(), class_names);
        self.file_to_functions.insert(path, func_names);
    }

    /// Remove a file from the index on file deletion or before re-indexing
    pub fn remove_file(&mut self, path: &Path) {
        if let Some(classes) = self.file_to_classes.remove(path) {
            for c_name in classes {
                if let Some(entries) = self.classes_by_name.get_mut(&c_name) {
                    entries.retain(|s| s.file_path != path);
                }
            }
        }

        if let Some(funcs) = self.file_to_functions.remove(path) {
            for f_name in funcs {
                if let Some(entries) = self.functions_by_name.get_mut(&f_name) {
                    entries.retain(|s| s.file_path != path);
                }
            }
        }
    }

    /// Lookup class by exact name (Go to Class - Ctrl+N)
    pub fn find_classes(&self, name: &str) -> &[PyClassStub] {
        self.classes_by_name
            .get(name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Lookup function/method by exact name (Go to Symbol - Ctrl+Alt+Shift+N)
    pub fn find_functions(&self, name: &str) -> &[PyFunctionStub] {
        self.functions_by_name
            .get(name)
            .map(|v| v.as_slice())
            .unwrap_or(&[])
    }

    /// Fuzzy search across all classes and functions (Search Everywhere - Shift+Shift)
    pub fn search_everywhere(&self, query: &str) -> Vec<SearchResult> {
        let query_lower = query.to_lowercase();
        let mut results = Vec::new();

        for (name, stubs) in &self.classes_by_name {
            if name.to_lowercase().contains(&query_lower) {
                for stub in stubs {
                    results.push(SearchResult::Class(stub.clone()));
                }
            }
        }

        for (name, stubs) in &self.functions_by_name {
            if name.to_lowercase().contains(&query_lower) {
                for stub in stubs {
                    results.push(SearchResult::Function(stub.clone()));
                }
            }
        }

        results
    }

    pub fn total_symbols(&self) -> (usize, usize) {
        let class_count: usize = self.classes_by_name.values().map(|v| v.len()).sum();
        let func_count: usize = self.functions_by_name.values().map(|v| v.len()).sum();
        (class_count, func_count)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SearchResult {
    Class(PyClassStub),
    Function(PyFunctionStub),
}
