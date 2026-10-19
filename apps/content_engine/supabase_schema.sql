-- ============================================================================
-- Shambhavi AI Labs — Content Engine Enterprise PostgreSQL Schema
-- Supabase Cloud Production Database (Autonomous Omnichannel Media Matrix)
-- Project: hbnwmjrrhvpibtqsucqz
-- ============================================================================

-- Enable pgcrypto for UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- 1. MASTER CONTENT PIECES TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.content_pieces (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic TEXT NOT NULL,
    status TEXT NOT NULL DEFAULT 'generated' CHECK (
        status IN ('draft', 'generated', 'reviewing', 'approved', 'published', 'archived')
    ),
    -- 60-Second Short Video Assets
    short_hook TEXT,
    short_cta TEXT,
    short_script_json JSONB DEFAULT '{}'::jsonb,
    full_spoken_transcript TEXT,

    -- Long-Form YouTube Script (Markdown with Timestamps & Chapters)
    long_form_script TEXT,

    -- 1,200-Word SEO Blog Article (Markdown with H1-H3 & FAQ)
    blog_article TEXT,

    -- Twitter (X) Viral Thread (Array of Tweets)
    twitter_thread JSONB DEFAULT '[]'::jsonb,

    -- Media URLs / Storage Paths
    audio_url TEXT,
    captions_ass TEXT,
    video_preview_url TEXT,

    -- Metadata & Analytics
    word_count INTEGER DEFAULT 0,
    estimated_duration_seconds NUMERIC(6, 2) DEFAULT 0.0,
    tags TEXT[] DEFAULT '{}',
    custom_metadata JSONB DEFAULT '{}'::jsonb,

    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- 2. PUBLISHING & SYNDICATION LOGS TABLE
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.publishing_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    content_id UUID NOT NULL REFERENCES public.content_pieces(id) ON DELETE CASCADE,
    platform TEXT NOT NULL CHECK (
        platform IN ('youtube_shorts', 'youtube_long', 'blog_seo', 'twitter_x', 'linkedin', 'instagram_reels')
    ),
    external_post_id TEXT,
    publish_status TEXT NOT NULL DEFAULT 'pending' CHECK (
        publish_status IN ('pending', 'scheduled', 'published', 'failed')
    ),
    scheduled_at TIMESTAMPTZ,
    published_at TIMESTAMPTZ,
    error_message TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- 3. TREND RECONNAISSANCE DOSSIERS TABLE (Research & Intel Radar)
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.trend_dossiers (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    topic TEXT NOT NULL,
    detected_viral_hooks JSONB DEFAULT '[]'::jsonb,
    competitor_videos JSONB DEFAULT '[]'::jsonb,
    news_articles JSONB DEFAULT '[]'::jsonb,
    reverse_engineered_tools TEXT[] DEFAULT '{}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- 4. HIGH-SPEED INDEXES
-- ============================================================================
CREATE INDEX IF NOT EXISTS idx_content_topic ON public.content_pieces (topic);
CREATE INDEX IF NOT EXISTS idx_content_status ON public.content_pieces (status, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_publishing_content ON public.publishing_logs (content_id, platform);
CREATE INDEX IF NOT EXISTS idx_trend_topic ON public.trend_dossiers (topic, created_at DESC);
