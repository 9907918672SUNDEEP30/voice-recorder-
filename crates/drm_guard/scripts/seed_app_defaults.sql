-- ============================================================================
-- Seed Out-of-the-Box Default Clients for All 8 Applications into Live D1
-- ============================================================================

INSERT OR REPLACE INTO clients (client_id, name, license_key, active, plan_tier, contact_phone, contact_email, created_at, updated_at)
VALUES
('cctv_guard_edge', 'CCTV Pehredar Default Node', 'PEHREDAR-SHAMBHAVI-24H-LEASE', 1, 'enterprise', '+919999999999', 'cctv@shambhaviailabs.com', 1770000000, 1770000000),
('nvr_core_box', 'NVR Core Appliance Node', 'NVR-SHAMBHAVI-24H-LEASE', 1, 'enterprise', '+919999999999', 'nvr@shambhaviailabs.com', 1770000000, 1770000000),
('voice_agent_cloud', 'Voice AI Calling Default Node', 'VOICE-SHAMBHAVI-24H-LEASE', 1, 'enterprise', '+919999999999', 'voice@shambhaviailabs.com', 1770000000, 1770000000),
('dsp_recorder_studio', 'DSP Audio Studio Default Node', 'DSP-SHAMBHAVI-24H-LEASE', 1, 'enterprise', '+919999999999', 'dsp@shambhaviailabs.com', 1770000000, 1770000000),
('video_studio_obs', 'Video Studio Default Node', 'VIDEO-SHAMBHAVI-24H-LEASE', 1, 'enterprise', '+919999999999', 'video@shambhaviailabs.com', 1770000000, 1770000000),
('financial_engine_pro', 'Financial Engine Default Node', 'FINANCIAL-SHAMBHAVI-24H-LEASE', 1, 'enterprise', '+919999999999', 'finance@shambhaviailabs.com', 1770000000, 1770000000),
('local_rag_vault', 'Local RAG Vault Default Node', 'RAG-SHAMBHAVI-24H-LEASE', 1, 'enterprise', '+919999999999', 'rag@shambhaviailabs.com', 1770000000, 1770000000),
('manifestor_user_01', 'Manifestation App Default User', 'MANIFEST-SHAMBHAVI-24H-LEASE', 1, 'enterprise', '+919999999999', 'manifest@shambhaviailabs.com', 1770000000, 1770000000),
('content_engine_creator', 'Content Engine Default Node', 'CONTENT-SHAMBHAVI-24H-LEASE', 1, 'enterprise', '+919999999999', 'content@shambhaviailabs.com', 1770000000, 1770000000);

-- Licenses and Quotas for each node
INSERT OR REPLACE INTO app_licenses (client_id, app_name, allowed, limits_json, created_at)
VALUES
('cctv_guard_edge', 'cctv_guard', 1, '{"max_cameras": 32}', 1770000000),
('nvr_core_box', 'nvr_core', 1, '{"max_cameras": 32}', 1770000000),
('voice_agent_cloud', 'voice_agent', 1, '{"max_daily_minutes": 5000, "max_calls": 2000}', 1770000000),
('dsp_recorder_studio', 'dsp_recorder', 1, '{"max_recording_hours": 500}', 1770000000),
('video_studio_obs', 'video_studio', 1, '{"max_render_minutes": 1000}', 1770000000),
('financial_engine_pro', 'financial_engine', 1, '{"max_special_searches": 5000}', 1770000000),
('local_rag_vault', 'local_rag_engine', 1, '{"max_documents": 500000, "max_daily_queries": 20000}', 1770000000),
('manifestor_user_01', 'manifestation_app', 1, '{"max_vision_renders": 1000}', 1770000000),
('content_engine_creator', 'content_engine', 1, '{"max_pipelines_daily": 200}', 1770000000);
