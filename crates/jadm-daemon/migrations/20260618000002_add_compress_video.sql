-- Add compress_video column to downloads
ALTER TABLE downloads ADD COLUMN compress_video BOOLEAN DEFAULT 0;
