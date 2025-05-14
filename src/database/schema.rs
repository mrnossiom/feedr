// @generated automatically by Diesel CLI.

diesel::table! {
    api_key (id) {
        id -> Int4,
        user_id -> Int4,
        name -> Text,
        secret -> Text,
        expires_at -> Nullable<Timestamptz>,
    }
}

diesel::table! {
    feed (id) {
        id -> Int4,
        url -> Text,
        status -> Text,
    }
}

diesel::table! {
    feed_entry (id) {
        id -> Int4,
        feed_id -> Int4,
        date -> Timestamptz,
        title -> Text,
        content -> Nullable<Text>,
    }
}

diesel::table! {
    session (id) {
        id -> Text,
        data -> Bytea,
        expiry_date -> Timestamptz,
    }
}

diesel::table! {
    user_ (id) {
        id -> Int4,
        username -> Text,
        basic_secret -> Nullable<Text>,
        dauth_secret -> Nullable<Text>,
    }
}

diesel::table! {
    user_feed (id) {
        id -> Int4,
        user_id -> Int4,
        feed_id -> Int4,
        folder_id -> Nullable<Int4>,
        title -> Text,
        description -> Nullable<Text>,
    }
}

diesel::table! {
    user_feed_entry_meta (id) {
        id -> Int4,
        user_id -> Int4,
        feed_entry_id -> Int4,
        read -> Int4,
        starred -> Int4,
    }
}

diesel::table! {
    user_feed_folder (id) {
        id -> Int4,
        user_id -> Int4,
        title -> Text,
    }
}

diesel::joinable!(api_key -> user_ (user_id));
diesel::joinable!(feed_entry -> feed (feed_id));
diesel::joinable!(user_feed -> feed (feed_id));
diesel::joinable!(user_feed -> user_ (user_id));
diesel::joinable!(user_feed -> user_feed_folder (folder_id));
diesel::joinable!(user_feed_entry_meta -> feed_entry (feed_entry_id));
diesel::joinable!(user_feed_entry_meta -> user_ (user_id));
diesel::joinable!(user_feed_folder -> user_ (user_id));

diesel::allow_tables_to_appear_in_same_query!(
    api_key,
    feed,
    feed_entry,
    session,
    user_,
    user_feed,
    user_feed_entry_meta,
    user_feed_folder,
);
