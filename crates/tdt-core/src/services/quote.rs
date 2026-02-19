//! Quote service - business logic for supplier quotation management
//!
//! Provides CRUD operations and price break management for quotes.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::core::cache::EntityCache;
use crate::core::entity::Status;
use crate::core::identity::{EntityId, EntityPrefix};
use crate::core::loader;
use crate::core::project::Project;
use crate::services::base::ServiceBase;
use crate::entities::quote::{Currency, NreCost, PriceBreak, Quote, QuoteLinks, QuoteStatus};

use super::common::{
    apply_pagination, CommonFilter, ListResult, ServiceError, ServiceResult, SortDirection,
};

/// Filter options specific to quotes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuoteFilter {
    /// Common filter options (status, author, search, etc.)
    #[serde(flatten)]
    pub common: CommonFilter,

    /// Filter by quote status
    pub quote_status: Option<QuoteStatus>,

    /// Filter by supplier ID
    pub supplier: Option<String>,

    /// Filter by component ID
    pub component: Option<String>,

    /// Filter by assembly ID
    pub assembly: Option<String>,

    /// Show only expired quotes
    pub expired_only: bool,

    /// Show only quotes with price breaks
    pub has_price_breaks: bool,

    /// Filter by currency
    pub currency: Option<Currency>,
}

impl QuoteFilter {
    /// Create a filter for quotes by supplier
    pub fn by_supplier(supplier_id: &str) -> Self {
        Self {
            supplier: Some(supplier_id.to_string()),
            ..Default::default()
        }
    }

    /// Create a filter for quotes by component
    pub fn by_component(component_id: &str) -> Self {
        Self {
            component: Some(component_id.to_string()),
            ..Default::default()
        }
    }

    /// Create a filter for quotes by assembly
    pub fn by_assembly(assembly_id: &str) -> Self {
        Self {
            assembly: Some(assembly_id.to_string()),
            ..Default::default()
        }
    }

    /// Create a filter for accepted quotes
    pub fn accepted() -> Self {
        Self {
            quote_status: Some(QuoteStatus::Accepted),
            ..Default::default()
        }
    }

    /// Create a filter for expired quotes
    pub fn expired() -> Self {
        Self {
            expired_only: true,
            ..Default::default()
        }
    }
}

/// Sort field for quotes
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuoteSortField {
    Id,
    #[default]
    Title,
    Supplier,
    Component,
    Price,
    QuoteStatus,
    Status,
    Author,
    Created,
}

/// Input for creating a new quote
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CreateQuote {
    /// Quote title
    pub title: String,

    /// Author name
    pub author: String,

    /// Supplier ID
    pub supplier: String,

    /// Component ID (mutually exclusive with assembly)
    #[serde(default)]
    pub component: Option<String>,

    /// Assembly ID (mutually exclusive with component)
    #[serde(default)]
    pub assembly: Option<String>,

    /// Supplier's quote reference number
    #[serde(default)]
    pub quote_ref: Option<String>,

    /// Description/notes
    #[serde(default)]
    pub description: Option<String>,

    /// Currency
    #[serde(default)]
    pub currency: Currency,

    /// Minimum order quantity
    #[serde(default)]
    pub moq: Option<u32>,

    /// Standard lead time in days
    #[serde(default)]
    pub lead_time_days: Option<u32>,

    /// Tooling cost
    #[serde(default)]
    pub tooling_cost: Option<f64>,

    /// Classification tags
    #[serde(default)]
    pub tags: Vec<String>,
}

impl Default for CreateQuote {
    fn default() -> Self {
        Self {
            title: String::new(),
            author: String::new(),
            supplier: String::new(),
            component: None,
            assembly: None,
            quote_ref: None,
            description: None,
            currency: Currency::default(),
            moq: None,
            lead_time_days: None,
            tooling_cost: None,
            tags: Vec::new(),
        }
    }
}

/// Input for updating an existing quote
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateQuote {
    /// Update title
    pub title: Option<String>,

    /// Update supplier
    pub supplier: Option<String>,

    /// Update quote reference
    pub quote_ref: Option<String>,

    /// Update description
    pub description: Option<String>,

    /// Update currency
    pub currency: Option<Currency>,

    /// Update MOQ
    pub moq: Option<u32>,

    /// Update lead time
    pub lead_time_days: Option<u32>,

    /// Update tooling cost
    pub tooling_cost: Option<f64>,

    /// Update quote status
    pub quote_status: Option<QuoteStatus>,

    /// Update entity status
    pub status: Option<Status>,

    /// Replace tags
    pub tags: Option<Vec<String>>,
}

/// Statistics about quotes
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuoteStats {
    pub total: usize,
    pub by_quote_status: QuoteStatusCounts,
    pub by_status: EntityStatusCounts,
    pub with_price_breaks: usize,
    pub with_nre: usize,
    pub expired: usize,
    pub for_components: usize,
    pub for_assemblies: usize,
}

/// Counts by quote status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct QuoteStatusCounts {
    pub pending: usize,
    pub received: usize,
    pub accepted: usize,
    pub rejected: usize,
    pub expired: usize,
}

/// Counts by entity status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EntityStatusCounts {
    pub draft: usize,
    pub review: usize,
    pub approved: usize,
    pub released: usize,
    pub obsolete: usize,
}

/// Result of quote comparison
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QuoteComparison {
    pub item_id: String,
    pub quantity: u32,
    pub quotes: Vec<ComparedQuote>,
    pub lowest_price_quote: Option<String>,
}

/// A quote in comparison results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComparedQuote {
    pub id: String,
    pub title: String,
    pub supplier: String,
    pub unit_price: Option<f64>,
    pub effective_price: Option<f64>,
    pub lead_time_days: Option<u32>,
    pub quote_status: QuoteStatus,
}

/// Service for quote management
pub struct QuoteService<'a> {
    project: &'a Project,
    cache: &'a EntityCache,
    base: ServiceBase<'a>,
}

impl<'a> QuoteService<'a> {
    /// Create a new quote service
    pub fn new(project: &'a Project, cache: &'a EntityCache) -> Self {
        Self {
            project,
            cache,
            base: ServiceBase::new(project, cache),
        }
    }

    /// Get the directory for storing quotes
    fn get_directory(&self) -> PathBuf {
        self.project.root().join("bom/quotes")
    }

    /// Get the file path for a quote
    fn get_file_path(&self, id: &EntityId) -> PathBuf {
        self.get_directory().join(format!("{}.tdt.yaml", id))
    }

    /// List quotes with filtering and pagination
    pub fn list(
        &self,
        filter: &QuoteFilter,
        sort_by: QuoteSortField,
        sort_dir: SortDirection,
    ) -> ServiceResult<ListResult<Quote>> {
        let mut quotes = self.load_all()?;

        // Apply filters
        quotes.retain(|q| self.matches_filter(q, filter));

        // Sort
        self.sort_quotes(&mut quotes, sort_by, sort_dir);

        // Paginate
        Ok(apply_pagination(
            quotes,
            filter.common.offset,
            filter.common.limit,
        ))
    }

    /// List quotes from cache (fast path for list display)
    ///
    /// Returns cached quote data without loading full YAML files.
    /// Use this for list commands where full entity data isn't needed.
    pub fn list_cached(&self) -> Vec<crate::core::CachedQuote> {
        self.cache
            .list_quotes(None, None, None, None, None, None, None)
    }

    /// Load all quotes from the filesystem
    pub fn load_all(&self) -> ServiceResult<Vec<Quote>> {
        let dir = self.get_directory();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        Ok(loader::load_all(&dir)?)
    }

    /// Get a single quote by ID
    pub fn get(&self, id: &str) -> ServiceResult<Option<Quote>> {
        let dir = self.get_directory();
        if let Some((_, quote)) = loader::load_entity::<Quote>(&dir, id)? {
            return Ok(Some(quote));
        }
        Ok(None)
    }

    /// Get a quote by ID, returning an error if not found
    pub fn get_required(&self, id: &str) -> ServiceResult<Quote> {
        self.get(id)?
            .ok_or_else(|| ServiceError::NotFound(id.to_string()))
    }

    /// Get quotes for a specific component
    pub fn get_by_component(&self, component_id: &str) -> ServiceResult<Vec<Quote>> {
        let quotes = self.load_all()?;
        Ok(quotes
            .into_iter()
            .filter(|q| {
                q.component
                    .as_ref()
                    .is_some_and(|c| c.contains(component_id))
            })
            .collect())
    }

    /// Get quotes for a specific assembly
    pub fn get_by_assembly(&self, assembly_id: &str) -> ServiceResult<Vec<Quote>> {
        let quotes = self.load_all()?;
        Ok(quotes
            .into_iter()
            .filter(|q| q.assembly.as_ref().is_some_and(|a| a.contains(assembly_id)))
            .collect())
    }

    /// Get quotes from a specific supplier
    pub fn get_by_supplier(&self, supplier_id: &str) -> ServiceResult<Vec<Quote>> {
        let quotes = self.load_all()?;
        Ok(quotes
            .into_iter()
            .filter(|q| q.supplier.contains(supplier_id))
            .collect())
    }

    /// Create a new quote
    pub fn create(&self, input: CreateQuote) -> ServiceResult<Quote> {
        // Validate: must have either component or assembly, not both
        if input.component.is_none() && input.assembly.is_none() {
            return Err(ServiceError::InvalidInput(
                "Either component or assembly must be specified".to_string(),
            ));
        }
        if input.component.is_some() && input.assembly.is_some() {
            return Err(ServiceError::InvalidInput(
                "Cannot specify both component and assembly".to_string(),
            ));
        }

        let id = EntityId::new(EntityPrefix::Quot);

        let quote = Quote {
            id: id.clone(),
            title: input.title,
            supplier: input.supplier,
            component: input.component,
            assembly: input.assembly,
            quote_ref: input.quote_ref,
            description: input.description,
            currency: input.currency,
            price_breaks: Vec::new(),
            moq: input.moq,
            tooling_cost: input.tooling_cost,
            nre_costs: Vec::new(),
            lead_time_days: input.lead_time_days,
            quote_date: None,
            valid_until: None,
            quote_status: QuoteStatus::Pending,
            tags: input.tags,
            status: Status::Draft,
            links: QuoteLinks::default(),
            created: Utc::now(),
            author: input.author,
            entity_revision: 1,
        };

        // Write to file
        let path = self.get_file_path(&id);
        self.base.save(&quote, &path, Some("QUOT"))?;

        Ok(quote)
    }

    /// Update an existing quote
    pub fn update(&self, id: &str, input: UpdateQuote) -> ServiceResult<Quote> {
        let (path, mut quote) = self.find_quote(id)?;

        // Apply updates
        if let Some(title) = input.title {
            quote.title = title;
        }
        if let Some(supplier) = input.supplier {
            quote.supplier = supplier;
        }
        if let Some(quote_ref) = input.quote_ref {
            quote.quote_ref = Some(quote_ref);
        }
        if let Some(description) = input.description {
            quote.description = Some(description);
        }
        if let Some(currency) = input.currency {
            quote.currency = currency;
        }
        if let Some(moq) = input.moq {
            quote.moq = Some(moq);
        }
        if let Some(lead_time_days) = input.lead_time_days {
            quote.lead_time_days = Some(lead_time_days);
        }
        if let Some(tooling_cost) = input.tooling_cost {
            quote.tooling_cost = Some(tooling_cost);
        }
        if let Some(quote_status) = input.quote_status {
            quote.quote_status = quote_status;
        }
        if let Some(status) = input.status {
            quote.status = status;
        }
        if let Some(tags) = input.tags {
            quote.tags = tags;
        }

        // Increment revision
        quote.entity_revision += 1;

        // Write back
        self.base.save(&quote, &path, None)?;

        Ok(quote)
    }

    /// Delete a quote
    pub fn delete(&self, id: &str, force: bool) -> ServiceResult<()> {
        let (path, quote) = self.find_quote(id)?;

        // Check for references unless force is true
        if !force && !quote.links.related_quotes.is_empty() {
            return Err(ServiceError::HasReferences);
        }

        // Delete the file
        fs::remove_file(&path)?;

        Ok(())
    }

    /// Add a price break to a quote
    pub fn add_price_break(
        &self,
        id: &str,
        min_qty: u32,
        unit_price: f64,
        lead_time_days: Option<u32>,
    ) -> ServiceResult<Quote> {
        let (path, mut quote) = self.find_quote(id)?;

        quote.price_breaks.push(PriceBreak {
            min_qty,
            unit_price,
            lead_time_days,
        });

        // Sort price breaks by quantity
        quote.price_breaks.sort_by_key(|pb| pb.min_qty);

        quote.entity_revision += 1;

        self.base.save(&quote, &path, None)?;

        Ok(quote)
    }

    /// Remove a price break from a quote by min_qty
    pub fn remove_price_break(&self, id: &str, min_qty: u32) -> ServiceResult<Quote> {
        let (path, mut quote) = self.find_quote(id)?;

        let initial_len = quote.price_breaks.len();
        quote.price_breaks.retain(|pb| pb.min_qty != min_qty);

        if quote.price_breaks.len() == initial_len {
            return Err(ServiceError::NotFound(format!(
                "Price break for qty {} not found",
                min_qty
            )));
        }

        quote.entity_revision += 1;

        self.base.save(&quote, &path, None)?;

        Ok(quote)
    }

    /// Add an NRE cost to a quote
    pub fn add_nre_cost(&self, id: &str, nre: NreCost) -> ServiceResult<Quote> {
        let (path, mut quote) = self.find_quote(id)?;

        quote.nre_costs.push(nre);
        quote.entity_revision += 1;

        self.base.save(&quote, &path, None)?;

        Ok(quote)
    }

    /// Remove an NRE cost from a quote by description
    pub fn remove_nre_cost(&self, id: &str, description: &str) -> ServiceResult<Quote> {
        let (path, mut quote) = self.find_quote(id)?;

        let initial_len = quote.nre_costs.len();
        quote.nre_costs.retain(|n| n.description != description);

        if quote.nre_costs.len() == initial_len {
            return Err(ServiceError::NotFound(format!(
                "NRE cost '{}' not found",
                description
            )));
        }

        quote.entity_revision += 1;

        self.base.save(&quote, &path, None)?;

        Ok(quote)
    }

    /// Set quote status (accept, reject, etc.)
    pub fn set_quote_status(&self, id: &str, status: QuoteStatus) -> ServiceResult<Quote> {
        let (path, mut quote) = self.find_quote(id)?;

        quote.quote_status = status;
        quote.entity_revision += 1;

        self.base.save(&quote, &path, None)?;

        Ok(quote)
    }

    /// Accept a quote
    pub fn accept(&self, id: &str) -> ServiceResult<Quote> {
        self.set_quote_status(id, QuoteStatus::Accepted)
    }

    /// Reject a quote
    pub fn reject(&self, id: &str) -> ServiceResult<Quote> {
        self.set_quote_status(id, QuoteStatus::Rejected)
    }

    /// Get price for a quantity from a specific quote
    pub fn get_price_for_qty(&self, id: &str, qty: u32) -> ServiceResult<Option<f64>> {
        let quote = self.get_required(id)?;
        Ok(quote.price_for_qty(qty))
    }

    /// Calculate effective unit price including amortized NRE
    pub fn effective_unit_price(
        &self,
        id: &str,
        qty: u32,
        amortize_qty: Option<u32>,
    ) -> ServiceResult<Option<f64>> {
        let quote = self.get_required(id)?;

        let base_price = match quote.price_for_qty(qty) {
            Some(p) => p,
            None => return Ok(None),
        };

        let nre_per_unit = match amortize_qty {
            Some(amort) if amort > 0 => quote.total_nre() / amort as f64,
            _ => 0.0,
        };

        Ok(Some(base_price + nre_per_unit))
    }

    /// Compare quotes for a component or assembly
    pub fn compare(
        &self,
        item_id: &str,
        qty: u32,
        amortize_qty: Option<u32>,
    ) -> ServiceResult<QuoteComparison> {
        // Get quotes for this item (check both component and assembly)
        let mut quotes: Vec<Quote> = self
            .load_all()?
            .into_iter()
            .filter(|q| {
                q.component.as_ref().is_some_and(|c| c.contains(item_id))
                    || q.assembly.as_ref().is_some_and(|a| a.contains(item_id))
            })
            .collect();

        // Sort by effective price
        quotes.sort_by(|a, b| {
            let price_a = self.calc_effective_price(a, qty, amortize_qty);
            let price_b = self.calc_effective_price(b, qty, amortize_qty);
            price_a
                .partial_cmp(&price_b)
                .unwrap_or(std::cmp::Ordering::Equal)
        });

        let lowest_price_quote = quotes.first().map(|q| q.id.to_string());

        let compared: Vec<ComparedQuote> = quotes
            .iter()
            .map(|q| ComparedQuote {
                id: q.id.to_string(),
                title: q.title.clone(),
                supplier: q.supplier.clone(),
                unit_price: q.price_for_qty(qty),
                effective_price: Some(self.calc_effective_price(q, qty, amortize_qty)),
                lead_time_days: q.lead_time_for_qty(qty),
                quote_status: q.quote_status,
            })
            .collect();

        Ok(QuoteComparison {
            item_id: item_id.to_string(),
            quantity: qty,
            quotes: compared,
            lowest_price_quote,
        })
    }

    /// Get expired quotes
    pub fn get_expired(&self) -> ServiceResult<Vec<Quote>> {
        let quotes = self.load_all()?;
        Ok(quotes.into_iter().filter(|q| q.is_expired()).collect())
    }

    /// Get statistics about quotes
    pub fn stats(&self) -> ServiceResult<QuoteStats> {
        let quotes = self.load_all()?;

        let mut stats = QuoteStats::default();
        stats.total = quotes.len();

        for quote in &quotes {
            // Count by quote status
            match quote.quote_status {
                QuoteStatus::Pending => stats.by_quote_status.pending += 1,
                QuoteStatus::Received => stats.by_quote_status.received += 1,
                QuoteStatus::Accepted => stats.by_quote_status.accepted += 1,
                QuoteStatus::Rejected => stats.by_quote_status.rejected += 1,
                QuoteStatus::Expired => stats.by_quote_status.expired += 1,
            }

            // Count by entity status
            match quote.status {
                Status::Draft => stats.by_status.draft += 1,
                Status::Review => stats.by_status.review += 1,
                Status::Approved => stats.by_status.approved += 1,
                Status::Released => stats.by_status.released += 1,
                Status::Obsolete => stats.by_status.obsolete += 1,
            }

            // Count features
            if !quote.price_breaks.is_empty() {
                stats.with_price_breaks += 1;
            }
            if quote.total_nre() > 0.0 {
                stats.with_nre += 1;
            }
            if quote.is_expired() {
                stats.expired += 1;
            }
            if quote.component.is_some() {
                stats.for_components += 1;
            }
            if quote.assembly.is_some() {
                stats.for_assemblies += 1;
            }
        }

        Ok(stats)
    }

    // --- Private helper methods ---

    /// Find a quote and its file path (cache-first lookup)
    fn find_quote(&self, id: &str) -> ServiceResult<(PathBuf, Quote)> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                if let Ok(quote) = crate::yaml::parse_yaml_file::<Quote>(&path) {
                    return Ok((path, quote));
                }
            }
        }

        // Fall back to directory scan
        let dir = self.get_directory();
        if let Some((path, quote)) = loader::load_entity::<Quote>(&dir, id)? {
            return Ok((path, quote));
        }
        Err(ServiceError::NotFound(id.to_string()))
    }

    /// Calculate effective price for sorting
    fn calc_effective_price(&self, quote: &Quote, qty: u32, amortize_qty: Option<u32>) -> f64 {
        let base = quote.price_for_qty(qty).unwrap_or(f64::MAX);
        let nre_per_unit = match amortize_qty {
            Some(amort) if amort > 0 => quote.total_nre() / amort as f64,
            _ => 0.0,
        };
        base + nre_per_unit
    }

    /// Check if a quote matches the given filter
    fn matches_filter(&self, quote: &Quote, filter: &QuoteFilter) -> bool {
        // Quote status filter
        if let Some(quote_status) = &filter.quote_status {
            if quote.quote_status != *quote_status {
                return false;
            }
        }

        // Supplier filter
        if let Some(supplier) = &filter.supplier {
            if !quote.supplier.contains(supplier) {
                return false;
            }
        }

        // Component filter
        if let Some(component) = &filter.component {
            if !quote
                .component
                .as_ref()
                .is_some_and(|c| c.contains(component))
            {
                return false;
            }
        }

        // Assembly filter
        if let Some(assembly) = &filter.assembly {
            if !quote
                .assembly
                .as_ref()
                .is_some_and(|a| a.contains(assembly))
            {
                return false;
            }
        }

        // Expired only filter
        if filter.expired_only && !quote.is_expired() {
            return false;
        }

        // Has price breaks filter
        if filter.has_price_breaks && quote.price_breaks.is_empty() {
            return false;
        }

        // Currency filter
        if let Some(currency) = &filter.currency {
            if quote.currency != *currency {
                return false;
            }
        }

        // Common filters
        if !filter.common.matches_status(&quote.status) {
            return false;
        }
        if !filter.common.matches_author(&quote.author) {
            return false;
        }
        if !filter.common.matches_tags(&quote.tags) {
            return false;
        }
        if !filter.common.matches_search(&[&quote.title]) {
            return false;
        }
        if !filter.common.matches_recent(&quote.created) {
            return false;
        }

        true
    }

    /// Sort quotes by the given field
    fn sort_quotes(&self, quotes: &mut [Quote], sort_by: QuoteSortField, sort_dir: SortDirection) {
        quotes.sort_by(|a, b| {
            let cmp = match sort_by {
                QuoteSortField::Id => a.id.to_string().cmp(&b.id.to_string()),
                QuoteSortField::Title => a.title.cmp(&b.title),
                QuoteSortField::Supplier => a.supplier.cmp(&b.supplier),
                QuoteSortField::Component => a.component.cmp(&b.component),
                QuoteSortField::Price => {
                    let price_a = a.price_for_qty(1).unwrap_or(0.0);
                    let price_b = b.price_for_qty(1).unwrap_or(0.0);
                    price_a
                        .partial_cmp(&price_b)
                        .unwrap_or(std::cmp::Ordering::Equal)
                }
                QuoteSortField::QuoteStatus => {
                    format!("{:?}", a.quote_status).cmp(&format!("{:?}", b.quote_status))
                }
                QuoteSortField::Status => format!("{:?}", a.status).cmp(&format!("{:?}", b.status)),
                QuoteSortField::Author => a.author.cmp(&b.author),
                QuoteSortField::Created => a.created.cmp(&b.created),
            };

            match sort_dir {
                SortDirection::Ascending => cmp,
                SortDirection::Descending => cmp.reverse(),
            }
        });
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn setup_test_project() -> (TempDir, Project, EntityCache) {
        let tmp = TempDir::new().unwrap();

        // Initialize project structure
        fs::create_dir_all(tmp.path().join(".tdt")).unwrap();
        fs::create_dir_all(tmp.path().join("bom/quotes")).unwrap();

        // Create config file
        fs::write(tmp.path().join(".tdt/config.yaml"), "author: Test Author\n").unwrap();

        let project = Project::discover_from(tmp.path()).unwrap();
        let cache = EntityCache::open(&project).unwrap();

        (tmp, project, cache)
    }

    #[test]
    fn test_create_quote_for_component() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let input = CreateQuote {
            title: "Bracket Quote".into(),
            author: "Test Author".into(),
            supplier: "SUP-123".into(),
            component: Some("CMP-456".into()),
            ..Default::default()
        };

        let quote = service.create(input).unwrap();

        assert_eq!(quote.title, "Bracket Quote");
        assert_eq!(quote.supplier, "SUP-123");
        assert_eq!(quote.component, Some("CMP-456".to_string()));
        assert_eq!(quote.assembly, None);
        assert_eq!(quote.quote_status, QuoteStatus::Pending);
    }

    #[test]
    fn test_create_quote_for_assembly() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let input = CreateQuote {
            title: "Assembly Quote".into(),
            author: "Test Author".into(),
            supplier: "SUP-123".into(),
            assembly: Some("ASM-789".into()),
            ..Default::default()
        };

        let quote = service.create(input).unwrap();

        assert_eq!(quote.component, None);
        assert_eq!(quote.assembly, Some("ASM-789".to_string()));
    }

    #[test]
    fn test_create_quote_requires_item() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let input = CreateQuote {
            title: "Bad Quote".into(),
            author: "Test".into(),
            supplier: "SUP-123".into(),
            // Neither component nor assembly
            ..Default::default()
        };

        let result = service.create(input);
        assert!(result.is_err());
    }

    #[test]
    fn test_get_quote() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let created = service
            .create(CreateQuote {
                title: "Find Me".into(),
                author: "Test".into(),
                supplier: "SUP-123".into(),
                component: Some("CMP-456".into()),
                ..Default::default()
            })
            .unwrap();

        let found = service.get(&created.id.to_string()).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().title, "Find Me");
    }

    #[test]
    fn test_update_quote() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let created = service
            .create(CreateQuote {
                title: "Original".into(),
                author: "Test".into(),
                supplier: "SUP-123".into(),
                component: Some("CMP-456".into()),
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .update(
                &created.id.to_string(),
                UpdateQuote {
                    title: Some("Updated Title".into()),
                    quote_status: Some(QuoteStatus::Received),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.title, "Updated Title");
        assert_eq!(updated.quote_status, QuoteStatus::Received);
        assert_eq!(updated.entity_revision, 2);
    }

    #[test]
    fn test_delete_quote() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let created = service
            .create(CreateQuote {
                title: "Delete Me".into(),
                author: "Test".into(),
                supplier: "SUP-123".into(),
                component: Some("CMP-456".into()),
                ..Default::default()
            })
            .unwrap();

        service.delete(&created.id.to_string(), false).unwrap();

        let found = service.get(&created.id.to_string()).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_add_price_break() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let created = service
            .create(CreateQuote {
                title: "With Prices".into(),
                author: "Test".into(),
                supplier: "SUP-123".into(),
                component: Some("CMP-456".into()),
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .add_price_break(&created.id.to_string(), 1, 10.0, Some(7))
            .unwrap();
        let updated = service
            .add_price_break(&updated.id.to_string(), 100, 8.0, Some(14))
            .unwrap();
        let updated = service
            .add_price_break(&updated.id.to_string(), 1000, 6.0, Some(21))
            .unwrap();

        assert_eq!(updated.price_breaks.len(), 3);

        // Test price lookups
        assert_eq!(updated.price_for_qty(1), Some(10.0));
        assert_eq!(updated.price_for_qty(50), Some(10.0));
        assert_eq!(updated.price_for_qty(100), Some(8.0));
        assert_eq!(updated.price_for_qty(500), Some(8.0));
        assert_eq!(updated.price_for_qty(1000), Some(6.0));
    }

    #[test]
    fn test_remove_price_break() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let created = service
            .create(CreateQuote {
                title: "Remove Price".into(),
                author: "Test".into(),
                supplier: "SUP-123".into(),
                component: Some("CMP-456".into()),
                ..Default::default()
            })
            .unwrap();

        service
            .add_price_break(&created.id.to_string(), 1, 10.0, None)
            .unwrap();
        service
            .add_price_break(&created.id.to_string(), 100, 8.0, None)
            .unwrap();

        let updated = service
            .remove_price_break(&created.id.to_string(), 1)
            .unwrap();

        assert_eq!(updated.price_breaks.len(), 1);
        assert_eq!(updated.price_breaks[0].min_qty, 100);
    }

    #[test]
    fn test_add_nre_cost() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let created = service
            .create(CreateQuote {
                title: "With NRE".into(),
                author: "Test".into(),
                supplier: "SUP-123".into(),
                component: Some("CMP-456".into()),
                tooling_cost: Some(5000.0),
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .add_nre_cost(
                &created.id.to_string(),
                NreCost {
                    description: "Design fee".into(),
                    cost: 2000.0,
                    one_time: true,
                },
            )
            .unwrap();

        assert_eq!(updated.nre_costs.len(), 1);
        assert_eq!(updated.total_nre(), 7000.0); // 5000 tooling + 2000 NRE
    }

    #[test]
    fn test_accept_reject_quote() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let created = service
            .create(CreateQuote {
                title: "Status Test".into(),
                author: "Test".into(),
                supplier: "SUP-123".into(),
                component: Some("CMP-456".into()),
                ..Default::default()
            })
            .unwrap();

        assert_eq!(created.quote_status, QuoteStatus::Pending);

        let accepted = service.accept(&created.id.to_string()).unwrap();
        assert_eq!(accepted.quote_status, QuoteStatus::Accepted);

        let rejected = service.reject(&created.id.to_string()).unwrap();
        assert_eq!(rejected.quote_status, QuoteStatus::Rejected);
    }

    #[test]
    fn test_get_by_component() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        // Create quotes for different components
        service
            .create(CreateQuote {
                title: "Quote 1".into(),
                author: "Test".into(),
                supplier: "SUP-A".into(),
                component: Some("CMP-100".into()),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateQuote {
                title: "Quote 2".into(),
                author: "Test".into(),
                supplier: "SUP-B".into(),
                component: Some("CMP-100".into()),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateQuote {
                title: "Quote 3".into(),
                author: "Test".into(),
                supplier: "SUP-C".into(),
                component: Some("CMP-200".into()),
                ..Default::default()
            })
            .unwrap();

        let cmp_100_quotes = service.get_by_component("CMP-100").unwrap();
        assert_eq!(cmp_100_quotes.len(), 2);

        let cmp_200_quotes = service.get_by_component("CMP-200").unwrap();
        assert_eq!(cmp_200_quotes.len(), 1);
    }

    #[test]
    fn test_compare_quotes() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        // Create competing quotes
        let q1 = service
            .create(CreateQuote {
                title: "Expensive".into(),
                author: "Test".into(),
                supplier: "SUP-A".into(),
                component: Some("CMP-100".into()),
                ..Default::default()
            })
            .unwrap();
        service
            .add_price_break(&q1.id.to_string(), 1, 15.0, None)
            .unwrap();

        let q2 = service
            .create(CreateQuote {
                title: "Cheap".into(),
                author: "Test".into(),
                supplier: "SUP-B".into(),
                component: Some("CMP-100".into()),
                ..Default::default()
            })
            .unwrap();
        service
            .add_price_break(&q2.id.to_string(), 1, 10.0, None)
            .unwrap();

        let q3 = service
            .create(CreateQuote {
                title: "Medium".into(),
                author: "Test".into(),
                supplier: "SUP-C".into(),
                component: Some("CMP-100".into()),
                ..Default::default()
            })
            .unwrap();
        service
            .add_price_break(&q3.id.to_string(), 1, 12.0, None)
            .unwrap();

        let comparison = service.compare("CMP-100", 1, None).unwrap();

        assert_eq!(comparison.quotes.len(), 3);
        assert_eq!(comparison.lowest_price_quote, Some(q2.id.to_string()));

        // First quote should be cheapest
        assert_eq!(comparison.quotes[0].title, "Cheap");
        assert_eq!(comparison.quotes[0].unit_price, Some(10.0));
    }

    #[test]
    fn test_effective_unit_price() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        let created = service
            .create(CreateQuote {
                title: "Effective Price Test".into(),
                author: "Test".into(),
                supplier: "SUP-123".into(),
                component: Some("CMP-456".into()),
                tooling_cost: Some(1000.0),
                ..Default::default()
            })
            .unwrap();

        service
            .add_price_break(&created.id.to_string(), 1, 10.0, None)
            .unwrap();

        // Without amortization
        let price = service
            .effective_unit_price(&created.id.to_string(), 1, None)
            .unwrap();
        assert_eq!(price, Some(10.0));

        // With amortization over 100 units: 10.0 + (1000/100) = 20.0
        let price = service
            .effective_unit_price(&created.id.to_string(), 1, Some(100))
            .unwrap();
        assert_eq!(price, Some(20.0));

        // With amortization over 1000 units: 10.0 + (1000/1000) = 11.0
        let price = service
            .effective_unit_price(&created.id.to_string(), 1, Some(1000))
            .unwrap();
        assert_eq!(price, Some(11.0));
    }

    #[test]
    fn test_list_with_filter() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        // Create quotes with different suppliers
        service
            .create(CreateQuote {
                title: "SUP-A Quote".into(),
                author: "Test".into(),
                supplier: "SUP-A".into(),
                component: Some("CMP-100".into()),
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateQuote {
                title: "SUP-B Quote".into(),
                author: "Test".into(),
                supplier: "SUP-B".into(),
                component: Some("CMP-200".into()),
                ..Default::default()
            })
            .unwrap();

        // List all
        let all = service
            .list(
                &QuoteFilter::default(),
                QuoteSortField::Title,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(all.items.len(), 2);

        // List by supplier
        let sup_a = service
            .list(
                &QuoteFilter::by_supplier("SUP-A"),
                QuoteSortField::Title,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(sup_a.items.len(), 1);
        assert_eq!(sup_a.items[0].title, "SUP-A Quote");
    }

    #[test]
    fn test_stats() {
        let (_tmp, project, cache) = setup_test_project();
        let service = QuoteService::new(&project, &cache);

        // Create quotes with different statuses
        let q1 = service
            .create(CreateQuote {
                title: "Pending Quote".into(),
                author: "Test".into(),
                supplier: "SUP-A".into(),
                component: Some("CMP-100".into()),
                tooling_cost: Some(1000.0),
                ..Default::default()
            })
            .unwrap();
        service
            .add_price_break(&q1.id.to_string(), 1, 10.0, None)
            .unwrap();

        let q2 = service
            .create(CreateQuote {
                title: "Assembly Quote".into(),
                author: "Test".into(),
                supplier: "SUP-B".into(),
                assembly: Some("ASM-200".into()),
                ..Default::default()
            })
            .unwrap();
        service.accept(&q2.id.to_string()).unwrap();

        let stats = service.stats().unwrap();

        assert_eq!(stats.total, 2);
        assert_eq!(stats.by_quote_status.pending, 1);
        assert_eq!(stats.by_quote_status.accepted, 1);
        assert_eq!(stats.with_price_breaks, 1);
        assert_eq!(stats.with_nre, 1);
        assert_eq!(stats.for_components, 1);
        assert_eq!(stats.for_assemblies, 1);
    }
}
