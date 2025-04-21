-- Insert dummy user
insert into user (username) values ('dummy');
insert into api_key (user_id, name, secret) values (1, 'dev', 'fdr_v0_dev');
update user set tmp_unencrypted_secret = 'yolo' where id = 1;

-- Insert some feeds
insert into feed (url, status)
values ('https://blog.tangled.sh/blog/feed.xml', 1),
       ('https://matklad.github.io/feed.xml', 0);

-- Insert a dummy user subscription to a feed
insert into user_feed (feed_id, user_id, title, description)
values (2, 1, 'Matklad', 'Masterclass!');

-- Select all feeds that a user subscribed to
select uf.id, feed.url, uf.title, uf.description
from user_feed uf
inner join feed
    on feed.id = uf.feed_id
where uf.user_id = 1;

-- TODO: Select or insert a feed and return a feed id

-- Insert two feed entries a feed
insert into feed_entry (feed_id, date, title, content)
values (2, datetime('now'), 'First look at Tangled', 'some content'),
       (2, datetime('now'), 'Second look at Tangled', 'some content but 2');

-- Select all feed entries of a specific feed for a user with optional meta
select feed_entry.title, meta.read, meta.starred
from feed_entry
left join user_feed_entry_meta meta
    on feed_entry.id = meta.feed_entry_id
where feed_entry.feed_id = 2
  and feed_entry.date > datetime('now', '-1 month');

-- Select all feed entries of a user with optional meta
select feed_entry.title, meta.read, meta.starred
from user_feed uf
inner join feed_entry
    on feed_entry.feed_id = uf.feed_id
left join user_feed_entry_meta meta
    on feed_entry.id = meta.feed_entry_id
where uf.user_id = 1
  and feed_entry.date > datetime('now', '-1 month');