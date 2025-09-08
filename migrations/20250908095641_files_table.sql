-- Add migration script here
CREATE TABLE files (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    user_id UUID REFERENCES users(id) ON DELETE CASCADE, -- Foreign key constraint for user_id
    file_name VARCHAR(255) NOT NULL,
    file_size BIGINT NOT NULL,
    encrypted_aes_key BYTEA NOT NULL, -- Store encrypted AES key
    encrypted_file BYTEA NOT NULL,    -- Store the actual encrypted file content
    iv BYTEA NOT NULL,                -- Initialization Vector for AES encryption
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
