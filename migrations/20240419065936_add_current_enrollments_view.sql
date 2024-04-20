CREATE VIEW
  current_enrollments AS
SELECT
  MAX(created_at) AS created_at,
  id,
  guild_id,
  user_id,
  channel_id,
  starting_at,
  interval_hours
FROM
  enrollments
GROUP BY
  guild_id,
  user_id;