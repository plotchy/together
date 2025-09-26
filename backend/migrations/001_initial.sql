-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Together attestations table
CREATE TABLE together_attestations (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    address_1 VARCHAR(42) NOT NULL, -- first address in the attestation
    address_2 VARCHAR(42) NOT NULL, -- second address in the attestation
    attestation_timestamp BIGINT NOT NULL, -- timestamp from the attestation event
    tx_hash VARCHAR(66), -- transaction hash where attestation was recorded
    block_number BIGINT, -- block number where attestation was recorded
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Together counts table - for quick lookups of total attestations per address
CREATE TABLE together_counts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    address VARCHAR(42) UNIQUE NOT NULL, -- address this count is for
    total_count BIGINT NOT NULL DEFAULT 0, -- total number of attestations this address is part of
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Watcher state table for resume functionality
CREATE TABLE watcher_state (
    id VARCHAR(50) PRIMARY KEY, -- e.g., 'attestation_watcher'
    last_processed_block BIGINT NOT NULL,
    chunk_size BIGINT NOT NULL DEFAULT 500, -- DEFAULT_WATCHER_CHUNK_SIZE
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for fast queries
CREATE INDEX idx_together_attestations_address_1 ON together_attestations(address_1);
CREATE INDEX idx_together_attestations_address_2 ON together_attestations(address_2);
CREATE INDEX idx_together_attestations_timestamp ON together_attestations(attestation_timestamp);
CREATE INDEX idx_together_attestations_block_number ON together_attestations(block_number);
CREATE INDEX idx_together_attestations_tx_hash ON together_attestations(tx_hash);
-- Composite index for efficient profile queries (address + timestamp desc for recent first)
CREATE INDEX idx_together_attestations_address_1_timestamp_desc ON together_attestations(address_1, attestation_timestamp DESC);
CREATE INDEX idx_together_attestations_address_2_timestamp_desc ON together_attestations(address_2, attestation_timestamp DESC);
-- Index for preventing duplicates
CREATE UNIQUE INDEX idx_together_attestations_unique ON together_attestations(address_1, address_2, attestation_timestamp);

CREATE UNIQUE INDEX idx_together_counts_address ON together_counts(address);
CREATE INDEX idx_together_counts_updated_at ON together_counts(updated_at);
CREATE INDEX idx_watcher_state_updated_at ON watcher_state(updated_at);
CREATE INDEX idx_watcher_state_last_processed_block ON watcher_state(last_processed_block);
CREATE INDEX idx_watcher_state_chunk_size ON watcher_state(chunk_size);

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_together_attestations_updated_at BEFORE UPDATE ON together_attestations FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_together_counts_updated_at BEFORE UPDATE ON together_counts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_watcher_state_updated_at BEFORE UPDATE ON watcher_state FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_watcher_state_last_processed_block BEFORE UPDATE ON watcher_state FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_watcher_state_chunk_size BEFORE UPDATE ON watcher_state FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
