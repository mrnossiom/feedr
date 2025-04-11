// @generated automatically by Diesel CLI.

diesel::table! {
    api_key (id) {
        id -> Nullable<Integer>,
        user_id -> Integer,
        name -> Text,
        secret -> Text,
    }
}

diesel::table! {
    feed (id) {
        id -> Nullable<Integer>,
        title -> Text,
        feed_url -> Text,
    }
}

diesel::table! {
    feed_entry (id) {
        id -> Nullable<Integer>,
        feed_id -> Integer,
        title -> Text,
        description -> Text,
        url -> Text,
    }
}

diesel::table! {
    user (id) {
        id -> Nullable<Integer>,
        username -> Text,
        d_auth_secret -> Nullable<Text>,
    }
}

diesel::table! {
    user_feed (rowid) {
        rowid -> Integer,
        user_id -> Integer,
        feed_id -> Integer,
    }
}

diesel::joinable!(api_key -> user (user_id));
diesel::joinable!(feed_entry -> feed (feed_id));
diesel::joinable!(user_feed -> feed (feed_id));
diesel::joinable!(user_feed -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    api_key,
    feed,
    feed_entry,
    user,
    user_feed,
);
