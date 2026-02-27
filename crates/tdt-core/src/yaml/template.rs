//! YAML template enhancement for newly created entities
//!
//! Adds inline comments with enum value hints, TODO placeholders for required
//! empty fields, and commented-out optional fields (category, tags) to help
//! users understand and fill in their entity files.

/// Enhance a newly-created entity YAML file with inline comments and TODO placeholders.
///
/// Only call this during entity creation (not updates), as the comments are
/// meant to guide users when first filling in their entity files.
pub fn enhance_new_entity_yaml(yaml: &str, prefix: &str) -> String {
    let lines: Vec<&str> = yaml.lines().collect();
    let hints = get_field_hints(prefix);
    let placeholders = get_placeholders(prefix);

    let has_category = lines.iter().any(|l| l.starts_with("category:"));
    let has_tags = lines.iter().any(|l| l.starts_with("tags:"));
    let add_category = !has_category && wants_category_comment(prefix);
    let add_tags = !has_tags && wants_tags_comment(prefix);

    let mut result: Vec<String> = Vec::with_capacity(lines.len() + 4);
    let mut inserted_optional = false;

    for line in &lines {
        // Insert commented-out optional fields before priority: or status:
        if !inserted_optional && (add_category || add_tags) {
            let trimmed = line.trim();
            if trimmed.starts_with("priority:") || trimmed.starts_with("status:") {
                if add_category {
                    result.push("# category:".to_string());
                }
                if add_tags {
                    result.push("# tags: []".to_string());
                }
                inserted_optional = true;
            }
        }

        // Check for TODO placeholders first
        let mut replaced = false;
        for (pattern, replacement) in &placeholders {
            if line.trim() == *pattern {
                result.push(replacement.to_string());
                replaced = true;
                break;
            }
        }
        if replaced {
            continue;
        }

        // Add inline enum hints
        let mut enhanced = false;
        for (field_prefix, hint) in &hints {
            if line.starts_with(field_prefix) && !line.contains('#') {
                result.push(format!("{}  # {}", line, hint));
                enhanced = true;
                break;
            }
        }
        if !enhanced {
            result.push(line.to_string());
        }
    }

    // Add wizard hint at end
    let cmd = prefix_to_command(prefix);
    result.push(format!(
        "# Use 'tdt {} new -i' for interactive mode with all fields",
        cmd
    ));

    let mut output = result.join("\n");
    if !output.ends_with('\n') {
        output.push('\n');
    }
    output
}

/// Get inline field hints (field prefix -> enum values) for a given entity type.
fn get_field_hints(prefix: &str) -> Vec<(&'static str, &'static str)> {
    let mut hints: Vec<(&str, &str)> = Vec::new();

    // Common status hint (most entities use the shared Status enum)
    match prefix {
        // These have custom status fields, don't add the common one
        "LOT" | "DEV" | "QUOT" => {}
        _ => {
            hints.push(("status:", "draft | review | approved | released | obsolete"));
        }
    }

    match prefix {
        "REQ" => {
            hints.push(("type:", "input | output"));
            hints.push((
                "level:",
                "stakeholder | system | subsystem | component | detail",
            ));
            hints.push(("priority:", "low | medium | high | critical"));
        }
        "TEST" => {
            hints.push(("type:", "verification | validation"));
            hints.push(("priority:", "low | medium | high | critical"));
        }
        "RISK" => {
            hints.push(("type:", "design | process | use | software"));
        }
        "RSLT" => {
            hints.push((
                "verdict:",
                "pass | fail | conditional | incomplete | not_applicable",
            ));
        }
        "CMP" => {
            hints.push(("make_buy:", "make | buy"));
            hints.push((
                "category:",
                "mechanical | electrical | software | fastener | consumable",
            ));
        }
        "HAZ" => {
            hints.push((
                "category:",
                "electrical | mechanical | thermal | chemical | biological | radiation | ergonomic | software | environmental",
            ));
            hints.push((
                "severity:",
                "negligible | minor | serious | severe | catastrophic",
            ));
        }
        "FEAT" => {
            hints.push(("feature_type:", "internal | external"));
        }
        "MATE" => {
            hints.push(("mate_type:", "clearance | transition | interference"));
        }
        "NCR" => {
            hints.push(("ncr_type:", "internal | supplier | customer"));
            hints.push(("severity:", "minor | major | critical"));
        }
        "CAPA" => {
            hints.push(("capa_type:", "corrective | preventive"));
        }
        "PROC" => {
            hints.push((
                "process_type:",
                "machining | assembly | inspection | test | finishing | packaging | handling | heat_treat | welding | coating",
            ));
        }
        "CTRL" => {
            hints.push((
                "control_type:",
                "spc | inspection | poka_yoke | visual | functional_test | attribute",
            ));
            hints.push(("control_category:", "variable | attribute"));
        }
        "LOT" => {
            hints.push((
                "lot_status:",
                "in_progress | on_hold | completed | scrapped",
            ));
        }
        "DEV" => {
            hints.push(("deviation_type:", "temporary | permanent | emergency"));
        }
        "QUOT" => {
            hints.push((
                "quote_status:",
                "pending | received | accepted | rejected | expired",
            ));
            hints.push(("currency:", "USD | EUR | GBP | CNY | JPY"));
        }
        _ => {}
    }

    hints
}

/// Get TODO placeholder replacements for required empty fields.
fn get_placeholders(prefix: &str) -> Vec<(&'static str, &'static str)> {
    match prefix {
        "REQ" => vec![("text: ''", "text: 'TODO: Describe this requirement'")],
        "TEST" => vec![(
            "objective: ''",
            "objective: 'TODO: Describe the test objective'",
        )],
        "RISK" => vec![("description: ''", "description: 'TODO: Describe this risk'")],
        "HAZ" => vec![(
            "description: ''",
            "description: 'TODO: Describe this hazard'",
        )],
        _ => vec![],
    }
}

/// Whether to add a commented-out `# category:` line for this entity type.
///
/// Only entities with an optional user-defined string category field get this.
/// Entities with enum category fields (CMP, HAZ, NCR, CTRL, DEV) don't need it
/// because their category is always present in the output.
fn wants_category_comment(prefix: &str) -> bool {
    matches!(prefix, "REQ" | "TEST" | "RISK" | "RSLT")
}

/// Whether to add a commented-out `# tags: []` line for this entity type.
///
/// Entities whose tags field uses `skip_serializing_if = "Vec::is_empty"` won't
/// show tags in the default output. This comment reminds users they can add tags.
fn wants_tags_comment(prefix: &str) -> bool {
    matches!(
        prefix,
        "REQ"
            | "TEST"
            | "RISK"
            | "RSLT"
            | "CMP"
            | "HAZ"
            | "PROC"
            | "WORK"
            | "SUP"
            | "NCR"
            | "CAPA"
            | "DEV"
            | "LOT"
            | "QUOT"
    )
}

/// Map entity prefix to CLI command name.
fn prefix_to_command(prefix: &str) -> &'static str {
    match prefix {
        "REQ" => "req",
        "TEST" => "test",
        "RISK" => "risk",
        "RSLT" => "rslt",
        "CMP" => "cmp",
        "ASM" => "asm",
        "HAZ" => "haz",
        "FEAT" => "feat",
        "MATE" => "mate",
        "TOL" => "tol",
        "NCR" => "ncr",
        "CAPA" => "capa",
        "PROC" => "proc",
        "CTRL" => "ctrl",
        "LOT" => "lot",
        "DEV" => "dev",
        "WORK" => "work",
        "SUP" => "sup",
        "QUOT" => "quot",
        _ => "entity",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_requirement_template() {
        let yaml = "\
id: REQ-01KHGW5G60A980RWF528NK82F5
type: input
level: system
title: Motor Torque
text: ''
priority: medium
status: draft
links: {}
created: '2026-02-15T14:44:39.232895865Z'
author: Jack Hale
revision: 1
";
        let result = enhance_new_entity_yaml(yaml, "REQ");

        assert!(result.contains("type: input  # input | output"));
        assert!(result
            .contains("level: system  # stakeholder | system | subsystem | component | detail"));
        assert!(result.contains("text: 'TODO: Describe this requirement'"));
        assert!(result.contains("priority: medium  # low | medium | high | critical"));
        assert!(result.contains("status: draft  # draft | review | approved | released | obsolete"));
        assert!(result.contains("# category:"));
        assert!(result.contains("# tags: []"));
        assert!(result.contains("# Use 'tdt req new -i' for interactive mode with all fields"));
        // Original data preserved
        assert!(result.contains("id: REQ-01KHGW5G60A980RWF528NK82F5"));
        assert!(result.contains("author: Jack Hale"));
    }

    #[test]
    fn test_test_template() {
        let yaml = "\
id: TEST-01KHGW5G60A980RWF528NK82F5
type: verification
title: Torque Test
objective: ''
priority: medium
status: draft
links: {}
created: '2026-02-15T14:44:39Z'
author: Jack Hale
revision: 1
";
        let result = enhance_new_entity_yaml(yaml, "TEST");

        assert!(result.contains("type: verification  # verification | validation"));
        assert!(result.contains("objective: 'TODO: Describe the test objective'"));
        assert!(result.contains("# category:"));
        assert!(result.contains("# tags: []"));
        assert!(result.contains("# Use 'tdt test new -i'"));
    }

    #[test]
    fn test_risk_template() {
        let yaml = "\
id: RISK-01KHGW5G60A980RWF528NK82F5
type: design
title: Battery Overheat
description: ''
status: draft
links: {}
created: '2026-02-15T14:44:39Z'
author: Jack Hale
revision: 1
";
        let result = enhance_new_entity_yaml(yaml, "RISK");

        assert!(result.contains("type: design  # design | process | use | software"));
        assert!(result.contains("description: 'TODO: Describe this risk'"));
        assert!(result.contains("# category:"));
        assert!(result.contains("# tags: []"));
        assert!(result.contains("# Use 'tdt risk new -i'"));
    }

    #[test]
    fn test_component_template() {
        let yaml = "\
id: CMP-01KHGW5G60A980RWF528NK82F5
part_number: PN-001
title: Motor Shaft
make_buy: buy
category: mechanical
status: draft
links: {}
created: '2026-02-15T14:44:39Z'
author: Jack Hale
entity_revision: 1
";
        let result = enhance_new_entity_yaml(yaml, "CMP");

        assert!(result.contains("make_buy: buy  # make | buy"));
        assert!(result.contains(
            "category: mechanical  # mechanical | electrical | software | fastener | consumable"
        ));
        assert!(result.contains("# tags: []"));
        // Should NOT have # category: since CMP already has category in output
        assert!(!result.contains("# category:\n"));
        assert!(result.contains("# Use 'tdt cmp new -i'"));
    }

    #[test]
    fn test_no_duplicate_comments_when_fields_present() {
        // When user provides category and tags, don't add commented-out versions
        let yaml = "\
id: REQ-01KHGW5G60A980RWF528NK82F5
type: input
level: system
title: Motor Torque
category: performance
tags:
- motor
- torque
text: The system shall provide 50Nm of torque.
priority: medium
status: draft
links: {}
created: '2026-02-15T14:44:39Z'
author: Jack Hale
revision: 1
";
        let result = enhance_new_entity_yaml(yaml, "REQ");

        // Should NOT have commented-out category/tags since they're already present
        assert!(!result.contains("# category:"));
        assert!(!result.contains("# tags: []"));
        // Should still have inline hints
        assert!(result.contains("type: input  # input | output"));
        // Should NOT replace text with TODO since it has content
        assert!(result.contains("text: The system shall provide 50Nm of torque."));
    }

    #[test]
    fn test_result_template() {
        let yaml = "\
id: RSLT-01KHGW5G60A980RWF528NK82F5
test_id: TEST-01KHGW5G60A980RWF528NK82F5
verdict: pass
executed_date: '2026-02-15T14:44:39Z'
executed_by: Jack Hale
status: draft
links: {}
created: '2026-02-15T14:44:39Z'
author: Jack Hale
revision: 1
";
        let result = enhance_new_entity_yaml(yaml, "RSLT");

        assert!(result
            .contains("verdict: pass  # pass | fail | conditional | incomplete | not_applicable"));
        assert!(result.contains("# Use 'tdt rslt new -i'"));
    }

    #[test]
    fn test_lot_uses_custom_status() {
        let yaml = "\
id: LOT-01KHGW5G60A980RWF528NK82F5
title: Production Lot 1
lot_status: in_progress
status: draft
links: {}
created: '2026-02-15T14:44:39Z'
author: Jack Hale
revision: 1
";
        let result = enhance_new_entity_yaml(yaml, "LOT");

        assert!(result
            .contains("lot_status: in_progress  # in_progress | on_hold | completed | scrapped"));
        // LOT should NOT have the common status hint (it has its own status semantics)
        assert!(!result.contains("status: draft  # draft"));
    }

    #[test]
    fn test_hazard_template() {
        let yaml = "\
id: HAZ-01KHGW5G60A980RWF528NK82F5
title: High Voltage
category: electrical
description: ''
severity: minor
status: draft
links: {}
created: '2026-02-15T14:44:39Z'
author: Jack Hale
revision: 1
";
        let result = enhance_new_entity_yaml(yaml, "HAZ");

        assert!(result.contains("category: electrical  #"));
        assert!(result
            .contains("severity: minor  # negligible | minor | serious | severe | catastrophic"));
        assert!(result.contains("description: 'TODO: Describe this hazard'"));
    }

    #[test]
    fn test_output_ends_with_newline() {
        let yaml = "id: REQ-123\nstatus: draft\n";
        let result = enhance_new_entity_yaml(yaml, "REQ");
        assert!(result.ends_with('\n'));
    }

    #[test]
    fn test_prefix_to_command() {
        assert_eq!(prefix_to_command("REQ"), "req");
        assert_eq!(prefix_to_command("TEST"), "test");
        assert_eq!(prefix_to_command("CMP"), "cmp");
        assert_eq!(prefix_to_command("UNKNOWN"), "entity");
    }
}
