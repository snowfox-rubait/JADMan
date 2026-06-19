-- Add live_support column to downloads
ALTER TABLE downloads ADD COLUMN live_support BOOLEAN DEFAULT 0;
