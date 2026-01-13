//! Schema introspection for AI agent ergonomics
//!
//! Provides commands to view entity schemas, making it easier for AI agents
//! and automation tools to understand entity structure without external documentation.

use clap::Subcommand;
use miette::{IntoDiagnostic, Result};
use serde_json::Value;
use std::collections::BTreeMap;
use tdt_core::{EntityPrefix, SchemaRegistry};

#[derive(Subcommand, Debug)]
pub enum SchemaCommands {
    /// List all available entity schemas
    List,

    /// Show detailed schema for an entity type
    Show(ShowArgs),
}

#[derive(clap::Args, Debug)]
pub struct ShowArgs {
    /// Entity type (req, risk, cmp, etc.)
    pub entity: String,

    /// Show raw JSON schema instead of formatted summary
    #[arg(long)]
    pub raw: bool,
}

pub fn run(cmd: SchemaCommands) -> Result<()> {
    match cmd {
        SchemaCommands::List => list_schemas(),
        SchemaCommands::Show(args) => show_schema(args),
    }
}

fn list_schemas() -> Result<()> {
    let registry = SchemaRegistry::new();

    println!("Available entity schemas:\n");
    println!("{:<8} {:<20} DESCRIPTION", "TYPE", "TITLE");
    println!("{}", "-".repeat(70));

    for prefix in EntityPrefix::all() {
        let name = prefix.as_str().to_lowercase();
        if let Some(content) = registry.get(*prefix) {
            let schema: Value = serde_json::from_str(content).into_diagnostic()?;
            let title = schema["title"].as_str().unwrap_or(&name);
            let desc = schema["description"].as_str().unwrap_or("");
            // Truncate description if too long
            let desc_short = if desc.len() > 40 {
                format!("{}...", &desc[..37])
            } else {
                desc.to_string()
            };
            println!("{:<8} {:<20} {}", name, title, desc_short);
        }
    }

    println!("\nUse 'tdt schema show <type>' for field details");
    Ok(())
}

fn show_schema(args: ShowArgs) -> Result<()> {
    let registry = SchemaRegistry::new();

    // Find the prefix that matches the entity name
    let prefix = EntityPrefix::all()
        .iter()
        .find(|p| p.as_str().eq_ignore_ascii_case(&args.entity))
        .copied();

    let Some(prefix) = prefix else {
        eprintln!("Unknown entity type: {}", args.entity);
        eprintln!("\nAvailable types:");
        for p in EntityPrefix::all() {
            eprintln!("  {}", p.as_str().to_lowercase());
        }
        std::process::exit(1);
    };

    let Some(content) = registry.get(prefix) else {
        eprintln!("No schema found for: {}", args.entity);
        std::process::exit(1);
    };

    if args.raw {
        println!("{}", content);
        return Ok(());
    }

    let schema: Value = serde_json::from_str(content).into_diagnostic()?;

    // Print header
    let title = schema["title"].as_str().unwrap_or(&args.entity);
    let desc = schema["description"].as_str().unwrap_or("");
    println!("{}", title);
    println!("{}", "=".repeat(title.len()));
    if !desc.is_empty() {
        println!("{}\n", desc);
    }

    // Required fields
    let required: Vec<&str> = schema["required"]
        .as_array()
        .map(|arr| arr.iter().filter_map(|v| v.as_str()).collect())
        .unwrap_or_default();

    // Properties
    if let Some(props) = schema["properties"].as_object() {
        println!("Fields:");
        println!("{:<20} {:<12} {:<8} DESCRIPTION", "NAME", "TYPE", "REQ");
        println!("{}", "-".repeat(80));

        // Sort properties for consistent display
        let sorted: BTreeMap<_, _> = props.iter().collect();

        for (name, prop) in &sorted {
            let prop_type = get_type_str(prop);
            let is_required = if required.contains(&name.as_str()) {
                "yes"
            } else {
                ""
            };
            let prop_desc = prop["description"].as_str().unwrap_or("");
            // Truncate description
            let desc_short = if prop_desc.len() > 38 {
                format!("{}...", &prop_desc[..35])
            } else {
                prop_desc.to_string()
            };
            println!(
                "{:<20} {:<12} {:<8} {}",
                name, prop_type, is_required, desc_short
            );
        }

        // Show enum values for relevant fields
        println!("\nEnum Values:");
        for (name, prop) in &sorted {
            if let Some(enum_vals) = prop["enum"].as_array() {
                let vals: Vec<&str> = enum_vals.iter().filter_map(|v| v.as_str()).collect();
                println!("  {}: {}", name, vals.join(", "));
            }
        }

        // Show links section if present
        if let Some(links) = props.get("links") {
            if let Some(link_props) = links["properties"].as_object() {
                println!("\nLink Types (in 'links' field):");
                for (link_name, link_prop) in link_props {
                    let link_desc = link_prop["description"].as_str().unwrap_or("");
                    println!("  {:<20} {}", link_name, link_desc);
                }
            }
        }
    }

    println!("\nUse --raw for full JSON schema");
    Ok(())
}

fn get_type_str(prop: &Value) -> String {
    if let Some(t) = prop["type"].as_str() {
        if t == "array" {
            if let Some(items_type) = prop["items"]["type"].as_str() {
                return format!("{}[]", items_type);
            }
            return "array".to_string();
        }
        if t == "object" {
            return "object".to_string();
        }
        return t.to_string();
    }
    "any".to_string()
}
