DELETE FROM links a
USING links b
WHERE a.long_url = b.long_url
  AND a.id > b.id;

CREATE UNIQUE INDEX IF NOT EXISTS links_long_url_key ON links (long_url);
