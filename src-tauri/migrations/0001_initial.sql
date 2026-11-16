-- Initial app-owned schema. Demo identities are explicitly separated from WeChat.
PRAGMA foreign_keys = ON;

CREATE TABLE schema_migrations (
    version INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    checksum TEXT NOT NULL,
    applied_at_ms INTEGER NOT NULL
);

CREATE TABLE local_accounts (
    id TEXT PRIMARY KEY,
    source_kind TEXT NOT NULL DEFAULT 'greenbubbles'
        CHECK (source_kind IN ('greenbubbles', 'demo')),
    source_account_key TEXT NOT NULL UNIQUE,
    display_name TEXT NOT NULL,
    self_source_id TEXT,
    query_profile_name TEXT NOT NULL UNIQUE,
    source_root TEXT NOT NULL,
    source_version TEXT NOT NULL,
    active INTEGER NOT NULL DEFAULT 0 CHECK (active IN (0, 1)),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
CREATE UNIQUE INDEX ux_accounts_one_active
    ON local_accounts(active) WHERE active = 1;

CREATE TABLE contacts (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES local_accounts(id) ON DELETE CASCADE,
    source_peer_id TEXT NOT NULL,
    display_name TEXT NOT NULL,
    remark TEXT,
    nickname TEXT,
    last_seen_at_ms INTEGER,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    UNIQUE (account_id, source_peer_id),
    UNIQUE (id, account_id)
);
CREATE INDEX ix_contacts_account_name ON contacts(account_id, display_name);

CREATE TABLE conversations (
    id TEXT PRIMARY KEY,
    account_id TEXT NOT NULL REFERENCES local_accounts(id) ON DELETE CASCADE,
    contact_id TEXT NOT NULL UNIQUE,
    source_conversation_id TEXT NOT NULL,
    kind TEXT NOT NULL DEFAULT 'private' CHECK (kind = 'private'),
    title TEXT NOT NULL,
    last_message_at_ms INTEGER,
    last_read_at_ms INTEGER,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    FOREIGN KEY (contact_id, account_id)
        REFERENCES contacts(id, account_id) ON DELETE CASCADE,
    UNIQUE (account_id, source_conversation_id)
);
CREATE INDEX ix_conversations_recent
    ON conversations(account_id, last_message_at_ms DESC);

CREATE TABLE contact_profile_settings (
    contact_id TEXT PRIMARY KEY REFERENCES contacts(id) ON DELETE CASCADE,
    auto_update_enabled INTEGER NOT NULL DEFAULT 0
        CHECK (auto_update_enabled IN (0, 1)),
    profile_revision INTEGER NOT NULL DEFAULT 1 CHECK (profile_revision > 0),
    analysis_checkpoint_json TEXT CHECK (
        analysis_checkpoint_json IS NULL OR json_valid(analysis_checkpoint_json)
    ),
    last_analyzed_at_ms INTEGER,
    last_analysis_status TEXT NOT NULL DEFAULT 'never'
        CHECK (last_analysis_status IN ('never', 'success', 'partial', 'failed')),
    updated_at_ms INTEGER NOT NULL
);

CREATE TABLE profile_items (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    category TEXT NOT NULL CHECK (
        category IN ('relationship', 'communication', 'preference', 'boundary', 'recent')
    ),
    subject_key TEXT NOT NULL,
    subject_label TEXT NOT NULL,
    content TEXT NOT NULL CHECK (length(content) BETWEEN 1 AND 1000),
    source_kind TEXT NOT NULL CHECK (
        source_kind IN ('manual', 'chat_fact', 'user_confirmed')
    ),
    locked INTEGER NOT NULL DEFAULT 1 CHECK (locked IN (0, 1)),
    scope TEXT NOT NULL DEFAULT 'long_term' CHECK (scope IN ('long_term', 'recent')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'archived')),
    valid_from_ms INTEGER NOT NULL,
    expires_at_ms INTEGER,
    revision INTEGER NOT NULL DEFAULT 1 CHECK (revision > 0),
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL,
    CHECK (expires_at_ms IS NULL OR expires_at_ms > valid_from_ms),
    CHECK (scope <> 'recent' OR expires_at_ms IS NOT NULL),
    CHECK (category <> 'recent' OR scope = 'recent')
);
CREATE UNIQUE INDEX ux_profile_items_active_subject
    ON profile_items(contact_id, subject_key, scope) WHERE status = 'active';
CREATE INDEX ix_profile_items_contact_category
    ON profile_items(contact_id, status, category, updated_at_ms DESC);

CREATE TABLE profile_proposals (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    kind TEXT NOT NULL CHECK (kind IN ('fact', 'observation', 'conflict')),
    operation TEXT NOT NULL CHECK (operation IN ('add', 'update')),
    target_item_id TEXT REFERENCES profile_items(id) ON DELETE SET NULL,
    target_revision INTEGER,
    applied_item_id TEXT REFERENCES profile_items(id) ON DELETE CASCADE,
    category TEXT NOT NULL CHECK (
        category IN ('relationship', 'communication', 'preference', 'boundary', 'recent')
    ),
    subject_key TEXT NOT NULL,
    subject_label TEXT NOT NULL,
    proposed_content TEXT NOT NULL CHECK (length(proposed_content) BETWEEN 1 AND 1000),
    scope TEXT NOT NULL DEFAULT 'long_term' CHECK (scope IN ('long_term', 'recent')),
    proposed_expires_at_ms INTEGER,
    status TEXT NOT NULL DEFAULT 'pending'
        CHECK (status IN ('pending', 'applied', 'rejected')),
    decision TEXT CHECK (
        decision IS NULL OR decision IN
            ('auto_apply', 'keep_existing', 'accept', 'accept_as_recent', 'ignore')
    ),
    dedupe_key TEXT NOT NULL,
    extractor_version TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    resolved_at_ms INTEGER,
    UNIQUE (contact_id, dedupe_key),
    CHECK (target_revision IS NULL OR target_revision > 0),
    CHECK (
        (status = 'pending' AND resolved_at_ms IS NULL AND decision IS NULL)
        OR (status <> 'pending' AND resolved_at_ms IS NOT NULL AND decision IS NOT NULL)
    ),
    CHECK (status <> 'applied' OR applied_item_id IS NOT NULL)
);
CREATE INDEX ix_proposals_pending
    ON profile_proposals(contact_id, status, kind, created_at_ms DESC);
CREATE INDEX ix_proposals_conversation ON profile_proposals(conversation_id);
CREATE INDEX ix_proposals_target ON profile_proposals(target_item_id);
CREATE INDEX ix_proposals_applied ON profile_proposals(applied_item_id);

CREATE TABLE profile_evidence (
    id TEXT PRIMARY KEY,
    proposal_id TEXT NOT NULL REFERENCES profile_proposals(id) ON DELETE CASCADE,
    item_id TEXT REFERENCES profile_items(id) ON DELETE CASCADE,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    source_message_id TEXT NOT NULL,
    sender_direction TEXT NOT NULL CHECK (sender_direction IN ('self', 'other')),
    message_at_ms INTEGER,
    quote TEXT NOT NULL CHECK (length(quote) BETWEEN 1 AND 300),
    created_at_ms INTEGER NOT NULL,
    UNIQUE (proposal_id, source_message_id)
);
CREATE INDEX ix_evidence_item ON profile_evidence(item_id);
CREATE INDEX ix_evidence_conversation ON profile_evidence(conversation_id);

CREATE TABLE profile_events (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    item_id TEXT REFERENCES profile_items(id) ON DELETE CASCADE,
    proposal_id TEXT REFERENCES profile_proposals(id) ON DELETE CASCADE,
    action TEXT NOT NULL CHECK (action IN (
        'manual_add', 'manual_edit', 'auto_add', 'auto_update', 'lock',
        'unlock', 'accept', 'keep_existing', 'accept_as_recent', 'ignore', 'archive'
    )),
    actor TEXT NOT NULL CHECK (actor IN ('user', 'system')),
    before_json TEXT CHECK (before_json IS NULL OR json_valid(before_json)),
    after_json TEXT CHECK (after_json IS NULL OR json_valid(after_json)),
    created_at_ms INTEGER NOT NULL,
    CHECK (item_id IS NOT NULL OR proposal_id IS NOT NULL)
);
CREATE INDEX ix_events_contact_time ON profile_events(contact_id, created_at_ms DESC);
CREATE INDEX ix_events_item ON profile_events(item_id);
CREATE INDEX ix_events_proposal ON profile_events(proposal_id);

CREATE TABLE profile_exclusions (
    id TEXT PRIMARY KEY,
    contact_id TEXT NOT NULL REFERENCES contacts(id) ON DELETE CASCADE,
    subject_key TEXT NOT NULL,
    subject_label TEXT NOT NULL,
    created_at_ms INTEGER NOT NULL,
    UNIQUE (contact_id, subject_key)
);

CREATE TABLE provider_configs (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL,
    adapter TEXT NOT NULL DEFAULT 'chat_completions'
        CHECK (adapter = 'chat_completions'),
    base_url TEXT NOT NULL,
    model TEXT NOT NULL,
    keychain_ref TEXT NOT NULL UNIQUE,
    is_default INTEGER NOT NULL DEFAULT 0 CHECK (is_default IN (0, 1)),
    config_revision INTEGER NOT NULL DEFAULT 1 CHECK (config_revision > 0),
    reply_consent_revision INTEGER,
    profile_consent_revision INTEGER,
    consented_at_ms INTEGER,
    created_at_ms INTEGER NOT NULL,
    updated_at_ms INTEGER NOT NULL
);
CREATE UNIQUE INDEX ux_providers_one_default
    ON provider_configs(is_default) WHERE is_default = 1;

CREATE TABLE generation_runs (
    id TEXT PRIMARY KEY,
    conversation_id TEXT NOT NULL REFERENCES conversations(id) ON DELETE CASCADE,
    provider_id TEXT REFERENCES provider_configs(id) ON DELETE SET NULL,
    purpose TEXT NOT NULL CHECK (purpose IN ('reply', 'profile')),
    task TEXT CHECK (task IS NULL OR task IN ('reply', 'rewrite', 'follow_up')),
    context_source TEXT NOT NULL CHECK (context_source IN ('wechat_db', 'manual')),
    context_count INTEGER NOT NULL CHECK (context_count >= 0),
    used_profile INTEGER NOT NULL DEFAULT 0 CHECK (used_profile IN (0, 1)),
    status TEXT NOT NULL CHECK (
        status IN ('running', 'success', 'failed', 'cancelled', 'interrupted')
    ),
    error_code TEXT,
    input_tokens INTEGER CHECK (input_tokens IS NULL OR input_tokens >= 0),
    output_tokens INTEGER CHECK (output_tokens IS NULL OR output_tokens >= 0),
    duration_ms INTEGER CHECK (duration_ms IS NULL OR duration_ms >= 0),
    started_at_ms INTEGER NOT NULL,
    finished_at_ms INTEGER,
    CHECK (
        (status = 'running' AND finished_at_ms IS NULL)
        OR (status <> 'running' AND finished_at_ms IS NOT NULL)
    )
);
CREATE INDEX ix_generation_runs_conversation
    ON generation_runs(conversation_id, started_at_ms DESC);
CREATE INDEX ix_generation_runs_provider ON generation_runs(provider_id);

CREATE TABLE app_settings (
    key TEXT PRIMARY KEY,
    value_json TEXT NOT NULL CHECK (json_valid(value_json)),
    updated_at_ms INTEGER NOT NULL
);
