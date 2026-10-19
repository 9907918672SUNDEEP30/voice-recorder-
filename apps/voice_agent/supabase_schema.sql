-- ============================================================================
-- Shambhavi AI Labs — Voice AI Calling Agent Enterprise PostgreSQL CRM Schema
-- Supabase Cloud Production Database (Self-Driving Autonomous Tele-Calling)
-- ============================================================================

-- Enable pgcrypto for UUID generation
CREATE EXTENSION IF NOT EXISTS "pgcrypto";

-- ============================================================================
-- 1. LEADS MASTER TABLE (10,000+ Bulk Numbers & Lead State Machine)
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.leads (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    phone_number TEXT NOT NULL UNIQUE,
    customer_name TEXT,
    company_name TEXT,
    custom_context JSONB DEFAULT '{}'::jsonb,
    status TEXT NOT NULL DEFAULT 'pending' CHECK (
        status IN (
            'pending',
            'in_progress',
            'completed',
            'scheduled_callback',
            'meeting_booked',
            'refund_requested',
            'not_interested',
            'dnd',
            'invalid_number'
        )
    ),
    call_attempts INTEGER NOT NULL DEFAULT 0,
    max_retries INTEGER NOT NULL DEFAULT 3,
    scheduled_at TIMESTAMPTZ,              -- Exact timestamp for scheduled callback (e.g. tomorrow at 4 PM)
    last_called_at TIMESTAMPTZ,
    priority INTEGER NOT NULL DEFAULT 1,   -- 1 = Normal, 2 = High Priority, 3 = Urgent
    assigned_agent TEXT DEFAULT 'voice_agent_core',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- 2. CALL LOGS & AI TRANSCRIPTS TABLE (Whisper + Qwen Deep Memory)
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.call_logs (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lead_id UUID REFERENCES public.leads(id) ON DELETE CASCADE,
    phone_number TEXT NOT NULL,
    direction TEXT NOT NULL DEFAULT 'outbound' CHECK (direction IN ('inbound', 'outbound', 'browser_test')),
    call_duration_seconds INTEGER NOT NULL DEFAULT 0,
    recording_url TEXT,
    stream_sid TEXT,
    status TEXT NOT NULL DEFAULT 'completed' CHECK (
        status IN ('completed', 'busy', 'no_answer', 'voicemail', 'failed', 'caller_hangup')
    ),
    transcript JSONB NOT NULL DEFAULT '[]'::jsonb, -- Array of turn records: { role, content, timestamp_ms }
    full_transcript_text TEXT,                     -- Full plaintext searchable transcript
    summary TEXT,                                  -- Qwen LLM generated concise business summary
    customer_intent TEXT,                          -- 'callback_requested', 'meeting_booked', 'refund_inquiry', 'not_interested', etc.
    sentiment TEXT CHECK (sentiment IN ('positive', 'neutral', 'negative', 'interested', 'hostile')),
    disposition TEXT,                              -- 'follow_up_needed', 'meeting_set', 'deal_closed', 'closed_lost', 'dnd'
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- 3. SCHEDULED CALLBACKS TABLE (Autonomous Precision Follow-Up Engine)
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.scheduled_callbacks (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lead_id UUID NOT NULL REFERENCES public.leads(id) ON DELETE CASCADE,
    call_log_id UUID REFERENCES public.call_logs(id) ON DELETE SET NULL,
    phone_number TEXT NOT NULL,
    customer_name TEXT,
    callback_time TIMESTAMPTZ NOT NULL,            -- e.g. "2026-10-20 16:00:00+05:30"
    reason TEXT,                                   -- e.g. "Customer requested call after 4 PM"
    status TEXT NOT NULL DEFAULT 'pending' CHECK (status IN ('pending', 'processing', 'completed', 'cancelled')),
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    completed_at TIMESTAMPTZ
);

-- ============================================================================
-- 4. MEETING BOOKINGS TABLE (Calendar & Strategic Meeting Closing)
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.meeting_bookings (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lead_id UUID REFERENCES public.leads(id) ON DELETE CASCADE,
    call_log_id UUID REFERENCES public.call_logs(id) ON DELETE SET NULL,
    attendee_name TEXT NOT NULL,
    attendee_phone TEXT NOT NULL,
    attendee_email TEXT,
    meeting_title TEXT NOT NULL DEFAULT 'Shambhavi AI Solutions Strategic Discussion',
    meeting_time TIMESTAMPTZ NOT NULL,             -- e.g. 11:00 AM meeting timestamp
    meeting_duration_minutes INTEGER NOT NULL DEFAULT 30,
    google_calendar_event_id TEXT,
    status TEXT NOT NULL DEFAULT 'confirmed' CHECK (status IN ('confirmed', 'rescheduled', 'cancelled')),
    notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- ============================================================================
-- 5. ACTION & REFUND TICKETS TABLE (Autonomous Operations Execution)
-- ============================================================================
CREATE TABLE IF NOT EXISTS public.action_tickets (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    lead_id UUID REFERENCES public.leads(id) ON DELETE CASCADE,
    call_log_id UUID REFERENCES public.call_logs(id) ON DELETE SET NULL,
    ticket_type TEXT NOT NULL CHECK (
        ticket_type IN ('refund_request', 'policy_inquiry', 'payment_support', 'tech_issue', 'general_support')
    ),
    description TEXT NOT NULL,
    amount NUMERIC(12, 2),                         -- e.g. 5000.00
    policy_clause_cited TEXT,                      -- e.g. "7-day money-back refund guarantee"
    status TEXT NOT NULL DEFAULT 'open' CHECK (status IN ('open', 'processing', 'resolved', 'rejected')),
    resolution_notes TEXT,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    resolved_at TIMESTAMPTZ
);

-- ============================================================================
-- 6. HIGH-SPEED INDEXES (Sub-5ms Batch Lead Retrieval for 10,000+ records)
-- ============================================================================
CREATE INDEX IF NOT EXISTS idx_leads_dialer ON public.leads (status, scheduled_at, priority DESC);
CREATE INDEX IF NOT EXISTS idx_leads_phone ON public.leads (phone_number);
CREATE INDEX IF NOT EXISTS idx_callbacks_due ON public.scheduled_callbacks (status, callback_time ASC);
CREATE INDEX IF NOT EXISTS idx_call_logs_lead ON public.call_logs (lead_id, created_at DESC);
CREATE INDEX IF NOT EXISTS idx_meetings_time ON public.meeting_bookings (meeting_time ASC);
CREATE INDEX IF NOT EXISTS idx_action_tickets_status ON public.action_tickets (status, ticket_type);

-- ============================================================================
-- 7. SEED INITIAL SAMPLE LEADS (To Test Immediate Dialing & Processing)
-- ============================================================================
INSERT INTO public.leads (phone_number, customer_name, company_name, custom_context, status, priority)
VALUES
('+919876543210', 'Rahul Sharma', 'Apex Logistics', '{"product": "CCTV Pehredar Box", "notes": "Interested in 16 camera model"}'::jsonb, 'pending', 2),
('+919812345678', 'Pooja Verma', 'Verma Tech Labs', '{"product": "Voice AI Agent", "notes": "Wants automated customer support"}'::jsonb, 'pending', 1),
('+919988776655', 'Anand Kulkarni', 'Kulkarni Enterprises', '{"product": "Financial Super Engine", "notes": "NSE/BSE portfolio tracking"}'::jsonb, 'pending', 3)
ON CONFLICT (phone_number) DO NOTHING;
