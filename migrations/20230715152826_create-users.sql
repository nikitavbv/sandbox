create table sandbox_users
(
    id    uuid not null
        constraint sandbox_users_pk
            primary key,
    email text  not null
);

create unique index sandbox_users_id_uindex
    on sandbox_users (id);

create unique index sandbox_users_email_uindex
    on sandbox_users (email);
