alter table slots
    add name varchar(40) default null;

alter table slots
    add added_by varchar(40) default null;

alter table slots
    add added_at timestamptz default null;

create index slots_name_index
    on slots (name);
