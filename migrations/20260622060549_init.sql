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
    -- "Completed", "Approved", "Pending", "Rejected"
    state TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL,
    -- The "claimer" or "bounty hunter" is the person who will or has completed the implementation of the bounty
    claimed_by BIGINT,
    related_message_id BIGINT,
    related_channel_id BIGINT
);

CREATE TABLE bounty_stakeholders (
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
    approved_bounties_channel BIGINT,
    claimed_bounties_channel BIGINT,
    completed_bounties_channel BIGINT,
    rejected_bounties_channel BIGINT,
    command_prefixes TEXT[] NOT NULL DEFAULT ARRAY['b!'],
    -- Serialized and deserialized from Rust
    bounty_submission_format JSONB NOT NULL,
    command_channels BIGINT[],
    current_bounty_number BIGINT NOT NULL DEFAULT 0
);

CREATE TABLE guild_permissions (
    guild_id BIGINT NOT NULL REFERENCES guilds (guild_id) ON DELETE CASCADE,
    -- "User" or "Role"
    kind TEXT NOT NULL,
    -- Either a user ID or a role ID, or the ID of the @everyone role (which is the same as the guild ID)
    entity_id BIGINT NOT NULL,
    -- Notably *not* the fluxer permissions, but rather the bounty bot permissions
    allow BIGINT NOT NULL
);