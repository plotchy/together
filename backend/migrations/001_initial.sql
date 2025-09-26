-- Create extensions
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Casts table
CREATE TABLE casts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cast_hash VARCHAR(66) NOT NULL UNIQUE, -- 0x + 64 hex chars
    author_fid BIGINT NOT NULL,
    author_username VARCHAR(255) NOT NULL,
    author_display_name VARCHAR(255) NOT NULL,
    author_pfp_url TEXT,
    text TEXT NOT NULL,
    timestamp TIMESTAMPTZ NOT NULL,
    thread_hash VARCHAR(66) NOT NULL,
    parent_hash VARCHAR(66), -- null if not a reply
    root_parent_url TEXT, -- useful for thread context
    likes_count BIGINT DEFAULT 0,
    recasts_count BIGINT DEFAULT 0,
    replies_count BIGINT DEFAULT 0,
    embeds JSONB DEFAULT '[]', -- store embed URLs and metadata
    mentioned_profiles JSONB DEFAULT '[]', -- store mentioned user info
    include BOOLEAN NOT NULL DEFAULT false, -- whether this cast passes our filters
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Auctions table
CREATE TABLE auctions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cast_hash VARCHAR(66) NOT NULL UNIQUE, -- no foreign key constraint
    creator_fid BIGINT, -- from AuctionStarted event
    settled BOOLEAN NOT NULL DEFAULT false,
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Metadata table
CREATE TABLE metadata (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cast_hash VARCHAR(66) NOT NULL UNIQUE, -- no foreign key constraint
    metadata_url TEXT NOT NULL,
    traits JSONB NOT NULL DEFAULT '{}',
    image_url TEXT NOT NULL,
    processed BOOLEAN NOT NULL DEFAULT false,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Whitelist table for presale eligibility
CREATE TABLE whitelist (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    address VARCHAR(42) NOT NULL UNIQUE, -- Ethereum address
    name VARCHAR(255), -- Optional name, can be null
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Presale NFTs table for managing available tokens
CREATE TABLE presale_nfts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    cast_hash VARCHAR(66) NOT NULL, -- 0x + 64 hex chars (bytes32 as hex)
    token_id VARCHAR(78) NOT NULL UNIQUE, -- U256 as decimal string (max 78 digits)
    status VARCHAR(20) NOT NULL DEFAULT 'available' CHECK (status IN ('available', 'designated', 'sold')),
    designated_to VARCHAR(42), -- Ethereum address, nullable
    expires_at TIMESTAMPTZ, -- When designation expires, nullable
    sold_tx_hash VARCHAR(66), -- Transaction hash when sold, nullable
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Watcher state table for resume functionality
CREATE TABLE watcher_state (
    id VARCHAR(50) PRIMARY KEY, -- e.g., 'auction_watcher'
    last_processed_block BIGINT NOT NULL,
    chunk_size BIGINT NOT NULL DEFAULT 500, -- DEFAULT_WATCHER_CHUNK_SIZE
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_casts_author_fid ON casts(author_fid);
CREATE INDEX idx_casts_timestamp ON casts(timestamp);
CREATE INDEX idx_casts_include ON casts(include);
CREATE INDEX idx_casts_thread_hash ON casts(thread_hash);
CREATE INDEX idx_casts_parent_hash ON casts(parent_hash);
CREATE INDEX idx_auctions_settled ON auctions(settled);
CREATE INDEX idx_auctions_cast_hash ON auctions(cast_hash); -- for joins
CREATE INDEX idx_metadata_processed ON metadata(processed);
CREATE INDEX idx_metadata_cast_hash ON metadata(cast_hash); -- for joins
CREATE INDEX idx_whitelist_address ON whitelist(address); -- for address lookups
CREATE INDEX idx_presale_nfts_status ON presale_nfts(status); -- for filtering by status
CREATE INDEX idx_presale_nfts_cast_hash ON presale_nfts(cast_hash); -- for cast hash lookups
CREATE INDEX idx_presale_nfts_token_id ON presale_nfts(token_id); -- for token ID lookups
CREATE INDEX idx_presale_nfts_designated_to ON presale_nfts(designated_to); -- for user lookups
CREATE INDEX idx_presale_nfts_expires_at ON presale_nfts(expires_at) WHERE expires_at IS NOT NULL; -- for expiration cleanup

-- Trigger to update updated_at
CREATE OR REPLACE FUNCTION update_updated_at_column()
RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ language 'plpgsql';

CREATE TRIGGER update_casts_updated_at BEFORE UPDATE ON casts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_auctions_updated_at BEFORE UPDATE ON auctions FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_metadata_updated_at BEFORE UPDATE ON metadata FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_watcher_state_updated_at BEFORE UPDATE ON watcher_state FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_whitelist_updated_at BEFORE UPDATE ON whitelist FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
CREATE TRIGGER update_presale_nfts_updated_at BEFORE UPDATE ON presale_nfts FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();
