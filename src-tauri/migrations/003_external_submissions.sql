CREATE UNIQUE INDEX IF NOT EXISTS idx_submissions_external_run_id
ON submissions(external_run_id)
WHERE external_run_id IS NOT NULL;

CREATE INDEX IF NOT EXISTS idx_submissions_submitted_at ON submissions(submitted_at);
