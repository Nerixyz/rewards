alter table timeouts drop constraint timeouts_pk;

alter table timeouts
    add constraint timeouts_pk
        unique (channel_id, user_id);
