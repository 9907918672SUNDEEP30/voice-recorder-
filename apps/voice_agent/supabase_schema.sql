-- =========================================================================
-- Shambhavi AI Labs: Cloud Voice AI Calling Agent Database Schema
-- Supabase PostgreSQL + pgvector
-- =========================================================================

-- 1. Enable pgvector extension
CREATE EXTENSION IF NOT EXISTS vector;

-- 2. Agent Configuration Table (Persona, System Prompt & Voice Settings)
CREATE TABLE IF NOT EXISTS agent_config (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id TEXT NOT NULL UNIQUE,
    persona_name TEXT NOT NULL DEFAULT 'Maya',
    language TEXT NOT NULL DEFAULT 'hi-IN',
    system_prompt TEXT NOT NULL,
    greeting_message TEXT NOT NULL,
    fallback_phone_number TEXT,
    voice_speed REAL DEFAULT 1.0,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- 3. Knowledge Base Documents (BGE Small 384-dimensional Embeddings)
CREATE TABLE IF NOT EXISTS knowledge_documents (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id TEXT NOT NULL,
    title TEXT NOT NULL,
    category TEXT DEFAULT 'general',
    chunk_content TEXT NOT NULL,
    embedding VECTOR(384) NOT NULL, -- 384 dimensions matching BGE Small Embedding
    metadata JSONB DEFAULT '{}'::JSONB,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Fast cosine similarity vector index using ivfflat
CREATE INDEX IF NOT EXISTS idx_knowledge_documents_embedding 
ON knowledge_documents 
USING ivfflat (embedding vector_cosine_ops) 
WITH (lists = 100);

-- 4. Call Logs & Live Transcripts
CREATE TABLE IF NOT EXISTS call_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    client_id TEXT NOT NULL,
    call_sid TEXT NOT NULL UNIQUE,
    caller_phone TEXT NOT NULL,
    direction TEXT NOT NULL CHECK (direction IN ('inbound', 'outbound')),
    status TEXT NOT NULL CHECK (status IN ('in_progress', 'completed', 'failed', 'busy', 'no_answer')),
    duration_seconds INTEGER DEFAULT 0,
    transcript TEXT,
    full_dialogue JSONB DEFAULT '[]'::JSONB,
    sentiment_score REAL DEFAULT 0.0,
    audio_recording_url TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    ended_at TIMESTAMPTZ
);

-- Index for instant Next.js dashboard query filtering
CREATE INDEX IF NOT EXISTS idx_call_logs_client_created 
ON call_logs (client_id, created_at DESC);

-- 5. Helper Function for Real-Time Semantic RAG Search during Phone Calls
CREATE OR REPLACE FUNCTION match_knowledge_documents (
    query_embedding VECTOR(384),
    match_threshold FLOAT,
    match_count INT,
    filter_client_id TEXT
)
RETURNS TABLE (
    id UUID,
    title TEXT,
    chunk_content TEXT,
    similarity FLOAT
)
LANGUAGE plpgsql
AS $$
BEGIN
    RETURN QUERY
    SELECT
        kd.id,
        kd.title,
        kd.chunk_content,
        1 - (kd.embedding <=> query_embedding) AS similarity
    FROM knowledge_documents kd
    WHERE kd.client_id = filter_client_id
      AND 1 - (kd.embedding <=> query_embedding) > match_threshold
    ORDER BY kd.embedding <=> query_embedding
    LIMIT match_count;
END;
$$;
