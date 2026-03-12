// @generated automatically by Diesel CLI.

diesel::table! {
    applied_goals (id) {
        id -> Int4,
        goal_id -> Int4,
        date -> Date,
        points_possible -> Int4,
        points_scored -> Int4,
    }
}

diesel::table! {
    goals (id) {
        id -> Int4,
        title -> Varchar,
        description -> Varchar,
        notes -> Nullable<Varchar>,
        user_id -> Int4,
    }
}

diesel::table! {
    users (id) {
        id -> Int4,
        username -> Varchar,
        hashed_password -> Nullable<Varchar>,
        email -> Nullable<Varchar>,
    }
}

diesel::joinable!(applied_goals -> goals (goal_id));
diesel::joinable!(goals -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    applied_goals,
    goals,
    users,
);
