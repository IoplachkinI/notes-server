-- DATABASE SCHEMA

CREATE TABLE notes (
    id BIGSERIAL PRIMARY KEY,
    content TEXT,
    created_at TIMESTAMP WITH TIME ZONE,
    updated_at TIMESTAMP WITH TIME ZONE
);

-- UPDATE TRIGGER

CREATE OR REPLACE FUNCTION set_updated_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_set_updated_at
BEFORE UPDATE ON notes
FOR EACH ROW
EXECUTE FUNCTION set_updated_at();

-- INSERT TRIGGER

CREATE OR REPLACE FUNCTION set_created_at() RETURNS TRIGGER AS $$
BEGIN
    NEW.created_at = NOW();
    NEW.updated_at = NOW();
    RETURN NEW;
END;
$$ LANGUAGE plpgsql;

CREATE TRIGGER trigger_set_created_at
BEFORE INSERT ON notes
FOR EACH ROW
EXECUTE FUNCTION set_created_at();