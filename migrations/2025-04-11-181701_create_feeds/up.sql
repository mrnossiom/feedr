create table feed (
    id integer primary key autoincrement,

    title text not null,
    feed_url text not null
);

create table feed_entry (
    id integer primary key autoincrement,
    feed_id integer not null,

    title text not null,
    description text not null,
    url text not null,

    foreign key (feed_id) references feed(id)
);

create table user (
    id integer primary key autoincrement,

    username text not null,

    d_auth_secret text
);

create table user_feed (
    user_id integer not null,
    feed_id integer not null,

    foreign key (user_id) references user(id),
    foreign key (feed_id) references feed(id)
);

create table api_key (
    id integer primary key autoincrement,
    user_id integer not null,

    name text not null,

    secret text not null,

    foreign key (user_id) references user(id)
)