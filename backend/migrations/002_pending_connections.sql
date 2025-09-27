-- Add users table with incrementing IDs
CREATE TABLE users (
    id SERIAL PRIMARY KEY, -- Auto-incrementing ID starting at 1
    wallet_address VARCHAR(42) UNIQUE NOT NULL, -- Wallet address from verified miniapp
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Add pending connections table
CREATE TABLE pending_connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    from_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    to_user_id INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    expires_at TIMESTAMPTZ NOT NULL DEFAULT (NOW() + INTERVAL '10 minutes')
);

-- Add optimistic connections table for user connections
CREATE TABLE optimistic_connections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id_1 INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    user_id_2 INTEGER NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    processed BOOLEAN NOT NULL DEFAULT FALSE, -- TRUE when verified on-chain
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes for efficient queries
CREATE UNIQUE INDEX idx_users_wallet_address ON users(wallet_address);
CREATE INDEX idx_users_created_at ON users(created_at);

-- Indexes for pending connections
CREATE INDEX idx_pending_connections_from_user_id ON pending_connections(from_user_id);
CREATE INDEX idx_pending_connections_to_user_id ON pending_connections(to_user_id);
CREATE INDEX idx_pending_connections_expires_at ON pending_connections(expires_at);
CREATE INDEX idx_pending_connections_created_at ON pending_connections(created_at);
-- Prevent duplicate pending connections
CREATE UNIQUE INDEX idx_pending_connections_unique ON pending_connections(from_user_id, to_user_id);

-- Indexes for optimistic connections
CREATE INDEX idx_optimistic_connections_user_id_1 ON optimistic_connections(user_id_1);
CREATE INDEX idx_optimistic_connections_user_id_2 ON optimistic_connections(user_id_2);
CREATE INDEX idx_optimistic_connections_processed ON optimistic_connections(processed);
CREATE INDEX idx_optimistic_connections_created_at ON optimistic_connections(created_at);
-- Prevent duplicate optimistic connections
CREATE UNIQUE INDEX idx_optimistic_connections_unique ON optimistic_connections(user_id_1, user_id_2);

-- Add triggers for updated_at
CREATE TRIGGER update_users_updated_at BEFORE UPDATE ON users FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Add constraint to prevent self-connections
ALTER TABLE pending_connections ADD CONSTRAINT chk_no_self_connection CHECK (from_user_id != to_user_id);
ALTER TABLE optimistic_connections ADD CONSTRAINT chk_no_self_optimistic_connection CHECK (user_id_1 != user_id_2);
