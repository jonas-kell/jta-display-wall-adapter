CREATE TABLE internal_wind_measurements (
  id VARCHAR NOT NULL PRIMARY KEY,
  data VARCHAR NOT NULL,
  wind_meas_time TIMESTAMP,
  stored_at_local TIMESTAMP NOT NULL
);