-- PPLNS settlement schema (PostgreSQL)
-- Facts are immutable: shares/candidates/ledger.
-- Reorg is handled by compensating ledger rows (negative amount).

create table if not exists pplns_shares (
    id bigserial primary key,
    account varchar(256) not null,
    worker_id varchar(128) not null,
    difficulty bigint not null check (difficulty > 0),
    accepted_at_millis bigint not null,
    created_at timestamptz not null default now()
);

create index if not exists idx_pplns_shares_accepted_at
    on pplns_shares (accepted_at_millis);

create table if not exists pplns_pending_submits (
    id bigserial primary key,
    job_id varchar(128) not null,
    nonce bigint not null,
    extra varchar(128) not null,
    account varchar(256) not null,
    worker_id varchar(128) not null,
    anchor_share_id bigint not null,
    expected_block_number bigint not null,
    submitted_at_millis bigint not null,
    created_at timestamptz not null default now(),
    unique (job_id, nonce, extra, worker_id)
);

create table if not exists pplns_candidates (
    block_hash varchar(128) primary key,
    block_number bigint not null,
    account varchar(256) not null,
    worker_id varchar(128) not null,
    anchor_share_id bigint not null,
    found_at_millis bigint not null,
    status varchar(32) not null,
    reward numeric(39, 0),
    settled_at_millis bigint,
    created_at timestamptz not null default now()
);

create index if not exists idx_pplns_candidates_block_number
    on pplns_candidates (block_number);

create index if not exists idx_pplns_candidates_status_block
    on pplns_candidates (status, block_number);

create table if not exists pplns_ledger_entries (
    id bigserial primary key,
    account varchar(256) not null,
    block_hash varchar(128) not null,
    amount numeric(39, 0) not null,
    entry_type varchar(32) not null,
    created_at_millis bigint not null,
    created_at timestamptz not null default now(),
    unique (block_hash, account, entry_type)
);

create index if not exists idx_pplns_ledger_account
    on pplns_ledger_entries (account);

create index if not exists idx_pplns_ledger_block_hash
    on pplns_ledger_entries (block_hash);

create table if not exists pplns_meta (
    meta_key varchar(128) primary key,
    meta_value text not null,
    updated_at timestamptz not null default now()
);
