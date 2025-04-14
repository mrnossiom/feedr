// @generated automatically by Diesel CLI.

diesel::table! {
    api_key (id) {
        id -> Integer,
        user_id -> Integer,
        name -> Text,
        secret -> Text,
    }
}

diesel::table! {
    feed (id) {
        id -> Integer,
        url -> Text,
    }
}

diesel::table! {
    feed_entry (id) {
        id -> Integer,
        feed_id -> Integer,
        user_id -> Integer,
        title -> Text,
        description -> Text,
    }
}

diesel::table! {
    user (id) {
        id -> Integer,
        username -> Text,
        d_auth_secret -> Nullable<Text>,
    }
}

diesel::joinable!(api_key -> user (user_id));
diesel::joinable!(feed_entry -> feed (feed_id));
diesel::joinable!(feed_entry -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    api_key,
    feed,
    feed_entry,
    user,
);
