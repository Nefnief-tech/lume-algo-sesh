-- Create custom event_type enum for match interactions
CREATE TYPE event_type AS ENUM (
    'viewed',
    'liked',
    'passed',
    'matched'
);

-- Create seen_profiles table to track profiles a user has already seen
-- This prevents the matching algorithm from returning the same profiles repeatedly
CREATE TABLE IF NOT EXISTS seen_profiles (
    user_id VARCHAR(255) NOT NULL,
    target_user_id VARCHAR(255) NOT NULL,
    event_type event_type NOT NULL DEFAULT 'viewed',
    seen_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    PRIMARY KEY (user_id, target_user_id)
);

-- Create index for fast lookups of seen profiles by user
CREATE INDEX IF NOT EXISTS idx_seen_profiles_user_id ON seen_profiles(user_id);

-- Create index for querying reverse lookups (who has seen a specific profile)
CREATE INDEX IF NOT EXISTS idx_seen_profiles_target_user_id ON seen_profiles(target_user_id);

-- Create index for event type filtering
CREATE INDEX IF NOT EXISTS idx_seen_profiles_event_type ON seen_profiles(event_type);

-- Create index for time-based queries (e.g., "show me profiles I saw in the last 7 days")
CREATE INDEX IF NOT EXISTS idx_seen_profiles_seen_at ON seen_profiles(seen_at DESC);

-- Add comment for documentation
COMMENT ON TABLE seen_profiles IS 'Tracks profiles users have seen to prevent repeats in matching results';
COMMENT ON COLUMN seen_profiles.user_id IS 'The user who viewed/saw the profile';
COMMENT ON COLUMN seen_profiles.target_user_id IS 'The profile that was viewed';
COMMENT ON COLUMN seen_profiles.event_type IS 'Type of interaction: viewed, liked, passed, matched';
COMMENT ON COLUMN seen_profiles.seen_at IS 'When the profile was first seen (updated on re-interaction)';
