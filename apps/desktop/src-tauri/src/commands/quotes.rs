//! Quote entity commands
//!
//! Provides commands for managing supplier quotations.

use serde::{Deserialize, Serialize};
use tauri::State;

use tdt_core::core::entity::Status;
use tdt_core::entities::quote::{Currency, Quote, QuoteStatus};
use tdt_core::services::common::SortDirection;
use tdt_core::services::quote::{
    CreateQuote, QuoteFilter, QuoteService, QuoteSortField, QuoteStats, UpdateQuote,
};

use crate::error::{CommandError, CommandResult};
use crate::state::AppState;

// ============================================================================
// Summary Types
// ============================================================================

/// Quote summary for list view
#[derive(Debug, Clone, Serialize)]
pub struct QuoteSummary {
    pub id: String,
    pub title: String,
    pub supplier: String,
    pub component: Option<String>,
    pub assembly: Option<String>,
    pub quote_ref: Option<String>,
    pub currency: String,
    pub moq: Option<u32>,
    pub lead_time_days: Option<u32>,
    pub quote_status: String,
    pub valid_until: Option<String>,
    pub status: String,
    pub author: String,
    pub created: String,
}

impl From<&Quote> for QuoteSummary {
    fn from(q: &Quote) -> Self {
        Self {
            id: q.id.to_string(),
            title: q.title.clone(),
            supplier: q.supplier.clone(),
            component: q.component.clone(),
            assembly: q.assembly.clone(),
            quote_ref: q.quote_ref.clone(),
            currency: q.currency.to_string(),
            moq: q.moq,
            lead_time_days: q.lead_time_days,
            quote_status: q.quote_status.to_string(),
            valid_until: q.valid_until.map(|d| d.format("%Y-%m-%d").to_string()),
            status: format!("{:?}", q.status).to_lowercase(),
            author: q.author.clone(),
            created: q.created.to_rfc3339(),
        }
    }
}

/// List result
#[derive(Debug, Clone, Serialize)]
pub struct ListQuotesResult {
    pub items: Vec<QuoteSummary>,
    pub total_count: usize,
}

// ============================================================================
// Input DTOs
// ============================================================================

#[derive(Debug, Clone, Default, Deserialize)]
pub struct ListQuotesParams {
    pub status: Option<Vec<String>>,
    pub quote_status: Option<String>,
    pub supplier: Option<String>,
    pub component: Option<String>,
    pub assembly: Option<String>,
    pub expired_only: Option<bool>,
    pub has_price_breaks: Option<bool>,
    pub currency: Option<String>,
    pub search: Option<String>,
    pub sort_by: Option<String>,
    pub sort_desc: Option<bool>,
    pub limit: Option<usize>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CreateQuoteInput {
    pub title: String,
    pub author: String,
    pub supplier: String,
    pub component: Option<String>,
    pub assembly: Option<String>,
    pub quote_ref: Option<String>,
    pub description: Option<String>,
    pub currency: Option<String>,
    pub moq: Option<u32>,
    pub lead_time_days: Option<u32>,
    pub tooling_cost: Option<f64>,
    pub tags: Option<Vec<String>>,
}

#[derive(Debug, Clone, Default, Deserialize)]
pub struct UpdateQuoteInput {
    pub title: Option<String>,
    pub quote_ref: Option<String>,
    pub description: Option<String>,
    pub currency: Option<String>,
    pub moq: Option<u32>,
    pub lead_time_days: Option<u32>,
    pub tooling_cost: Option<f64>,
    pub quote_status: Option<String>,
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

fn parse_quote_status(s: &str) -> Option<QuoteStatus> {
    match s.to_lowercase().as_str() {
        "pending" => Some(QuoteStatus::Pending),
        "received" => Some(QuoteStatus::Received),
        "accepted" => Some(QuoteStatus::Accepted),
        "rejected" => Some(QuoteStatus::Rejected),
        "expired" => Some(QuoteStatus::Expired),
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

fn parse_sort_field(s: &str) -> QuoteSortField {
    match s.to_lowercase().as_str() {
        "id" => QuoteSortField::Id,
        "title" => QuoteSortField::Title,
        "supplier" => QuoteSortField::Supplier,
        "component" => QuoteSortField::Component,
        "price" => QuoteSortField::Price,
        "quote_status" => QuoteSortField::QuoteStatus,
        "status" => QuoteSortField::Status,
        "author" => QuoteSortField::Author,
        "created" => QuoteSortField::Created,
        _ => QuoteSortField::Title,
    }
}

fn build_quote_filter(params: &ListQuotesParams) -> QuoteFilter {
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

    QuoteFilter {
        common,
        quote_status: params.quote_status.as_ref().and_then(|s| parse_quote_status(s)),
        supplier: params.supplier.clone(),
        component: params.component.clone(),
        assembly: params.assembly.clone(),
        expired_only: params.expired_only.unwrap_or(false),
        has_price_breaks: params.has_price_breaks.unwrap_or(false),
        currency: params.currency.as_ref().and_then(|c| parse_currency(c)),
    }
}

// ============================================================================
// Commands
// ============================================================================

#[tauri::command]
pub async fn list_quotes(
    params: Option<ListQuotesParams>,
    state: State<'_, AppState>,
) -> CommandResult<ListQuotesResult> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = QuoteService::new(project, cache);
    let params = params.unwrap_or_default();
    let filter = build_quote_filter(&params);

    let sort = params.sort_by.as_ref().map(|s| parse_sort_field(s)).unwrap_or_default();
    let sort_direction = if params.sort_desc.unwrap_or(false) {
        SortDirection::Descending
    } else {
        SortDirection::Ascending
    };

    let quotes = service.list(&filter, sort, sort_direction)?;

    Ok(ListQuotesResult {
        total_count: quotes.items.len(),
        items: quotes.items.iter().map(QuoteSummary::from).collect(),
    })
}

#[tauri::command]
pub async fn get_quote(id: String, state: State<'_, AppState>) -> CommandResult<Option<Quote>> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = QuoteService::new(project, cache);
    Ok(service.get(&id)?)
}

#[tauri::command]
pub async fn create_quote(input: CreateQuoteInput, state: State<'_, AppState>) -> CommandResult<Quote> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let quote = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = QuoteService::new(project, cache);

        let create = CreateQuote {
            title: input.title,
            author: input.author,
            supplier: input.supplier,
            component: input.component,
            assembly: input.assembly,
            quote_ref: input.quote_ref,
            description: input.description,
            currency: input.currency.and_then(|c| parse_currency(&c)).unwrap_or_default(),
            moq: input.moq,
            lead_time_days: input.lead_time_days,
            tooling_cost: input.tooling_cost,
            tags: input.tags.unwrap_or_default(),
        };
        service.create(create)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(quote)
}

#[tauri::command]
pub async fn update_quote(id: String, input: UpdateQuoteInput, state: State<'_, AppState>) -> CommandResult<Quote> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    let quote = {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = QuoteService::new(project, cache);

        let update = UpdateQuote {
            title: input.title,
            supplier: None,
            quote_ref: input.quote_ref,
            description: input.description,
            currency: input.currency.and_then(|c| parse_currency(&c)),
            moq: input.moq,
            lead_time_days: input.lead_time_days,
            tooling_cost: input.tooling_cost,
            quote_status: input.quote_status.and_then(|s| parse_quote_status(&s)),
            status: input.status.and_then(|s| parse_status(&s)),
            tags: input.tags,
        };
        service.update(&id, update)?
    };

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(quote)
}

#[tauri::command]
pub async fn delete_quote(id: String, force: Option<bool>, state: State<'_, AppState>) -> CommandResult<()> {
    let project_guard = state.project.lock().unwrap();
    let project = project_guard.as_ref().ok_or(CommandError::NoProject)?;

    {
        let cache_guard = state.cache.lock().unwrap();
        let cache = cache_guard.as_ref().ok_or(CommandError::NoProject)?;
        let service = QuoteService::new(project, cache);
        service.delete(&id, force.unwrap_or(false))?;
    }

    drop(project_guard);
    let mut cache_guard = state.cache.lock().unwrap();
    if let Some(cache) = cache_guard.as_mut() { let _ = cache.sync(); }

    Ok(())
}

#[tauri::command]
pub async fn get_quote_stats(state: State<'_, AppState>) -> CommandResult<QuoteStats> {
    let project = state.project.lock().unwrap();
    let cache = state.cache.lock().unwrap();
    let project = project.as_ref().ok_or(CommandError::NoProject)?;
    let cache = cache.as_ref().ok_or(CommandError::NoProject)?;

    let service = QuoteService::new(project, cache);
    Ok(service.stats()?)
}
