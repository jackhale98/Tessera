//! Name and field mapping utilities for SysML v2 interchange

use std::collections::HashMap;

/// Convert a TDT entity title to a valid SysML PascalCase identifier.
///
/// Examples:
/// - "Stroke Length" -> "StrokeLength"
/// - "Min Operating Temp" -> "MinOperatingTemp"
/// - "shaft-bushing clearance" -> "ShaftBushingClearance"
pub fn title_to_sysml_name(title: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for ch in title.chars() {
        if ch.is_alphanumeric() {
            if capitalize_next {
                for upper in ch.to_uppercase() {
                    result.push(upper);
                }
                capitalize_next = false;
            } else {
                result.push(ch);
            }
        } else {
            // Non-alphanumeric character acts as word separator
            capitalize_next = true;
        }
    }

    // Ensure the name starts with a letter
    if result.starts_with(|c: char| c.is_ascii_digit()) {
        result.insert(0, 'X');
    }

    if result.is_empty() {
        result = "Unnamed".to_string();
    }

    result
}

/// Convert a SysML PascalCase name back to a human-readable title.
///
/// Examples:
/// - "StrokeLength" -> "Stroke Length"
/// - "ShaftBushingClearance" -> "Shaft Bushing Clearance"
pub fn sysml_name_to_title(name: &str) -> String {
    let mut result = String::new();

    for (i, ch) in name.chars().enumerate() {
        if i > 0 && ch.is_uppercase() {
            // Check if previous char was lowercase (word boundary)
            let prev = name.chars().nth(i - 1).unwrap_or(' ');
            if prev.is_lowercase() {
                result.push(' ');
            }
        }
        result.push(ch);
    }

    result
}

/// Build a name map from TDT IDs to SysML names, resolving collisions.
///
/// If two entities produce the same PascalCase name, the suffix `_<short_id>` is appended.
pub fn build_name_map(entries: &[(String, String)]) -> HashMap<String, String> {
    let mut name_to_ids: HashMap<String, Vec<String>> = HashMap::new();
    let mut id_to_name: HashMap<String, String> = HashMap::new();

    // First pass: generate names and detect collisions
    for (id, title) in entries {
        let name = title_to_sysml_name(title);
        name_to_ids
            .entry(name.clone())
            .or_default()
            .push(id.clone());
        id_to_name.insert(id.clone(), name);
    }

    // Second pass: resolve collisions by appending a suffix
    let mut result = HashMap::new();
    for (id, base_name) in &id_to_name {
        let ids = &name_to_ids[base_name];
        if ids.len() > 1 {
            // Collision: append short suffix from ID
            let suffix = id
                .split('-')
                .next_back()
                .map(|s| &s[..s.len().min(6)])
                .unwrap_or("X");
            result.insert(id.clone(), format!("{}_{}", base_name, suffix));
        } else {
            result.insert(id.clone(), base_name.clone());
        }
    }

    result
}

/// Map TDT verdict to SysML verdict kind string.
pub fn verdict_to_sysml(verdict: &str) -> &str {
    match verdict.to_lowercase().as_str() {
        "pass" => "pass",
        "fail" => "fail",
        "conditional" | "incomplete" => "inconclusive",
        "not_applicable" => "error",
        _ => "inconclusive",
    }
}

/// Map SysML verdict kind back to TDT verdict string.
pub fn sysml_to_verdict(sysml_verdict: &str) -> &str {
    match sysml_verdict.to_lowercase().as_str() {
        "pass" => "pass",
        "fail" => "fail",
        "inconclusive" => "conditional",
        "error" => "not_applicable",
        _ => "incomplete",
    }
}

/// Map TDT test method to SysML verification method kind.
pub fn method_to_sysml(method: &str) -> &str {
    match method.to_lowercase().as_str() {
        "inspection" => "inspect",
        "analysis" => "analyze",
        "demonstration" => "demo",
        "test" => "test",
        _ => "test",
    }
}

/// Map SysML verification method kind back to TDT test method.
pub fn sysml_to_method(sysml_method: &str) -> &str {
    match sysml_method.to_lowercase().as_str() {
        "inspect" | "inspection" => "inspection",
        "analyze" | "analysis" => "analysis",
        "demo" | "demonstration" => "demonstration",
        "test" => "test",
        _ => "test",
    }
}

/// Escape a string for use in a SysML doc comment block.
pub fn escape_sysml_doc(text: &str) -> String {
    text.replace("*/", "* /")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_title_to_sysml_name() {
        assert_eq!(title_to_sysml_name("Stroke Length"), "StrokeLength");
        assert_eq!(
            title_to_sysml_name("shaft-bushing clearance"),
            "ShaftBushingClearance"
        );
        assert_eq!(
            title_to_sysml_name("Min Operating Temp"),
            "MinOperatingTemp"
        );
        assert_eq!(title_to_sysml_name("100mm stroke"), "X100mmStroke");
        assert_eq!(title_to_sysml_name(""), "Unnamed");
    }

    #[test]
    fn test_sysml_name_to_title() {
        assert_eq!(sysml_name_to_title("StrokeLength"), "Stroke Length");
        assert_eq!(
            sysml_name_to_title("ShaftBushingClearance"),
            "Shaft Bushing Clearance"
        );
    }

    #[test]
    fn test_build_name_map_no_collision() {
        let entries = vec![
            ("REQ-001".to_string(), "Stroke Length".to_string()),
            ("REQ-002".to_string(), "Operating Temp".to_string()),
        ];
        let map = build_name_map(&entries);
        assert_eq!(map["REQ-001"], "StrokeLength");
        assert_eq!(map["REQ-002"], "OperatingTemp");
    }

    #[test]
    fn test_build_name_map_with_collision() {
        let entries = vec![
            ("REQ-AAAAAA".to_string(), "Stroke Length".to_string()),
            ("REQ-BBBBBB".to_string(), "Stroke Length".to_string()),
        ];
        let map = build_name_map(&entries);
        // Both should have suffixes
        assert!(map["REQ-AAAAAA"].starts_with("StrokeLength_"));
        assert!(map["REQ-BBBBBB"].starts_with("StrokeLength_"));
        assert_ne!(map["REQ-AAAAAA"], map["REQ-BBBBBB"]);
    }

    #[test]
    fn test_verdict_mapping() {
        assert_eq!(verdict_to_sysml("pass"), "pass");
        assert_eq!(verdict_to_sysml("fail"), "fail");
        assert_eq!(verdict_to_sysml("conditional"), "inconclusive");
        assert_eq!(sysml_to_verdict("pass"), "pass");
        assert_eq!(sysml_to_verdict("inconclusive"), "conditional");
    }

    #[test]
    fn test_method_mapping() {
        assert_eq!(method_to_sysml("inspection"), "inspect");
        assert_eq!(method_to_sysml("analysis"), "analyze");
        assert_eq!(sysml_to_method("inspect"), "inspection");
        assert_eq!(sysml_to_method("demo"), "demonstration");
    }
}
