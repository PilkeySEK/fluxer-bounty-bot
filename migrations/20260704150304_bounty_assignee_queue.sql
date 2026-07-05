-- Add migration script here

CREATE TABLE bounty_assignee_queue (
    bounty_id BIGINT NOT NULL REFERENCES bounties (bounty_id) ON DELETE CASCADE,
    user_id BIGINT NOT NULL,
    queued_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX bounty_assignee_queue_by_queued_at ON bounty_assignee_queue (queued_at);
CREATE INDEX bounty_assignee_queue_by_bounty_id ON bounty_assignee_queue (bounty_id);
CREATE UNIQUE INDEX bounty_assignee_queue_by_bounty_id_and_user_id ON bounty_assignee_queue (bounty_id, user_id);

BEGIN;

INSERT INTO bounty_assignee_queue (bounty_id, user_id, queued_at)
SELECT bounty_id, assigned_to, NOW() FROM bounties WHERE assigned_to IS NOT NULL;

ALTER TABLE bounties DROP COLUMN assigned_to;

COMMIT;
