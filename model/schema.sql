-- Allowed things and links for the shared brain.
-- origin: extracted = read from a project file; canon = kept on purpose.

PRAGMA foreign_keys = ON;

CREATE TABLE nodes (
  id TEXT PRIMARY KEY,
  kind TEXT NOT NULL CHECK (kind IN (
    'Repository',
    'Service',
    'API',
    'Database',
    'Package',
    'Skill',
    'Agent',
    'Rule',
    'Concept'
  )),
  name TEXT NOT NULL,
  detail TEXT NOT NULL DEFAULT ''
);

CREATE TABLE edges (
  id INTEGER PRIMARY KEY,
  from_id TEXT NOT NULL REFERENCES nodes(id),
  to_id TEXT NOT NULL REFERENCES nodes(id),
  relation TEXT NOT NULL CHECK (relation IN (
    'contains',
    'uses',
    'stores',
    'describes'
  )),
  origin TEXT NOT NULL CHECK (origin IN ('extracted', 'canon')),
  UNIQUE (from_id, to_id, relation)
);
