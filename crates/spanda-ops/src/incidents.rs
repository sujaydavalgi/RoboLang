//! SRE incident workflow for Control Center operators.
//!
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;
use std::time::{SystemTime, UNIX_EPOCH};

/// Lifecycle state for an operational incident.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentStatus {
    Open,
    Acknowledged,
    Resolved,
}

/// Incident severity for paging and SLO impact.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum IncidentSeverity {
    Info,
    Warning,
    Critical,
}

/// Operator incident record linked to alerts and audit evidence.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Incident {
    pub id: String,
    pub title: String,
    pub description: String,
    pub severity: IncidentSeverity,
    pub status: IncidentStatus,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub source_alert_id: Option<String>,
    pub created_at_ms: f64,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub acknowledged_at_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolved_at_ms: Option<f64>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub assignee: Option<String>,
}

/// Ring buffer of incidents for Control Center SRE workflow.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct IncidentStore {
    incidents: VecDeque<Incident>,
    pub max_entries: usize,
}

impl IncidentStore {
    pub fn new(max_entries: usize) -> Self {
        Self {
            incidents: VecDeque::new(),
            max_entries,
        }
    }

    pub fn from_records(max_entries: usize, incidents: Vec<Incident>) -> Self {
        let mut store = Self::new(max_entries);
        for incident in incidents {
            store.push(incident);
        }
        store
    }

    pub fn push(&mut self, incident: Incident) {
        if self.incidents.len() >= self.max_entries {
            self.incidents.pop_front();
        }
        self.incidents.push_back(incident);
    }

    pub fn list_owned(&self) -> Vec<Incident> {
        self.incidents.iter().cloned().collect()
    }

    pub fn get_mut(&mut self, incident_id: &str) -> Option<&mut Incident> {
        self.incidents
            .iter_mut()
            .find(|incident| incident.id == incident_id)
    }

    pub fn create(
        &mut self,
        title: String,
        description: String,
        severity: IncidentSeverity,
        source_alert_id: Option<String>,
    ) -> Incident {
        let incident = Incident {
            id: format!("incident-{}", now_ms()),
            title,
            description,
            severity,
            status: IncidentStatus::Open,
            source_alert_id,
            created_at_ms: now_ms(),
            acknowledged_at_ms: None,
            resolved_at_ms: None,
            assignee: None,
        };
        self.push(incident.clone());
        incident
    }

    pub fn acknowledge(&mut self, incident_id: &str, assignee: Option<String>) -> Option<Incident> {
        let incident = self.get_mut(incident_id)?;
        if incident.status == IncidentStatus::Resolved {
            return None;
        }
        incident.status = IncidentStatus::Acknowledged;
        incident.acknowledged_at_ms = Some(now_ms());
        incident.assignee = assignee;
        Some(incident.clone())
    }

    pub fn resolve(&mut self, incident_id: &str) -> Option<Incident> {
        let incident = self.get_mut(incident_id)?;
        incident.status = IncidentStatus::Resolved;
        incident.resolved_at_ms = Some(now_ms());
        Some(incident.clone())
    }

    pub fn open_count(&self) -> usize {
        self.incidents
            .iter()
            .filter(|incident| incident.status == IncidentStatus::Open)
            .count()
    }

    pub fn acknowledged_count(&self) -> usize {
        self.incidents
            .iter()
            .filter(|incident| incident.status == IncidentStatus::Acknowledged)
            .count()
    }

    /// Mean time to resolve for resolved incidents (milliseconds), when available.
    pub fn mttr_hint_ms(&self) -> Option<f64> {
        let resolved: Vec<f64> = self
            .incidents
            .iter()
            .filter_map(|incident| {
                let resolved_at = incident.resolved_at_ms?;
                Some(resolved_at - incident.created_at_ms)
            })
            .collect();
        if resolved.is_empty() {
            return None;
        }
        Some(resolved.iter().sum::<f64>() / resolved.len() as f64)
    }

    /// Open an incident from a critical alert when no active incident references it.
    pub fn maybe_open_from_alert(&mut self, alert: &crate::alerting::Alert) -> Option<Incident> {
        if alert.severity != crate::alerting::AlertSeverity::Critical {
            return None;
        }
        let already_open = self.incidents.iter().any(|incident| {
            incident.source_alert_id.as_deref() == Some(alert.id.as_str())
                && incident.status != IncidentStatus::Resolved
        });
        if already_open {
            return None;
        }
        let severity = match alert.severity {
            crate::alerting::AlertSeverity::Critical => IncidentSeverity::Critical,
            crate::alerting::AlertSeverity::Warning => IncidentSeverity::Warning,
            crate::alerting::AlertSeverity::Info => IncidentSeverity::Info,
        };
        Some(self.create(
            format!("Alert: {:?}", alert.alert_type),
            alert.message.clone(),
            severity,
            Some(alert.id.clone()),
        ))
    }
}

pub fn now_ms() -> f64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs_f64() * 1000.0)
        .unwrap_or(0.0)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn incident_lifecycle_and_mttr() {
        let mut store = IncidentStore::new(10);
        let created = store.create(
            "fleet offline".into(),
            "scout agent unreachable".into(),
            IncidentSeverity::Critical,
            None,
        );
        store.acknowledge(&created.id, Some("oncall".into()));
        store.resolve(&created.id);
        assert!(store.mttr_hint_ms().is_some());
        assert_eq!(store.open_count(), 0);
    }

    #[test]
    fn incident_from_critical_alert() {
        let mut store = IncidentStore::new(10);
        let alert = crate::alerting::Alert {
            id: "alert-1".into(),
            alert_type: crate::alerting::AlertType::HealthCritical,
            severity: crate::alerting::AlertSeverity::Critical,
            message: "battery critical".into(),
            source: "rover".into(),
            timestamp_ms: 1.0,
            delivered_via: vec![],
        };
        assert!(store.maybe_open_from_alert(&alert).is_some());
        assert_eq!(store.open_count(), 1);
        assert!(store.maybe_open_from_alert(&alert).is_none());
    }
}
