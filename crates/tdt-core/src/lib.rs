//! TDT Core Library
//!
//! Core library for TDT (Tessera Design Toolkit) - a Unix-style toolkit for
//! managing engineering artifacts as plain text files under git version control.
//!
//! This crate provides the fundamental types, entity definitions, and business
//! logic that can be shared between the CLI, GUI, and API interfaces.
//!
//! # Modules
//!
//! - [`core`] - Fundamental types: Entity trait, IDs, Project, Config, Cache
//! - [`entities`] - Data model structs for all 20 entity types
//! - [`schema`] - JSON Schema validation and template generation
//! - [`yaml`] - YAML parsing utilities

pub mod core;
pub mod entities;
pub mod schema;
pub mod services;
pub mod yaml;

// Re-export core types for convenience
pub use crate::core::{
    bounds_approx_equal,
    check_stale_bounds,
    clear_link_suspect,
    compute_torsor_bounds,
    create_execution_steps_from_routing,
    get_suspect_links,
    has_suspect_links,
    mark_link_suspect,
    step_min_approvals,
    step_required_roles,
    step_requires_approval,
    step_requires_signature,
    // Cache
    CachedComponent,
    CachedEntity,
    CachedFeature,
    CachedLink,
    CachedQuote,
    CachedRequirement,
    CachedRisk,
    CachedSupplier,
    CachedTest,
    // Config
    Config,
    // Entity trait
    Entity,
    EntityCache,
    EntityFilter,
    // Identity
    EntityId,
    EntityPrefix,
    // Suspect
    ExtendedLinkRef,
    // GDT
    GdtTorsorResult,
    // Git
    Git,
    GitError,
    IdParseError,
    LinkRef,
    LinkType,
    // Manufacturing
    LotWorkflow,
    LotWorkflowConfig,
    // Provider
    PrInfo,
    PrState,
    // Project
    Project,
    ProjectError,
    Provider,
    ProviderClient,
    ProviderError,
    // Team
    Role,
    // ShortId
    ShortIdIndex,
    SuspectError,
    SuspectReason,
    SuspectSummary,
    SyncStats,
    TeamMember,
    TeamRoster,
    // Workflow
    WorkflowConfig,
    WorkflowEngine,
    WorkflowError,
};

// Re-export all entity types
pub use entities::{
    Asil, Assembly, Capa, Component, ComponentSupplier, Control, Dal, Dev, DimensionRef, Feature,
    Hazard, Lot, Mate, Ncr, Process, Quote, Requirement, Result, Risk, Stackup, Supplier, SwClass,
    Test, WorkInstruction,
};

// Re-export schema types
pub use schema::{SchemaRegistry, TemplateContext, TemplateGenerator, ValidationError, Validator};
