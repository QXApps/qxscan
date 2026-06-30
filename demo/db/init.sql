-- qxscan demo database initialization
CREATE TABLE IF NOT EXISTS users (
    id SERIAL PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    email VARCHAR(255) NOT NULL UNIQUE,
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

INSERT INTO users (name, email) VALUES
    ('Alice Johnson', 'alice@demo.quantx.dev'),
    ('Bob Martinez', 'bob@demo.quantx.dev'),
    ('Carol Chen', 'carol@demo.quantx.dev'),
    ('David Park', 'david@demo.quantx.dev'),
    ('Eve Thompson', 'eve@demo.quantx.dev')
ON CONFLICT (email) DO NOTHING;
