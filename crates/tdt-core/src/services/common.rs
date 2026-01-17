//! Common types and traits for services

use crate::core::entity::{Priority, Status};
use miette::Diagnostic;
use serde::{Deserialize, Serialize};
use thiserror::Error;

/// Result type for service operations
pub type ServiceResult<T> = std::result::Result<T, ServiceError>;

/// Errors that can occur in service operations
#[derive(Debug, Error, Diagnostic)]
pub enum ServiceError {
    #[error("Entity not found: {0}")]
    #[diagnostic(code(tdt::service::not_found))]
    NotFound(String),

    #[error("Entity already exists: {0}")]
    #[diagnostic(code(tdt::service::already_exists))]
    AlreadyExists(String),

    #[error("Invalid input: {0}")]
    #[diagnostic(code(tdt::service::invalid_input))]
    InvalidInput(String),

    #[error("Validation failed: {0}")]
    #[diagnostic(code(tdt::service::validation_failed))]
    ValidationFailed(String),

    #[error("IO error: {0}")]
    #[diagnostic(code(tdt::service::io_error))]
    Io(#[from] std::io::Error),

    #[error("YAML parsing error: {0}")]
    #[diagnostic(code(tdt::service::yaml_error))]
    Yaml(String),

    #[error("Project error: {0}")]
    #[diagnostic(code(tdt::service::project_error))]
    Project(String),

    #[error("Entity is referenced by other entities")]
    #[diagnostic(code(tdt::service::has_references))]
    HasReferences,

    #[error("{0}")]
    #[diagnostic(code(tdt::service::other))]
    Other(String),
}

impl From<miette::Report> for ServiceError {
    fn from(err: miette::Report) -> Self {
        ServiceError::Other(err.to_string())
    }
}

/// Common filter options for listing entities
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CommonFilter {
    /// Filter by status
    pub status: Option<Vec<Status>>,

    /// Filter by priority
    pub priority: Option<Vec<Priority>>,

    /// Filter by author (case-insensitive substring match)
    pub author: Option<String>,

    /// Filter by tags (entities must have at least one of these tags)
    pub tags: Option<Vec<String>>,

    /// Full-text search in title and description fields
    pub search: Option<String>,

    /// Only show entities created in the last N days
    pub recent_days: Option<u32>,

    /// Maximum number of results to return
    pub limit: Option<usize>,

    /// Number of results to skip (for pagination)
    pub offset: Option<usize>,
}

impl CommonFilter {
    /// Check if an entity matches the common filters using the Entity trait
    ///
    /// This method checks: status (via string), author, search (on title), and recent_days.
    /// Tags and priority must be checked separately as they're not in the Entity trait.
    ///
    /// # Example
    ///
    /// ```ignore
    /// let filter = CommonFilter { status: Some(vec![Status::Draft]), ..Default::default() };
    /// if filter.matches_entity(&requirement) {
    ///     // Entity passes common filters
    /// }
    /// ```
    pub fn matches_entity<E: crate::core::entity::Entity>(&self, entity: &E) -> bool {
        // Status filter (parse string to Status)
        if let Some(ref statuses) = self.status {
            if let Ok(entity_status) = entity.status().parse::<Status>() {
                if !statuses.contains(&entity_status) {
                    return false;
                }
            } else {
                // If status can't be parsed, entity doesn't match
                return false;
            }
        }

        // Author filter
        if !self.matches_author(entity.author()) {
            return false;
        }

        // Search filter (on title only - entities can add more fields in entity-specific filters)
        if let Some(ref search) = self.search {
            let search_lower = search.to_lowercase();
            if !entity.title().to_lowercase().contains(&search_lower) {
                return false;
            }
        }

        // Recent days filter
        if !self.matches_recent(&entity.created()) {
            return false;
        }

        true
    }

    /// Check if an entity's status matches the filter
    pub fn matches_status(&self, status: &Status) -> bool {
        self.status
            .as_ref()
            .map(|statuses| statuses.contains(status))
            .unwrap_or(true)
    }

    /// Check if a status string matches the filter
    pub fn matches_status_str(&self, status_str: &str) -> bool {
        match &self.status {
            None => true,
            Some(statuses) => {
                if let Ok(status) = status_str.parse::<Status>() {
                    statuses.contains(&status)
                } else {
                    false
                }
            }
        }
    }

    /// Check if an entity's priority matches the filter
    pub fn matches_priority(&self, priority: &Priority) -> bool {
        self.priority
            .as_ref()
            .map(|priorities| priorities.contains(priority))
            .unwrap_or(true)
    }

    /// Check if an entity's author matches the filter
    pub fn matches_author(&self, author: &str) -> bool {
        self.author
            .as_ref()
            .map(|filter| author.to_lowercase().contains(&filter.to_lowercase()))
            .unwrap_or(true)
    }

    /// Check if an entity's tags match the filter
    pub fn matches_tags(&self, entity_tags: &[String]) -> bool {
        self.tags.as_ref().map_or(true, |filter_tags| {
            filter_tags.iter().any(|ft| {
                entity_tags
                    .iter()
                    .any(|et| et.to_lowercase() == ft.to_lowercase())
            })
        })
    }

    /// Check if a text field matches the search filter
    pub fn matches_search(&self, texts: &[&str]) -> bool {
        self.search.as_ref().map_or(true, |search| {
            let search_lower = search.to_lowercase();
            texts
                .iter()
                .any(|text| text.to_lowercase().contains(&search_lower))
        })
    }

    /// Check if a creation date is within the recent_days filter
    pub fn matches_recent(&self, created: &chrono::DateTime<chrono::Utc>) -> bool {
        self.recent_days.map_or(true, |days| {
            let cutoff = chrono::Utc::now() - chrono::Duration::days(days as i64);
            *created >= cutoff
        })
    }
}

/// Sort direction for list operations
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum SortDirection {
    #[default]
    Ascending,
    Descending,
}

/// Wrapper for Option that sorts None values last instead of first
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct NoneLast<T>(pub Option<T>);

impl<T: Ord> Ord for NoneLast<T> {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        match (&self.0, &other.0) {
            (None, None) => std::cmp::Ordering::Equal,
            (None, Some(_)) => std::cmp::Ordering::Greater, // None sorts after Some
            (Some(_), None) => std::cmp::Ordering::Less,    // Some sorts before None
            (Some(a), Some(b)) => a.cmp(b),
        }
    }
}

impl<T: Ord> PartialOrd for NoneLast<T> {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

/// A comparable key for sorting entities
///
/// Supports different types of sortable values with consistent ordering.
/// None values sort last in ascending order (after all Some values).
#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
pub enum SortKey {
    /// String comparison (case-sensitive)
    String(String),
    /// Numeric comparison (using i64 to handle most cases)
    Number(i64),
    /// Optional string - None values sort last
    OptionalString(NoneLast<String>),
    /// Optional number - None values sort last
    OptionalNumber(NoneLast<i64>),
    /// Date/time comparison (milliseconds since epoch)
    DateTime(i64),
    /// Multi-field comparison - compares in order
    Composite(Vec<SortKey>),
}

impl SortKey {
    /// Create a string sort key
    pub fn string(s: impl Into<String>) -> Self {
        SortKey::String(s.into())
    }

    /// Create a sort key from an optional string
    pub fn optional_string(s: Option<impl Into<String>>) -> Self {
        SortKey::OptionalString(NoneLast(s.map(|s| s.into())))
    }

    /// Create a numeric sort key
    pub fn number(n: i64) -> Self {
        SortKey::Number(n)
    }

    /// Create a sort key from an optional number
    pub fn optional_number(n: Option<i64>) -> Self {
        SortKey::OptionalNumber(NoneLast(n))
    }

    /// Create a datetime sort key from a chrono DateTime
    pub fn datetime(dt: &chrono::DateTime<chrono::Utc>) -> Self {
        SortKey::DateTime(dt.timestamp_millis())
    }

    /// Create a composite sort key from multiple keys
    pub fn composite(keys: Vec<SortKey>) -> Self {
        SortKey::Composite(keys)
    }

    /// Create a sort key from a float (scaled to preserve precision)
    pub fn float(f: Option<f64>) -> Self {
        // Scale by 1000 to preserve 3 decimal places
        SortKey::OptionalNumber(NoneLast(f.map(|v| (v * 1000.0) as i64)))
    }
}

/// Trait for entities that can be sorted
///
/// Implement this trait to enable generic sorting for your entity type.
/// Each entity defines its own SortField enum and implements sort_key().
///
/// # Example
///
/// ```ignore
/// impl Sortable for Requirement {
///     type SortField = RequirementSortField;
///
///     fn sort_key(&self, field: &Self::SortField) -> SortKey {
///         match field {
///             RequirementSortField::Title => SortKey::string(&self.title),
///             RequirementSortField::Created => SortKey::datetime(&self.created),
///             // ...
///         }
///     }
/// }
/// ```
pub trait Sortable {
    /// The enum type defining valid sort fields for this entity
    type SortField;

    /// Get a comparable sort key for the given field
    fn sort_key(&self, field: &Self::SortField) -> SortKey;
}

/// Sort a slice of entities by the given field and direction
///
/// This function provides generic sorting for any entity implementing Sortable.
///
/// # Example
///
/// ```ignore
/// let mut requirements = service.load_all()?;
/// sort_entities(&mut requirements, RequirementSortField::Title, SortDirection::Ascending);
/// ```
pub fn sort_entities<T: Sortable>(items: &mut [T], field: T::SortField, direction: SortDirection)
where
    T::SortField: Copy,
{
    items.sort_by(|a, b| {
        let key_a = a.sort_key(&field);
        let key_b = b.sort_key(&field);

        match direction {
            SortDirection::Ascending => key_a.cmp(&key_b),
            SortDirection::Descending => key_b.cmp(&key_a),
        }
    });
}

/// Result of a list operation with pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ListResult<T> {
    /// The items matching the filter
    pub items: Vec<T>,

    /// Total count before pagination
    pub total_count: usize,

    /// Whether there are more items
    pub has_more: bool,
}

impl<T> ListResult<T> {
    pub fn new(items: Vec<T>, total_count: usize, has_more: bool) -> Self {
        Self {
            items,
            total_count,
            has_more,
        }
    }

    pub fn empty() -> Self {
        Self {
            items: Vec::new(),
            total_count: 0,
            has_more: false,
        }
    }
}

/// Trait for services that support listing entities
///
/// This trait provides a common interface for list operations across all entity services.
/// Both full entity loading and cached (fast) loading are supported.
///
/// # Example
///
/// ```ignore
/// fn list_entities<S, E, C, F, SF>(service: &S, filter: F, sort: SF) -> ServiceResult<Vec<E>>
/// where
///     S: ListableService<E, C, F, SF>,
/// {
///     let result = service.list(&filter, sort, SortDirection::Ascending)?;
///     Ok(result.items)
/// }
/// ```
pub trait ListableService<Entity, CachedEntity, Filter, SortField> {
    /// List entities with full data (required for JSON/YAML output)
    fn list(
        &self,
        filter: &Filter,
        sort_by: SortField,
        sort_dir: SortDirection,
    ) -> ServiceResult<ListResult<Entity>>;

    /// List entities using cached data (fast path for table output)
    fn list_cached(&self, filter: &Filter) -> ServiceResult<ListResult<CachedEntity>>;
}

/// Helper to apply pagination to a vector
pub fn apply_pagination<T>(items: Vec<T>, offset: Option<usize>, limit: Option<usize>) -> ListResult<T> {
    let total_count = items.len();
    let offset = offset.unwrap_or(0);

    let items: Vec<T> = items.into_iter().skip(offset).collect();
    let items_after_offset = items.len();

    let (items, has_more) = if let Some(limit) = limit {
        let has_more = items_after_offset > limit;
        (items.into_iter().take(limit).collect(), has_more)
    } else {
        (items, false)
    };

    ListResult::new(items, total_count, has_more)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_common_filter_status() {
        let filter = CommonFilter {
            status: Some(vec![Status::Draft, Status::Review]),
            ..Default::default()
        };

        assert!(filter.matches_status(&Status::Draft));
        assert!(filter.matches_status(&Status::Review));
        assert!(!filter.matches_status(&Status::Approved));
    }

    #[test]
    fn test_common_filter_author() {
        let filter = CommonFilter {
            author: Some("john".into()),
            ..Default::default()
        };

        assert!(filter.matches_author("John Smith"));
        assert!(filter.matches_author("johnson"));
        assert!(!filter.matches_author("Jane Doe"));
    }

    #[test]
    fn test_common_filter_search() {
        let filter = CommonFilter {
            search: Some("safety".into()),
            ..Default::default()
        };

        assert!(filter.matches_search(&["This is a safety requirement"]));
        assert!(filter.matches_search(&["SAFETY CRITICAL"]));
        assert!(!filter.matches_search(&["This is a performance requirement"]));
    }

    #[test]
    fn test_pagination() {
        let items: Vec<i32> = (1..=10).collect();

        // No pagination
        let result = apply_pagination(items.clone(), None, None);
        assert_eq!(result.items.len(), 10);
        assert_eq!(result.total_count, 10);
        assert!(!result.has_more);

        // With limit
        let result = apply_pagination(items.clone(), None, Some(5));
        assert_eq!(result.items, vec![1, 2, 3, 4, 5]);
        assert_eq!(result.total_count, 10);
        assert!(result.has_more);

        // With offset
        let result = apply_pagination(items.clone(), Some(5), None);
        assert_eq!(result.items, vec![6, 7, 8, 9, 10]);
        assert_eq!(result.total_count, 10);
        assert!(!result.has_more);

        // With both
        let result = apply_pagination(items, Some(2), Some(3));
        assert_eq!(result.items, vec![3, 4, 5]);
        assert_eq!(result.total_count, 10);
        assert!(result.has_more);
    }

    #[test]
    fn test_sort_key_ordering() {
        // String keys
        assert!(SortKey::string("apple") < SortKey::string("banana"));
        assert!(SortKey::string("zebra") > SortKey::string("apple"));

        // Number keys
        assert!(SortKey::number(1) < SortKey::number(10));
        assert!(SortKey::number(-5) < SortKey::number(0));

        // Optional string - None sorts after Some
        assert!(SortKey::optional_string(Some("a")) < SortKey::optional_string(None::<String>));
        assert!(SortKey::optional_string(Some("a")) < SortKey::optional_string(Some("b")));

        // Optional number - None sorts after Some
        assert!(SortKey::optional_number(Some(1)) < SortKey::optional_number(None));
        assert!(SortKey::optional_number(Some(5)) < SortKey::optional_number(Some(10)));
    }

    #[test]
    fn test_sort_key_float() {
        // Float with 3 decimal precision
        let key1 = SortKey::float(Some(1.234));
        let key2 = SortKey::float(Some(1.235));
        let key3 = SortKey::float(None);

        assert!(key1 < key2);
        assert!(key2 < key3); // None sorts last
    }

    // Test struct for sort_entities
    #[derive(Debug, Clone, PartialEq)]
    struct TestItem {
        name: String,
        value: i64,
    }

    #[derive(Debug, Clone, Copy)]
    enum TestSortField {
        Name,
        Value,
    }

    impl Sortable for TestItem {
        type SortField = TestSortField;

        fn sort_key(&self, field: &Self::SortField) -> SortKey {
            match field {
                TestSortField::Name => SortKey::string(&self.name),
                TestSortField::Value => SortKey::number(self.value),
            }
        }
    }

    #[test]
    fn test_sort_entities_ascending() {
        let mut items = vec![
            TestItem { name: "Charlie".into(), value: 3 },
            TestItem { name: "Alice".into(), value: 1 },
            TestItem { name: "Bob".into(), value: 2 },
        ];

        sort_entities(&mut items, TestSortField::Name, SortDirection::Ascending);

        assert_eq!(items[0].name, "Alice");
        assert_eq!(items[1].name, "Bob");
        assert_eq!(items[2].name, "Charlie");
    }

    #[test]
    fn test_sort_entities_descending() {
        let mut items = vec![
            TestItem { name: "A".into(), value: 1 },
            TestItem { name: "B".into(), value: 2 },
            TestItem { name: "C".into(), value: 3 },
        ];

        sort_entities(&mut items, TestSortField::Value, SortDirection::Descending);

        assert_eq!(items[0].value, 3);
        assert_eq!(items[1].value, 2);
        assert_eq!(items[2].value, 1);
    }

    #[test]
    fn test_sort_key_composite() {
        // Composite keys compare element by element
        let key1 = SortKey::composite(vec![SortKey::string("A"), SortKey::number(1)]);
        let key2 = SortKey::composite(vec![SortKey::string("A"), SortKey::number(2)]);
        let key3 = SortKey::composite(vec![SortKey::string("B"), SortKey::number(1)]);

        assert!(key1 < key2); // Same first element, compare second
        assert!(key2 < key3); // Different first element wins
    }

    #[test]
    fn test_matches_status_str() {
        let filter = CommonFilter {
            status: Some(vec![Status::Draft, Status::Review]),
            ..Default::default()
        };

        assert!(filter.matches_status_str("draft"));
        assert!(filter.matches_status_str("review"));
        assert!(!filter.matches_status_str("approved"));
        assert!(!filter.matches_status_str("invalid")); // Invalid status doesn't match
    }

    #[test]
    fn test_matches_status_str_no_filter() {
        let filter = CommonFilter::default();

        // When no filter is set, everything matches
        assert!(filter.matches_status_str("draft"));
        assert!(filter.matches_status_str("approved"));
    }

    // Test entity for matches_entity tests
    use crate::core::identity::{EntityId, EntityPrefix};
    use chrono::{DateTime, Utc};
    use serde::{Deserialize, Serialize};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestEntity {
        id: EntityId,
        title: String,
        status: Status,
        author: String,
        created: DateTime<Utc>,
    }

    impl crate::core::entity::Entity for TestEntity {
        const PREFIX: &'static str = "TEST";

        fn id(&self) -> &EntityId {
            &self.id
        }

        fn title(&self) -> &str {
            &self.title
        }

        fn status(&self) -> &str {
            match self.status {
                Status::Draft => "draft",
                Status::Review => "review",
                Status::Approved => "approved",
                Status::Released => "released",
                Status::Obsolete => "obsolete",
            }
        }

        fn created(&self) -> DateTime<Utc> {
            self.created
        }

        fn author(&self) -> &str {
            &self.author
        }
    }

    fn make_test_entity(title: &str, status: Status, author: &str) -> TestEntity {
        TestEntity {
            id: EntityId::new(EntityPrefix::Test),
            title: title.to_string(),
            status,
            author: author.to_string(),
            created: Utc::now(),
        }
    }

    #[test]
    fn test_matches_entity_no_filters() {
        let filter = CommonFilter::default();
        let entity = make_test_entity("Test Entity", Status::Draft, "John");

        assert!(filter.matches_entity(&entity));
    }

    #[test]
    fn test_matches_entity_status_filter() {
        let filter = CommonFilter {
            status: Some(vec![Status::Draft]),
            ..Default::default()
        };

        let draft = make_test_entity("Draft", Status::Draft, "John");
        let approved = make_test_entity("Approved", Status::Approved, "John");

        assert!(filter.matches_entity(&draft));
        assert!(!filter.matches_entity(&approved));
    }

    #[test]
    fn test_matches_entity_author_filter() {
        let filter = CommonFilter {
            author: Some("john".to_string()),
            ..Default::default()
        };

        let john = make_test_entity("Test", Status::Draft, "John Smith");
        let jane = make_test_entity("Test", Status::Draft, "Jane Doe");

        assert!(filter.matches_entity(&john));
        assert!(!filter.matches_entity(&jane));
    }

    #[test]
    fn test_matches_entity_search_filter() {
        let filter = CommonFilter {
            search: Some("safety".to_string()),
            ..Default::default()
        };

        let safety = make_test_entity("Safety Requirement", Status::Draft, "John");
        let performance = make_test_entity("Performance Test", Status::Draft, "John");

        assert!(filter.matches_entity(&safety));
        assert!(!filter.matches_entity(&performance));
    }

    #[test]
    fn test_matches_entity_combined_filters() {
        let filter = CommonFilter {
            status: Some(vec![Status::Draft, Status::Review]),
            author: Some("john".to_string()),
            search: Some("safety".to_string()),
            ..Default::default()
        };

        // Matches all
        let entity1 = make_test_entity("Safety Check", Status::Draft, "John Smith");
        assert!(filter.matches_entity(&entity1));

        // Wrong status
        let entity2 = make_test_entity("Safety Check", Status::Approved, "John Smith");
        assert!(!filter.matches_entity(&entity2));

        // Wrong author
        let entity3 = make_test_entity("Safety Check", Status::Draft, "Jane Doe");
        assert!(!filter.matches_entity(&entity3));

        // Wrong title (search doesn't match)
        let entity4 = make_test_entity("Performance Test", Status::Draft, "John Smith");
        assert!(!filter.matches_entity(&entity4));
    }
}
