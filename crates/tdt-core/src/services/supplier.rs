//! Supplier service - business logic for approved supplier management
//!
//! Provides CRUD operations and contact/certification management for suppliers.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

use crate::core::cache::EntityCache;
use crate::core::entity::Status;
use crate::core::identity::{EntityId, EntityPrefix};
use crate::core::loader;
use crate::core::project::Project;
use crate::entities::supplier::{
    Address, Capability, Certification, Contact, Currency, Supplier, SupplierLinks,
};
use crate::services::base::ServiceBase;

use super::common::{
    apply_pagination, CommonFilter, ListResult, ServiceError, ServiceResult, SortDirection,
};

/// Filter options specific to suppliers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SupplierFilter {
    /// Common filter options (status, author, search, etc.)
    #[serde(flatten)]
    pub common: CommonFilter,

    /// Filter by capability
    pub capability: Option<Capability>,

    /// Show only suppliers with expired certifications
    pub expired_certs: bool,

    /// Show only suppliers with certifications expiring within N days
    pub certs_expiring_days: Option<i64>,

    /// Filter by currency
    pub currency: Option<Currency>,
}

impl SupplierFilter {
    /// Create a filter for suppliers with a specific capability
    pub fn with_capability(capability: Capability) -> Self {
        Self {
            capability: Some(capability),
            ..Default::default()
        }
    }

    /// Create a filter for suppliers with expired certifications
    pub fn expired() -> Self {
        Self {
            expired_certs: true,
            ..Default::default()
        }
    }

    /// Create a filter for suppliers with certifications expiring soon
    pub fn expiring_soon(days: i64) -> Self {
        Self {
            certs_expiring_days: Some(days),
            ..Default::default()
        }
    }
}

/// Sort field for suppliers
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SupplierSortField {
    Id,
    #[default]
    Name,
    ShortName,
    Status,
    Website,
    Capabilities,
    Author,
    Created,
}

/// Input for creating a new supplier
#[derive(Debug, Clone, Serialize, Deserialize)]
#[derive(Default)]
pub struct CreateSupplier {
    /// Company name
    pub name: String,

    /// Author name
    pub author: String,

    /// Short name for display
    #[serde(default)]
    pub short_name: Option<String>,

    /// Website URL
    #[serde(default)]
    pub website: Option<String>,

    /// Default payment terms
    #[serde(default)]
    pub payment_terms: Option<String>,

    /// Preferred currency
    #[serde(default)]
    pub currency: Currency,

    /// Notes about the supplier
    #[serde(default)]
    pub notes: Option<String>,

    /// Manufacturing capabilities
    #[serde(default)]
    pub capabilities: Vec<Capability>,

    /// Classification tags
    #[serde(default)]
    pub tags: Vec<String>,
}


/// Input for updating an existing supplier
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UpdateSupplier {
    /// Update company name
    pub name: Option<String>,

    /// Update short name
    pub short_name: Option<String>,

    /// Update website
    pub website: Option<String>,

    /// Update payment terms
    pub payment_terms: Option<String>,

    /// Update currency
    pub currency: Option<Currency>,

    /// Update notes
    pub notes: Option<String>,

    /// Replace capabilities
    pub capabilities: Option<Vec<Capability>>,

    /// Update status
    pub status: Option<Status>,

    /// Replace tags
    pub tags: Option<Vec<String>>,
}

/// Statistics about suppliers
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SupplierStats {
    pub total: usize,
    pub by_status: SupplierStatusCounts,
    pub by_capability: CapabilityCounts,
    pub with_certifications: usize,
    pub with_expired_certs: usize,
    pub with_contacts: usize,
    pub with_addresses: usize,
}

/// Counts by status
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct SupplierStatusCounts {
    pub draft: usize,
    pub review: usize,
    pub approved: usize,
    pub released: usize,
    pub obsolete: usize,
}

/// Counts by capability
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CapabilityCounts {
    pub machining: usize,
    pub sheet_metal: usize,
    pub casting: usize,
    pub injection: usize,
    pub extrusion: usize,
    pub pcb: usize,
    pub pcb_assembly: usize,
    pub cable_assembly: usize,
    pub assembly: usize,
    pub testing: usize,
    pub finishing: usize,
    pub packaging: usize,
}

/// Service for supplier management
pub struct SupplierService<'a> {
    project: &'a Project,
    cache: &'a EntityCache,
    base: ServiceBase<'a>,
}

impl<'a> SupplierService<'a> {
    /// Create a new supplier service
    pub fn new(project: &'a Project, cache: &'a EntityCache) -> Self {
        Self {
            project,
            cache,
            base: ServiceBase::new(project, cache),
        }
    }

    /// Get the directory for storing suppliers
    fn get_directory(&self) -> PathBuf {
        self.project.root().join("bom/suppliers")
    }

    /// Get the file path for a supplier
    fn get_file_path(&self, id: &EntityId) -> PathBuf {
        self.get_directory().join(format!("{}.tdt.yaml", id))
    }

    /// List suppliers with filtering and pagination
    pub fn list(
        &self,
        filter: &SupplierFilter,
        sort_by: SupplierSortField,
        sort_dir: SortDirection,
    ) -> ServiceResult<ListResult<Supplier>> {
        let mut suppliers = self.load_all()?;

        // Apply filters
        suppliers.retain(|sup| self.matches_filter(sup, filter));

        // Sort
        self.sort_suppliers(&mut suppliers, sort_by, sort_dir);

        // Paginate
        Ok(apply_pagination(
            suppliers,
            filter.common.offset,
            filter.common.limit,
        ))
    }

    /// List suppliers from cache (fast path for list display)
    ///
    /// Returns cached supplier data without loading full YAML files.
    /// Use this for list commands where full entity data isn't needed.
    pub fn list_cached(&self) -> Vec<crate::core::CachedSupplier> {
        self.cache.list_suppliers(None, None, None, None, None)
    }

    /// Load all suppliers from the filesystem
    pub fn load_all(&self) -> ServiceResult<Vec<Supplier>> {
        let dir = self.get_directory();
        if !dir.exists() {
            return Ok(Vec::new());
        }
        Ok(loader::load_all(&dir)?)
    }

    /// Get a single supplier by ID
    pub fn get(&self, id: &str) -> ServiceResult<Option<Supplier>> {
        let dir = self.get_directory();
        if let Some((_, sup)) = loader::load_entity::<Supplier>(&dir, id)? {
            return Ok(Some(sup));
        }
        Ok(None)
    }

    /// Get a supplier by ID, returning an error if not found
    pub fn get_required(&self, id: &str) -> ServiceResult<Supplier> {
        self.get(id)?
            .ok_or_else(|| ServiceError::NotFound(id.to_string()))
    }

    /// Get suppliers with a specific capability
    pub fn get_by_capability(&self, capability: Capability) -> ServiceResult<Vec<Supplier>> {
        let suppliers = self.load_all()?;
        Ok(suppliers
            .into_iter()
            .filter(|s| s.capabilities.contains(&capability))
            .collect())
    }

    /// Create a new supplier
    pub fn create(&self, input: CreateSupplier) -> ServiceResult<Supplier> {
        let id = EntityId::new(EntityPrefix::Sup);

        let supplier = Supplier {
            id: id.clone(),
            name: input.name,
            short_name: input.short_name,
            website: input.website,
            contacts: Vec::new(),
            addresses: Vec::new(),
            payment_terms: input.payment_terms,
            currency: input.currency,
            certifications: Vec::new(),
            capabilities: input.capabilities,
            notes: input.notes,
            tags: input.tags,
            status: Status::Draft,
            links: SupplierLinks::default(),
            created: Utc::now(),
            author: input.author,
            entity_revision: 1,
        };

        // Write to file
        let path = self.get_file_path(&id);
        self.base.save(&supplier, &path, Some("SUP"))?;

        Ok(supplier)
    }

    /// Update an existing supplier
    pub fn update(&self, id: &str, input: UpdateSupplier) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        // Apply updates
        if let Some(name) = input.name {
            supplier.name = name;
        }
        if let Some(short_name) = input.short_name {
            supplier.short_name = Some(short_name);
        }
        if let Some(website) = input.website {
            supplier.website = Some(website);
        }
        if let Some(payment_terms) = input.payment_terms {
            supplier.payment_terms = Some(payment_terms);
        }
        if let Some(currency) = input.currency {
            supplier.currency = currency;
        }
        if let Some(notes) = input.notes {
            supplier.notes = Some(notes);
        }
        if let Some(capabilities) = input.capabilities {
            supplier.capabilities = capabilities;
        }
        if let Some(status) = input.status {
            supplier.status = status;
        }
        if let Some(tags) = input.tags {
            supplier.tags = tags;
        }

        // Increment revision
        supplier.entity_revision += 1;

        // Write back
        self.base.save(&supplier, &path, None)?;

        Ok(supplier)
    }

    /// Delete a supplier
    pub fn delete(&self, id: &str, force: bool) -> ServiceResult<()> {
        let (path, supplier) = self.find_supplier(id)?;

        // Check for references unless force is true
        if !force && !supplier.links.approved_for.is_empty() {
            return Err(ServiceError::HasReferences);
        }

        // Delete the file
        fs::remove_file(&path)?;

        Ok(())
    }

    /// Add a contact to a supplier
    pub fn add_contact(&self, id: &str, contact: Contact) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        supplier.contacts.push(contact);
        supplier.entity_revision += 1;

        self.base.save(&supplier, &path, None)?;

        Ok(supplier)
    }

    /// Remove a contact from a supplier by name
    pub fn remove_contact(&self, id: &str, contact_name: &str) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        let initial_len = supplier.contacts.len();
        supplier.contacts.retain(|c| c.name != contact_name);

        if supplier.contacts.len() == initial_len {
            return Err(ServiceError::NotFound(format!(
                "Contact '{}' not found",
                contact_name
            )));
        }

        supplier.entity_revision += 1;

        self.base.save(&supplier, &path, None)?;

        Ok(supplier)
    }

    /// Set the primary contact for a supplier
    pub fn set_primary_contact(&self, id: &str, contact_name: &str) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        let mut found = false;
        for contact in &mut supplier.contacts {
            if contact.name == contact_name {
                contact.primary = true;
                found = true;
            } else {
                contact.primary = false;
            }
        }

        if !found {
            return Err(ServiceError::NotFound(format!(
                "Contact '{}' not found",
                contact_name
            )));
        }

        supplier.entity_revision += 1;

        self.base.save(&supplier, &path, None)?;

        Ok(supplier)
    }

    /// Add an address to a supplier
    pub fn add_address(&self, id: &str, address: Address) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        supplier.addresses.push(address);
        supplier.entity_revision += 1;

        self.base.save(&supplier, &path, None)?;

        Ok(supplier)
    }

    /// Remove an address from a supplier by type
    pub fn remove_address(
        &self,
        id: &str,
        address_type: crate::entities::supplier::AddressType,
    ) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        let initial_len = supplier.addresses.len();
        supplier
            .addresses
            .retain(|a| a.address_type != address_type);

        if supplier.addresses.len() == initial_len {
            return Err(ServiceError::NotFound(format!(
                "Address of type '{:?}' not found",
                address_type
            )));
        }

        supplier.entity_revision += 1;

        self.base.save(&supplier, &path, None)?;

        Ok(supplier)
    }

    /// Add a certification to a supplier
    pub fn add_certification(&self, id: &str, cert: Certification) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        supplier.certifications.push(cert);
        supplier.entity_revision += 1;

        self.base.save(&supplier, &path, None)?;

        Ok(supplier)
    }

    /// Remove a certification from a supplier by name
    pub fn remove_certification(&self, id: &str, cert_name: &str) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        let initial_len = supplier.certifications.len();
        supplier.certifications.retain(|c| c.name != cert_name);

        if supplier.certifications.len() == initial_len {
            return Err(ServiceError::NotFound(format!(
                "Certification '{}' not found",
                cert_name
            )));
        }

        supplier.entity_revision += 1;

        self.base.save(&supplier, &path, None)?;

        Ok(supplier)
    }

    /// Add a capability to a supplier
    pub fn add_capability(&self, id: &str, capability: Capability) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        if !supplier.capabilities.contains(&capability) {
            supplier.capabilities.push(capability);
            supplier.entity_revision += 1;

            self.base.save(&supplier, &path, None)?;
        }

        Ok(supplier)
    }

    /// Remove a capability from a supplier
    pub fn remove_capability(&self, id: &str, capability: Capability) -> ServiceResult<Supplier> {
        let (path, mut supplier) = self.find_supplier(id)?;

        let initial_len = supplier.capabilities.len();
        supplier.capabilities.retain(|c| *c != capability);

        if supplier.capabilities.len() == initial_len {
            return Err(ServiceError::NotFound(format!(
                "Capability '{:?}' not found",
                capability
            )));
        }

        supplier.entity_revision += 1;

        self.base.save(&supplier, &path, None)?;

        Ok(supplier)
    }

    /// Get suppliers with expired certifications
    pub fn get_with_expired_certs(&self) -> ServiceResult<Vec<Supplier>> {
        let suppliers = self.load_all()?;
        Ok(suppliers
            .into_iter()
            .filter(|s| s.has_expired_certs())
            .collect())
    }

    /// Get suppliers with certifications expiring within N days
    pub fn get_with_expiring_certs(&self, days: i64) -> ServiceResult<Vec<Supplier>> {
        let suppliers = self.load_all()?;
        Ok(suppliers
            .into_iter()
            .filter(|s| !s.certs_expiring_soon(days).is_empty())
            .collect())
    }

    /// Get statistics about suppliers
    pub fn stats(&self) -> ServiceResult<SupplierStats> {
        let suppliers = self.load_all()?;

        let mut stats = SupplierStats {
            total: suppliers.len(),
            ..Default::default()
        };

        for sup in &suppliers {
            // Count by status
            match sup.status {
                Status::Draft => stats.by_status.draft += 1,
                Status::Review => stats.by_status.review += 1,
                Status::Approved => stats.by_status.approved += 1,
                Status::Released => stats.by_status.released += 1,
                Status::Obsolete => stats.by_status.obsolete += 1,
            }

            // Count capabilities
            for cap in &sup.capabilities {
                match cap {
                    Capability::Machining => stats.by_capability.machining += 1,
                    Capability::SheetMetal => stats.by_capability.sheet_metal += 1,
                    Capability::Casting => stats.by_capability.casting += 1,
                    Capability::Injection => stats.by_capability.injection += 1,
                    Capability::Extrusion => stats.by_capability.extrusion += 1,
                    Capability::Pcb => stats.by_capability.pcb += 1,
                    Capability::PcbAssembly => stats.by_capability.pcb_assembly += 1,
                    Capability::CableAssembly => stats.by_capability.cable_assembly += 1,
                    Capability::Assembly => stats.by_capability.assembly += 1,
                    Capability::Testing => stats.by_capability.testing += 1,
                    Capability::Finishing => stats.by_capability.finishing += 1,
                    Capability::Packaging => stats.by_capability.packaging += 1,
                }
            }

            // Count features
            if !sup.certifications.is_empty() {
                stats.with_certifications += 1;
            }
            if sup.has_expired_certs() {
                stats.with_expired_certs += 1;
            }
            if !sup.contacts.is_empty() {
                stats.with_contacts += 1;
            }
            if !sup.addresses.is_empty() {
                stats.with_addresses += 1;
            }
        }

        Ok(stats)
    }

    // --- Private helper methods ---

    /// Find a supplier and its file path (cache-first lookup)
    fn find_supplier(&self, id: &str) -> ServiceResult<(PathBuf, Supplier)> {
        // Try to find in cache first for fast path lookup
        if let Some(cached) = self.cache.get_entity(id) {
            let path = if cached.file_path.is_absolute() {
                cached.file_path.clone()
            } else {
                self.project.root().join(&cached.file_path)
            };
            if path.exists() {
                if let Ok(supplier) = crate::yaml::parse_yaml_file::<Supplier>(&path) {
                    return Ok((path, supplier));
                }
            }
        }

        // Fall back to directory scan
        let dir = self.get_directory();
        if let Some((path, sup)) = loader::load_entity::<Supplier>(&dir, id)? {
            return Ok((path, sup));
        }
        Err(ServiceError::NotFound(id.to_string()))
    }

    /// Check if a supplier matches the given filter
    fn matches_filter(&self, sup: &Supplier, filter: &SupplierFilter) -> bool {
        // Capability filter
        if let Some(capability) = &filter.capability {
            if !sup.capabilities.contains(capability) {
                return false;
            }
        }

        // Expired certs filter
        if filter.expired_certs && !sup.has_expired_certs() {
            return false;
        }

        // Certs expiring soon filter
        if let Some(days) = filter.certs_expiring_days {
            if sup.certs_expiring_soon(days).is_empty() {
                return false;
            }
        }

        // Currency filter
        if let Some(currency) = &filter.currency {
            if sup.currency != *currency {
                return false;
            }
        }

        // Common filters
        if !filter.common.matches_status(&sup.status) {
            return false;
        }
        if !filter.common.matches_author(&sup.author) {
            return false;
        }
        if !filter.common.matches_tags(&sup.tags) {
            return false;
        }
        if !filter.common.matches_search(&[&sup.name]) {
            return false;
        }
        if !filter.common.matches_recent(&sup.created) {
            return false;
        }

        true
    }

    /// Sort suppliers by the given field
    fn sort_suppliers(
        &self,
        suppliers: &mut [Supplier],
        sort_by: SupplierSortField,
        sort_dir: SortDirection,
    ) {
        suppliers.sort_by(|a, b| {
            let cmp = match sort_by {
                SupplierSortField::Id => a.id.to_string().cmp(&b.id.to_string()),
                SupplierSortField::Name => a.name.cmp(&b.name),
                SupplierSortField::ShortName => a.short_name.cmp(&b.short_name),
                SupplierSortField::Status => {
                    format!("{:?}", a.status).cmp(&format!("{:?}", b.status))
                }
                SupplierSortField::Website => a.website.cmp(&b.website),
                SupplierSortField::Capabilities => a.capabilities.len().cmp(&b.capabilities.len()),
                SupplierSortField::Author => a.author.cmp(&b.author),
                SupplierSortField::Created => a.created.cmp(&b.created),
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
    use crate::entities::supplier::AddressType;
    use chrono::NaiveDate;
    use tempfile::TempDir;

    fn setup_test_project() -> (TempDir, Project, EntityCache) {
        let tmp = TempDir::new().unwrap();

        // Initialize project structure
        fs::create_dir_all(tmp.path().join(".tdt")).unwrap();
        fs::create_dir_all(tmp.path().join("bom/suppliers")).unwrap();

        // Create config file
        fs::write(tmp.path().join(".tdt/config.yaml"), "author: Test Author\n").unwrap();

        let project = Project::discover_from(tmp.path()).unwrap();
        let cache = EntityCache::open(&project).unwrap();

        (tmp, project, cache)
    }

    #[test]
    fn test_create_supplier() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let input = CreateSupplier {
            name: "Acme Corp".into(),
            author: "Test Author".into(),
            capabilities: vec![Capability::Machining, Capability::SheetMetal],
            ..Default::default()
        };

        let sup = service.create(input).unwrap();

        assert_eq!(sup.name, "Acme Corp");
        assert_eq!(sup.capabilities.len(), 2);
        assert_eq!(sup.status, Status::Draft);
    }

    #[test]
    fn test_get_supplier() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "Find Me Inc".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let found = service.get(&created.id.to_string()).unwrap();
        assert!(found.is_some());
        assert_eq!(found.unwrap().name, "Find Me Inc");
    }

    #[test]
    fn test_update_supplier() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "Original Name".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .update(
                &created.id.to_string(),
                UpdateSupplier {
                    name: Some("Updated Name".into()),
                    short_name: Some("Updated".into()),
                    ..Default::default()
                },
            )
            .unwrap();

        assert_eq!(updated.name, "Updated Name");
        assert_eq!(updated.short_name, Some("Updated".to_string()));
        assert_eq!(updated.entity_revision, 2);
    }

    #[test]
    fn test_delete_supplier() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "Delete Me".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service.delete(&created.id.to_string(), false).unwrap();

        let found = service.get(&created.id.to_string()).unwrap();
        assert!(found.is_none());
    }

    #[test]
    fn test_add_contact() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "With Contact".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let contact = Contact {
            name: "John Smith".into(),
            role: Some("Sales".into()),
            email: Some("john@acme.com".into()),
            phone: Some("555-1234".into()),
            primary: true,
        };

        let updated = service
            .add_contact(&created.id.to_string(), contact)
            .unwrap();

        assert_eq!(updated.contacts.len(), 1);
        assert_eq!(updated.contacts[0].name, "John Smith");
        assert!(updated.contacts[0].primary);
    }

    #[test]
    fn test_remove_contact() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "With Contacts".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .add_contact(
                &created.id.to_string(),
                Contact {
                    name: "John".into(),
                    ..Default::default()
                },
            )
            .unwrap();
        service
            .add_contact(
                &created.id.to_string(),
                Contact {
                    name: "Jane".into(),
                    ..Default::default()
                },
            )
            .unwrap();

        let updated = service
            .remove_contact(&created.id.to_string(), "John")
            .unwrap();

        assert_eq!(updated.contacts.len(), 1);
        assert_eq!(updated.contacts[0].name, "Jane");
    }

    #[test]
    fn test_set_primary_contact() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "Primary Contact Test".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        service
            .add_contact(
                &created.id.to_string(),
                Contact {
                    name: "John".into(),
                    primary: true,
                    ..Default::default()
                },
            )
            .unwrap();
        service
            .add_contact(
                &created.id.to_string(),
                Contact {
                    name: "Jane".into(),
                    primary: false,
                    ..Default::default()
                },
            )
            .unwrap();

        let updated = service
            .set_primary_contact(&created.id.to_string(), "Jane")
            .unwrap();

        let john = updated.contacts.iter().find(|c| c.name == "John").unwrap();
        let jane = updated.contacts.iter().find(|c| c.name == "Jane").unwrap();

        assert!(!john.primary);
        assert!(jane.primary);
    }

    #[test]
    fn test_add_address() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "With Address".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let address = Address {
            address_type: AddressType::Headquarters,
            street: Some("123 Main St".into()),
            city: Some("Springfield".into()),
            state: Some("IL".into()),
            postal: Some("62701".into()),
            country: Some("USA".into()),
        };

        let updated = service
            .add_address(&created.id.to_string(), address)
            .unwrap();

        assert_eq!(updated.addresses.len(), 1);
        assert_eq!(updated.addresses[0].city, Some("Springfield".to_string()));
    }

    #[test]
    fn test_add_certification() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "Certified Supplier".into(),
                author: "Test".into(),
                ..Default::default()
            })
            .unwrap();

        let cert = Certification {
            name: "ISO 9001:2015".into(),
            expiry: Some(NaiveDate::from_ymd_opt(2027, 12, 31).unwrap()),
            certificate_number: Some("CERT-12345".into()),
        };

        let updated = service
            .add_certification(&created.id.to_string(), cert)
            .unwrap();

        assert_eq!(updated.certifications.len(), 1);
        assert_eq!(updated.certifications[0].name, "ISO 9001:2015");
    }

    #[test]
    fn test_add_capability() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "Capability Test".into(),
                author: "Test".into(),
                capabilities: vec![Capability::Machining],
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .add_capability(&created.id.to_string(), Capability::SheetMetal)
            .unwrap();

        assert_eq!(updated.capabilities.len(), 2);
        assert!(updated.capabilities.contains(&Capability::SheetMetal));
    }

    #[test]
    fn test_remove_capability() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        let created = service
            .create(CreateSupplier {
                name: "Remove Cap Test".into(),
                author: "Test".into(),
                capabilities: vec![Capability::Machining, Capability::SheetMetal],
                ..Default::default()
            })
            .unwrap();

        let updated = service
            .remove_capability(&created.id.to_string(), Capability::Machining)
            .unwrap();

        assert_eq!(updated.capabilities.len(), 1);
        assert!(!updated.capabilities.contains(&Capability::Machining));
    }

    #[test]
    fn test_list_with_filter() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        // Create suppliers with different capabilities
        service
            .create(CreateSupplier {
                name: "Machining Shop".into(),
                author: "Test".into(),
                capabilities: vec![Capability::Machining],
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateSupplier {
                name: "PCB House".into(),
                author: "Test".into(),
                capabilities: vec![Capability::Pcb, Capability::PcbAssembly],
                ..Default::default()
            })
            .unwrap();

        // List all
        let all = service
            .list(
                &SupplierFilter::default(),
                SupplierSortField::Name,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(all.items.len(), 2);

        // List by capability
        let machining = service
            .list(
                &SupplierFilter::with_capability(Capability::Machining),
                SupplierSortField::Name,
                SortDirection::Ascending,
            )
            .unwrap();
        assert_eq!(machining.items.len(), 1);
        assert_eq!(machining.items[0].name, "Machining Shop");
    }

    #[test]
    fn test_get_by_capability() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        service
            .create(CreateSupplier {
                name: "Multi-cap".into(),
                author: "Test".into(),
                capabilities: vec![Capability::Machining, Capability::Assembly],
                ..Default::default()
            })
            .unwrap();

        service
            .create(CreateSupplier {
                name: "Assembly Only".into(),
                author: "Test".into(),
                capabilities: vec![Capability::Assembly],
                ..Default::default()
            })
            .unwrap();

        let assembly_suppliers = service.get_by_capability(Capability::Assembly).unwrap();
        assert_eq!(assembly_suppliers.len(), 2);

        let machining_suppliers = service.get_by_capability(Capability::Machining).unwrap();
        assert_eq!(machining_suppliers.len(), 1);
    }

    #[test]
    fn test_stats() {
        let (_tmp, project, cache) = setup_test_project();
        let service = SupplierService::new(&project, &cache);

        // Create supplier with capabilities
        let sup = service
            .create(CreateSupplier {
                name: "Stats Test".into(),
                author: "Test".into(),
                capabilities: vec![Capability::Machining, Capability::SheetMetal],
                ..Default::default()
            })
            .unwrap();

        // Add contact
        service
            .add_contact(
                &sup.id.to_string(),
                Contact {
                    name: "Test Contact".into(),
                    ..Default::default()
                },
            )
            .unwrap();

        // Add certification
        service
            .add_certification(
                &sup.id.to_string(),
                Certification {
                    name: "ISO 9001".into(),
                    expiry: None,
                    certificate_number: None,
                },
            )
            .unwrap();

        let stats = service.stats().unwrap();

        assert_eq!(stats.total, 1);
        assert_eq!(stats.by_status.draft, 1);
        assert_eq!(stats.by_capability.machining, 1);
        assert_eq!(stats.by_capability.sheet_metal, 1);
        assert_eq!(stats.with_certifications, 1);
        assert_eq!(stats.with_contacts, 1);
    }
}
