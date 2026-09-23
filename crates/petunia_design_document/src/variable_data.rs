//! Variable Data and Data Merge Engine: Sources, Bindings, Generation & Preflight (10.11).
//!
//! Provides CSV/TSV/JSON tabular parsing, stable typed field identities, declarative formatters,
//! path security, preflight validation, and non-destructive record preview & batch generation.

use std::collections::HashMap;
use std::path::Path;

use petunia_design_foundation::{PetuniaError, ObjectId};
use serde::{Deserialize, Serialize};

use crate::document_object::DocumentObject;

/// Stable identifier for a variable data source (10.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct DataSourceId(pub u64);

impl DataSourceId {
    /// Creates a new data source ID.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns the underlying raw integer.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for DataSourceId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "DataSourceId:{}", self.0)
    }
}

/// Stable internal field identity on a data source schema (10.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct FieldId(pub u32);

impl FieldId {
    /// Creates a new field ID.
    #[must_use]
    pub const fn new(id: u32) -> Self {
        Self(id)
    }

    /// Returns the raw field ID.
    #[must_use]
    pub const fn raw(self) -> u32 {
        self.0
    }
}

impl std::fmt::Display for FieldId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FieldId:{}", self.0)
    }
}

/// Stable identifier for an active data binding (10.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct BindingId(pub u64);

impl BindingId {
    /// Creates a new binding ID.
    #[must_use]
    pub const fn new(id: u64) -> Self {
        Self(id)
    }

    /// Returns raw ID.
    #[must_use]
    pub const fn raw(self) -> u64 {
        self.0
    }
}

impl std::fmt::Display for BindingId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "BindingId:{}", self.0)
    }
}

/// V1 Supported field data types (10.11).
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum FieldType {
    /// Arbitrary Unicode text.
    #[default]
    Text,
    /// Numeric value (integer or floating point).
    Number,
    /// ISO-8601 date or date-time string.
    Date,
    /// Boolean flag (true/false, 1/0).
    Boolean,
    /// Relative file path to local image asset.
    ImageRef,
    /// External web reference.
    Url,
}

/// Metadata descriptor for a single field in a data schema.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct FieldDescriptor {
    /// Stable internal field ID.
    pub id: FieldId,
    /// Original header or display title.
    pub name: String,
    /// Inferred or user-specified semantic type.
    pub field_type: FieldType,
}

/// Complete schema contract of a variable data source (10.11).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
pub struct DataSourceSchema {
    /// Ordered list of fields.
    pub fields: Vec<FieldDescriptor>,
    /// Optional field used as primary unique record key.
    pub key_field: Option<FieldId>,
}

impl DataSourceSchema {
    /// Finds a field descriptor by stable ID.
    #[must_use]
    pub fn field(&self, id: FieldId) -> Option<&FieldDescriptor> {
        self.fields.iter().find(|f| f.id == id)
    }

    /// Finds a field descriptor by name (case-insensitive).
    #[must_use]
    pub fn field_by_name(&self, name: &str) -> Option<&FieldDescriptor> {
        self.fields
            .iter()
            .find(|f| f.name.eq_ignore_ascii_case(name))
    }
}

/// Strong-typed field values parsed from tabular or structured source data.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value", rename_all = "kebab-case")]
pub enum FieldValue {
    /// Text value.
    Text(String),
    /// Numeric value.
    Number(f64),
    /// Date or date-time ISO string.
    Date(String),
    /// Boolean flag.
    Boolean(bool),
    /// Local image asset reference.
    ImageRef(String),
    /// URL link.
    Url(String),
    /// Missing or null value.
    Null,
}

impl FieldValue {
    /// Returns string representation.
    #[must_use]
    pub fn as_text(&self) -> String {
        match self {
            Self::Text(s) | Self::Date(s) | Self::ImageRef(s) | Self::Url(s) => s.clone(),
            Self::Number(n) => n.to_string(),
            Self::Boolean(b) => b.to_string(),
            Self::Null => String::new(),
        }
    }
}

/// Format of the imported data source.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DataSourceFormat {
    /// Comma-separated values.
    Csv,
    /// Tab-separated values.
    Tsv,
    /// JSON array of objects.
    Json,
}

/// Single record (row) in a data source.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataRecord {
    /// Stable record key (either derived from key field or 1-based index).
    pub key: String,
    /// Map from FieldId to parsed typed value.
    pub values: HashMap<FieldId, FieldValue>,
}

impl DataRecord {
    /// Gets value for a field.
    #[must_use]
    pub fn get(&self, field_id: FieldId) -> Option<&FieldValue> {
        self.values.get(&field_id)
    }
}

/// Canonical data source definition attached to a document.
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataSourceDefinition {
    /// Unique stable data source ID.
    pub id: DataSourceId,
    /// Display name (e.g. filename).
    pub name: String,
    /// Source format.
    pub format: DataSourceFormat,
    /// Inferred/confirmed schema.
    pub schema: DataSourceSchema,
    /// Parsed records.
    pub records: Vec<DataRecord>,
}

impl DataSourceDefinition {
    /// Returns number of records in this source.
    #[must_use]
    pub fn record_count(&self) -> usize {
        self.records.len()
    }
}

/// Document object target property eligible for data binding (10.11).
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TargetProperty {
    /// Object text content (for text elements or object name).
    TextContent,
    /// Fill color or token reference.
    FillColor,
    /// Stroke color or token reference.
    StrokeColor,
    /// Opacity factor `[0.0, 1.0]`.
    Opacity,
    /// Visibility boolean flag.
    Visible,
    /// Image asset file reference.
    ImageSource,
}

/// Declarative, pure value formatters without side-effects or scripting (10.11).
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
pub enum ValueFormatter {
    /// No transformation.
    #[default]
    None,
    /// Uppercase text conversion.
    Uppercase,
    /// Lowercase text conversion.
    Lowercase,
    /// Currency formatting with prefix symbol and fixed decimal places.
    Currency {
        /// Currency symbol (e.g. "$", "R$", "€").
        symbol: String,
        /// Fixed decimal digits (typically 2).
        decimals: usize,
    },
    /// Fixed decimal places for numbers.
    NumberDecimals(usize),
    /// Prepend a literal prefix.
    Prefix(String),
    /// Append a literal suffix.
    Suffix(String),
}

/// Policy for handling missing/null values during merge.
#[derive(Clone, Debug, Default, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum MissingValuePolicy {
    /// Fall back to a default literal string.
    UseDefault(String),
    /// Skip updating this property if missing.
    #[default]
    Skip,
    /// Treat missing value as preflight error.
    Fail,
}

/// Data binding linking a source field to a document object property (10.11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct DataBinding {
    /// Stable binding ID.
    pub id: BindingId,
    /// Bound data source ID.
    pub source_id: DataSourceId,
    /// Bound field ID on the data source.
    pub field_id: FieldId,
    /// Target document object ID.
    pub target_object: ObjectId,
    /// Target property on the object.
    pub target_property: TargetProperty,
    /// Pure value formatter pipeline.
    pub formatter: ValueFormatter,
    /// Missing value fallback policy.
    pub missing_policy: MissingValuePolicy,
}

/// Path security checks against malicious path traversal (10.11).
pub struct PathSecurity;

impl PathSecurity {
    /// Sanitizes an image/asset relative path, preventing directory traversal or absolute escapes.
    pub fn sanitize_relative_path(path_str: &str) -> Result<String, PetuniaError> {
        let p = Path::new(path_str);
        if p.is_absolute() {
            return Err(PetuniaError::invalid_input(format!(
                "absolute paths not permitted in variable data: `{path_str}`"
            )));
        }
        for component in p.components() {
            if let std::path::Component::ParentDir = component {
                return Err(PetuniaError::invalid_input(format!(
                    "path traversal `..` forbidden in variable data: `{path_str}`"
                )));
            }
        }
        Ok(path_str.trim().replace('\\', "/"))
    }
}

/// Pure tabular parser for CSV, TSV, and JSON formats (10.11).
pub struct DataSourceParser;

impl DataSourceParser {
    /// Parses a delimiter-separated text stream (CSV or TSV) with quoted cell support.
    pub fn parse_delimited(
        source_id: DataSourceId,
        name: impl Into<String>,
        content: &str,
        delimiter: char,
    ) -> Result<DataSourceDefinition, PetuniaError> {
        let raw_rows = Self::parse_csv_rows(content, delimiter)?;
        if raw_rows.is_empty() {
            return Err(PetuniaError::invalid_input("data source content is empty"));
        }

        let headers = &raw_rows[0];
        if headers.is_empty() {
            return Err(PetuniaError::invalid_input(
                "data source has no header columns",
            ));
        }

        let mut fields = Vec::new();
        for (i, header) in headers.iter().enumerate() {
            let field_id = FieldId::new(u32::try_from(i + 1).unwrap_or(1));
            let clean_name = header.trim();
            let col_name = if clean_name.is_empty() {
                format!("Column{}", i + 1)
            } else {
                clean_name.to_string()
            };
            fields.push(FieldDescriptor {
                id: field_id,
                name: col_name,
                field_type: FieldType::Text, // Refined below by inference
            });
        }

        let data_rows = &raw_rows[1..];
        let mut records = Vec::new();

        for (row_idx, row) in data_rows.iter().enumerate() {
            let mut values = HashMap::new();
            for (col_idx, field) in fields.iter().enumerate() {
                let cell_raw = row.get(col_idx).map(|s| s.trim()).unwrap_or("");
                let val = Self::infer_value(cell_raw);
                values.insert(field.id, val);
            }
            let key = format!("row-{}", row_idx + 1);
            records.push(DataRecord { key, values });
        }

        // Infer column types based on parsed records
        for field in &mut fields {
            let non_empty_values: Vec<&FieldValue> = records
                .iter()
                .filter_map(|r| r.get(field.id))
                .filter(|v| !matches!(v, FieldValue::Null))
                .collect();

            if !non_empty_values.is_empty() {
                if non_empty_values
                    .iter()
                    .all(|v| matches!(v, FieldValue::Number(_)))
                {
                    field.field_type = FieldType::Number;
                } else if non_empty_values
                    .iter()
                    .all(|v| matches!(v, FieldValue::Boolean(_)))
                {
                    field.field_type = FieldType::Boolean;
                } else if non_empty_values
                    .iter()
                    .all(|v| matches!(v, FieldValue::Date(_)))
                {
                    field.field_type = FieldType::Date;
                }
            }
        }

        let format = if delimiter == '\t' {
            DataSourceFormat::Tsv
        } else {
            DataSourceFormat::Csv
        };

        Ok(DataSourceDefinition {
            id: source_id,
            name: name.into(),
            format,
            schema: DataSourceSchema {
                fields,
                key_field: None,
            },
            records,
        })
    }

    /// Parses a JSON array of objects `[ { "name": "Val", ... } ]`.
    pub fn parse_json(
        source_id: DataSourceId,
        name: impl Into<String>,
        content: &str,
    ) -> Result<DataSourceDefinition, PetuniaError> {
        let json_val: serde_json::Value = serde_json::from_str(content).map_err(|e| {
            PetuniaError::invalid_input(format!("invalid JSON in data source: {e}"))
        })?;

        let array = json_val.as_array().ok_or_else(|| {
            PetuniaError::invalid_input("JSON data source must be an array of objects")
        })?;

        if array.is_empty() {
            return Err(PetuniaError::invalid_input("JSON array is empty"));
        }

        let mut field_names: Vec<String> = Vec::new();
        for item in array {
            if let Some(obj) = item.as_object() {
                for key in obj.keys() {
                    if !field_names.contains(key) {
                        field_names.push(key.clone());
                    }
                }
            }
        }

        let mut fields: Vec<FieldDescriptor> = field_names
            .iter()
            .enumerate()
            .map(|(i, k)| FieldDescriptor {
                id: FieldId::new(u32::try_from(i + 1).unwrap_or(1)),
                name: k.clone(),
                field_type: FieldType::Text,
            })
            .collect();

        let mut records = Vec::new();
        for (idx, item) in array.iter().enumerate() {
            let mut values = HashMap::new();
            if let Some(obj) = item.as_object() {
                for field in &fields {
                    let val = match obj.get(&field.name) {
                        Some(serde_json::Value::String(s)) => Self::infer_value(s),
                        Some(serde_json::Value::Number(n)) => {
                            FieldValue::Number(n.as_f64().unwrap_or(0.0))
                        }
                        Some(serde_json::Value::Bool(b)) => FieldValue::Boolean(*b),
                        Some(serde_json::Value::Null) | None => FieldValue::Null,
                        Some(other) => FieldValue::Text(other.to_string()),
                    };
                    values.insert(field.id, val);
                }
            }
            records.push(DataRecord {
                key: format!("record-{}", idx + 1),
                values,
            });
        }

        // Infer field types
        for field in &mut fields {
            let non_empty: Vec<&FieldValue> = records
                .iter()
                .filter_map(|r| r.get(field.id))
                .filter(|v| !matches!(v, FieldValue::Null))
                .collect();
            if !non_empty.is_empty() {
                if non_empty.iter().all(|v| matches!(v, FieldValue::Number(_))) {
                    field.field_type = FieldType::Number;
                } else if non_empty
                    .iter()
                    .all(|v| matches!(v, FieldValue::Boolean(_)))
                {
                    field.field_type = FieldType::Boolean;
                }
            }
        }

        Ok(DataSourceDefinition {
            id: source_id,
            name: name.into(),
            format: DataSourceFormat::Json,
            schema: DataSourceSchema {
                fields,
                key_field: None,
            },
            records,
        })
    }

    /// Robust CSV row tokenizer handling quoted strings and newlines inside quotes.
    fn parse_csv_rows(content: &str, delimiter: char) -> Result<Vec<Vec<String>>, PetuniaError> {
        let mut rows = Vec::new();
        let mut current_row = Vec::new();
        let mut current_field = String::new();
        let mut in_quotes = false;
        let mut chars = content.chars().peekable();

        while let Some(ch) = chars.next() {
            match ch {
                '"' => {
                    if in_quotes {
                        if chars.peek() == Some(&'"') {
                            // Escaped double quote `""`
                            current_field.push('"');
                            chars.next();
                        } else {
                            in_quotes = false;
                        }
                    } else {
                        in_quotes = true;
                    }
                }
                c if c == delimiter && !in_quotes => {
                    current_row.push(current_field);
                    current_field = String::new();
                }
                '\r' => {
                    if in_quotes {
                        current_field.push('\r');
                    } else if chars.peek() == Some(&'\n') {
                        chars.next();
                        current_row.push(current_field);
                        current_field = String::new();
                        rows.push(current_row);
                        current_row = Vec::new();
                    } else {
                        current_row.push(current_field);
                        current_field = String::new();
                        rows.push(current_row);
                        current_row = Vec::new();
                    }
                }
                '\n' => {
                    if in_quotes {
                        current_field.push('\n');
                    } else {
                        current_row.push(current_field);
                        current_field = String::new();
                        rows.push(current_row);
                        current_row = Vec::new();
                    }
                }
                other => {
                    current_field.push(other);
                }
            }
        }

        if in_quotes {
            return Err(PetuniaError::invalid_input("unclosed quote in CSV data"));
        }

        if !current_field.is_empty() || !current_row.is_empty() {
            current_row.push(current_field);
            rows.push(current_row);
        }

        Ok(rows)
    }

    /// Value type inference from text representation.
    fn infer_value(raw: &str) -> FieldValue {
        if raw.is_empty() {
            return FieldValue::Null;
        }

        // Boolean
        if raw.eq_ignore_ascii_case("true") {
            return FieldValue::Boolean(true);
        }
        if raw.eq_ignore_ascii_case("false") {
            return FieldValue::Boolean(false);
        }

        // Number
        if let Ok(num) = raw.parse::<f64>() {
            if num.is_finite() {
                return FieldValue::Number(num);
            }
        }

        // Date (basic ISO YYYY-MM-DD check)
        if raw.len() == 10 && raw.chars().nth(4) == Some('-') && raw.chars().nth(7) == Some('-') {
            return FieldValue::Date(raw.to_string());
        }

        // URL
        if raw.starts_with("http://") || raw.starts_with("https://") {
            return FieldValue::Url(raw.to_string());
        }

        FieldValue::Text(raw.to_string())
    }
}

/// Evaluator applying formatters and binding mappings into derived document views (10.11).
pub struct DataMergeEvaluator;

impl DataMergeEvaluator {
    /// Formats a typed field value through declarative formatter.
    #[must_use]
    pub fn format_value(value: &FieldValue, formatter: &ValueFormatter) -> String {
        let base = value.as_text();
        match formatter {
            ValueFormatter::None => base,
            ValueFormatter::Uppercase => base.to_uppercase(),
            ValueFormatter::Lowercase => base.to_lowercase(),
            ValueFormatter::Prefix(p) => format!("{p}{base}"),
            ValueFormatter::Suffix(s) => format!("{base}{s}"),
            ValueFormatter::NumberDecimals(decimals) => {
                if let FieldValue::Number(n) = value {
                    format!("{n:.decimals$}")
                } else if let Ok(n) = base.parse::<f64>() {
                    format!("{n:.decimals$}")
                } else {
                    base
                }
            }
            ValueFormatter::Currency { symbol, decimals } => {
                let formatted_num = if let FieldValue::Number(n) = value {
                    format!("{n:.decimals$}")
                } else if let Ok(n) = base.parse::<f64>() {
                    format!("{n:.decimals$}")
                } else {
                    base
                };
                format!("{symbol}{formatted_num}")
            }
        }
    }

    /// Applies bindings from a record to a target document object.
    pub fn apply_to_object(
        object: &mut DocumentObject,
        record: &DataRecord,
        bindings: &[&DataBinding],
    ) -> Result<(), PetuniaError> {
        for binding in bindings {
            if binding.target_object != object.id {
                continue;
            }

            let raw_value = match record.get(binding.field_id) {
                Some(FieldValue::Null) | None => match &binding.missing_policy {
                    MissingValuePolicy::UseDefault(def) => FieldValue::Text(def.clone()),
                    MissingValuePolicy::Skip => continue,
                    MissingValuePolicy::Fail => {
                        return Err(PetuniaError::invalid_input(format!(
                            "missing required value for field `{}` on object `{}`",
                            binding.field_id, object.id
                        )))
                    }
                },
                Some(v) => v.clone(),
            };

            let formatted = Self::format_value(&raw_value, &binding.formatter);

            match binding.target_property {
                TargetProperty::TextContent => {
                    object.name = formatted;
                }
                TargetProperty::FillColor => {
                    object.fill = Some(formatted);
                }
                TargetProperty::StrokeColor => {
                    object.stroke = Some(formatted);
                }
                TargetProperty::Opacity => {
                    if let Ok(opacity) = formatted.parse::<f64>() {
                        object.opacity = opacity.clamp(0.0, 1.0);
                    }
                }
                TargetProperty::Visible => {
                    if let Ok(vis) = formatted.parse::<bool>() {
                        object.visible = vis;
                    } else if formatted == "0" || formatted.eq_ignore_ascii_case("false") {
                        object.visible = false;
                    } else if formatted == "1" || formatted.eq_ignore_ascii_case("true") {
                        object.visible = true;
                    }
                }
                TargetProperty::ImageSource => {
                    let sanitized = PathSecurity::sanitize_relative_path(&formatted)?;
                    object.name = format!("[Image: {sanitized}]");
                }
            }
        }
        Ok(())
    }
}

/// Preflight finding for validation reporting (10.11).
#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct PreflightFinding {
    /// Associated record key.
    pub record_key: String,
    /// Target object ID, if finding relates to a specific object.
    pub object_id: Option<ObjectId>,
    /// Bound field ID, if applicable.
    pub field_id: Option<FieldId>,
    /// Human-readable message.
    pub message: String,
    /// True if this finding blocks batch generation.
    pub is_blocking: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_csv_with_quotes_and_newlines() {
        let csv = "Name,Age,\"City, Country\",Notes\nAlice,30,\"Sao Paulo, Brazil\",\"Line 1\nLine 2\"\nBob,25,\"Paris, France\",\"Simple note\"";
        let ds = DataSourceParser::parse_delimited(DataSourceId::new(1), "people.csv", csv, ',')
            .unwrap();

        assert_eq!(ds.records.len(), 2);
        assert_eq!(ds.schema.fields.len(), 4);
        assert_eq!(ds.schema.fields[0].name, "Name");
        assert_eq!(ds.schema.fields[1].name, "Age");
        assert_eq!(ds.schema.fields[1].field_type, FieldType::Number);
        assert_eq!(ds.schema.fields[2].name, "City, Country");

        let r1 = &ds.records[0];
        assert_eq!(
            r1.get(FieldId::new(1)),
            Some(&FieldValue::Text("Alice".to_string()))
        );
        assert_eq!(r1.get(FieldId::new(2)), Some(&FieldValue::Number(30.0)));
        assert_eq!(
            r1.get(FieldId::new(3)),
            Some(&FieldValue::Text("Sao Paulo, Brazil".to_string()))
        );
        assert_eq!(
            r1.get(FieldId::new(4)),
            Some(&FieldValue::Text("Line 1\nLine 2".to_string()))
        );
    }

    #[test]
    fn parse_json_array() {
        let json_data = r#"[
            {"sku": "SKU-001", "price": 49.90, "in_stock": true},
            {"sku": "SKU-002", "price": 12.50, "in_stock": false}
        ]"#;

        let ds =
            DataSourceParser::parse_json(DataSourceId::new(2), "products.json", json_data).unwrap();

        assert_eq!(ds.records.len(), 2);
        assert_eq!(ds.schema.fields.len(), 3);
        let price_field = ds.schema.field_by_name("price").unwrap();
        assert_eq!(price_field.field_type, FieldType::Number);

        let stock_field = ds.schema.field_by_name("in_stock").unwrap();
        assert_eq!(stock_field.field_type, FieldType::Boolean);
    }

    #[test]
    fn path_security_sanitization() {
        assert!(PathSecurity::sanitize_relative_path("images/hero.png").is_ok());
        assert!(PathSecurity::sanitize_relative_path("assets\\logo.svg").is_ok());
        assert!(PathSecurity::sanitize_relative_path("/etc/passwd").is_err());
        assert!(PathSecurity::sanitize_relative_path("../escape.png").is_err());
        assert!(PathSecurity::sanitize_relative_path("folder/../../escape.png").is_err());
    }

    #[test]
    fn formatters_and_evaluation() {
        let num_val = FieldValue::Number(1234.5);
        let curr = ValueFormatter::Currency {
            symbol: "$".to_string(),
            decimals: 2,
        };
        assert_eq!(
            DataMergeEvaluator::format_value(&num_val, &curr),
            "$1234.50"
        );

        let text_val = FieldValue::Text("hello world".to_string());
        assert_eq!(
            DataMergeEvaluator::format_value(&text_val, &ValueFormatter::Uppercase),
            "HELLO WORLD"
        );
    }
}
