-- Migration 010: Grounding System
-- Adds citation grounding (source quotes), factual grounding (verification),
-- and the memory_citations table for multi-source citation linking.

-- ── Add grounding columns to memories ──────────────────────────────────────

ALTER TABLE memories
  ADD COLUMN IF NOT EXISTS source_quote       TEXT,
  ADD COLUMN IF NOT EXISTS source_offset_start INTEGER,
  ADD COLUMN IF NOT EXISTS source_offset_end   INTEGER,
  ADD COLUMN IF NOT EXISTS verification_status TEXT NOT NULL DEFAULT 'unverified'
    CHECK (verification_status IN ('unverified', 'verified', 'contested', 'failed')),
  ADD COLUMN IF NOT EXISTS verification_score  REAL;

CREATE INDEX IF NOT EXISTS idx_memories_verification_status
  ON memories (verification_status) WHERE verification_status != 'unverified';

-- ── Memory citations table ─────────────────────────────────────────────────
-- Links a memory to one or more source episodes with exact quotes and offsets.

CREATE TABLE IF NOT EXISTS memory_citations (
  id                UUID PRIMARY KEY DEFAULT gen_random_uuid(),
  memory_id         UUID NOT NULL REFERENCES memories(id) ON DELETE CASCADE,
  episode_id        UUID NOT NULL REFERENCES episodes(id) ON DELETE CASCADE,
  source_quote      TEXT NOT NULL,
  offset_start      INTEGER,
  offset_end        INTEGER,
  confidence        REAL DEFAULT 1.0 CHECK (confidence >= 0 AND confidence <= 1),
  created_at        TIMESTAMPTZ NOT NULL DEFAULT now()
);

CREATE INDEX IF NOT EXISTS idx_memory_citations_memory
  ON memory_citations (memory_id);
CREATE INDEX IF NOT EXISTS idx_memory_citations_episode
  ON memory_citations (episode_id);

-- ── Seed grounding config keys into system_config ──────────────────────────

INSERT INTO system_config (key, value, value_type, category, description, env_override)
VALUES
  ('grounding.enable_citations',      'true',        'boolean', 'grounding',
   'Enable citation grounding — extract source quotes for each memory', ''),
  ('grounding.enable_verification',   'false',       'boolean', 'grounding',
   'Enable factual verification — second LLM pass to verify extracted memories', ''),
  ('grounding.verification_model',    '"same"',      'string',  'grounding',
   'Model to use for verification (\"same\" = use extraction model)', ''),
  ('grounding.min_verification_score', '0.5',        'number',  'grounding',
   'Minimum verification score to mark a memory as verified (0.0-1.0)', '')
ON CONFLICT (key) DO NOTHING;
