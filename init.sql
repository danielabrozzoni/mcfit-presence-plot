create table city (
    id_city integer not null generated always as identity,
    name varchar(255) not null
);

alter table only city
    add constraint pk_city primary key(id_city);

create table presence (
    id_presence integer not null generated always as identity,
    time timestamp not null,
    presence real not null,
    id_city integer not null
);

alter table only presence
    add constraint pk_presence primary key(id_presence);

alter table only presence
    add constraint fk_presence_city
    foreign key(id_city) references city(id_city);
