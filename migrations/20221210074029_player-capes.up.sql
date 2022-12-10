create table player_capes
(
    player uuid not null,
    cape   bigint default null,
    constraint fk_cape
        foreign key (cape)
            references capes (id)
            on delete set null
);