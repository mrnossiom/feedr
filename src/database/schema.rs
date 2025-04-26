// @generated automatically by Diesel CLI.

diesel::table! {
	api_key (id) {
		id -> Integer,
		user_id -> Integer,
		name -> Text,
		secret -> Text,
		expires_at -> Nullable<Date>,
	}
}

diesel::table! {
	feed (id) {
		id -> Integer,
		url -> Text,
		status -> Text,
	}
}

diesel::table! {
	feed_entry (id) {
		id -> Integer,
		feed_id -> Integer,
		date -> Timestamp,
		title -> Text,
		content -> Nullable<Text>,
	}
}

diesel::table! {
	session (id) {
		id -> Text,
		data -> Binary,
		expiry_date -> TimestamptzSqlite,
	}
}

diesel::table! {
	user (id) {
		id -> Integer,
		username -> Text,
		basic_secret -> Nullable<Text>,
		dauth_secret -> Nullable<Text>,
	}
}

diesel::table! {
	user_feed (id) {
		id -> Integer,
		user_id -> Integer,
		feed_id -> Integer,
		folder_id -> Nullable<Integer>,
		title -> Text,
		description -> Nullable<Text>,
	}
}

diesel::table! {
	user_feed_entry_meta (id) {
		id -> Integer,
		user_id -> Integer,
		feed_entry_id -> Integer,
		read -> Integer,
		starred -> Integer,
	}
}

diesel::table! {
	user_feed_folder (id) {
		id -> Integer,
		user_id -> Integer,
		title -> Text,
	}
}

diesel::joinable!(api_key -> user (user_id));
diesel::joinable!(feed_entry -> feed (feed_id));
diesel::joinable!(user_feed -> feed (feed_id));
diesel::joinable!(user_feed -> user (user_id));
diesel::joinable!(user_feed -> user_feed_folder (folder_id));
diesel::joinable!(user_feed_entry_meta -> feed_entry (feed_entry_id));
diesel::joinable!(user_feed_entry_meta -> user (user_id));
diesel::joinable!(user_feed_folder -> user (user_id));

diesel::allow_tables_to_appear_in_same_query!(
	api_key,
	feed,
	feed_entry,
	session,
	user,
	user_feed,
	user_feed_entry_meta,
	user_feed_folder,
);
