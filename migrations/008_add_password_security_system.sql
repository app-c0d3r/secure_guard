-- migrations/008_add_password_security_system.sql

-- Add password security columns to users table
ALTER TABLE users.users 
ADD COLUMN IF NOT EXISTS must_change_password BOOLEAN NOT NULL DEFAULT FALSE,
ADD COLUMN IF NOT EXISTS password_last_changed TIMESTAMPTZ DEFAULT now(),
ADD COLUMN IF NOT EXISTS failed_login_attempts INTEGER DEFAULT 0,
ADD COLUMN IF NOT EXISTS last_failed_login TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS account_locked_until TIMESTAMPTZ,
ADD COLUMN IF NOT EXISTS role VARCHAR(50) NOT NULL DEFAULT 'user',
ADD COLUMN IF NOT EXISTS password_history JSONB DEFAULT '[]'::jsonb;

-- Create password policy settings table
CREATE TABLE IF NOT EXISTS users.password_policies (
    policy_id UUID PRIMARY KEY DEFAULT gen_random_uuid(),
    min_length INTEGER NOT NULL DEFAULT 12,
    require_uppercase BOOLEAN NOT NULL DEFAULT TRUE,
    require_lowercase BOOLEAN NOT NULL DEFAULT TRUE,
    require_numbers BOOLEAN NOT NULL DEFAULT TRUE,
    require_special_chars BOOLEAN NOT NULL DEFAULT TRUE,
    max_age_days INTEGER NOT NULL DEFAULT 90,
    history_count INTEGER NOT NULL DEFAULT 5,
    max_failed_attempts INTEGER NOT NULL DEFAULT 5,
    lockout_duration_minutes INTEGER NOT NULL DEFAULT 30,
    created_at TIMESTAMPTZ NOT NULL DEFAULT now(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT now()
);

-- Insert default password policy
INSERT INTO users.password_policies (
    min_length, 
    require_uppercase, 
    require_lowercase, 
    require_numbers, 
    require_special_chars,
    max_age_days,
    history_count,
    max_failed_attempts,
    lockout_duration_minutes
) VALUES (
    12, TRUE, TRUE, TRUE, TRUE, 90, 5, 5, 30
) ON CONFLICT DO NOTHING;

-- Create default admin user with secure random password that must be changed
DO $$
DECLARE
    random_password TEXT;
    password_hash TEXT;
BEGIN
    -- Generate a secure random password (32 characters)
    random_password := encode(gen_random_bytes(24), 'base64');
    
    -- Hash the password using crypt (we'll use Argon2 in the application)
    password_hash := crypt(random_password, gen_salt('bf', 10));
    
    -- Insert admin user if not exists
    INSERT INTO users.users (
        username, 
        password_hash, 
        email, 
        role, 
        must_change_password,
        is_active
    ) VALUES (
        'admin',
        password_hash,
        'admin@secureguard.local',
        'system_admin',
        TRUE,
        TRUE
    ) ON CONFLICT (username) DO UPDATE SET
        must_change_password = TRUE,
        role = 'system_admin';
    
    -- Log the default password (this should be displayed during first setup)
    RAISE NOTICE 'Default admin password: %', random_password;
    RAISE NOTICE 'IMPORTANT: Change this password immediately after first login!';
END $$;

-- Create function to validate password strength
CREATE OR REPLACE FUNCTION users.validate_password_strength(
    password TEXT,
    min_length INTEGER DEFAULT 12,
    require_uppercase BOOLEAN DEFAULT TRUE,
    require_lowercase BOOLEAN DEFAULT TRUE,
    require_numbers BOOLEAN DEFAULT TRUE,
    require_special_chars BOOLEAN DEFAULT TRUE
) RETURNS BOOLEAN AS $$
BEGIN
    -- Check minimum length
    IF length(password) < min_length THEN
        RETURN FALSE;
    END IF;
    
    -- Check for uppercase letter
    IF require_uppercase AND password !~ '[A-Z]' THEN
        RETURN FALSE;
    END IF;
    
    -- Check for lowercase letter
    IF require_lowercase AND password !~ '[a-z]' THEN
        RETURN FALSE;
    END IF;
    
    -- Check for numbers
    IF require_numbers AND password !~ '[0-9]' THEN
        RETURN FALSE;
    END IF;
    
    -- Check for special characters
    IF require_special_chars AND password !~ '[^a-zA-Z0-9]' THEN
        RETURN FALSE;
    END IF;
    
    RETURN TRUE;
END;
$$ LANGUAGE plpgsql;

-- Create function to handle failed login attempts
CREATE OR REPLACE FUNCTION users.handle_failed_login(username_param TEXT) RETURNS VOID AS $$
DECLARE
    max_attempts INTEGER;
    lockout_duration INTEGER;
BEGIN
    -- Get policy settings
    SELECT max_failed_attempts, lockout_duration_minutes 
    INTO max_attempts, lockout_duration
    FROM users.password_policies 
    LIMIT 1;
    
    -- Update failed attempt count
    UPDATE users.users 
    SET 
        failed_login_attempts = failed_login_attempts + 1,
        last_failed_login = now()
    WHERE username = username_param;
    
    -- Lock account if max attempts reached
    UPDATE users.users 
    SET account_locked_until = now() + INTERVAL '1 minute' * lockout_duration
    WHERE username = username_param 
    AND failed_login_attempts >= max_attempts;
END;
$$ LANGUAGE plpgsql;

-- Create function to handle successful login
CREATE OR REPLACE FUNCTION users.handle_successful_login(username_param TEXT) RETURNS VOID AS $$
BEGIN
    UPDATE users.users 
    SET 
        failed_login_attempts = 0,
        last_failed_login = NULL,
        account_locked_until = NULL
    WHERE username = username_param;
END;
$$ LANGUAGE plpgsql;

-- Create indexes for performance
CREATE INDEX IF NOT EXISTS idx_users_username ON users.users(username);
CREATE INDEX IF NOT EXISTS idx_users_email ON users.users(email);
CREATE INDEX IF NOT EXISTS idx_users_role ON users.users(role);
CREATE INDEX IF NOT EXISTS idx_users_account_locked ON users.users(account_locked_until) WHERE account_locked_until IS NOT NULL;