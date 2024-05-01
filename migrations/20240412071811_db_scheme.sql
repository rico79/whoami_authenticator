-- Users
CREATE TABLE IF NOT EXISTS users (
    id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    name VARCHAR NOT NULL,
    email VARCHAR NOT NULL UNIQUE,
    email_confirmed BOOLEAN NOT NULL DEFAULT FALSE,
    encrypted_password VARCHAR NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP
);

-- Apps
CREATE TABLE IF NOT EXISTS apps (
    id SERIAL PRIMARY KEY,
    name VARCHAR NOT NULL,
    description VARCHAR NOT NULL,
    base_url VARCHAR NOT NULL,
    redirect_endpoint VARCHAR NOT NULL,
    logo_endpoint VARCHAR NOT NULL,
    jwt_secret VARCHAR NOT NULL,
    jwt_seconds_to_expire INTEGER NOT NULL,
    created_at TIMESTAMP WITH TIME ZONE DEFAULT CURRENT_TIMESTAMP,
    owner_id UUID REFERENCES users ON DELETE CASCADE
);