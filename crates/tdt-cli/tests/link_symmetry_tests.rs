//! Link order symmetry tests
//!
//! Verifies that `tdt link add A B` and `tdt link add B A` both produce
//! the correct forward and reciprocal links, regardless of argument order.

mod common;

use common::{
    create_test_capa, create_test_component, create_test_control, create_test_ncr,
    create_test_process, create_test_protocol, create_test_requirement, create_test_risk,
    setup_test_project, tdt,
};

/// Helper: verify that link add produces both forward and reciprocal links
fn assert_link_symmetry(
    tmp: &tempfile::TempDir,
    source_short: &str,
    target_short: &str,
    expected_forward: &str,
    expected_reciprocal: &str,
) {
    let output = tdt()
        .current_dir(tmp.path())
        .args(["link", "add", source_short, target_short])
        .output()
        .unwrap();

    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.contains("Added link") && stdout.contains(expected_forward),
        "Expected forward link type '{}' in output: {}",
        expected_forward,
        stdout
    );
    assert!(
        stdout.contains("Added reciprocal") && stdout.contains(expected_reciprocal),
        "Expected reciprocal link type '{}' in output: {}",
        expected_reciprocal,
        stdout
    );
}

// ============================================================================
// REQ <-> TEST
// ============================================================================

#[test]
fn test_link_order_symmetry_req_test() {
    let tmp = setup_test_project();
    create_test_requirement(&tmp, "Spec", "input");
    create_test_protocol(&tmp, "Verify", "verification");

    assert_link_symmetry(&tmp, "REQ@1", "TEST@1", "verified_by", "verifies");
}

#[test]
fn test_link_order_symmetry_test_req() {
    let tmp = setup_test_project();
    create_test_requirement(&tmp, "Spec", "input");
    create_test_protocol(&tmp, "Verify", "verification");

    assert_link_symmetry(&tmp, "TEST@1", "REQ@1", "verifies", "verified_by");
}

// ============================================================================
// CMP <-> RISK
// ============================================================================

#[test]
fn test_link_order_symmetry_cmp_risk() {
    let tmp = setup_test_project();
    create_test_component(&tmp, "P-001", "Widget");
    create_test_risk(&tmp, "Failure", "design");

    assert_link_symmetry(&tmp, "CMP@1", "RISK@1", "risks", "component");
}

#[test]
fn test_link_order_symmetry_risk_cmp() {
    let tmp = setup_test_project();
    create_test_component(&tmp, "P-001", "Widget");
    create_test_risk(&tmp, "Failure", "design");

    assert_link_symmetry(&tmp, "RISK@1", "CMP@1", "component", "risks");
}

// ============================================================================
// NCR <-> CAPA
// ============================================================================

#[test]
fn test_link_order_symmetry_ncr_capa() {
    let tmp = setup_test_project();
    create_test_ncr(&tmp, "Defect");
    create_test_capa(&tmp, "Fix");

    assert_link_symmetry(&tmp, "NCR@1", "CAPA@1", "capa", "ncrs");
}

#[test]
fn test_link_order_symmetry_capa_ncr() {
    let tmp = setup_test_project();
    create_test_ncr(&tmp, "Defect");
    create_test_capa(&tmp, "Fix");

    assert_link_symmetry(&tmp, "CAPA@1", "NCR@1", "ncrs", "capa");
}

// ============================================================================
// CTRL <-> PROC
// ============================================================================

#[test]
fn test_link_order_symmetry_ctrl_proc() {
    let tmp = setup_test_project();
    create_test_process(&tmp, "Assembly", "assembly");
    create_test_control(&tmp, "Inspect");

    assert_link_symmetry(&tmp, "CTRL@1", "PROC@1", "process", "controls");
}

#[test]
fn test_link_order_symmetry_proc_ctrl() {
    let tmp = setup_test_project();
    create_test_process(&tmp, "Assembly", "assembly");
    create_test_control(&tmp, "Inspect");

    assert_link_symmetry(&tmp, "PROC@1", "CTRL@1", "controls", "process");
}

// ============================================================================
// CTRL <-> RISK (mitigates / mitigated_by)
// ============================================================================

#[test]
fn test_link_order_symmetry_ctrl_risk() {
    let tmp = setup_test_project();
    create_test_control(&tmp, "Inspect");
    create_test_risk(&tmp, "Failure", "design");

    assert_link_symmetry(&tmp, "CTRL@1", "RISK@1", "mitigates", "mitigated_by");
}

#[test]
fn test_link_order_symmetry_risk_ctrl() {
    let tmp = setup_test_project();
    create_test_control(&tmp, "Inspect");
    create_test_risk(&tmp, "Failure", "design");

    assert_link_symmetry(&tmp, "RISK@1", "CTRL@1", "mitigated_by", "mitigates");
}

// ============================================================================
// CAPA <-> PROC (processes_modified / modified_by_capa)
// ============================================================================

#[test]
fn test_link_order_symmetry_capa_proc() {
    let tmp = setup_test_project();
    create_test_capa(&tmp, "Fix");
    create_test_process(&tmp, "Assembly", "assembly");

    assert_link_symmetry(
        &tmp,
        "CAPA@1",
        "PROC@1",
        "processes_modified",
        "modified_by_capa",
    );
}

#[test]
fn test_link_order_symmetry_proc_capa() {
    let tmp = setup_test_project();
    create_test_capa(&tmp, "Fix");
    create_test_process(&tmp, "Assembly", "assembly");

    assert_link_symmetry(
        &tmp,
        "PROC@1",
        "CAPA@1",
        "modified_by_capa",
        "processes_modified",
    );
}
