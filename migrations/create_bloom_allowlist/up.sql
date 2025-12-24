-- Create the bloom_allowlist table
CREATE TABLE bloom_allowlist (
                                 id SERIAL PRIMARY KEY,
                                 wallet_address TEXT NOT NULL UNIQUE,
                                 created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

-- Create an index for faster lookups on wallet_address
CREATE INDEX idx_bloom_allowlist_wallet ON bloom_allowlist(wallet_address);

-- Optional: Add a comment to the table
COMMENT ON TABLE bloom_allowlist IS 'Stores wallet addresses for allowlist verification with bloom filter';