create table if not exists words (
    id serial primary key,
    word text not null
);

create table if not exists word_entries (
    id serial primary key,
    word text not null
);

create table if not exists source_urls (
    id serial primary key,
    word_entry_id int references word_entries (id) on delete cascade,
    url text not null
);

create table if not exists meanings (
    id serial primary key,
    word_entry_id int references word_entries (id) on delete cascade,
    part_of_speech text not null
);

create table if not exists synonyms (
    id serial primary key,
    meaning_id int references meanings (id) on delete cascade,
    synonym text not null
);

create table if not exists antonyms (
    id serial primary key,
    meaning_id int references meanings (id) on delete cascade,
    antonym text not null
);

create table if not exists definitions (
    id serial primary key,
    meaning_id int references meanings (id) on delete cascade,
    definition text not null,
    example text
);
