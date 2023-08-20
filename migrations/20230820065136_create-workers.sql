create table sandbox_workers
(
    last_ping_at timestamp with time zone default now() not null
);

insert into sandbox_workers (last_ping_at) values (now());
