//! Variable-data preflight checks shared by panels and MCP (10.11, Table B).
//!
//! Pure over `&Document`: unbound-field warnings plus blocking record
//! findings (missing required values, image path-security violations).

use aubrieta_document::{DataSourceId, Document, FieldValue, MissingValuePolicy, PreflightFinding};
use aubrieta_foundation::AubrietaError;

/// Runs merge preflight for one data source.
/// Returns findings; `is_blocking` entries must abort materialization.
pub fn preflight(
    document: &Document,
    source_id: DataSourceId,
) -> Result<Vec<PreflightFinding>, AubrietaError> {
    let source = document.data_source(source_id).ok_or_else(|| {
        AubrietaError::not_found(format!("data source `{source_id}` not found"))
    })?;

    let mut findings = Vec::new();

    // 1. Check for unbound fields
    for field in &source.schema.fields {
        let has_binding = document
            .bindings()
            .iter()
            .any(|b| b.source_id == source_id && b.field_id == field.id);
        if !has_binding {
            findings.push(PreflightFinding {
                record_key: "*".to_string(),
                object_id: None,
                field_id: Some(field.id),
                message: format!("Field `{}` has no document property bindings", field.name),
                is_blocking: false,
            });
        }
    }

    // 2. Check each record for missing required values or bad paths
    for record in &source.records {
        for binding in document.bindings() {
            if binding.source_id != source_id {
                continue;
            }
            match record.get(binding.field_id) {
                None | Some(FieldValue::Null) => {
                    if matches!(binding.missing_policy, MissingValuePolicy::Fail) {
                        findings.push(PreflightFinding {
                            record_key: record.key.clone(),
                            object_id: Some(binding.target_object),
                            field_id: Some(binding.field_id),
                            message: format!(
                                "Record `{}` missing required value for field `{}`",
                                record.key, binding.field_id
                            ),
                            is_blocking: true,
                        });
                    }
                }
                Some(FieldValue::ImageRef(path)) => {
                    if let Err(e) = aubrieta_document::PathSecurity::sanitize_relative_path(path) {
                        findings.push(PreflightFinding {
                            record_key: record.key.clone(),
                            object_id: Some(binding.target_object),
                            field_id: Some(binding.field_id),
                            message: format!(
                                "Record `{}` image path security error: {e}",
                                record.key
                            ),
                            is_blocking: true,
                        });
                    }
                }
                _ => {}
            }
        }
    }

    Ok(findings)
}
