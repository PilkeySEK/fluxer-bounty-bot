-- Add migration script here

CREATE TABLE bounties (
    -- the internal ID of this bounty, unique across all guilds
    bounty_id BIGSERIAL PRIMARY KEY,
    -- the number of this bounty on a per-guild basis
    bounty_number BIGINT NOT NULL,
    guild_id BIGINT NOT NULL,
    created_by BIGINT NOT NULL,
    -- The content like title, description, etc., serialized and deserialized from Rust
    content JSONB NOT NULL,
    -- "Finished", "Approved", "Pending", "Rejected"
    state TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    -- The "claimer" is the person who will or has completed the implementation of the bounty
    claimed_by BIGINT,
    related_message_id BIGINT,
    related_channel_id BIGINT
);

CREATE TABLE bounty_payers (
    bounty_id BIGINT NOT NULL REFERENCES bounties (bounty_id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL,
    amount INTEGER NOT NULL,
    note TEXT,
    -- Can also be seen as "expiration". When this is reached, this record will be deleted because the person will no longer pay the amount
    deadline TIMESTAMPTZ
);

CREATE TABLE guilds (
    guild_id BIGINT PRIMARY KEY,
    bounty_submission_channel BIGINT,
    approval_queue_channel BIGINT,
    claimed_bounties_channel BIGINT,
    completed_bounties_channel BIGINT,
    denied_bounties_channel BIGINT,
    command_prefixes TEXT[] NOT NULL DEFAULT ARRAY['b!'],
    -- Serialized and deserialized from Rust
    bounty_submission_format JSONB NOT NULL,
    command_channels BIGINT[],
    current_bounty_number BIGINT NOT NULL DEFAULT 0
);