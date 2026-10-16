//! # CRM Customer Intelligence Module
//!
//! Provides thread-safe customer lifecycle tracking, document associations,
//! deal metrics, and interaction history for local SME businesses.

use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerNote {
    pub timestamp: String,
    pub author: String,
    pub note: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CustomerRecord {
    pub id: String,
    pub name: String,
    pub company: String,
    pub email: String,
    pub phone: String,
    pub deal_value: f64,
    pub tags: Vec<String>,
    pub associated_docs: Vec<String>,
    pub notes: Vec<CustomerNote>,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Clone)]
pub struct CrmManager {
    records: Arc<RwLock<HashMap<String, CustomerRecord>>>,
}

impl Default for CrmManager {
    fn default() -> Self {
        Self::new()
    }
}

impl CrmManager {
    pub fn new() -> Self {
        let mut initial_map = HashMap::new();

        // Seed with sample SME business accounts
        let now = Utc::now().to_rfc3339();
        let sample1 = CustomerRecord {
            id: "CUST-001".to_string(),
            name: "Rajesh Sharma".to_string(),
            company: "Sharma Engineering Works".to_string(),
            email: "rajesh@sharmaeng.in".to_string(),
            phone: "+91 98765 43210".to_string(),
            deal_value: 2500000.0,
            tags: vec!["Enterprise".to_string(), "High-Value".to_string(), "GST-Compliant".to_string()],
            associated_docs: vec!["DOC-INV-2026-001".to_string(), "DOC-CONTRACT-SEW".to_string()],
            notes: vec![
                CustomerNote {
                    timestamp: now.clone(),
                    author: "Sales Lead".to_string(),
                    note: "Annual manufacturing contract finalized. Awaiting final Q1 invoice reconciliation.".to_string(),
                }
            ],
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        let sample2 = CustomerRecord {
            id: "CUST-002".to_string(),
            name: "Priya Nair".to_string(),
            company: "Nair Cloud Solutions".to_string(),
            email: "priya@naircloud.com".to_string(),
            phone: "+91 91234 56789".to_string(),
            deal_value: 1200000.0,
            tags: vec!["SME".to_string(), "Cloud-Services".to_string(), "Active-Client".to_string()],
            associated_docs: vec!["DOC-SLA-NCS".to_string()],
            notes: vec![
                CustomerNote {
                    timestamp: now.clone(),
                    author: "Account Manager".to_string(),
                    note: "Migrating on-premise inventory to local encrypted database node.".to_string(),
                }
            ],
            created_at: now.clone(),
            updated_at: now.clone(),
        };

        initial_map.insert(sample1.id.clone(), sample1);
        initial_map.insert(sample2.id.clone(), sample2);

        Self {
            records: Arc::new(RwLock::new(initial_map)),
        }
    }

    /// Retrieves all customer records sorted by update time
    pub async fn list_customers(&self) -> Vec<CustomerRecord> {
        let map = self.records.read().await;
        let mut list: Vec<CustomerRecord> = map.values().cloned().collect();
        list.sort_by(|a, b| b.updated_at.cmp(&a.updated_at));
        list
    }

    /// Finds customer by ID
    pub async fn get_customer(&self, id: &str) -> Option<CustomerRecord> {
        let map = self.records.read().await;
        map.get(id).cloned()
    }

    /// Upserts a customer record
    pub async fn upsert_customer(&self, mut customer: CustomerRecord) {
        customer.updated_at = Utc::now().to_rfc3339();
        if customer.created_at.is_empty() {
            customer.created_at = customer.updated_at.clone();
        }
        let mut map = self.records.write().await;
        map.insert(customer.id.clone(), customer);
    }

    /// Links an ingested document ID to a customer
    pub async fn associate_document(&self, customer_id: &str, doc_id: &str) -> bool {
        let mut map = self.records.write().await;
        if let Some(record) = map.get_mut(customer_id) {
            if !record.associated_docs.contains(&doc_id.to_string()) {
                record.associated_docs.push(doc_id.to_string());
                record.updated_at = Utc::now().to_rfc3339();
            }
            true
        } else {
            false
        }
    }

    /// Appends an interaction note to a customer profile
    pub async fn add_note(&self, customer_id: &str, author: &str, note: &str) -> bool {
        let mut map = self.records.write().await;
        if let Some(record) = map.get_mut(customer_id) {
            record.notes.push(CustomerNote {
                timestamp: Utc::now().to_rfc3339(),
                author: author.to_string(),
                note: note.to_string(),
            });
            record.updated_at = Utc::now().to_rfc3339();
            true
        } else {
            false
        }
    }
}
