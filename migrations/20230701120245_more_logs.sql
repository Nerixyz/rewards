create or replace function clear_logs() returns trigger as $$
    declare
        max_log_size int := 20;
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
