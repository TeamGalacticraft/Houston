create sequence capes_id_seq;
alter table capes alter column id set default nextval('capes_id_seq');
alter sequence capes_id_seq owned by capes.id;