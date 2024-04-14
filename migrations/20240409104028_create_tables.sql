CREATE TABLE enrollments(
  id integer PRIMARY KEY AUTOINCREMENT NOT NULL,
  guild_id text NOT NULL,
  user_id text NOT NULL,
  channel_id text NOT NULL,
  created_at integer NOT NULL,
  starting_at integer NOT NULL,
  interval_hours integer NOT NULL
);

CREATE TABLE shares(
  id integer PRIMARY KEY AUTOINCREMENT NOT NULL,
  enrollment_id integer NOT NULL,
  created_at integer NOT NULL,
  FOREIGN KEY (enrollment_id) REFERENCES enrollments(id)
);

