CREATE TABLE database_state (
  id INTEGER NOT NULL PRIMARY KEY,
  created_with_version VARCHAR NOT NULL,
  data VARCHAR NOT NULL
);