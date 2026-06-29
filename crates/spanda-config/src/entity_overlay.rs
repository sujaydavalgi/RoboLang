//! Runtime entity mutation overlay and bi-directional TOML sync.
//!
use crate::entity::{
    EntityHealthStatus, EntityKind, EntityLifecycleState, EntityReadinessStatus, EntityRecord,
    EntityRegistry, EntityRelationship, EntityRelationshipKind, EntityTrustStatus,
};
use crate::error::{ConfigError, ConfigResult};
use crate::manifest::SpandaManifest;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};

/// Durable overlay applied on top of configuration-backed entity registry.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityMutationStore {
    pub entities: HashMap<String, EntityRecord>,
    pub relationships: Vec<EntityRelationship>,
    #[serde(default)]
    pub version: u32,
}

/// Register a new or updated entity through the mutation API.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityRegisterRequest {
    pub id: String,
    pub entity_type: String,
    #[serde(default)]
    pub display_name: Option<String>,
    #[serde(default)]
    pub parent_id: Option<String>,
    #[serde(default)]
    pub capabilities: Vec<String>,
    #[serde(default)]
    pub metadata: HashMap<String, String>,
    #[serde(default)]
    pub tags: Vec<String>,
    #[serde(default)]
    pub persist: bool,
}

/// Add or remove tags on an entity overlay.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, Default)]
pub struct EntityTagRequest {
    #[serde(default)]
    pub add: Vec<String>,
    #[serde(default)]
    pub remove: Vec<String>,
}

/// Create a directed relationship between two entities.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntityRelateRequest {
    pub from_id: String,
    pub to_id: String,
    pub kind: String,
    #[serde(default)]
    pub label: Option<String>,
}

/// Result of syncing overlay entities back to TOML fragments.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EntitySyncResult {
    pub path: PathBuf,
    pub entities_written: usize,
    pub relationships_written: usize,
    pub created_fragment: bool,
}

/// Default path for persisted entity mutation overlay.
pub fn default_entity_overlay_path() -> PathBuf {
    std::env::var("SPANDA_ENTITY_OVERLAY_PATH")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from(".spanda/entity-overlays.json"))
}

/// Load overlay from disk, or empty store when missing.
pub fn load_entity_overlay(path: &Path) -> EntityMutationStore {
    let Ok(content) = std::fs::read_to_string(path) else {
        return EntityMutationStore::default();
    };
    serde_json::from_str(&content).unwrap_or_default()
}

/// Persist overlay JSON to disk.
pub fn save_entity_overlay(path: &Path, store: &EntityMutationStore) -> Result<(), String> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|e| e.to_string())?;
    }
    let content = serde_json::to_string_pretty(store).map_err(|e| e.to_string())?;
    std::fs::write(path, content).map_err(|e| e.to_string())
}

/// Merge mutation overlay entities and relationships into a registry.
pub fn apply_entity_mutation_overlay(registry: &mut EntityRegistry, store: &EntityMutationStore) {
    for (id, overlay) in &store.entities {
        registry.entities.insert(id.clone(), overlay.clone());
    }
    for edge in &store.relationships {
        if registry.entities.contains_key(&edge.from_id)
            && registry.entities.contains_key(&edge.to_id)
            && !registry
                .relationships
                .iter()
                .any(|existing| existing == edge)
        {
            registry.relationships.push(edge.clone());
        }
    }
}

/// Register or update an entity in the mutation overlay.
pub fn register_entity_overlay(
    store: &mut EntityMutationStore,
    request: &EntityRegisterRequest,
) -> EntityRecord {
    let entity_type = EntityKind::parse(&request.entity_type);
    let record = EntityRecord {
        id: request.id.clone(),
        name: Some(request.id.clone()),
        display_name: request
            .display_name
            .clone()
            .or_else(|| Some(request.id.clone())),
        entity_type,
        parent_id: request.parent_id.clone(),
        capabilities: request.capabilities.clone(),
        metadata: request.metadata.clone(),
        tags: request.tags.clone(),
        lifecycle_state: EntityLifecycleState::Active,
        health_status: EntityHealthStatus::Healthy,
        readiness_status: EntityReadinessStatus::Ready,
        trust_status: EntityTrustStatus::Trusted,
        ..Default::default()
    };
    store.entities.insert(request.id.clone(), record.clone());
    store.version = store.version.saturating_add(1);
    if let Some(parent_id) = request.parent_id.as_ref() {
        let edge = EntityRelationship {
            from_id: parent_id.clone(),
            to_id: request.id.clone(),
            kind: EntityRelationshipKind::Contains,
            label: Some("overlay_register".into()),
        };
        push_relationship(store, edge);
    }
    record
}

/// Apply tag additions and removals in the overlay.
pub fn tag_entity_overlay(
    store: &mut EntityMutationStore,
    entity_id: &str,
    request: &EntityTagRequest,
) -> Option<EntityRecord> {
    let record = store.entities.get_mut(entity_id)?;
    for tag in &request.add {
        if !record.tags.contains(tag) {
            record.tags.push(tag.clone());
        }
    }
    if !request.remove.is_empty() {
        let remove: HashSet<_> = request.remove.iter().cloned().collect();
        record.tags.retain(|tag| !remove.contains(tag));
    }
    store.version = store.version.saturating_add(1);
    Some(record.clone())
}

/// Add a relationship edge to the overlay store.
pub fn relate_entities_overlay(
    store: &mut EntityMutationStore,
    request: &EntityRelateRequest,
) -> EntityRelationship {
    let kind =
        EntityRelationshipKind::parse(&request.kind).unwrap_or(EntityRelationshipKind::DependsOn);
    let edge = EntityRelationship {
        from_id: request.from_id.clone(),
        to_id: request.to_id.clone(),
        kind,
        label: request.label.clone(),
    };
    push_relationship(store, edge.clone());
    store.version = store.version.saturating_add(1);
    edge
}

fn push_relationship(store: &mut EntityMutationStore, edge: EntityRelationship) {
    if store.relationships.iter().any(|existing| existing == &edge) {
        return;
    }
    store.relationships.push(edge);
}

/// Resolve TOML fragment paths for entity overlay sync.
pub fn entity_fragment_paths(project_root: &Path, manifest: &SpandaManifest) -> Vec<PathBuf> {
    let mut paths = Vec::new();
    if let Some(ref rel) = manifest.config.facilities {
        paths.push(project_root.join(rel));
    }
    paths.push(project_root.join(".spanda/entity-overrides.toml"));
    paths
}

/// Write overlay entities into a facilities or overrides TOML fragment.
pub fn sync_entity_overlay_to_toml(
    project_root: &Path,
    manifest: &SpandaManifest,
    store: &EntityMutationStore,
) -> ConfigResult<EntitySyncResult> {
    let paths = entity_fragment_paths(project_root, manifest);
    let target = paths
        .iter()
        .find(|path| path.exists())
        .cloned()
        .unwrap_or_else(|| {
            paths
                .last()
                .cloned()
                .unwrap_or_else(|| project_root.join(".spanda/entity-overrides.toml"))
        });
    let created_fragment = !target.exists();
    let mut value: toml::Value = if target.exists() {
        let content = std::fs::read_to_string(&target).map_err(|source| ConfigError::Io {
            path: target.clone(),
            source,
        })?;
        toml::from_str(&content).map_err(|source| ConfigError::TomlParse {
            path: target.clone(),
            source,
        })?
    } else {
        toml::Value::Table(toml::map::Map::new())
    };
    let mut entities_written = 0usize;
    for record in store.entities.values() {
        if write_entity_to_toml(&mut value, record) {
            entities_written += 1;
        }
    }
    let relationships_written = store.relationships.len();
    merge_relationship_hints(&mut value, &store.relationships);
    let out = toml::to_string_pretty(&value).map_err(|e| ConfigError::InvalidManifest {
        detail: e.to_string(),
    })?;
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).map_err(|source| ConfigError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    std::fs::write(&target, out).map_err(|source| ConfigError::Io {
        path: target.clone(),
        source,
    })?;
    Ok(EntitySyncResult {
        path: target,
        entities_written,
        relationships_written,
        created_fragment,
    })
}

fn write_entity_to_toml(value: &mut toml::Value, record: &EntityRecord) -> bool {
    match record.entity_type {
        EntityKind::Facility => upsert_table_row(value, "facilities", facility_row(record)),
        EntityKind::Building => upsert_table_row(value, "buildings", building_row(record)),
        EntityKind::Zone | EntityKind::Hazard => upsert_table_row(value, "zones", zone_row(record)),
        _ => upsert_table_row(value, "entity_kinds", entity_kind_row(record)),
    }
}

fn upsert_table_row(
    value: &mut toml::Value,
    table: &str,
    row: toml::map::Map<String, toml::Value>,
) -> bool {
    let id = row
        .get("id")
        .and_then(|v| v.as_str())
        .unwrap_or("")
        .to_string();
    if id.is_empty() {
        return false;
    }
    let root = value.as_table_mut().expect("toml root table");
    let entries = root
        .entry(table.to_string())
        .or_insert_with(|| toml::Value::Array(Vec::new()));
    let Some(arr) = entries.as_array_mut() else {
        return false;
    };
    if let Some(existing) = arr
        .iter_mut()
        .find(|entry| entry.get("id").and_then(|v| v.as_str()) == Some(id.as_str()))
    {
        *existing = toml::Value::Table(row);
    } else {
        arr.push(toml::Value::Table(row));
    }
    true
}

fn facility_row(record: &EntityRecord) -> toml::map::Map<String, toml::Value> {
    let mut row = toml::map::Map::new();
    row.insert("id".into(), toml::Value::String(record.id.clone()));
    if let Some(name) = record.display_name.as_ref() {
        row.insert("name".into(), toml::Value::String(name.clone()));
    }
    if let Some(profile) = record.metadata.get("compliance.profile") {
        row.insert(
            "compliance_profile".into(),
            toml::Value::String(profile.clone()),
        );
    }
    row
}

fn building_row(record: &EntityRecord) -> toml::map::Map<String, toml::Value> {
    let mut row = toml::map::Map::new();
    row.insert("id".into(), toml::Value::String(record.id.clone()));
    if let Some(name) = record.display_name.as_ref() {
        row.insert("name".into(), toml::Value::String(name.clone()));
    }
    if let Some(facility_id) = record.parent_id.as_ref() {
        row.insert(
            "facility_id".into(),
            toml::Value::String(facility_id.clone()),
        );
    }
    row
}

fn zone_row(record: &EntityRecord) -> toml::map::Map<String, toml::Value> {
    let mut row = toml::map::Map::new();
    row.insert("id".into(), toml::Value::String(record.id.clone()));
    if let Some(zone_type) = record.metadata.get("zone_type") {
        row.insert("type".into(), toml::Value::String(zone_type.clone()));
    }
    if let Some(building_id) = record.parent_id.as_ref() {
        row.insert(
            "building_id".into(),
            toml::Value::String(building_id.clone()),
        );
    }
    row
}

fn entity_kind_row(record: &EntityRecord) -> toml::map::Map<String, toml::Value> {
    let mut row = toml::map::Map::new();
    row.insert("id".into(), toml::Value::String(record.id.clone()));
    row.insert(
        "kind".into(),
        toml::Value::String(record.entity_type.as_str().to_string()),
    );
    if let Some(name) = record.display_name.as_ref() {
        row.insert("display_name".into(), toml::Value::String(name.clone()));
    }
    if !record.capabilities.is_empty() {
        row.insert(
            "capabilities".into(),
            toml::Value::Array(
                record
                    .capabilities
                    .iter()
                    .cloned()
                    .map(toml::Value::String)
                    .collect(),
            ),
        );
    }
    if let Some(package) = record.metadata.get("package") {
        row.insert("package".into(), toml::Value::String(package.clone()));
    }
    row
}

fn merge_relationship_hints(value: &mut toml::Value, relationships: &[EntityRelationship]) {
    if relationships.is_empty() {
        return;
    }
    let root = value.as_table_mut().expect("toml root table");
    let hints = relationships
        .iter()
        .map(|edge| {
            let mut row = toml::map::Map::new();
            row.insert("from_id".into(), toml::Value::String(edge.from_id.clone()));
            row.insert("to_id".into(), toml::Value::String(edge.to_id.clone()));
            row.insert(
                "kind".into(),
                toml::Value::String(edge.kind.as_str().to_string()),
            );
            if let Some(label) = edge.label.as_ref() {
                row.insert("label".into(), toml::Value::String(label.clone()));
            }
            toml::Value::Table(row)
        })
        .collect();
    root.insert(
        "entity_relationships".to_string(),
        toml::Value::Array(hints),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn register_and_apply_overlay() {
        let mut store = EntityMutationStore::default();
        let record = register_entity_overlay(
            &mut store,
            &EntityRegisterRequest {
                id: "bay-1".into(),
                entity_type: "custom".into(),
                display_name: Some("Bay 1".into()),
                parent_id: Some("warehouse-a".into()),
                capabilities: vec!["inspect".into()],
                metadata: HashMap::new(),
                tags: vec!["overlay".into()],
                persist: false,
            },
        );
        assert_eq!(record.id, "bay-1");
        let mut registry = EntityRegistry::default();
        registry.entities.insert(
            "warehouse-a".into(),
            EntityRecord {
                id: "warehouse-a".into(),
                entity_type: EntityKind::Facility,
                ..Default::default()
            },
        );
        apply_entity_mutation_overlay(&mut registry, &store);
        assert!(registry.get("bay-1").is_some());
    }

    #[test]
    fn sync_overlay_writes_entity_kinds_table() {
        let dir = std::env::temp_dir().join(format!("spanda-entity-sync-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        std::fs::create_dir_all(&dir).unwrap();
        let mut store = EntityMutationStore::default();
        register_entity_overlay(
            &mut store,
            &EntityRegisterRequest {
                id: "bay-2".into(),
                entity_type: "calibration_station".into(),
                display_name: Some("Bay 2".into()),
                parent_id: None,
                capabilities: vec!["calibrate".into()],
                metadata: HashMap::new(),
                tags: Vec::new(),
                persist: true,
            },
        );
        let manifest = SpandaManifest::default();
        let result = sync_entity_overlay_to_toml(&dir, &manifest, &store).unwrap();
        assert!(result.path.exists());
        assert!(result.entities_written >= 1);
        let content = std::fs::read_to_string(result.path).unwrap();
        assert!(content.contains("bay-2"));
    }
}
