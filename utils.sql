-- Add one dummy user
insert into user (username) values ('dummy');
insert into api_key (user_id, name, secret) values (1, 'dev', 'fdr_v0_dev');

-- Add one feed
insert into feed (url) values ('https://blog.tangled.sh/blog/feed.xml');

-- Add one feed and create a feed_entry for the dummy user
insert into feed (url) values ('https://matklad.github.io/feed.xml');
insert into feed_entry (feed_id, user_id, title, description)
values (2, 1, 'Matklad', 'Masterclass!');

-- Return all feed_entry for a user
select entry.id, feed.url, entry.title, entry.description
from feed_entry entry
inner join feed
    on feed.id = entry.feed_id
where entry.user_id = 1;