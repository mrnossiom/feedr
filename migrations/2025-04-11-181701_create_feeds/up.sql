-- feeds source of truth to share entries
create table feed (
    id integer not null primary key autoincrement,

    url text not null,

    -- replace with an enum when switching database
    -- 0 is fine, 1 is fetching, 2 is fetch failed
    status integer not null
);

-- idx ensures feeds are unique
create unique index feed_url_idx
on feed (url);

-- a single feed entry fetched
create table feed_entry (
    id integer not null primary key autoincrement,
    feed_id integer not null,

    title text not null,
    -- cache summary? (first 50 chars of content)
    -- summary text,
    content text,

    foreign key (feed_id) references feed(id)
);

-- users
create table user (
    id integer not null primary key autoincrement,

    username text not null,

    d_auth_secret text
);

-- feed with user information referencing a `feed`
create table user_feed (
    id integer not null primary key autoincrement,
    user_id integer not null,
    feed_id integer not null,

    title text not null,
    description text,

    foreign key (user_id) references user(id),
    foreign key (feed_id) references feed(id)
);

-- idx ensures user has no entries that point to the same feed
create unique index user_feed_idx
on user_feed (user_id, feed_id);

-- feed with user information referencing a `feed_entry`
create table user_feed_entry (
    id integer not null primary key autoincrement,
    user_id integer not null,
    feed_entry_id integer not null,

    -- this is a boolean
    is_read integer not null,

    foreign key (user_id) references user(id),
    foreign key (feed_entry_id) references feed_entry(id)
);

-- user api keys
create table api_key (
    id integer not null primary key autoincrement,
    user_id integer not null,

    name text not null,

    secret text not null,

    foreign key (user_id) references user(id)
)
