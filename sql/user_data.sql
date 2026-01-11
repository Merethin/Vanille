CREATE TABLE user_data (
    queue      BIGINT NOT NULL,
    user_id     BIGINT NOT NULL,
    nation     TEXT   NOT NULL,
    founded    BIGINT NOT NULL,
    newfounds  TEXT[] NOT NULL,
    refounds   TEXT[] NOT NULL,

    CONSTRAINT user_data_pkey PRIMARY KEY (queue, user_id)
);