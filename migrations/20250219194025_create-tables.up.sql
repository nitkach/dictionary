create table if not exists word_entry (
    id serial primary key,
    word text not null,
    phonetic text not null
);

create table if not exists source_url (
    id serial primary key,
    word_entry_id int references word_entry(id) on delete cascade,
    url text not null
);

create table if not exists phonetic (
    id serial primary key,
    word_entry_id int references word_entry(id) on delete cascade,
    text text not null,
    audio text not null,
    source_url text not null
);

create table if not exists meaning (
    id serial primary key,
    word_entry_id int references word_entry(id) on delete cascade,
    part_of_speech text not null
);

create table if not exists definition (
    id serial primary key,
    meaning_id int references meaning(id) on delete cascade,
    definition text not null,
    example text not null
);

create table if not exists synonym (
    id serial primary key,
    meaning_id int references meaning(id) on delete cascade,
    synonym text not null
);

create table if not exists antonym (
    id serial primary key,
    meaning_id int references meaning(id) on delete cascade,
    antonym text not null
);
