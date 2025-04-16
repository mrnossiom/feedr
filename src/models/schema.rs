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
        status -> Integer,
    }
}

diesel::table! {
    feed_entry (id) {
        id -> Integer,
        feed_id -> Integer,
        title -> Text,
        content -> Nullable<Text>,
    }
}

diesel::table! {
    user (id) {
        id -> Integer,
        username -> Text,
        d_auth_secret -> Nullable<Text>,
    }
}

diesel::table! {
    user_feed (id) {
        id -> Integer,
        user_id -> Integer,
        feed_id -> Integer,
        title -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    user_feed_entry (id) {
        id -> Integer,
        user_id -> Integer,
        feed_entry_id -> Integer,
        is_read -> Integer,
    }
}

diesel::joinable!(api_key -> user (user_id));
diesel::joinable!(feed_entry -> feed (feed_id));
diesel::joinable!(user_feed -> feed (feed_id));
diesel::joinable!(user_feed -> user (user_id));
diesel::joinable!(user_feed_entry -> feed_entry (feed_entry_id));
diesel::joinable!(user_feed_entry -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    api_key,
    feed,
    feed_entry,
    user,
    user_feed,
    user_feed_entry,
);
