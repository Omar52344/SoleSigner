-- Enable UUID extension
CREATE EXTENSION IF NOT EXISTS "uuid-ossp";

-- Enums
CREATE TYPE election_status AS ENUM ('DRAFT', 'OPEN', 'CLOSING', 'SEALED');
CREATE TYPE access_type AS ENUM ('PUBLIC', 'PRIVATE');

-- 1. Elections Table
CREATE TABLE elections (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    title VARCHAR NOT NULL,
    form_config JSONB NOT NULL,
    status election_status NOT NULL DEFAULT 'DRAFT',
    start_date TIMESTAMPTZ NOT NULL,
    end_date TIMESTAMPTZ NOT NULL,
    access_type access_type NOT NULL DEFAULT 'PUBLIC',
    merkle_root VARCHAR,
    election_salt VARCHAR NOT NULL
);

-- 2. Voter Registry Table
CREATE TABLE voter_registry (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL REFERENCES elections(id),
    nullifier_hash VARCHAR NOT NULL,
    identity_status VARCHAR NOT NULL,
    voted_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    location_zone VARCHAR,
    UNIQUE(election_id, nullifier_hash)
);

-- 3. Ballots Table
CREATE TABLE ballots (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL REFERENCES elections(id),
    encrypted_choices JSONB NOT NULL,
    ballot_hash VARCHAR NOT NULL,
    merkle_proof JSONB, -- Can be updated after tree construction or stored initially if doing incremental
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 4. Whitelist Table
CREATE TABLE whitelist (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    election_id UUID NOT NULL REFERENCES elections(id),
    document_id_hash VARCHAR NOT NULL,
    has_voted BOOLEAN NOT NULL DEFAULT FALSE
);

-- 5. Audit Logs Table
CREATE TABLE audit_logs (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    admin_id UUID NOT NULL, -- Assuming managed externally or simple ID for now
    action VARCHAR NOT NULL,
    payload JSONB NOT NULL,
    signature VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);
