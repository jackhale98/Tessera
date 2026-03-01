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

pub mod assembly;
pub mod base;
pub mod capa;
pub mod common;
pub mod component;
pub mod control;
pub mod deviation;
pub mod feature;
pub mod hazard;
pub mod lot;
pub mod mate;
pub mod ncr;
pub mod process;
pub mod quote;
pub mod requirement;
pub mod result;
pub mod risk;
pub mod stackup;
pub mod supplier;
#[path = "test.rs"]
pub mod test_protocol;
pub mod traceability;
pub mod work_instruction;
pub mod workflow;
pub mod workflow_guard;

pub use assembly::{
    AssemblyFilter, AssemblyService, AssemblySortField, AssemblyStats, BomCostResult,
    BomCostResultDetailed, BomMassResult, BomNode, ComponentCostLine, CreateAssembly,
    UpdateAssembly,
};
pub use base::ServiceBase;
pub use capa::{
    AddActionInput, CapaFilter, CapaService, CapaSortField, CapaStats, CapaTypeCounts,
    CapaWorkflowCounts, CreateCapa, UpdateCapa,
};
pub use common::*;
pub use component::{
    BomCostSummary, ComponentFilter, ComponentService, ComponentSortField, ComponentStats,
    CreateComponent, UpdateComponent,
};
pub use control::{
    ControlCategoryCounts, ControlFilter, ControlService, ControlSortField, ControlStats,
    ControlTypeCounts, CreateControl, StatusCounts, UpdateControl,
};
pub use deviation::{
    CreateDeviation, DevStatusCounts, DeviationCategoryCounts, DeviationFilter, DeviationService,
    DeviationSortField, DeviationStats, DeviationTypeCounts, RiskLevelCounts, UpdateDeviation,
};
pub use feature::{
    CreateFeature, FeatureFilter, FeatureService, FeatureSortField, FeatureStats,
    FeatureStatusCounts, FeatureTypeCounts, UpdateFeature,
};
pub use hazard::{
    CreateHazard, HazardCategoryCounts, HazardFilter, HazardService, HazardSeverityCounts,
    HazardSortField, HazardStats, UpdateHazard,
};
pub use lot::{
    ApproveWiStepInput, CreateLot, ExecuteWiStepInput, LotFilter, LotService, LotSortField,
    LotStats, LotStatusCounts, UpdateLot, UpdateStepInput, WiStepExecutionResult,
};
pub use mate::{
    CreateMate, FitResultCounts, MateFilter, MateService, MateSortField, MateStats, MateTypeCounts,
    RecalcResult, UpdateMate,
};
pub use ncr::{
    CreateNcr, NcrCategoryCounts, NcrFilter, NcrService, NcrSeverityCounts, NcrSortField, NcrStats,
    NcrStatusCounts, NcrTypeCounts, UpdateNcr,
};
pub use process::{
    CreateProcess, ProcessFilter, ProcessService, ProcessSortField, ProcessStats, UpdateProcess,
};
pub use quote::{
    ComparedQuote, CreateQuote, EntityStatusCounts, QuoteComparison, QuoteFilter, QuoteService,
    QuoteSortField, QuoteStats, QuoteStatusCounts, UpdateQuote,
};
pub use requirement::{
    CreateRequirement, RequirementFilter, RequirementService, RequirementSortField,
    RequirementStats, UpdateRequirement,
};
pub use result::{
    CreateResult, ResultFilter, ResultService, ResultSortField, ResultStats, ResultStatusCounts,
    UpdateResult, VerdictCounts,
};
pub use risk::{
    CreateRisk, RiskFilter, RiskMatrix, RiskService, RiskSortField, RiskStats, UpdateRisk,
};
pub use stackup::{
    AddContributorInput, CreateStackup, DispositionCounts, ResultCounts, StackupFilter,
    StackupService, StackupSortField, StackupStats, UpdateStackup,
};
pub use supplier::{
    CapabilityCounts, CreateSupplier, SupplierFilter, SupplierService, SupplierSortField,
    SupplierStats, SupplierStatusCounts, UpdateSupplier,
};
pub use test_protocol::{
    CreateTest, RunTestInput, TestFilter, TestLevelCounts, TestMethodCounts, TestPriorityCounts,
    TestService, TestSortField, TestStats, TestStatusCounts, TestTypeCounts, UpdateTest,
};
pub use traceability::{
    BrokenLink, CoverageReport, CoverageStats, DesignStructureMatrix, DomainMappingMatrix,
    LinkValidationResult, TraceDirection, TraceLink, TraceOptions, TraceResult,
    TraceabilityService, TracedEntity,
};
pub use work_instruction::{
    CreateWorkInstruction, UpdateWorkInstruction, WorkInstructionFilter, WorkInstructionService,
    WorkInstructionSortField, WorkInstructionStats, WorkInstructionStatusCounts,
};
pub use workflow::{
    ApprovalResult, ApproveEntityInput, ReleaseEntityInput, RejectEntityInput, SubmitEntityInput,
    WorkflowService,
};
pub use workflow_guard::{AuthorizedUser, SignatureCheck, WorkflowGuard};
