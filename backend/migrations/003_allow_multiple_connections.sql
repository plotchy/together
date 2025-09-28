-- Remove unique constraints that prevent multiple connections between same addresses
DROP INDEX IF EXISTS idx_together_attestations_unique;
DROP INDEX IF EXISTS idx_pending_connections_unique;
DROP INDEX IF EXISTS idx_optimistic_connections_unique;

-- Add connection strength tracking to together_counts
ALTER TABLE together_counts ADD COLUMN IF NOT EXISTS connection_pairs JSONB DEFAULT '{}';

-- Create a new index for efficient duplicate checking (but allow duplicates with different timestamps)
-- This helps with performance but doesn't prevent duplicates
CREATE INDEX IF NOT EXISTS idx_together_attestations_pair_timestamp ON together_attestations(address_1, address_2, attestation_timestamp);

-- Allow multiple pending connections between same users (they can connect multiple times)
-- But add a constraint to prevent spam (max 3 pending at once between same pair)
CREATE OR REPLACE FUNCTION check_pending_connection_limit()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if there are already 3+ pending connections between these users
    IF (SELECT COUNT(*) FROM pending_connections 
        WHERE (from_user_id = NEW.from_user_id AND to_user_id = NEW.to_user_id)
           OR (from_user_id = NEW.to_user_id AND to_user_id = NEW.from_user_id)) >= 3 THEN
        RAISE EXCEPTION 'Too many pending connections between these users';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER check_pending_limit 
    BEFORE INSERT ON pending_connections 
    FOR EACH ROW 
    EXECUTE FUNCTION check_pending_connection_limit();

-- Allow multiple optimistic connections but limit active ones (max 50 unprocessed between same pair)
CREATE OR REPLACE FUNCTION check_optimistic_connection_limit()
RETURNS TRIGGER AS $$
BEGIN
    -- Check if there are already 50+ unprocessed optimistic connections between these users
    IF (SELECT COUNT(*) FROM optimistic_connections 
        WHERE processed = FALSE
        AND ((user_id_1 = NEW.user_id_1 AND user_id_2 = NEW.user_id_2)
           OR (user_id_1 = NEW.user_id_2 AND user_id_2 = NEW.user_id_1))) >= 50 THEN
        RAISE EXCEPTION 'Too many unprocessed optimistic connections between these users';
    END IF;
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER check_optimistic_limit 
    BEFORE INSERT ON optimistic_connections 
    FOR EACH ROW 
    EXECUTE FUNCTION check_optimistic_connection_limit();

-- Function to update connection strength when attestations are added
CREATE OR REPLACE FUNCTION update_connection_strength()
RETURNS TRIGGER AS $$
DECLARE
    addr1 VARCHAR(42);
    addr2 VARCHAR(42);
    connection_count INTEGER;
BEGIN
    addr1 := NEW.address_1;
    addr2 := NEW.address_2;
    
    -- Get current connection count between these addresses
    SELECT COUNT(*) INTO connection_count 
    FROM together_attestations 
    WHERE (address_1 = addr1 AND address_2 = addr2) 
       OR (address_1 = addr2 AND address_2 = addr1);
    
    -- Update connection_pairs JSONB for addr1
    INSERT INTO together_counts (address, total_count, connection_pairs) 
    VALUES (addr1, 1, jsonb_build_object(addr2, connection_count))
    ON CONFLICT (address) DO UPDATE SET
        total_count = together_counts.total_count + 1,
        connection_pairs = jsonb_set(
            COALESCE(together_counts.connection_pairs, '{}'),
            ARRAY[addr2],
            to_jsonb(connection_count)
        );
    
    -- Update connection_pairs JSONB for addr2
    INSERT INTO together_counts (address, total_count, connection_pairs) 
    VALUES (addr2, 1, jsonb_build_object(addr1, connection_count))
    ON CONFLICT (address) DO UPDATE SET
        total_count = together_counts.total_count + 1,
        connection_pairs = jsonb_set(
            COALESCE(together_counts.connection_pairs, '{}'),
            ARRAY[addr1],
            to_jsonb(connection_count)
        );
    
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

-- Create trigger to automatically update connection strength
CREATE TRIGGER update_connection_strength_trigger
    AFTER INSERT ON together_attestations
    FOR EACH ROW
    EXECUTE FUNCTION update_connection_strength();
