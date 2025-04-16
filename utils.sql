-- Add one dummy user
insert into user (username) values ('dummy');
insert into api_key (user_id, name, secret) values (1, 'dev', 'fdr_v0_dev');

-- Add one feed
insert into feed (url, status) values ('https://blog.tangled.sh/blog/feed.xml', 1);

-- Add one feed and create a feed_entry for the dummy user
insert into feed (url, status) values ('https://matklad.github.io/feed.xml', 0);
insert into user_feed (feed_id, user_id, title, description)
values (2, 1, 'Matklad', 'Masterclass!');

-- Return all feed_entry for a user
select uf.id, feed.url, uf.title, uf.description
from user_feed uf
inner join feed
    on feed.id = uf.feed_id
where uf.user_id = 1;