-- Test database setup script
-- Run this after PostgreSQL is running to create test database

-- Connect to default postgres database first
-- psql -U secureguard -h localhost -p 5432

-- Create test database
CREATE DATABASE secureguard_test;

-- Grant permissions
GRANT ALL PRIVILEGES ON DATABASE secureguard_test TO secureguard;