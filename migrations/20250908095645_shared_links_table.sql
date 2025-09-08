-- Add migration script here
CREATE TABLE shared_links (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    file_id UUID REFERENCES files(id) ON DELETE CASCADE,           -- Foreign key to files table
    recipient_id UUID REFERENCES users(id) ON DELETE CASCADE, -- Foreign key to users table
    password VARCHAR(255) NOT NULL,                           -- Password protection (required)
    expiration_date TIMESTAMP WITH TIME ZONE NOT NULL,        -- Expiration date (required)
    created_at TIMESTAMP WITH TIME ZONE DEFAULT NOW()
);
