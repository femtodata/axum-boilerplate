CREATE TABLE "goals"(
  "id" SERIAL PRIMARY KEY,
  "title" VARCHAR NOT NULL,
  "description" VARCHAR NOT NULL,
  "notes" VARCHAR,
  "user_id" INTEGER NOT NULL REFERENCES users(id),
  UNIQUE (user_id,title)
);
