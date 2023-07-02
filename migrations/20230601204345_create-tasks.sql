create table sandbox_tasks
(
    task_id    text                                   not null
        constraint sandbox_tasks_pk
            primary key,
    created_at timestamp with time zone default now() not null,
    prompt     text                                   not null,
    user_id    text,
    status     jsonb                                  not null,
    is_pending boolean                                not null
);

create unique index sandbox_tasks_task_id_uindex
    on sandbox_tasks (task_id);
