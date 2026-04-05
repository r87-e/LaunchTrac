-- LaunchTrac v2 database schema

CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

CREATE TABLE users (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    email VARCHAR(255) UNIQUE NOT NULL,
    display_name VARCHAR(100),
    auth_provider VARCHAR(50) DEFAULT 'email',
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE devices (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    hardware_version VARCHAR(20) NOT NULL,
    firmware_version VARCHAR(20) NOT NULL,
    last_seen TIMESTAMPTZ,
    api_key_hash VARCHAR(255) NOT NULL,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE sessions (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id),
    device_id UUID REFERENCES devices(id),
    started_at TIMESTAMPTZ DEFAULT NOW(),
    ended_at TIMESTAMPTZ,
    location_name VARCHAR(255),
    notes TEXT
);

CREATE TABLE shots (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    session_id UUID REFERENCES sessions(id),
    device_id UUID REFERENCES devices(id) NOT NULL,
    shot_number INTEGER NOT NULL,
    speed_mph DOUBLE PRECISION NOT NULL,
    vla_deg DOUBLE PRECISION NOT NULL,
    hla_deg DOUBLE PRECISION NOT NULL,
    backspin_rpm INTEGER NOT NULL,
    sidespin_rpm INTEGER NOT NULL,
    spin_axis_deg DOUBLE PRECISION,
    total_spin_rpm DOUBLE PRECISION,
    club VARCHAR(20),
    confidence DOUBLE PRECISION,
    processing_time_ms INTEGER,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

CREATE TABLE firmware_releases (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    version VARCHAR(20) NOT NULL,
    channel VARCHAR(20) NOT NULL DEFAULT 'stable',
    checksum_sha256 VARCHAR(64) NOT NULL,
    download_url TEXT NOT NULL,
    size_bytes BIGINT,
    release_notes TEXT,
    created_at TIMESTAMPTZ DEFAULT NOW()
);

-- Indexes for common queries
CREATE INDEX idx_shots_device_id ON shots(device_id);
CREATE INDEX idx_shots_session_id ON shots(session_id);
CREATE INDEX idx_shots_created_at ON shots(created_at DESC);
CREATE INDEX idx_devices_user_id ON devices(user_id);
CREATE INDEX idx_sessions_user_id ON sessions(user_id);
