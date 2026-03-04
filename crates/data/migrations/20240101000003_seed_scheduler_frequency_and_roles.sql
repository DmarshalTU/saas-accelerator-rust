-- Seed scheduler_frequency and roles for admin UI
INSERT INTO scheduler_frequency (frequency) VALUES ('Daily'), ('Weekly'), ('Monthly')
ON CONFLICT (frequency) DO NOTHING;

INSERT INTO roles (name) VALUES ('Admin'), ('User')
ON CONFLICT (name) DO NOTHING;
