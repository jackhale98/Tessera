//! Supplier entity commands
//!
//! Provides commands for managing suppliers and vendor information.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::entities::supplier::{Capability, Currency, Supplier};
use tdt_core::services::common::SortDirection;
use tdt_core::services::supplier::{
    CreateSupplier, SupplierFilter, SupplierService, SupplierSortField, SupplierStats, UpdateSupplier,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Supplier summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct SupplierSummary {
    pub id: String,
    pub name: String,
    pub short_name: Option<String>,
    pub website: Option<String>,
    pub capabilities: Vec<String>,
    pub currency: String,
    pub certification_count: usize,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Supplier> for SupplierSummary {
    fn from(s: &Supplier) -> Self {
        Self {
            id: s.id.to_string(),
            name: s.name.clone(),
            short_name: s.short_name.clone(),
            website: s.website.clone(),
            capabilities: s.capabilities.iter().map(|c| c.to_string()).collect(),
            currency: s.currency.to_string(),
            certification_count: s.certifications.len(),
            status: format!("{:?}", s.status).to_lowercase(),
            author: s.author.clone(),
            created: s.created.to_rfc3339(),
        }
    }
}

/// List result
#[derive(Debug, Clone, Serialize)]
pub struct ListSuppliersResult {
    pub items: Vec<SupplierSummary>,
    pub total_count: usize,
}

// ============================================================================
// Input DTOs
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListSuppliersParams {
    pub status: Option<Vec<String>>,
    pub capability: Option<String>,
    pub expired_certs: Option<bool>,
    pub certs_expiring_days: Option<i64>,
    pub currency: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateSupplierInput {
    pub name: String,
    pub author: String,
    pub short_name: Option<String>,
    pub website: Option<String>,
    pub payment_terms: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub capabilities: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateSupplierInput {
    pub name: Option<String>,
    pub short_name: Option<String>,
    pub website: Option<String>,
    pub payment_terms: Option<String>,
    pub currency: Option<String>,
    pub notes: Option<String>,
    pub status: Option<String>,
    pub tags: Option<Vec<String>>,
}

// ============================================================================
// Conversion helpers
// ============================================================================

fn parse_status(s: &str) -> Option<Status> {
    match s.to_lowercase().as_str() {
        "draft" => Some(Status::Draft),
        "review" => Some(Status::Review),
        "approved" => Some(Status::Approved),
        "released" => Some(Status::Released),
        "obsolete" => Some(Status::Obsolete),
        _ => None,
    }
}

fn parse_capability(s: &str) -> Option<Capability> {
    match s.to_lowercase().as_str() {
        "machining" => Some(Capability::Machining),
        "sheet_metal" => Some(Capability::SheetMetal),
        "casting" => Some(Capability::Casting),
        "injection" => Some(Capability::Injection),
        "extrusion" => Some(Capability::Extrusion),
        "pcb" => Some(Capability::Pcb),
        "pcb_assembly" => Some(Capability::PcbAssembly),
        "cable_assembly" => Some(Capability::CableAssembly),
        "assembly" => Some(Capability::Assembly),
        "testing" => Some(Capability::Testing),
        "finishing" => Some(Capability::Finishing),
        "packaging" => Some(Capability::Packaging),
        _ => None,
    }
}

fn parse_currency(s: &str) -> Option<Currency> {
    match s.to_uppercase().as_str() {
        "USD" => Some(Currency::Usd),
        "EUR" => Some(Currency::Eur),
        "GBP" => Some(Currency::Gbp),
        "JPY" => Some(Currency::Jpy),
        "CNY" => Some(Currency::Cny),
        _ => None,
    }
}

fn parse_sort_field(s: &str) -> SupplierSortField {
    match s.to_lowercase().as_str() {
        "id" => SupplierSortField::Id,
        "name" => SupplierSortField::Name,
        "short_name" => SupplierSortField::ShortName,
        "website" => SupplierSortField::Website,
        "capabilities" => SupplierSortField::Capabilities,
        "status" => SupplierSortField::Status,
        "author" => SupplierSortField::Author,
        "created" => SupplierSortField::Created,
        _ => SupplierSortField::Name,
    }
}

fn build_supplier_filter(params: &ListSuppliersParams) -> SupplierFilter {
    use tdt_core::services::common::CommonFilter;

    let common = CommonFilter {
        status: params.status.as_ref().and_then(|v| {
            let statuses: Vec<Status> = v.iter().filter_map(|s| parse_status(s)).collect();
            if statuses.is_empty() { None } else { Some(statuses) }
        }),
        search: params.search.clone(),
        limit: params.limit,
        ..Default::default()
    };

    SupplierFilter {
        common,
        capability: params.capability.as_ref().and_then(|c| parse_capability(c)),
        expired_certs: params.expired_certs.unwrap_or(false),
        certs_expiring_days: params.certs_expiring_days,
        currency: params.currency.as_ref().and_then(|c| parse_currency(c)),
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn list_suppliers(
    params: Option<ListSuppliersParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListSuppliersResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = SupplierService::new(project, cache);
    let params = params.unwrap_or_default();
    let filter = build_supplier_filter(&params);

    let sort = params.sort_by.as_ref().map(|s| parse_sort_field(s)).unwrap_or_default();
    let sort_direction = if params.sort_desc.unwrap_or(false) {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };

    let suppliers = service.list(&filter, sort, sort_direction)?;

    Ok(ListSuppliersResult {
        total_count: suppliers.items.len(),
        items: suppliers.items.iter().map(SupplierSummary::from).collect(),
    })
}

#[tauri::command]
pub async fn get_supplier(id: String, state: State<'_, AppState>) -> CommandResult<Option<Supplier>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = SupplierService::new(project, cache);
    Ok(service.get(&id)?)
}

#[tauri::command]
pub async fn create_supplier(input: CreateSupplierInput, state: State<'_, AppState>) -> CommandResult<Supplier> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let supplier = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = SupplierService::new(project, cache);

        let capabilities: Vec<Capability> = input.capabilities
            .unwrap_or_default()
            .iter()
            .filter_map(|c| parse_capability(c))
            .collect();

        let create = CreateSupplier {
            name: input.name,
            author: input.author,
            short_name: input.short_name,
            website: input.website,
            payment_terms: input.payment_terms,
            currency: input.currency.and_then(|c| parse_currency(&c)).unwrap_or_default(),
            notes: input.notes,
            capabilities,
            tags: input.tags.unwrap_or_default(),
        };
        service.create(create)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(supplier)
}

#[tauri::command]
pub async fn update_supplier(id: String, input: UpdateSupplierInput, state: State<'_, AppState>) -> CommandResult<Supplier> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let supplier = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = SupplierService::new(project, cache);

        let update = UpdateSupplier {
            name: input.name,
            short_name: input.short_name,
            website: input.website,
            payment_terms: input.payment_terms,
            currency: input.currency.and_then(|c| parse_currency(&c)),
            notes: input.notes,
            capabilities: None,
            status: input.status.and_then(|s| parse_status(&s)),
            tags: input.tags,
        };
        service.update(&id, update)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(supplier)
}

#[tauri::command]
pub async fn delete_supplier(id: String, force: Option<bool>, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = SupplierService::new(project, cache);
        service.delete(&id, force.unwrap_or(false))?;
    }

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(())
}

#[tauri::command]
pub async fn get_supplier_stats(state: State<'_, AppState>) -> CommandResult<SupplierStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = SupplierService::new(project, cache);
    Ok(service.stats()?)
}
