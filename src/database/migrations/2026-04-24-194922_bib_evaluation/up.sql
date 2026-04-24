CREATE TABLE bib_data_points (
  id VARCHAR NOT NULL PRIMARY KEY,
  belongs_to_id VARCHAR NOT NULL,
  data VARCHAR NOT NULL
);

CREATE INDEX idx_bib_data_points_belongs_to_id
ON bib_data_points(belongs_to_id);

CREATE TABLE bib_equivalences (
  id VARCHAR NOT NULL PRIMARY KEY,
  belongs_to_id VARCHAR NOT NULL,
  data VARCHAR NOT NULL
);

CREATE INDEX idx_bib_equivalences_belongs_to_id
ON bib_equivalences(belongs_to_id);