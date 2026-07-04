-- Add migration script here

CREATE TABLE bounty_assignee_queue (
    bounty_id BIGINT NOT NULL REFERENCES bounties (bounty_id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL,
    queued_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX bounty_asignee_queue_by_queued_at ON bounty_assignee_queue (queued_at);