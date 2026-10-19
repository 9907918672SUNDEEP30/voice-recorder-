-- ============================================================================
-- Shambhavi AI Labs — Unified Auth, DRM & Billing Security Ledger
-- Central Database Schema (Supabase PostgreSQL / Edge D1 Compatible)
-- ============================================================================

CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- 1. Users Master Profile Table
CREATE TABLE IF NOT EXISTS public.users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    email TEXT NOT NULL UNIQUE,
    password_hash TEXT NOT NULL,
    full_name TEXT,
    tier_level TEXT NOT NULL DEFAULT 'tier2' CHECK (tier_level IN ('tier1', 'tier2', 'tier3', 'admin')),
    is_active BOOLEAN NOT NULL DEFAULT TRUE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 2. User Credits Ledger Table (Tier 1 Live Dual-Gate Metering)
CREATE TABLE IF NOT EXISTS public.user_credits (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    app_name TEXT NOT NULL, -- 'voice_agent' | 'content_engine' | 'all'
    voice_minutes_balance NUMERIC(10, 2) NOT NULL DEFAULT 60.00,  -- 60 free minutes initial
    video_units_balance NUMERIC(10, 2) NOT NULL DEFAULT 50.00,    -- 50 free video generation units
    reserved_credits NUMERIC(10, 2) NOT NULL DEFAULT 0.00,
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'suspended', 'depleted')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    CONSTRAINT unique_user_app UNIQUE (user_id, app_name)
);

-- 3. Hardware Bound Licenses Table (Tier 3 DRM Heartbeats)
CREATE TABLE IF NOT EXISTS public.licenses (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    license_key TEXT NOT NULL UNIQUE,
    machine_fingerprint TEXT,                      -- Bound CPU ID + Motherboard Serial SHA-256
    app_name TEXT NOT NULL,                        -- 'cctv_guard', 'nvr_core', 'dsp_recorder', 'video_studio', 'local_rag_engine'
    organization_name TEXT DEFAULT 'Enterprise Customer',
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked', 'expired')),
    max_devices INTEGER NOT NULL DEFAULT 1,
    heartbeat_interval_secs INTEGER NOT NULL DEFAULT 86400, -- 24 hours
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '365 days'),
    last_heartbeat_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. Credit Audit & Deduction Transactions Table (Tier 1 Metering)
CREATE TABLE IF NOT EXISTS public.credit_transactions (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    user_id UUID NOT NULL REFERENCES public.users(id) ON DELETE CASCADE,
    permit_id TEXT NOT NULL,
    app_name TEXT NOT NULL,
    units_deducted NUMERIC(10, 2) NOT NULL,
    balance_after NUMERIC(10, 2) NOT NULL,
    operation_type TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- High-speed indexes
CREATE INDEX IF NOT EXISTS idx_users_email ON public.users(email);
CREATE INDEX IF NOT EXISTS idx_user_credits_user ON public.user_credits(user_id, app_name);
CREATE INDEX IF NOT EXISTS idx_licenses_key ON public.licenses(license_key);
CREATE INDEX IF NOT EXISTS idx_licenses_machine ON public.licenses(machine_fingerprint, status);
CREATE INDEX IF NOT EXISTS idx_credit_tx_user ON public.credit_transactions(user_id, created_at DESC);
