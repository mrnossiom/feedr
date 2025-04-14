create table user (
    id integer not null primary key autoincrement,

    username text not null,

    d_auth_secret text
);

create table feed (
    id integer not null primary key autoincrement,

    url text not null
);

create unique index feed_url_idx
on feed (url);

create table feed_entry (
    id integer not null primary key autoincrement,
    feed_id integer not null,
    user_id integer not null,

    title text not null,
    description text not null,

    foreign key (feed_id) references feed(id),
    foreign key (user_id) references user(id)
);


create table api_key (
    id integer not null primary key autoincrement,
    user_id integer not null,

    name text not null,

    secret text not null,

    foreign key (user_id) references user(id)
)