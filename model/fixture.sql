-- One hand-built example. Not a scan of a folder.
-- Service and API are allowed by the schema and left out here:
-- this project does not have either yet.

INSERT INTO nodes (id, kind, name, detail) VALUES
  ('repo:npin', 'Repository', 'neural-project-intelligent-network', 'This project, the shared brain.'),
  ('rule:secrets', 'Rule', 'npin-secrets', 'Never read passwords or key files while scanning.'),
  ('skill:ponytail', 'Skill', 'ponytail', 'Prefer the smallest change that still finishes the job.'),
  ('package:graphifyy', 'Package', 'graphifyy', 'Local tool that will map code into a graph.'),
  ('agent:cursor', 'Agent', 'Cursor', 'The coding assistant that uses the installed skills.'),
  ('concept:shared-brain', 'Concept', 'shared brain', 'One map of your projects, plus memory a new project can inherit.'),
  ('database:canon', 'Database', 'canon store', 'The single local database that will hold this map.');

INSERT INTO edges (from_id, to_id, relation, origin) VALUES
  ('repo:npin', 'rule:secrets', 'contains', 'extracted'),
  ('repo:npin', 'package:graphifyy', 'uses', 'canon'),
  ('repo:npin', 'skill:ponytail', 'uses', 'canon'),
  ('agent:cursor', 'skill:ponytail', 'uses', 'canon'),
  ('concept:shared-brain', 'repo:npin', 'describes', 'canon'),
  ('database:canon', 'repo:npin', 'stores', 'canon');
