CREATE TABLE events (
    id INTEGER PRIMARY KEY,
    name TEXT NOT NULL,
    svg_icon TEXT,
    created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    edited_at DATETIME DEFAULT CURRENT_TIMESTAMP,
    sub_event_id INTEGER
);

CREATE TABLE event_occurs (
    id INTEGER PRIMARY KEY,
    event_id INTEGER REFERENCES events(id),
    timestamp DATETIME DEFAULT CURRENT_TIMESTAMP,
    FOREIGN KEY(event_id) REFERENCES events(id)
);