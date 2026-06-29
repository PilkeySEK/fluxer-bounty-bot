CREATE TABLE bounty_reviews (
    bounty_id BIGINT NOT NULL REFERENCES bounties (bounty_id) ON DELETE CASCADE,
    reviewer_id BIGINT NOT NULL,
    -- "Approval" or "Denial"
    decision TEXT NOT NULL,
    comment TEXT,
    bypass BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (bounty_id, reviewer_id)
);

CREATE INDEX idx_bounty_reviews_by_bounty_id_and_decision ON bounty_reviews (bounty_id, decision);
