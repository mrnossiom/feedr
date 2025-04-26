-- feeds source of truth to share entries
create table feed (
    id integer not null primary key autoincrement,

    url text not null,

    -- replace with an enum when switching database
    status text check(status in ('ok', 'fetching', 'failed')) not null

    -- todo: add titles values coming from the feed
);

-- idx ensures feeds are unique
create unique index feed_url_idx
on feed (url);

-- a single feed entry fetched
create table feed_entry (
    id integer not null primary key autoincrement,
    feed_id integer not null,

    date datetime not null,

    title text not null,
    -- cache summary? (first 50 chars of content)
    -- summary text,
    content text,

    foreign key (feed_id) references feed(id)
        on delete cascade
);

-- users
create table user (
    id integer not null primary key autoincrement,

    username text not null,

    basic_secret text,
    dauth_secret text
);

create table user_feed_folder (
    id integer not null primary key autoincrement,
    user_id integer not null,

    title text not null,

    foreign key (user_id) references user(id)
        on delete cascade
);

-- feed with user information referencing a `feed`
create table user_feed (
    id integer not null primary key autoincrement,
    user_id integer not null,
    feed_id integer not null,
    -- null is `Default` folder
    folder_id integer,

    title text not null,
    description text,

    foreign key (user_id) references user(id)
        on delete cascade,
    foreign key (feed_id) references feed(id)
        on delete restrict,
    foreign key (folder_id) references user_feed_folder(id)
        on delete set null
);

-- idx ensures user has no entries that point to the same feed
create unique index user_feed_idx
on user_feed (user_id, feed_id);

-- meta information when user has interacted with a `feed_entry`
create table user_feed_entry_meta (
    id integer not null primary key autoincrement,
    user_id integer not null,
    feed_entry_id integer not null,

    read integer not null,
    starred integer not null,

    foreign key (user_id) references user(id)
        on delete cascade,
    foreign key (feed_entry_id) references feed_entry(id)
        on delete restrict
);

-- user sessions
create table session (
    id text primary key not null,
    data blob not null,
    expiry_date timestamp not null
);

-- user api keys
create table api_key (
    id integer not null primary key autoincrement,
    user_id integer not null,

    name text not null,
    secret text not null,
    expires_at date,

    foreign key (user_id) references user(id)
        on delete cascade
);
