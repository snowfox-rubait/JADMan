-- Add live_from_start column to downloads
ALTER TABLE downloads ADD COLUMN live_from_start BOOLEAN DEFAULT 0;
