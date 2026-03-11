CREATE TABLE "applied_goals" (
  "id" SERIAL PRIMARY KEY,
  "goal_id" INTEGER NOT NULL REFERENCES goals(id),
  "date" DATE NOT NULL,
  "points_possible" INTEGER NOT NULL DEFAULT 0,
  "points_scored" INTEGER NOT NULL DEFAULT 0,
  UNIQUE (goal_id,date)
);

CREATE INDEX idx_applied_goals_goal_id ON applied_goals(goal_id);
CREATE INDEX idx_applied_goals_date ON applied_goals(date);
