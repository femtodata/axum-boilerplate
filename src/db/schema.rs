// @generated automatically by Diesel CLI.

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

diesel::joinable!(goals -> users (user_id));

diesel::allow_tables_to_appear_in_same_query!(goals, users,);
