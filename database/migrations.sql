-- Users Table
CREATE TABLE users (
    id SERIAL PRIMARY KEY,
    address VARCHAR(42) NOT NULL UNIQUE,
    subscription_expiry TIMESTAMP NOT NULL,
    tier INT NOT NULL,
    min_liquidity FLOAT DEFAULT 0.0 -- Column for minimum liquidity
);

-- Trades Table
CREATE TABLE trades (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(42) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    amount FLOAT NOT NULL,
    price FLOAT NOT NULL,
    liquidity FLOAT DEFAULT NULL, -- Column for liquidity at trade time
    status VARCHAR(20) NOT NULL CHECK (status IN ('success', 'failed')), -- Validates status values
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL,
    CONSTRAINT fk_user_address FOREIGN KEY (user_address) REFERENCES users(address) -- Links to users table
);

-- Failed Trades Table
CREATE TABLE failed_trades (
    id SERIAL PRIMARY KEY,
    user_address VARCHAR(42) NOT NULL,
    symbol VARCHAR(10) NOT NULL,
    amount FLOAT NOT NULL,       
    price FLOAT NOT NULL,        
    liquidity FLOAT DEFAULT NULL,
    reason TEXT NOT NULL,
    timestamp TIMESTAMP DEFAULT CURRENT_TIMESTAMP NOT NULL
);

-- Indexes for performance
CREATE INDEX idx_trades_user_address ON trades(user_address);
CREATE INDEX idx_failed_trades_user_address ON failed_trades(user_address);
CREATE INDEX idx_trades_symbol ON trades(symbol); -- Index on symbol for better querying
CREATE INDEX idx_failed_trades_symbol ON failed_trades(symbol); -- Index for symbol in failed trades