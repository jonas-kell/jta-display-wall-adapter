CREATE TABLE heat_evaluations (
  id VARCHAR NOT NULL PRIMARY KEY,
  belongs_to_id VARCHAR NOT NULL,
  data VARCHAR NOT NULL
);

CREATE INDEX idx_heat_evaluations_belongs_to_id
ON heat_evaluations(belongs_to_id);