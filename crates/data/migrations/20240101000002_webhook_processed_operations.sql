-- Idempotency for webhook processing: skip duplicate delivery by operation_id
CREATE TABLE IF NOT EXISTS webhook_processed_operations (
    operation_id UUID PRIMARY KEY,
    processed_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE INDEX IF NOT EXISTS idx_webhook_processed_operations_processed_at
    ON webhook_processed_operations(processed_at);
