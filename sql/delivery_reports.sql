CREATE TABLE delivery_reports (
    id BIGSERIAL PRIMARY KEY,
    name TEXT NOT NULL,
    event TEXT NOT NULL,
    origin TEXT NOT NULL,
    queue BIGINT NOT NULL,
    queue_time BIGINT NOT NULL,
    recruiter BIGINT NOT NULL,
    sender TEXT NOT NULL,
    template TEXT NOT NULL,
    sent_time BIGINT NOT NULL,
    moved BOOLEAN DEFAULT FALSE,
    moved_time BIGINT
);