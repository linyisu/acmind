CREATE TABLE IF NOT EXISTS problems (
    id TEXT PRIMARY KEY,
    source TEXT NOT NULL,
    source_problem_id TEXT NOT NULL,
    title TEXT NOT NULL,
    url TEXT,
    difficulty INTEGER,
    tags TEXT NOT NULL DEFAULT '[]',
    statement_path TEXT,
    created_at DATETIME NOT NULL DEFAULT (datetime('now')),
    updated_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS submissions (
    id TEXT PRIMARY KEY,
    problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    status TEXT NOT NULL CHECK(status IN ('AC','WA','TLE','RE','MLE','CE')),
    language TEXT NOT NULL DEFAULT 'C++',
    code_path TEXT NOT NULL,
    runtime INTEGER,
    memory INTEGER,
    note TEXT,
    submitted_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS solution_notes (
    id TEXT PRIMARY KEY,
    problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    note_type TEXT NOT NULL CHECK(note_type IN ('official','community','self','ai')),
    content TEXT NOT NULL DEFAULT '',
    source_url TEXT,
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS error_analyses (
    id TEXT PRIMARY KEY,
    problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    submission_id TEXT NOT NULL REFERENCES submissions(id) ON DELETE CASCADE,
    error_type TEXT NOT NULL,
    root_cause TEXT NOT NULL DEFAULT '',
    fix_summary TEXT NOT NULL DEFAULT '',
    related_knowledge TEXT NOT NULL DEFAULT '[]',
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS knowledge_points (
    id TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE,
    category TEXT NOT NULL,
    parent_id TEXT
);

CREATE TABLE IF NOT EXISTS problem_knowledge (
    problem_id TEXT NOT NULL REFERENCES problems(id) ON DELETE CASCADE,
    knowledge_point_id TEXT NOT NULL REFERENCES knowledge_points(id) ON DELETE CASCADE,
    confidence REAL NOT NULL DEFAULT 1.0,
    PRIMARY KEY (problem_id, knowledge_point_id)
);

CREATE TABLE IF NOT EXISTS reports (
    id TEXT PRIMARY KEY,
    report_type TEXT NOT NULL,
    title TEXT NOT NULL,
    content TEXT NOT NULL DEFAULT '',
    start_date TEXT NOT NULL,
    end_date TEXT NOT NULL,
    created_at DATETIME NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_submissions_problem ON submissions(problem_id);
CREATE INDEX IF NOT EXISTS idx_notes_problem ON solution_notes(problem_id);
CREATE INDEX IF NOT EXISTS idx_errors_problem ON error_analyses(problem_id);

-- Seed knowledge points
INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES ('DP', 'Dynamic Programming', 'DP', NULL);
INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES ('Graph', 'Graph Theory', 'Graph', NULL);
INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES ('Math', 'Mathematics', 'Math', NULL);
INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES ('DS', 'Data Structures', 'DS', NULL);
INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES ('String', 'String Algorithms', 'String', NULL);
INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES ('Greedy', 'Greedy Algorithms', 'Greedy', NULL);
INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES ('Geometry', 'Computational Geometry', 'Geometry', NULL);
INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES ('Search', 'Search & Enumeration', 'Search', NULL);
INSERT OR IGNORE INTO knowledge_points (id, name, category, parent_id) VALUES ('Other', 'Other', 'Other', NULL);
