create type chat_role as enum('system', 'user', 'assistant');

create table sandbox_chat_messages
(
    task_id text not null,
    message_id text not null,
    content text not null,
    message_role chat_role not null,
    created_at timestamp with time zone default now() not null,
    constraint sandbox_chat_messages_pk
        primary key (message_id, task_id)
);

create unique index sandbox_chat_messages_message_id_uindex
    on sandbox_chat_messages (message_id);
