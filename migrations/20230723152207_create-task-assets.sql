create table sandbox_task_assets
(
    task_id    text                                   not null,
    asset_id   text                                   not null,
    created_at timestamp with time zone default now() not null,
    constraint sandbox_task_assets_pk
        primary key (asset_id, task_id)
);

create unique index sandbox_task_assets_asset_id_uindex
    on sandbox_task_assets (asset_id);
