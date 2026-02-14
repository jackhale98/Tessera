//! SysML v2 export - generates SysML textual notation from TDT entities

use std::collections::HashMap;
use std::fmt::Write;

use crate::entities::component::Component;
use crate::entities::requirement::Requirement;
use crate::entities::result::Result as TestResult;
use crate::entities::test::Test;
use crate::sysml::mapping::{
    build_name_map, escape_sysml_doc, method_to_sysml, title_to_sysml_name, verdict_to_sysml,
};
// model types are not currently needed directly in export

/// Context for SysML export containing all loaded entities
pub struct ExportContext {
    pub package_name: String,
    pub requirements: Vec<Requirement>,
    pub tests: Vec<Test>,
    pub results: Vec<TestResult>,
    pub components: Vec<Component>,
}

/// Generate SysML v2 textual notation from loaded TDT entities.
pub fn generate_sysml(ctx: &ExportContext) -> String {
    let mut out = String::new();

    // Build name maps for all entity types
    let req_entries: Vec<(String, String)> = ctx
        .requirements
        .iter()
        .map(|r| (r.id.to_string(), r.title.clone()))
        .collect();
    let test_entries: Vec<(String, String)> = ctx
        .tests
        .iter()
        .map(|t| (t.id.to_string(), t.title.clone()))
        .collect();
    let cmp_entries: Vec<(String, String)> = ctx
        .components
        .iter()
        .map(|c| (c.id.to_string(), c.title.clone()))
        .collect();

    let req_names = build_name_map(&req_entries);
    let test_names = build_name_map(&test_entries);
    let cmp_names = build_name_map(&cmp_entries);

    // Build reverse maps: SysML name -> TDT ID
    let req_id_to_name: HashMap<String, String> = req_names.clone();
    let cmp_id_to_name: HashMap<String, String> = cmp_names.clone();

    // Build test->result lookup (latest result per test)
    let mut latest_results: HashMap<String, &TestResult> = HashMap::new();
    for result in &ctx.results {
        let test_id = result.test_id.to_string();
        if let Some(existing) = latest_results.get(&test_id) {
            if result.executed_date > existing.executed_date {
                latest_results.insert(test_id, result);
            }
        } else {
            latest_results.insert(test_id, result);
        }
    }

    // Package header
    let _ = writeln!(out, "package {} {{", ctx.package_name);
    let _ = writeln!(out, "    import Requirements::*;");
    let _ = writeln!(out, "    import VerificationCases::*;");
    let _ = writeln!(out);

    // Emit requirement definitions
    for req in &ctx.requirements {
        let id_str = req.id.to_string();
        let name = req_names
            .get(&id_str)
            .cloned()
            .unwrap_or_else(|| title_to_sysml_name(&req.title));
        emit_requirement(&mut out, req, &name);
        let _ = writeln!(out);
    }

    // Emit part definitions for components
    for cmp in &ctx.components {
        let id_str = cmp.id.to_string();
        let name = cmp_names
            .get(&id_str)
            .cloned()
            .unwrap_or_else(|| title_to_sysml_name(&cmp.title));
        emit_part_def(&mut out, cmp, &name);
        let _ = writeln!(out);
    }

    // Emit verification definitions
    for test in &ctx.tests {
        let id_str = test.id.to_string();
        let name = test_names
            .get(&id_str)
            .cloned()
            .unwrap_or_else(|| title_to_sysml_name(&test.title));
        let latest_result = latest_results.get(&id_str).copied();
        emit_verification(&mut out, test, &name, &req_id_to_name, latest_result);
        let _ = writeln!(out);
    }

    // Emit satisfy relationships from requirement satisfied_by links
    let mut has_satisfy = false;
    for req in &ctx.requirements {
        let req_id_str = req.id.to_string();
        let req_name = req_names
            .get(&req_id_str)
            .cloned()
            .unwrap_or_else(|| title_to_sysml_name(&req.title));

        for satisfied_by_id in &req.links.satisfied_by {
            let sb_id_str = satisfied_by_id.to_string();
            // Check if it's a component
            if let Some(cmp_name) = cmp_id_to_name.get(&sb_id_str) {
                if !has_satisfy {
                    let _ = writeln!(out, "    // Satisfy relationships");
                    has_satisfy = true;
                }
                let _ = writeln!(
                    out,
                    "    satisfy requirement : {} by {};",
                    req_name, cmp_name
                );
            }
        }
    }

    if has_satisfy {
        let _ = writeln!(out);
    }

    let _ = writeln!(out, "}}");

    out
}

fn emit_requirement(out: &mut String, req: &Requirement, name: &str) {
    let id_str = req.id.to_string();
    let _ = writeln!(out, "    requirement def <'{}'> {} {{", id_str, name);

    // Doc block
    let doc_text = escape_sysml_doc(&req.text);
    let _ = writeln!(out, "        doc /* {} */", doc_text);

    // TDT metadata annotation
    let _ = write!(out, "        @TdtMetadata {{ ");
    let _ = write!(out, "status = {}; ", req.status);
    let _ = write!(out, "author = \"{}\"; ", req.author);
    let _ = write!(out, "priority = {}; ", req.priority);
    let _ = write!(out, "level = {}; ", req.level);
    let _ = write!(out, "req_type = {}; ", req.req_type);
    if let Some(ref cat) = req.category {
        let _ = write!(out, "category = \"{}\"; ", cat);
    }
    if let Some(ref rat) = req.rationale {
        let _ = write!(out, "rationale = \"{}\"; ", escape_sysml_doc(rat));
    }
    if !req.tags.is_empty() {
        let _ = write!(out, "tags = \"{}\"; ", req.tags.join(","));
    }
    let _ = writeln!(out, "}}");

    let _ = writeln!(out, "    }}");
}

fn emit_part_def(out: &mut String, cmp: &Component, name: &str) {
    let id_str = cmp.id.to_string();
    let desc = cmp.description.as_deref().unwrap_or(&cmp.title);

    if desc.is_empty() || desc == cmp.title {
        let _ = writeln!(out, "    part def <'{}'> {};", id_str, name);
    } else {
        let _ = writeln!(out, "    part def <'{}'> {} {{", id_str, name);
        let _ = writeln!(out, "        doc /* {} */", escape_sysml_doc(desc));
        let _ = writeln!(out, "    }}");
    }
}

fn emit_verification(
    out: &mut String,
    test: &Test,
    name: &str,
    req_id_to_name: &HashMap<String, String>,
    latest_result: Option<&TestResult>,
) {
    let id_str = test.id.to_string();
    let _ = writeln!(out, "    verification def <'{}'> {} {{", id_str, name);

    // Doc block
    let doc_text = escape_sysml_doc(&test.objective);
    let _ = writeln!(out, "        doc /* {} */", doc_text);

    // Verification method annotation
    if let Some(ref method) = test.test_method {
        let method_str = method.to_string();
        let sysml_method = method_to_sysml(&method_str);
        let _ = writeln!(
            out,
            "        @VerificationMethod {{ kind = VerificationMethodKind::{}; }}",
            sysml_method
        );
    }

    // TDT metadata
    let _ = write!(out, "        @TdtMetadata {{ ");
    let _ = write!(out, "status = {}; ", test.status);
    let _ = write!(out, "author = \"{}\"; ", test.author);
    let _ = writeln!(out, "}}");

    // Objective: verify requirement references
    if !test.links.verifies.is_empty() {
        let _ = writeln!(out, "        objective {{");
        for req_id in &test.links.verifies {
            let req_id_str = req_id.to_string();
            if let Some(req_name) = req_id_to_name.get(&req_id_str) {
                let _ = writeln!(out, "            verify requirement : {};", req_name);
            }
        }
        let _ = writeln!(out, "        }}");
    }

    let _ = writeln!(out, "        return verdict : VerdictKind;");

    // Emit latest result as structured comment
    if let Some(result) = latest_result {
        let verdict_string = result.verdict.to_string();
        let verdict_str = verdict_to_sysml(&verdict_string);
        let _ = writeln!(
            out,
            "        // Result: {} = {} (executed {} by {})",
            result.id,
            verdict_str,
            result.executed_date.format("%Y-%m-%d"),
            result.executed_by
        );
    }

    let _ = writeln!(out, "    }}");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_empty_package() {
        let ctx = ExportContext {
            package_name: "TestExport".to_string(),
            requirements: vec![],
            tests: vec![],
            results: vec![],
            components: vec![],
        };

        let output = generate_sysml(&ctx);
        assert!(output.contains("package TestExport {"));
        assert!(output.contains("import Requirements::*;"));
        assert!(output.contains("import VerificationCases::*;"));
        assert!(output.ends_with("}\n"));
    }

    #[test]
    fn test_generate_with_requirement() {
        let req = Requirement::new(
            crate::entities::requirement::RequirementType::Input,
            "Stroke Length".to_string(),
            "The actuator shall have a minimum stroke length of 100mm.".to_string(),
            "Jack Hale".to_string(),
        );

        let ctx = ExportContext {
            package_name: "TestExport".to_string(),
            requirements: vec![req],
            tests: vec![],
            results: vec![],
            components: vec![],
        };

        let output = generate_sysml(&ctx);
        assert!(output.contains("requirement def <'REQ-"));
        assert!(output.contains("StrokeLength"));
        assert!(
            output.contains("doc /* The actuator shall have a minimum stroke length of 100mm. */")
        );
        assert!(output.contains("@TdtMetadata"));
        assert!(output.contains("status = draft"));
    }

    #[test]
    fn test_generate_with_verification() {
        use crate::entities::test::{TestMethod, TestType};

        let mut test = Test::new(
            TestType::Verification,
            "Shaft Clearance Check".to_string(),
            "Verify shaft-bushing clearance fits within specification.".to_string(),
            "Jack Hale".to_string(),
        );
        test.test_method = Some(TestMethod::Inspection);

        let ctx = ExportContext {
            package_name: "TestExport".to_string(),
            requirements: vec![],
            tests: vec![test],
            results: vec![],
            components: vec![],
        };

        let output = generate_sysml(&ctx);
        assert!(output.contains("verification def <'TEST-"));
        assert!(output.contains("ShaftClearanceCheck"));
        assert!(output.contains("@VerificationMethod { kind = VerificationMethodKind::inspect; }"));
        assert!(output.contains("return verdict : VerdictKind;"));
    }
}
