CREATE TABLE queues (
    channel_id BIGINT PRIMARY KEY,
    message_id BIGINT NOT NULL,
    region TEXT NOT NULL,
    size BIGINT NOT NULL,
    excluded_regions TEXT[] NOT NULL DEFAULT '{}',
    fill_threshold BIGINT,
    time_threshold BIGINT,
    ping_channel BIGINT,
    ping_role BIGINT
);