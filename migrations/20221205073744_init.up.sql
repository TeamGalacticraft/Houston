create table users
(
    id         uuid primary key not null,
    username   text             not null,
    avatar_url text             not null, /* probably a visage face url */
    roles      text array       not null default array[]::text[], /* modern_capes, legacy_capes, developer, admin */
    created    timestamptz      not null default current_timestamp
);

create table capes
(
    id          bigint primary key not null,
    name        text               not null,
    category    text               not null, /* modern, legacy, developer */
    texture_url text               not null
);