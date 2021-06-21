alter table timed_modes drop constraint timed_mode_users_id_fk;

alter table timed_modes
    add constraint timed_mode_users_id_fk
        foreign key (user_id) references users
            on delete cascade;
