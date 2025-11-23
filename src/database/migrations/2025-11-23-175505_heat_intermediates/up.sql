CREATE TABLE heat_intermediates (
  id VARCHAR NOT NULL PRIMARY KEY,
  belongs_to_id VARCHAR NOT NULL,
  data VARCHAR NOT NULL
);

CREATE INDEX idx_heat_intermediates_belongs_to_id
ON heat_intermediates(belongs_to_id);