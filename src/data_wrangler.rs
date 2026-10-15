//! Data Wrangler Pro feature for PyCharm.
//! Reverse engineered from JetBrains `dataWrangler-plugin.jar`
//! (`com.intellij.dataWrangler.core.engine.CoreDataWranglerEngine`,
//! `CoreStatisticsHeader`, `DataWranglerCommandFilter`).
//!
//! Provides DataFrame profiling, column data type inference, descriptive statistics
//! (mean, median, std dev, missing value %), histograms, interactive transformations,
//! and reproducible Python Pandas code generation.

use serde::{Deserialize, Serialize};

/// Supported column types in Data Wrangler
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum DataType {
    Integer,
    Float,
    String,
    Boolean,
}

/// Dynamic cell value in DataFrame
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum CellValue {
    Integer(i64),
    Float(f64),
    String(String),
    Boolean(bool),
    Null,
}

impl CellValue {
    pub fn to_display_string(&self) -> String {
        match self {
            CellValue::Integer(i) => i.to_string(),
            CellValue::Float(f) => format!("{:.2}", f),
            CellValue::String(s) => s.clone(),
            CellValue::Boolean(b) => b.to_string(),
            CellValue::Null => "NaN".to_string(),
        }
    }

    pub fn is_null(&self) -> bool {
        matches!(self, CellValue::Null)
    }

    pub fn as_f64(&self) -> Option<f64> {
        match self {
            CellValue::Integer(i) => Some(*i as f64),
            CellValue::Float(f) => Some(*f),
            _ => None,
        }
    }
}

/// Descriptive statistics header per column (PyCharm Data Wrangler Header)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ColumnStats {
    pub name: String,
    pub data_type: DataType,
    pub total_count: usize,
    pub non_null_count: usize,
    pub missing_count: usize,
    pub missing_pct: f64,
    pub unique_count: usize,
    pub min: Option<f64>,
    pub max: Option<f64>,
    pub mean: Option<f64>,
    pub median: Option<f64>,
    pub std_dev: Option<f64>,
    pub histogram: Vec<(String, usize)>,
}

/// Transformation filter operators
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum FilterOp {
    GreaterThan(f64),
    LessThan(f64),
    Equals(String),
    Contains(String),
    NotNull,
}

/// Recorded transformation operation for script generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransformationStep {
    Filter { column: String, op: FilterOp },
    Sort { column: String, ascending: bool },
    DropNulls { column: String },
    FillNulls { column: String, value: String },
}

/// Pure Rust DataFrame engine
#[derive(Debug, Clone)]
pub struct DataFrame {
    columns: Vec<(String, DataType)>,
    rows: Vec<Vec<CellValue>>,
    history: Vec<TransformationStep>,
}

impl DataFrame {
    pub fn new(columns: Vec<(&str, DataType)>, rows: Vec<Vec<CellValue>>) -> Self {
        Self {
            columns: columns.into_iter().map(|(n, t)| (n.to_string(), t)).collect(),
            rows,
            history: Vec::new(),
        }
    }

    /// Parses CSV into a DataFrame
    pub fn from_csv(csv_str: &str) -> Result<Self, String> {
        let mut lines = csv_str.lines().filter(|l| !l.trim().is_empty());
        let header_line = lines.next().ok_or("Empty CSV data")?;
        let col_names: Vec<String> = header_line.split(',').map(|s| s.trim().to_string()).collect();

        let mut raw_rows: Vec<Vec<String>> = Vec::new();
        for line in lines {
            let fields: Vec<String> = line.split(',').map(|s| s.trim().to_string()).collect();
            if fields.len() == col_names.len() {
                raw_rows.push(fields);
            }
        }

        // Infer types
        let mut columns = Vec::new();
        for (i, name) in col_names.iter().enumerate() {
            let mut is_int = true;
            let mut is_float = true;
            let mut is_bool = true;

            for r in &raw_rows {
                let v = &r[i];
                if v.is_empty() || v.eq_ignore_ascii_case("nan") || v.eq_ignore_ascii_case("null") {
                    continue;
                }
                if v.parse::<i64>().is_err() {
                    is_int = false;
                }
                if v.parse::<f64>().is_err() {
                    is_float = false;
                }
                if v != "true" && v != "false" {
                    is_bool = false;
                }
            }

            let dt = if is_int {
                DataType::Integer
            } else if is_float {
                DataType::Float
            } else if is_bool {
                DataType::Boolean
            } else {
                DataType::String
            };
            columns.push((name.clone(), dt));
        }

        // Parse cells
        let mut rows = Vec::new();
        for r in raw_rows {
            let mut row = Vec::new();
            for (i, v) in r.iter().enumerate() {
                if v.is_empty() || v.eq_ignore_ascii_case("nan") || v.eq_ignore_ascii_case("null") {
                    row.push(CellValue::Null);
                } else {
                    let cell = match columns[i].1 {
                        DataType::Integer => v.parse::<i64>().map(CellValue::Integer).unwrap_or(CellValue::Null),
                        DataType::Float => v.parse::<f64>().map(CellValue::Float).unwrap_or(CellValue::Null),
                        DataType::Boolean => CellValue::Boolean(v == "true"),
                        DataType::String => CellValue::String(v.clone()),
                    };
                    row.push(cell);
                }
            }
            rows.push(row);
        }

        Ok(Self {
            columns,
            rows,
            history: Vec::new(),
        })
    }

    pub fn column_index(&self, name: &str) -> Option<usize> {
        self.columns.iter().position(|(c, _)| c == name)
    }

    /// Computes column statistics (PyCharm Data Wrangler statistics header)
    pub fn compute_stats(&self) -> Vec<ColumnStats> {
        let mut stats = Vec::new();
        let total = self.rows.len();

        for (idx, (col_name, dt)) in self.columns.iter().enumerate() {
            let mut non_null = 0;
            let mut unique_vals = std::collections::HashSet::new();
            let mut numeric_vals = Vec::new();

            for row in &self.rows {
                let cell = &row[idx];
                if !cell.is_null() {
                    non_null += 1;
                    unique_vals.insert(cell.to_display_string());
                    if let Some(num) = cell.as_f64() {
                        numeric_vals.push(num);
                    }
                }
            }

            let missing = total - non_null;
            let missing_pct = if total > 0 { (missing as f64 / total as f64) * 100.0 } else { 0.0 };

            let (min, max, mean, median, std_dev) = if !numeric_vals.is_empty() {
                numeric_vals.sort_by(|a, b| a.partial_cmp(b).unwrap());
                let min = *numeric_vals.first().unwrap();
                let max = *numeric_vals.last().unwrap();
                let sum: f64 = numeric_vals.iter().sum();
                let mean = sum / numeric_vals.len() as f64;
                let median = if numeric_vals.len() % 2 == 1 {
                    numeric_vals[numeric_vals.len() / 2]
                } else {
                    (numeric_vals[numeric_vals.len() / 2 - 1] + numeric_vals[numeric_vals.len() / 2]) / 2.0
                };
                let variance = numeric_vals.iter().map(|v| (v - mean).powi(2)).sum::<f64>() / numeric_vals.len() as f64;
                let std = variance.sqrt();
                (Some(min), Some(max), Some(mean), Some(median), Some(std))
            } else {
                (None, None, None, None, None)
            };

            // Build histogram buckets
            let histogram = if let (Some(lo), Some(hi)) = (min, max) {
                if lo < hi {
                    let bucket_count = 4;
                    let step = (hi - lo) / bucket_count as f64;
                    let mut buckets = vec![0; bucket_count];
                    for &val in &numeric_vals {
                        let b = (((val - lo) / step).floor() as usize).min(bucket_count - 1);
                        buckets[b] += 1;
                    }
                    buckets
                        .into_iter()
                        .enumerate()
                        .map(|(b, count)| {
                            let label = format!("{:.0}-{:.0}", lo + b as f64 * step, lo + (b + 1) as f64 * step);
                            (label, count)
                        })
                        .collect()
                } else {
                    vec![(format!("{:.0}", lo), numeric_vals.len())]
                }
            } else {
                Vec::new()
            };

            stats.push(ColumnStats {
                name: col_name.clone(),
                data_type: *dt,
                total_count: total,
                non_null_count: non_null,
                missing_count: missing,
                missing_pct,
                unique_count: unique_vals.len(),
                min,
                max,
                mean,
                median,
                std_dev,
                histogram,
            });
        }

        stats
    }

    /// Filters rows by condition
    pub fn filter(&mut self, col_name: &str, op: FilterOp) -> Result<&mut Self, String> {
        let idx = self.column_index(col_name).ok_or_else(|| format!("Unknown column '{}'", col_name))?;

        self.rows.retain(|row| {
            let cell = &row[idx];
            match &op {
                FilterOp::GreaterThan(limit) => cell.as_f64().map(|v| v > *limit).unwrap_or(false),
                FilterOp::LessThan(limit) => cell.as_f64().map(|v| v < *limit).unwrap_or(false),
                FilterOp::Equals(target) => cell.to_display_string() == *target,
                FilterOp::Contains(sub) => cell.to_display_string().contains(sub),
                FilterOp::NotNull => !cell.is_null(),
            }
        });

        self.history.push(TransformationStep::Filter {
            column: col_name.to_string(),
            op,
        });

        Ok(self)
    }

    /// Sorts DataFrame by column
    pub fn sort_by(&mut self, col_name: &str, ascending: bool) -> Result<&mut Self, String> {
        let idx = self.column_index(col_name).ok_or_else(|| format!("Unknown column '{}'", col_name))?;

        self.rows.sort_by(|a, b| {
            let cmp = match (&a[idx], &b[idx]) {
                (CellValue::Null, CellValue::Null) => std::cmp::Ordering::Equal,
                (CellValue::Null, _) => std::cmp::Ordering::Greater,
                (_, CellValue::Null) => std::cmp::Ordering::Less,
                (CellValue::Integer(x), CellValue::Integer(y)) => x.cmp(y),
                (CellValue::Float(x), CellValue::Float(y)) => x.partial_cmp(y).unwrap_or(std::cmp::Ordering::Equal),
                (CellValue::String(x), CellValue::String(y)) => x.cmp(y),
                (CellValue::Boolean(x), CellValue::Boolean(y)) => x.cmp(y),
                _ => a[idx].to_display_string().cmp(&b[idx].to_display_string()),
            };
            if ascending { cmp } else { cmp.reverse() }
        });

        self.history.push(TransformationStep::Sort {
            column: col_name.to_string(),
            ascending,
        });

        Ok(self)
    }

    /// Generates reproducible Python Pandas transformation code
    pub fn generate_python_script(&self) -> String {
        let mut script = String::new();
        script.push_str("# Auto-generated by PyCharm Data Wrangler\n");
        script.push_str("import pandas as pd\n\n");
        script.push_str("def transform(df: pd.DataFrame) -> pd.DataFrame:\n");

        if self.history.is_empty() {
            script.push_str("    return df\n");
            return script;
        }

        for step in &self.history {
            match step {
                TransformationStep::Filter { column, op } => match op {
                    FilterOp::GreaterThan(v) => {
                        script.push_str(&format!("    df = df[df['{}'] > {}]\n", column, v));
                    }
                    FilterOp::LessThan(v) => {
                        script.push_str(&format!("    df = df[df['{}'] < {}]\n", column, v));
                    }
                    FilterOp::Equals(s) => {
                        script.push_str(&format!("    df = df[df['{}'] == '{}']\n", column, s));
                    }
                    FilterOp::Contains(s) => {
                        script.push_str(&format!("    df = df[df['{}'].astype(str).str.contains('{}')]\n", column, s));
                    }
                    FilterOp::NotNull => {
                        script.push_str(&format!("    df = df[df['{}'].notna()]\n", column));
                    }
                },
                TransformationStep::Sort { column, ascending } => {
                    script.push_str(&format!(
                        "    df = df.sort_values(by='{}', ascending={})\n",
                        column,
                        if *ascending { "True" } else { "False" }
                    ));
                }
                TransformationStep::DropNulls { column } => {
                    script.push_str(&format!("    df = df.dropna(subset=['{}'])\n", column));
                }
                TransformationStep::FillNulls { column, value } => {
                    script.push_str(&format!("    df['{}'] = df['{}'].fillna({})\n", column, column, value));
                }
            }
        }
        script.push_str("    return df\n");
        script
    }

    /// Renders visual Data Wrangler grid (with Header Statistics)
    pub fn render_summary_grid(&self) -> String {
        let mut out = String::new();
        let stats = self.compute_stats();

        out.push_str("┌─── PyCharm Data Wrangler Grid ──────────────────────────────────────────────┐\n");
        // Column names & types
        out.push_str("│ Columns: ");
        for (name, dt) in &self.columns {
            out.push_str(&format!("{}: {:?} | ", name, dt));
        }
        out.push_str("\n│ ── Statistics Header ────────────────────────────────────────────────────────\n");
        for st in &stats {
            let mean_str = st.mean.map(|m| format!("mean: {:.1}, ", m)).unwrap_or_default();
            let min_max = match (st.min, st.max) {
                (Some(lo), Some(hi)) => format!("range: [{:.0}..{:.0}], ", lo, hi),
                _ => String::new(),
            };
            out.push_str(&format!(
                "│ [{}] total: {}, non-null: {}, missing: {:.1}%, {}{}{}\n",
                st.name, st.total_count, st.non_null_count, st.missing_pct, min_max, mean_str,
                if !st.histogram.is_empty() {
                    let bins = st.histogram.iter().map(|(l, c)| format!("{}:{}", l, c)).collect::<Vec<_>>().join(" ");
                    format!("hist: [{}]", bins)
                } else {
                    String::new()
                }
            ));
        }
        out.push_str("│ ── Data Rows (preview) ─────────────────────────────────────────────────────\n");
        for (r_idx, row) in self.rows.iter().take(5).enumerate() {
            let line = row.iter().map(|c| c.to_display_string()).collect::<Vec<_>>().join(" | ");
            out.push_str(&format!("│ {:>2}: {}\n", r_idx + 1, line));
        }
        out.push_str("└─────────────────────────────────────────────────────────────────────────────┘\n");
        out
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_csv_parsing_and_statistics() {
        let csv = "\
name,age,salary,is_active
Alice,30,85000,true
Bob,25,62000,false
Charlie,40,,true
David,,95000,true";

        let df = DataFrame::from_csv(csv).unwrap();
        assert_eq!(df.columns.len(), 4);
        assert_eq!(df.rows.len(), 4);

        let stats = df.compute_stats();
        let age_stat = stats.iter().find(|s| s.name == "age").unwrap();
        assert_eq!(age_stat.missing_count, 1);
        assert_eq!(age_stat.missing_pct, 25.0);
        assert_eq!(age_stat.mean, Some(31.666666666666668));
    }

    #[test]
    fn test_transformations_and_code_generation() {
        let csv = "\
product,price,rating
Widget A,15.5,4.8
Widget B,8.0,3.5
Widget C,45.0,4.9
Widget D,12.0,";

        let mut df = DataFrame::from_csv(csv).unwrap();
        df.filter("price", FilterOp::GreaterThan(10.0)).unwrap();
        df.sort_by("price", false).unwrap();

        assert_eq!(df.rows.len(), 3);
        assert_eq!(df.rows[0][0].to_display_string(), "Widget C");

        let code = df.generate_python_script();
        assert!(code.contains("df = df[df['price'] > 10]"));
        assert!(code.contains("df = df.sort_values(by='price', ascending=False)"));
    }
}
