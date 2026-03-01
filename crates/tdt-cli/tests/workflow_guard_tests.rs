//! Integration tests for workflow guard enforcement
//!
//! Tests the unified workflow enforcement system: WorkflowGuard + WorkflowService,
//! `tdt check` command, pre-push hook installation, and signing method support.

mod common;

use common::{create_test_requirement, create_test_risk, setup_test_project, tdt};
use predicates::prelude::*;
use std::fs;
use tempfile::TempDir;

/// Helper to find entity file by ID prefix
fn find_entity_file(tmp: &TempDir, id: &str, prefix: &str) -> Option<std::path::PathBuf> {
    if !id.starts_with(prefix) {
        return None;
    }

    for entry in walkdir::WalkDir::new(tmp.path())
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(|e| e.path().to_string_lossy().ends_with(".tdt.yaml"))
    {
        if let Ok(content) = fs::read_to_string(entry.path()) {
            if content.contains(&format!("id: {}", id))
                || content.contains(&format!("id: \"{}\"", id))
            {
                return Some(entry.path().to_path_buf());
            }
        }
    }
    None
}

/// Helper: set up a project with workflow enabled, git initialized, and a team roster
fn setup_workflow_project() -> TempDir {
    let tmp = setup_test_project();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    // Enable workflow
    let config_path = tmp.path().join(".tdt/config.yaml");
    fs::write(
        &config_path,
        "workflow:\n  enabled: true\n  provider: none\n  auto_commit: true\n",
    )
    .unwrap();

    // Create team roster with Test User having engineering role
    let team_yaml = tmp.path().join(".tdt/team.yaml");
    fs::write(
        &team_yaml,
        "version: 1\nmembers:\n  - name: Test User\n    email: test@example.com\n    username: test\n    roles: [engineering]\n",
    )
    .unwrap();

    // Initial commit
    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    tmp
}

// ============================================================================
// Check Command Tests
// ============================================================================

#[test]
fn test_check_command_exists() {
    tdt()
        .args(["check", "--help"])
        .assert()
        .success()
        .stdout(predicate::str::contains("--branch"))
        .stdout(predicate::str::contains("--push-guard"));
}

#[test]
fn test_check_disabled_workflow() {
    let tmp = setup_test_project();

    // With default config (workflow disabled), check should succeed with info message
    tdt()
        .current_dir(tmp.path())
        .args(["check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("not enabled"));
}

#[test]
fn test_check_no_entities() {
    let tmp = setup_test_project();

    // Enable workflow with team roster
    let config_path = tmp.path().join(".tdt/config.yaml");
    fs::write(
        &config_path,
        "workflow:\n  enabled: true\n  provider: none\n",
    )
    .unwrap();

    let team_yaml = tmp.path().join(".tdt/team.yaml");
    fs::write(
        &team_yaml,
        "version: 1\nmembers:\n  - name: Test\n    email: t@x.com\n    username: t\n    roles: [engineering]\n",
    )
    .unwrap();

    tdt()
        .current_dir(tmp.path())
        .args(["check"])
        .assert()
        .success()
        .stdout(predicate::str::contains("No entity files"));
}

#[test]
fn test_check_detects_missing_approvals() {
    let tmp = setup_test_project();

    // Enable workflow with 2 required approvals for RISK
    let config_path = tmp.path().join(".tdt/config.yaml");
    fs::write(
        &config_path,
        "workflow:\n  enabled: true\n  provider: none\n  approvals:\n    RISK:\n      min_approvals: 2\n",
    )
    .unwrap();

    let team_yaml = tmp.path().join(".tdt/team.yaml");
    fs::write(
        &team_yaml,
        "version: 1\nmembers:\n  - name: Test\n    email: t@x.com\n    username: t\n    roles: [engineering]\n",
    )
    .unwrap();

    // Create a risk
    let risk_id = create_test_risk(&tmp, "Check Test Risk", "design");
    if risk_id.is_empty() {
        return;
    }

    // Set it to approved status with only 1 approval (needs 2)
    let risk_file = find_entity_file(&tmp, &risk_id, "RISK-");
    if let Some(path) = risk_file {
        let content = fs::read_to_string(&path).unwrap();
        let updated = content.replace(
            "status: draft",
            "status: approved\napprovals:\n  - approver: alice\n    role: engineering\n    timestamp: 2024-01-15T10:30:00Z",
        );
        fs::write(&path, updated).unwrap();

        // Check should detect violation: 1/2 approvals
        tdt()
            .current_dir(tmp.path())
            .args(["check"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("missing approvals"));
    }
}

#[test]
fn test_check_passes_with_sufficient_approvals() {
    let tmp = setup_test_project();

    // Enable workflow with 1 required approval for RISK
    let config_path = tmp.path().join(".tdt/config.yaml");
    fs::write(
        &config_path,
        "workflow:\n  enabled: true\n  provider: none\n  approvals:\n    RISK:\n      min_approvals: 1\n",
    )
    .unwrap();

    let team_yaml = tmp.path().join(".tdt/team.yaml");
    fs::write(
        &team_yaml,
        "version: 1\nmembers:\n  - name: Test\n    email: t@x.com\n    username: t\n    roles: [engineering]\n",
    )
    .unwrap();

    let risk_id = create_test_risk(&tmp, "Sufficient Approvals Risk", "design");
    if risk_id.is_empty() {
        return;
    }

    let risk_file = find_entity_file(&tmp, &risk_id, "RISK-");
    if let Some(path) = risk_file {
        let content = fs::read_to_string(&path).unwrap();
        let updated = content.replace(
            "status: draft",
            "status: approved\napprovals:\n  - approver: alice\n    role: engineering\n    timestamp: 2024-01-15T10:30:00Z",
        );
        fs::write(&path, updated).unwrap();

        // Check should pass: 1/1 approvals met
        tdt()
            .current_dir(tmp.path())
            .args(["check"])
            .assert()
            .success()
            .stdout(predicate::str::contains("pass"));
    }
}

#[test]
fn test_check_push_guard_mode_quiet_on_success() {
    let tmp = setup_test_project();

    // With workflow disabled, push-guard should produce no output
    let output = tdt()
        .current_dir(tmp.path())
        .args(["check", "--push-guard"])
        .output()
        .unwrap();

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(
        stdout.trim().is_empty(),
        "Expected empty stdout in push-guard mode, got: {}",
        stdout
    );
}

#[test]
fn test_check_push_guard_mode_fails_on_violations() {
    let tmp = setup_test_project();

    // Enable workflow with 2 required approvals
    let config_path = tmp.path().join(".tdt/config.yaml");
    fs::write(
        &config_path,
        "workflow:\n  enabled: true\n  provider: none\n  approvals:\n    RISK:\n      min_approvals: 2\n",
    )
    .unwrap();

    let team_yaml = tmp.path().join(".tdt/team.yaml");
    fs::write(
        &team_yaml,
        "version: 1\nmembers:\n  - name: Test\n    email: t@x.com\n    username: t\n    roles: [engineering]\n",
    )
    .unwrap();

    let risk_id = create_test_risk(&tmp, "Push Guard Fail Risk", "design");
    if risk_id.is_empty() {
        return;
    }

    let risk_file = find_entity_file(&tmp, &risk_id, "RISK-");
    if let Some(path) = risk_file {
        let content = fs::read_to_string(&path).unwrap();
        let updated = content.replace("status: draft", "status: approved");
        fs::write(&path, updated).unwrap();

        // Push-guard should fail with exit code
        tdt()
            .current_dir(tmp.path())
            .args(["check", "--push-guard"])
            .assert()
            .failure();
    }
}

// ============================================================================
// Approval Authorization Tests (via CLI approve command)
// ============================================================================

#[test]
fn test_approve_authorized_user_succeeds() {
    let tmp = setup_workflow_project();

    let req_id = create_test_requirement(&tmp, "Auth Approve Test", "input");
    if req_id.is_empty() {
        return;
    }

    let req_file = find_entity_file(&tmp, &req_id, "REQ-");
    if let Some(path) = req_file {
        let content = fs::read_to_string(&path).unwrap();
        let updated = content.replace("status: draft", "status: review");
        fs::write(&path, updated).unwrap();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Set to review"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Test User has engineering role, which is valid for REQ (no specific role required)
        tdt()
            .current_dir(tmp.path())
            .args(["approve", &req_id, "-y"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Recorded approval"));
    }
}

#[test]
fn test_approve_blocked_unauthorized_role() {
    let tmp = setup_workflow_project();

    // Require management or quality role for RISK approvals
    let team_yaml = tmp.path().join(".tdt/team.yaml");
    fs::write(
        &team_yaml,
        "version: 1\nmembers:\n  - name: Test User\n    email: test@example.com\n    username: test\n    roles: [engineering]\napproval_matrix:\n  RISK: [management, quality]\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "Update roster"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let risk_id = create_test_risk(&tmp, "Blocked Approve Test", "design");
    if risk_id.is_empty() {
        return;
    }

    let risk_file = find_entity_file(&tmp, &risk_id, "RISK-");
    if let Some(path) = risk_file {
        let content = fs::read_to_string(&path).unwrap();
        let updated = content.replace("status: draft", "status: review");
        fs::write(&path, updated).unwrap();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Set risk to review"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Should fail: engineering role doesn't match management/quality requirement
        tdt()
            .current_dir(tmp.path())
            .args(["approve", &risk_id, "-y"])
            .assert()
            .failure();
    }
}

#[test]
fn test_approve_force_bypasses_auth() {
    let tmp = setup_workflow_project();

    // Require management for RISK approvals
    let team_yaml = tmp.path().join(".tdt/team.yaml");
    fs::write(
        &team_yaml,
        "version: 1\nmembers:\n  - name: Test User\n    email: test@example.com\n    username: test\n    roles: [engineering]\napproval_matrix:\n  RISK: [management]\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "Update roster"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let risk_id = create_test_risk(&tmp, "Force Approve Test", "design");
    if risk_id.is_empty() {
        return;
    }

    let risk_file = find_entity_file(&tmp, &risk_id, "RISK-");
    if let Some(path) = risk_file {
        let content = fs::read_to_string(&path).unwrap();
        let updated = content.replace("status: draft", "status: review");
        fs::write(&path, updated).unwrap();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Set to review"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // --force should bypass authorization check
        tdt()
            .current_dir(tmp.path())
            .args(["approve", &risk_id, "-y", "--force"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Recorded approval"));
    }
}

#[test]
fn test_approve_without_roster_permits_all() {
    let tmp = setup_test_project();

    // Initialize git but do NOT create team roster (guard disabled)
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.email", "test@example.com"])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["config", "user.name", "Test User"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let config_path = tmp.path().join(".tdt/config.yaml");
    fs::write(
        &config_path,
        "workflow:\n  enabled: true\n  provider: none\n  auto_commit: true\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "Initial commit"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let req_id = create_test_requirement(&tmp, "No Roster Test", "input");
    if req_id.is_empty() {
        return;
    }

    let req_file = find_entity_file(&tmp, &req_id, "REQ-");
    if let Some(path) = req_file {
        let content = fs::read_to_string(&path).unwrap();
        let updated = content.replace("status: draft", "status: review");
        fs::write(&path, updated).unwrap();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Set to review"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Approve should succeed: no roster means guard disabled, no auth check
        tdt()
            .current_dir(tmp.path())
            .args(["approve", &req_id, "-y"])
            .assert()
            .success()
            .stdout(predicate::str::contains("Recorded approval"));
    }
}

// ============================================================================
// Signature Requirement Tests
// ============================================================================

#[test]
fn test_signature_required_blocks_unsigned_approval() {
    let tmp = setup_workflow_project();

    // Require signature for REQ approvals
    let config_path = tmp.path().join(".tdt/config.yaml");
    fs::write(
        &config_path,
        "workflow:\n  enabled: true\n  provider: none\n  auto_commit: true\n  approvals:\n    REQ:\n      min_approvals: 1\n      require_signature: true\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "Update config"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    let req_id = create_test_requirement(&tmp, "Sig Required Test", "input");
    if req_id.is_empty() {
        return;
    }

    let req_file = find_entity_file(&tmp, &req_id, "REQ-");
    if let Some(path) = req_file {
        let content = fs::read_to_string(&path).unwrap();
        let updated = content.replace("status: draft", "status: review");
        fs::write(&path, updated).unwrap();

        std::process::Command::new("git")
            .args(["add", "."])
            .current_dir(tmp.path())
            .output()
            .unwrap();
        std::process::Command::new("git")
            .args(["commit", "-m", "Set to review"])
            .current_dir(tmp.path())
            .output()
            .unwrap();

        // Approve without --sign should fail when signature is required
        tdt()
            .current_dir(tmp.path())
            .args(["approve", &req_id, "-y"])
            .assert()
            .failure()
            .stderr(predicate::str::contains("signature").or(predicate::str::contains("GPG")));
    }
}

// ============================================================================
// Deviation Workflow Guard Tests
// ============================================================================

#[test]
fn test_dev_approve_with_authorized_role() {
    let tmp = setup_workflow_project();

    // User has quality role which matches DEV requirement
    let team_yaml = tmp.path().join(".tdt/team.yaml");
    fs::write(
        &team_yaml,
        "version: 1\nmembers:\n  - name: Test User\n    email: test@example.com\n    username: test\n    roles: [quality]\napproval_matrix:\n  DEV: [quality]\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "Update roster"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    // Create a deviation
    tdt()
        .current_dir(tmp.path())
        .args([
            "dev",
            "new",
            "--title",
            "Workflow Dev Test",
            "--dev-type",
            "temporary",
            "--no-edit",
        ])
        .assert()
        .success();

    // List to register short IDs
    tdt()
        .current_dir(tmp.path())
        .args(["dev", "list"])
        .assert()
        .success();

    // Approve should succeed: quality role matches DEV requirement
    tdt()
        .current_dir(tmp.path())
        .args(["dev", "approve", "DEV@1", "-y"])
        .assert()
        .success();
}

#[test]
fn test_dev_approve_blocked_without_required_role() {
    let tmp = setup_workflow_project();

    // User has engineering but DEV requires management
    let team_yaml = tmp.path().join(".tdt/team.yaml");
    fs::write(
        &team_yaml,
        "version: 1\nmembers:\n  - name: Test User\n    email: test@example.com\n    username: test\n    roles: [engineering]\napproval_matrix:\n  DEV: [management]\n",
    )
    .unwrap();

    std::process::Command::new("git")
        .args(["add", "."])
        .current_dir(tmp.path())
        .output()
        .unwrap();
    std::process::Command::new("git")
        .args(["commit", "-m", "Update roster"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    tdt()
        .current_dir(tmp.path())
        .args([
            "dev",
            "new",
            "--title",
            "Blocked Dev Test",
            "--dev-type",
            "temporary",
            "--no-edit",
        ])
        .assert()
        .success();

    tdt()
        .current_dir(tmp.path())
        .args(["dev", "list"])
        .assert()
        .success();

    // Should fail: engineering doesn't match management
    tdt()
        .current_dir(tmp.path())
        .args(["dev", "approve", "DEV@1", "-y"])
        .assert()
        .failure()
        .stderr(predicate::str::contains("Authorization").or(predicate::str::contains("role")));
}

// ============================================================================
// Pre-push Hook Installation Test
// ============================================================================

#[test]
fn test_cache_install_hooks_includes_pre_push() {
    let tmp = setup_test_project();

    // Initialize git repo
    std::process::Command::new("git")
        .args(["init"])
        .current_dir(tmp.path())
        .output()
        .unwrap();

    // Install hooks
    tdt()
        .current_dir(tmp.path())
        .args(["cache", "install-hooks"])
        .assert()
        .success();

    // Verify pre-push hook was installed
    let hook_path = tmp.path().join(".git/hooks/pre-push");
    assert!(hook_path.exists(), "pre-push hook should be installed");

    let content = fs::read_to_string(&hook_path).unwrap();
    assert!(
        content.contains("tdt check"),
        "pre-push hook should contain 'tdt check'"
    );
    assert!(
        content.contains("--push-guard"),
        "pre-push hook should use --push-guard mode"
    );
}
