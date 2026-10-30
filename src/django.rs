use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents a detected Django or FastAPI/Flask route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebRoute {
    pub pattern: String,
    pub http_method: String, // GET, POST, ANY
    pub view_target: String, // e.g. views.user_list or UsersView.as_view
    pub name: Option<String>,
    pub file_path: PathBuf,
    pub line_number: usize,
}

/// Represents a Django ORM Field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjangoField {
    pub name: String,
    pub field_type: String, // CharField, IntegerField, ForeignKey, DateTimeField, etc.
    pub is_primary_key: bool,
    pub related_model: Option<String>,
}

/// Represents a Django ORM Model (matching PyCharm Pro Django model binding)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjangoModel {
    pub name: String,
    pub table_name: String,
    pub app_name: String,
    pub fields: Vec<DjangoField>,
    pub file_path: PathBuf,
    pub line_number: usize,
}

/// Represents a Django Project Configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DjangoProject {
    pub root_dir: PathBuf,
    pub manage_py: PathBuf,
    pub settings_module: Option<String>,
    pub apps: Vec<String>,
    pub models: Vec<DjangoModel>,
    pub routes: Vec<WebRoute>,
}

pub struct DjangoInspector;

impl DjangoInspector {
    /// Detect if a directory is a Django project and inspect its manage.py and apps
    pub fn detect_project<P: AsRef<Path>>(root: P) -> Option<DjangoProject> {
        let root_path = root.as_ref().to_path_buf();
        let manage_py = root_path.join("manage.py");

        if !manage_py.exists() {
            return None;
        }

        let mut project = DjangoProject {
            root_dir: root_path.clone(),
            manage_py,
            settings_module: None,
            apps: Vec::new(),
            models: Vec::new(),
            routes: Vec::new(),
        };

        // Scan manage.py for DJANGO_SETTINGS_MODULE
        if let Ok(content) = std::fs::read_to_string(&project.manage_py) {
            for line in content.lines() {
                if line.contains("DJANGO_SETTINGS_MODULE") {
                    if let Some(pos) = line.find("DJANGO_SETTINGS_MODULE") {
                        let remainder = &line[pos..];
                        if let Some(quote_start) = remainder.find(['\'', '"']) {
                            let after_quote = &remainder[quote_start + 1..];
                            if let Some(quote_end) = after_quote.find(['\'', '"']) {
                                project.settings_module =
                                    Some(after_quote[..quote_end].to_string());
                            }
                        }
                    }
                }
            }
        }

        Some(project)
    }

    /// Parse a Django urls.py source string and extract URL routes (matching PyCharm Pro URL reference provider)
    pub fn parse_urls(source: &str, file_path: &Path) -> Vec<WebRoute> {
        let mut routes = Vec::new();

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();
            // Look for path('pattern', view, name='name') or re_path(...)
            if trimmed.starts_with("path(") || trimmed.starts_with("re_path(") {
                let inside = trimmed
                    .strip_prefix("path(")
                    .or_else(|| trimmed.strip_prefix("re_path("))
                    .unwrap_or_default();

                let mut parts = inside.split(',');
                if let Some(raw_pattern) = parts.next() {
                    let pattern = raw_pattern.trim().trim_matches(['\'', '"']).to_string();
                    if let Some(raw_view) = parts.next() {
                        let view_target = raw_view.trim().to_string();
                        let mut route_name = None;

                        // Check for name='...'
                        for part in parts {
                            let p_trim = part.trim();
                            if let Some(name_val_raw) = p_trim.strip_prefix("name=") {
                                let name_val = name_val_raw.trim().trim_matches(['\'', '"', ')']);
                                route_name = Some(name_val.to_string());
                            }
                        }

                        routes.push(WebRoute {
                            pattern,
                            http_method: "ANY".to_string(),
                            view_target,
                            name: route_name,
                            file_path: file_path.to_path_buf(),
                            line_number: line_idx + 1,
                        });
                    }
                }
            }
            // Also support FastAPI / Flask decorators: @app.get('/route'), @app.post(...)
            else if trimmed.starts_with("@app.") || trimmed.starts_with("@router.") {
                let is_app = trimmed.starts_with("@app.");
                let prefix_len = if is_app { 5 } else { 8 };
                let after_prefix = &trimmed[prefix_len..];

                if let Some(paren_pos) = after_prefix.find('(') {
                    let method = after_prefix[..paren_pos].to_uppercase();
                    let inside = &after_prefix[paren_pos + 1..];
                    if let Some(quote_start) = inside.find(['\'', '"']) {
                        let after_quote = &inside[quote_start + 1..];
                        if let Some(quote_end) = after_quote.find(['\'', '"']) {
                            let route_path = after_quote[..quote_end].to_string();
                            routes.push(WebRoute {
                                pattern: route_path,
                                http_method: method,
                                view_target: "endpoint".to_string(),
                                name: None,
                                file_path: file_path.to_path_buf(),
                                line_number: line_idx + 1,
                            });
                        }
                    }
                }
            }
        }

        routes
    }

    /// Parse a Django models.py source string and extract ORM models and fields
    pub fn parse_models(source: &str, file_path: &Path, app_name: &str) -> Vec<DjangoModel> {
        let mut models = Vec::new();
        let mut current_model: Option<DjangoModel> = None;

        for (line_idx, line) in source.lines().enumerate() {
            let trimmed = line.trim();

            if trimmed.starts_with("class ") && trimmed.contains("(models.Model)") {
                if let Some(m) = current_model.take() {
                    models.push(m);
                }

                let class_name = trimmed[6..trimmed.find('(').unwrap()].trim().to_string();
                let table_name =
                    format!("{}_{}", app_name.to_lowercase(), class_name.to_lowercase());

                current_model = Some(DjangoModel {
                    name: class_name,
                    table_name,
                    app_name: app_name.to_string(),
                    fields: Vec::new(),
                    file_path: file_path.to_path_buf(),
                    line_number: line_idx + 1,
                });
            } else if let Some(ref mut model) = current_model {
                // Check if inside class indentation
                if line.starts_with("    ") && trimmed.contains("models.") && trimmed.contains('=')
                {
                    let mut parts = trimmed.split('=');
                    if let Some(field_name_raw) = parts.next() {
                        let field_name = field_name_raw.trim().to_string();
                        if let Some(field_type_raw) = parts.next() {
                            let ft = field_type_raw.trim();
                            let is_pk = ft.contains("primary_key=True");
                            let mut related_model = None;

                            if ft.contains("ForeignKey(") || ft.contains("OneToOneField(") {
                                if let Some(start) = ft.find('(') {
                                    let arg_part = &ft[start + 1..];
                                    if let Some(comma_or_paren) = arg_part.find([',', ')']) {
                                        related_model = Some(
                                            arg_part[..comma_or_paren]
                                                .trim()
                                                .trim_matches(['\'', '"'])
                                                .to_string(),
                                        );
                                    }
                                }
                            }

                            let clean_type = if let Some(paren) = ft.find('(') {
                                ft[..paren].replace("models.", "")
                            } else {
                                ft.replace("models.", "")
                            };

                            model.fields.push(DjangoField {
                                name: field_name,
                                field_type: clean_type,
                                is_primary_key: is_pk,
                                related_model,
                            });
                        }
                    }
                } else if !line.starts_with(' ') && !line.is_empty() && !line.starts_with('#') {
                    // Exited class definition
                    if let Some(m) = current_model.take() {
                        models.push(m);
                    }
                }
            }
        }

        if let Some(m) = current_model.take() {
            models.push(m);
        }

        models
    }
}
