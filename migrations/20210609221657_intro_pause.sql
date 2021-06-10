alter table rewards
    add live_delay varchar(255) default null;

alter table rewards
    add unpause_at timestamptz default null;
