alter table if exists bttv_slots rename to slots;

create type slot_platform as enum ('bttv', 'ffz', '7tv');

alter table slots
    add if not exists platform slot_platform default 'bttv' not null;

alter table slots alter column platform drop default;