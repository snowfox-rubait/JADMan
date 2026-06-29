-- Add download_playlist and referer columns to downloads table
ALTER TABLE downloads ADD COLUMN download_playlist BOOLEAN NOT NULL DEFAULT 0;
ALTER TABLE downloads ADD COLUMN referer TEXT;
