-- 6. Admins Table
CREATE TABLE admins (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    username VARCHAR NOT NULL UNIQUE,
    password_hash VARCHAR NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- 7. Add admin_id to Elections
ALTER TABLE elections ADD COLUMN admin_id UUID REFERENCES admins(id);

-- Optional: If we want to enforce every election belongs to an admin, we would add NOT NULL later.
-- For now, existing elections might have NULL admin_id. 
-- We can eventually fill them or leave them as "System/Public" elections.
