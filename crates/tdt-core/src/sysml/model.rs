//! Intermediate representation types for SysML v2 interchange

/// A SysML v2 package containing all exported/imported elements
#[derive(Debug, Clone, Default)]
pub struct SysmlPackage {
    pub name: String,
    pub requirements: Vec<SysmlRequirement>,
    pub verifications: Vec<SysmlVerificationCase>,
    pub parts: Vec<SysmlPartDef>,
    pub satisfy_rels: Vec<SatisfyRelationship>,
    /// Number of unrecognized SysML constructs that were skipped during parsing
    pub skipped_count: usize,
}

/// A SysML requirement definition
#[derive(Debug, Clone)]
pub struct SysmlRequirement {
    /// TDT entity ID (e.g. "REQ-01KC8FF44V...")
    pub tdt_id: String,
    /// SysML short ID (same as tdt_id for round-trip)
    pub short_id: String,
    /// PascalCase SysML name
    pub name: String,
    /// Requirement text (doc block)
    pub doc: String,
    /// TDT priority
    pub priority: Option<String>,
    /// TDT level
    pub level: Option<String>,
    /// TDT status
    pub status: Option<String>,
    /// TDT author
    pub author: Option<String>,
    /// TDT category
    pub category: Option<String>,
    /// TDT tags
    pub tags: Vec<String>,
    /// TDT rationale
    pub rationale: Option<String>,
    /// TDT requirement type (input/output)
    pub req_type: Option<String>,
}

/// A SysML verification case definition
#[derive(Debug, Clone)]
pub struct SysmlVerificationCase {
    /// TDT entity ID (e.g. "TEST-01KC99PD5A...")
    pub tdt_id: String,
    /// SysML short ID (same as tdt_id for round-trip)
    pub short_id: String,
    /// PascalCase SysML name
    pub name: String,
    /// Objective text (doc block)
    pub doc: String,
    /// Verification method (inspection, analysis, demonstration, test)
    pub method: Option<String>,
    /// SysML names of requirements this verifies
    pub verifies: Vec<String>,
    /// TDT status
    pub status: Option<String>,
    /// TDT author
    pub author: Option<String>,
}

/// A SysML part definition
#[derive(Debug, Clone)]
pub struct SysmlPartDef {
    /// TDT entity ID (e.g. "CMP-01KC8FHP5Z...")
    pub tdt_id: String,
    /// SysML short ID (same as tdt_id for round-trip)
    pub short_id: String,
    /// PascalCase SysML name
    pub name: String,
    /// Description text (doc block)
    pub doc: Option<String>,
}

/// A satisfy relationship between a requirement and a part/design element
#[derive(Debug, Clone)]
pub struct SatisfyRelationship {
    /// SysML name of the requirement being satisfied
    pub requirement_name: String,
    /// SysML name of the element satisfying the requirement
    pub satisfied_by: String,
}

/// Results from importing a SysML file into TDT entities
#[derive(Debug, Default)]
pub struct ImportResult {
    /// YAML content keyed by (prefix, entity_id, title) for writing to files
    pub entities: Vec<ImportedEntity>,
    /// Warnings generated during import
    pub warnings: Vec<String>,
    /// Number of constructs that were skipped
    pub skipped_constructs: usize,
}

/// A single entity produced by import
#[derive(Debug)]
pub struct ImportedEntity {
    /// Entity prefix (e.g. "REQ", "TEST", "CMP")
    pub prefix: String,
    /// Entity ID
    pub id: String,
    /// Entity title
    pub title: String,
    /// Serialized YAML content
    pub yaml: String,
}
