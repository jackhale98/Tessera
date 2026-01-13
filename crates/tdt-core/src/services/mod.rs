//! Service layer for TDT business logic
//!
//! This module provides high-level services that encapsulate business logic
//! for entity management. Services are designed to be used by both the CLI
//! and GUI interfaces.
//!
//! # Architecture
//!
//! Each service takes references to [`Project`] and [`EntityCache`] and provides
//! CRUD operations plus domain-specific business logic.
//!
//! # Example
//!
//! ```ignore
//! use tdt_core::services::RequirementService;
//! use tdt_core::{Project, EntityCache};
//!
//! let project = Project::discover()?;
//! let cache = EntityCache::open(&project)?;
//! let service = RequirementService::new(&project, &cache);
//!
//! // List with filters
//! let reqs = service.list(&RequirementFilter::default())?;
//!
//! // Create new requirement
//! let req = service.create(CreateRequirement {
//!     title: "My Requirement".into(),
//!     req_type: RequirementType::Input,
//!     ..Default::default()
//! })?;
//! ```

pub mod common;
pub mod component;
pub mod requirement;
pub mod risk;
pub mod traceability;

pub use common::*;
pub use component::{
    BomCostSummary, ComponentFilter, ComponentService, ComponentSortField, ComponentStats,
    CreateComponent, UpdateComponent,
};
pub use requirement::{
    CreateRequirement, RequirementFilter, RequirementService, RequirementSortField,
    RequirementStats, UpdateRequirement,
};
pub use risk::{
    CreateRisk, RiskFilter, RiskMatrix, RiskService, RiskSortField, RiskStats, UpdateRisk,
};
pub use traceability::{
    CoverageReport, CoverageStats, DesignStructureMatrix, TraceDirection, TraceLink, TraceOptions,
    TraceResult, TraceabilityService, TracedEntity,
};
