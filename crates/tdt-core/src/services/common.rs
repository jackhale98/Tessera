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
    /// Check if an entity's status matches the filter
    pub fn matches_status(&self, status: &Status) -> bool {
        self.status
            .as_ref()
            .map(|statuses| statuses.contains(status))
            .unwrap_or(true)
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
}
