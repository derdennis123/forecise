-- Enable extensions
-- TimescaleDB is optional - use if available
DO $$ BEGIN
  CREATE EXTENSION IF NOT EXISTS timescaledb;
EXCEPTION WHEN OTHERS THEN
  RAISE NOTICE 'TimescaleDB not available, using regular tables';
END $$;
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Sources (Polymarket, Kalshi, Metaculus, etc.)
CREATE TABLE sources (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug VARCHAR(50) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    source_type VARCHAR(50) NOT NULL, -- 'prediction_market', 'forecast_platform', 'analyst'
    api_base_url TEXT,
    is_active BOOLEAN DEFAULT true,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Categories
CREATE TABLE categories (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug VARCHAR(100) UNIQUE NOT NULL,
    name VARCHAR(255) NOT NULL,
    description TEXT,
    icon VARCHAR(10),
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Markets (unified representation of questions across platforms)
CREATE TABLE markets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    slug VARCHAR(500) UNIQUE NOT NULL,
    title TEXT NOT NULL,
    description TEXT,
    category_id UUID REFERENCES categories(id),
    status VARCHAR(50) DEFAULT 'active', -- 'active', 'resolved', 'closed', 'cancelled'
    resolution_value DECIMAL(10, 6), -- NULL if unresolved, 0-1 for resolved
    resolution_date TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Source Markets (individual market on each platform, linked to unified market)
CREATE TABLE source_markets (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    market_id UUID REFERENCES markets(id) ON DELETE CASCADE,
    source_id UUID REFERENCES sources(id) ON DELETE CASCADE,
    external_id VARCHAR(500) NOT NULL,
    external_url TEXT,
    title TEXT NOT NULL,
    current_probability DECIMAL(10, 6),
    volume DECIMAL(20, 2),
    liquidity DECIMAL(20, 2),
    status VARCHAR(50) DEFAULT 'active',
    resolution_value DECIMAL(10, 6),
    resolution_date TIMESTAMPTZ,
    metadata JSONB DEFAULT '{}',
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(source_id, external_id)
);

-- Odds History (time-series - will be hypertable)
CREATE TABLE odds_history (
    time TIMESTAMPTZ NOT NULL,
    source_market_id UUID NOT NULL REFERENCES source_markets(id) ON DELETE CASCADE,
    probability DECIMAL(10, 6) NOT NULL,
    volume DECIMAL(20, 2),
    trade_count INTEGER
);

-- Convert to TimescaleDB hypertable if available
DO $$ BEGIN
  PERFORM create_hypertable('odds_history', 'time');
EXCEPTION WHEN OTHERS THEN
  RAISE NOTICE 'TimescaleDB not available, using regular table for odds_history';
END $$;

-- Create index for efficient lookups
CREATE INDEX idx_odds_history_source_market ON odds_history (source_market_id, time DESC);

-- Accuracy Records (per source, per category)
CREATE TABLE accuracy_records (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_id UUID NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    category_id UUID REFERENCES categories(id),
    total_resolved INTEGER DEFAULT 0,
    correct_predictions INTEGER DEFAULT 0,
    brier_score DECIMAL(10, 6),
    accuracy_pct DECIMAL(10, 4),
    last_calculated_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(source_id, category_id)
);

-- Individual prediction scores (for Brier score calculation)
CREATE TABLE prediction_scores (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_market_id UUID NOT NULL REFERENCES source_markets(id) ON DELETE CASCADE,
    source_id UUID NOT NULL REFERENCES sources(id) ON DELETE CASCADE,
    market_id UUID NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    category_id UUID REFERENCES categories(id),
    predicted_probability DECIMAL(10, 6) NOT NULL, -- probability at time of resolution
    actual_outcome DECIMAL(10, 6) NOT NULL, -- 0 or 1
    brier_score DECIMAL(10, 6) NOT NULL, -- (predicted - actual)^2
    resolved_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Consensus snapshots
CREATE TABLE consensus_snapshots (
    time TIMESTAMPTZ NOT NULL,
    market_id UUID NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    consensus_probability DECIMAL(10, 6) NOT NULL,
    confidence_score DECIMAL(10, 4),
    source_count INTEGER NOT NULL,
    agreement_score DECIMAL(10, 4), -- 0-1, how much sources agree
    outlier_sources JSONB DEFAULT '[]',
    weights JSONB NOT NULL, -- {"source_id": weight, ...}
    created_at TIMESTAMPTZ DEFAULT NOW()
);

DO $$ BEGIN
  PERFORM create_hypertable('consensus_snapshots', 'time');
EXCEPTION WHEN OTHERS THEN
  RAISE NOTICE 'TimescaleDB not available, using regular table for consensus_snapshots';
END $$;

-- Movement events (for "Why It Moved")
CREATE TABLE movement_events (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_market_id UUID NOT NULL REFERENCES source_markets(id) ON DELETE CASCADE,
    market_id UUID NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    probability_before DECIMAL(10, 6) NOT NULL,
    probability_after DECIMAL(10, 6) NOT NULL,
    change_pct DECIMAL(10, 4) NOT NULL,
    detected_at TIMESTAMPTZ NOT NULL,
    explanation TEXT,
    related_news JSONB DEFAULT '[]',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Whale trades (for Polymarket on-chain tracking)
CREATE TABLE whale_trades (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    source_market_id UUID REFERENCES source_markets(id),
    wallet_address VARCHAR(42) NOT NULL,
    trade_type VARCHAR(10) NOT NULL, -- 'buy', 'sell'
    position VARCHAR(10) NOT NULL, -- 'yes', 'no'
    amount DECIMAL(20, 6) NOT NULL,
    price DECIMAL(10, 6),
    tx_hash VARCHAR(66),
    block_number BIGINT,
    traded_at TIMESTAMPTZ NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Wallet accuracy tracking
CREATE TABLE wallet_accuracy (
    wallet_address VARCHAR(42) PRIMARY KEY,
    total_trades INTEGER DEFAULT 0,
    resolved_trades INTEGER DEFAULT 0,
    correct_trades INTEGER DEFAULT 0,
    accuracy_pct DECIMAL(10, 4),
    total_volume DECIMAL(20, 2) DEFAULT 0,
    pnl DECIMAL(20, 2) DEFAULT 0,
    is_smart_money BOOLEAN DEFAULT false,
    last_active_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Users (for future auth)
CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    name VARCHAR(255),
    tier VARCHAR(50) DEFAULT 'free', -- 'free', 'pro', 'team', 'enterprise'
    created_at TIMESTAMPTZ DEFAULT NOW(),
    updated_at TIMESTAMPTZ DEFAULT NOW()
);

-- Watchlists
CREATE TABLE watchlist_items (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    market_id UUID NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    created_at TIMESTAMPTZ DEFAULT NOW(),
    UNIQUE(user_id, market_id)
);

-- Alerts
CREATE TABLE alerts (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID NOT NULL REFERENCES users(id) ON DELETE CASCADE,
    market_id UUID NOT NULL REFERENCES markets(id) ON DELETE CASCADE,
    alert_type VARCHAR(50) NOT NULL, -- 'threshold', 'movement', 'whale'
    threshold_value DECIMAL(10, 4),
    channel VARCHAR(50) DEFAULT 'email', -- 'email', 'telegram', 'slack'
    is_active BOOLEAN DEFAULT true,
    last_triggered_at TIMESTAMPTZ,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Insert default sources
INSERT INTO sources (slug, name, source_type, api_base_url) VALUES
    ('polymarket', 'Polymarket', 'prediction_market', 'https://clob.polymarket.com'),
    ('kalshi', 'Kalshi', 'prediction_market', 'https://api.elections.kalshi.com'),
    ('metaculus', 'Metaculus', 'forecast_platform', 'https://www.metaculus.com/api2'),
    ('manifold', 'Manifold Markets', 'prediction_market', 'https://api.manifold.markets/v0'),
    ('predictit', 'PredictIt', 'prediction_market', 'https://www.predictit.org/api');

-- Insert default categories
INSERT INTO categories (slug, name, description, icon) VALUES
    ('politics', 'Politics', 'Elections, policy, geopolitics', '🏛️'),
    ('economics', 'Economics', 'Central banks, GDP, inflation, markets', '📈'),
    ('technology', 'Technology', 'AI, space, science, crypto', '💻'),
    ('crypto', 'Crypto', 'Bitcoin, Ethereum, DeFi, regulation', '🪙'),
    ('climate', 'Climate & Energy', 'Climate policy, energy markets', '🌍'),
    ('sports', 'Sports', 'Major sports events and tournaments', '⚽'),
    ('entertainment', 'Entertainment', 'Awards, media, culture', '🎬'),
    ('science', 'Science & Health', 'Research, health, pandemics', '🔬');

-- Morning Briefings
CREATE TABLE morning_briefings (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    briefing_date DATE UNIQUE NOT NULL,
    total_markets_tracked INTEGER NOT NULL DEFAULT 0,
    new_markets_24h INTEGER NOT NULL DEFAULT 0,
    top_movers JSONB NOT NULL DEFAULT '[]',
    high_volume_markets JSONB NOT NULL DEFAULT '[]',
    source_agreement JSONB NOT NULL DEFAULT '[]',
    key_stats JSONB NOT NULL DEFAULT '{}',
    summary TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE INDEX idx_morning_briefings_date ON morning_briefings(briefing_date DESC);

-- Create useful indexes
CREATE INDEX idx_source_markets_market ON source_markets(market_id);
CREATE INDEX idx_source_markets_source ON source_markets(source_id);
CREATE INDEX idx_markets_category ON markets(category_id);
CREATE INDEX idx_markets_status ON markets(status);
CREATE INDEX idx_prediction_scores_source ON prediction_scores(source_id);
CREATE INDEX idx_movement_events_market ON movement_events(market_id, detected_at DESC);
CREATE INDEX idx_whale_trades_wallet ON whale_trades(wallet_address, traded_at DESC);
CREATE INDEX idx_whale_trades_market ON whale_trades(source_market_id, traded_at DESC);
