create table logs
(
    id serial not null,
    user_id varchar(16) not null
        constraint logs_users_id_fk
            references users
            on delete cascade,
    date timestamptz not null,
    content text not null
);

create index logs_date_index
    on logs (date desc);

create unique index logs_id_uindex
    on logs (id);

create index logs_user_id_index
    on logs (user_id);

alter table logs
    add constraint logs_pk
        primary key (id);


create or replace function clear_logs() returns trigger as $$
    declare
        max_log_size int := 10;
        current_log_size int := 0;
    begin
        lock table logs in exclusive mode;

        select into current_log_size count(*)
        from logs
        where user_id = new.user_id;

        if current_log_size >= max_log_size then
            delete from logs
            where
                  user_id = new.user_id
              and
                  id <= (select id from logs
                            where user_id = new.user_id
                            order by date desc, id desc
                            offset (max_log_size - 1) limit 1
                        );
        end if;

        return new;
    end;
$$ language plpgsql;

drop trigger if exists clear_logs on logs;
create trigger clear_logs
    after insert
    on logs
        for each row
            execute procedure clear_logs();
