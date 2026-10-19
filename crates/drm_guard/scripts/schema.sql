-- ============================================================================
-- Shambhavi AI Labs — Cloudflare D1 Universal DRM & Metered Usage SQL Schema
-- Pure Serverless Edge Relational Database (SQLite Engine on Cloudflare D1)
-- ============================================================================

-- 1. Clients & Organizations Master Ledger
CREATE TABLE IF NOT EXISTS clients (
    client_id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    license_key TEXT NOT NULL UNIQUE,
    active INTEGER NOT NULL DEFAULT 1,             -- 1 = Active, 0 = Suspended / Overdue
    plan_tier TEXT NOT NULL DEFAULT 'enterprise',  -- 'starter', 'pro', 'enterprise'
    contact_phone TEXT,
    contact_email TEXT,
    created_at INTEGER NOT NULL,                  -- Unix timestamp in seconds
    updated_at INTEGER NOT NULL
);

-- 2. App Permissions & Dynamic Metered Quotas (All 8 Applications)
CREATE TABLE IF NOT EXISTS app_licenses (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id TEXT NOT NULL,
    app_name TEXT NOT NULL,                        -- 'cctv_guard', 'nvr_core', 'voice_agent', 'dsp_recorder',
                                                  -- 'video_studio', 'financial_engine', 'local_rag_engine',
                                                  -- 'manifestation_app', 'content_engine'
    allowed INTEGER NOT NULL DEFAULT 1,
    limits_json TEXT NOT NULL DEFAULT '{}',       -- JSON string with app-specific limits
    created_at INTEGER NOT NULL,
    FOREIGN KEY(client_id) REFERENCES clients(client_id) ON DELETE CASCADE,
    UNIQUE(client_id, app_name)
);

-- 3. Hardware Device Bindings (Strict Anti-Piracy Single-Machine UUID Lock)
CREATE TABLE IF NOT EXISTS device_bindings (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id TEXT NOT NULL,
    app_name TEXT NOT NULL,
    machine_uuid TEXT NOT NULL,
    bound_at INTEGER NOT NULL,
    last_seen_at INTEGER NOT NULL,
    FOREIGN KEY(client_id) REFERENCES clients(client_id) ON DELETE CASCADE,
    UNIQUE(client_id, app_name)
);

-- 4. 10-Minute Health & Metered Usage Telemetry Logs
CREATE TABLE IF NOT EXISTS telemetry_logs (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    client_id TEXT NOT NULL,
    app_name TEXT NOT NULL,
    machine_uuid TEXT NOT NULL,
    timestamp INTEGER NOT NULL,
    metrics_json TEXT NOT NULL,
    wire_faults INTEGER DEFAULT 0,
    offline_cameras_json TEXT,
    storage_pct REAL DEFAULT 0.0,
    alert_dispatched INTEGER DEFAULT 0,
    created_at INTEGER NOT NULL
);

-- 5. Cryptographically Signed OTA Update Catalog
CREATE TABLE IF NOT EXISTS ota_releases (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    app_name TEXT NOT NULL,
    version TEXT NOT NULL,
    download_url TEXT NOT NULL,
    sha256 TEXT NOT NULL,
    signature_hex TEXT NOT NULL,
    mandatory INTEGER NOT NULL DEFAULT 0,
    release_notes TEXT,
    created_at INTEGER NOT NULL,
    UNIQUE(app_name, version)
);

-- High-Performance Binary Indexes (<2ms Edge Traversal)
CREATE INDEX IF NOT EXISTS idx_client_license ON clients(license_key);
CREATE INDEX IF NOT EXISTS idx_device_binding ON device_bindings(client_id, app_name);
CREATE INDEX IF NOT EXISTS idx_telemetry_client_app ON telemetry_logs(client_id, app_name, timestamp DESC);
CREATE INDEX IF NOT EXISTS idx_ota_app_version ON ota_releases(app_name, version);

-- ============================================================================
-- Seed Production & Testing Clients (Pre-loaded with Enterprise Quotas)
-- ============================================================================

-- 1. Default Enterprise Client (Production Ready)
INSERT OR REPLACE INTO clients (client_id, name, license_key, active, plan_tier, contact_phone, contact_email, created_at, updated_at)
VALUES (
    'sme_client_001',
    'Shambhavi Enterprise Alpha Partner',
    'KEY-ENTERPRISE-2026',
    1,
    'enterprise',
    '+919999999999',
    'enterprise@shambhaviailabs.com',
    1770000000,
    1770000000
);

-- 2. Demo Evaluation Client
INSERT OR REPLACE INTO clients (client_id, name, license_key, active, plan_tier, contact_phone, contact_email, created_at, updated_at)
VALUES (
    'client_demo',
    'Shambhavi Demo Evaluation Lab',
    'DEMO-KEY-2026',
    1,
    'starter',
    '+918888888888',
    'demo@shambhaviailabs.com',
    1770000000,
    1770000000
);

-- Enterprise Limits (All 8 Apps Enabled with High Quotas)
INSERT OR REPLACE INTO app_licenses (client_id, app_name, allowed, limits_json, created_at)
VALUES
('sme_client_001', 'cctv_guard', 1, '{"max_cameras": 32}', 1770000000),
('sme_client_001', 'nvr_core', 1, '{"max_cameras": 32}', 1770000000),
('sme_client_001', 'voice_agent', 1, '{"max_daily_minutes": 5000, "max_calls": 2000}', 1770000000),
('sme_client_001', 'dsp_recorder', 1, '{"max_recording_hours": 500}', 1770000000),
('sme_client_001', 'video_studio', 1, '{"max_render_minutes": 1000}', 1770000000),
('sme_client_001', 'financial_engine', 1, '{"max_special_searches": 5000}', 1770000000),
('sme_client_001', 'local_rag_engine', 1, '{"max_documents": 500000, "max_daily_queries": 20000}', 1770000000),
('sme_client_001', 'manifestation_app', 1, '{"max_vision_renders": 1000}', 1770000000),
('sme_client_001', 'content_engine', 1, '{"max_pipelines_daily": 200}', 1770000000);

-- Demo Limits (Restricted Quotas)
INSERT OR REPLACE INTO app_licenses (client_id, app_name, allowed, limits_json, created_at)
VALUES
('client_demo', 'cctv_guard', 1, '{"max_cameras": 4}', 1770000000),
('client_demo', 'nvr_core', 1, '{"max_cameras": 4}', 1770000000),
('client_demo', 'voice_agent', 1, '{"max_daily_minutes": 60, "max_calls": 50}', 1770000000),
('client_demo', 'dsp_recorder', 1, '{"max_recording_hours": 5}', 1770000000),
('client_demo', 'video_studio', 1, '{"max_render_minutes": 30}', 1770000000),
('client_demo', 'financial_engine', 1, '{"max_special_searches": 25}', 1770000000),
('client_demo', 'local_rag_engine', 1, '{"max_documents": 500, "max_daily_queries": 100}', 1770000000),
('client_demo', 'manifestation_app', 1, '{"max_vision_renders": 20}', 1770000000),
('client_demo', 'content_engine', 1, '{"max_pipelines_daily": 5}', 1770000000);
