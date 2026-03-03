use serde::{Deserialize, Serialize};
use sqlx::{sqlite::SqliteConnection, types::Json};
use time::OffsetDateTime;

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RawEvent {
    pub id: i32,
    pub name: String,
    pub svg_icon: String,
    pub created_at: OffsetDateTime,
    pub edited_at: OffsetDateTime,
    pub user_enabled: bool,
    pub event_occurrence: i32,
    pub sub_events_count: i32,
}

#[derive(Debug, Serialize, Deserialize, sqlx::FromRow)]
pub struct RawEventWithChildren {
    pub id: i32,
    pub name: String,
    pub svg_icon: String,
    pub created_at: OffsetDateTime,
    pub edited_at: OffsetDateTime,
    pub user_enabled: bool,
    pub event_occurrence: i32,
    pub sub_events: Json<Vec<RawEvent>>,
}

/// Lists all events with their occurrence count for today
pub async fn events(
    mut e: impl AsMut<SqliteConnection>,
    offset: u32,
    limit: u32,
) -> Result<Vec<RawEvent>, sqlx::Error> {
    sqlx::query_as::<_, RawEvent>(
        r#"
        SELECT 
            e.id, e.name, e.svg_icon, e.created_at, e.edited_at, e.user_enabled,
            (SELECT COUNT(*) FROM event_occurs WHERE event_id = e.id AND DATE(timestamp) = DATE('now')) AS event_occurrence,
            (SELECT COUNT(*) FROM events ee WHERE ee.id = e.parent_id) AS sub_events_count
        FROM events e
        WHERE e.parent_id IS NULL
        ORDER BY e.name 
        LIMIT $1 OFFSET $2"#,
    )
    .bind(limit)
    .bind(offset)
    .fetch_all(e.as_mut())
    .await
}

/// Records a new occurrence of an event
pub async fn event_occurrence_create(
    mut e: impl AsMut<SqliteConnection>,
    event_id: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO event_occurs (event_id, timestamp) VALUES (?, ?);")
        .bind(event_id)
        .bind(OffsetDateTime::now_utc())
        .execute(e.as_mut())
        .await?;

    Ok(())
}

/// Fetches a specific event and all its direct children
pub async fn event_with_children(
    mut e: impl AsMut<SqliteConnection>,
    event_id: u32,
) -> Result<RawEventWithChildren, sqlx::Error> {
    let row = sqlx::query_as::<_, RawEventWithChildren>(
        r#"
        WITH daily_counts AS (
            SELECT event_id, COUNT(*) as cnt 
            FROM event_occurs 
            WHERE DATE(timestamp) = DATE('now') 
            GROUP BY event_id
        )
        SELECT 
            e.id, e.name, e.svg_icon, e.created_at, e.edited_at, e.user_enabled, e.parent_id,
            COALESCE(dc.cnt, 0) as event_occurrence,
            COALESCE(
                (
                    SELECT json_group_array(
                        json_object(
                            'id', s.id,
                            'name', s.name,
                            'svg_icon', s.svg_icon,
                            'created_at', s.created_at,
                            'edited_at', s.edited_at,
                            'user_enabled', s.user_enabled,
                            'event_occurrence', COALESCE(sc.cnt, 0)
                        )
                    )
                    FROM events s
                    LEFT JOIN daily_counts sc ON s.id = sc.event_id
                    WHERE s.parent_id = e.id
                ),
                '[]'
            ) as sub_events
        FROM events e
        LEFT JOIN daily_counts dc ON e.id = dc.event_id
        WHERE e.id = $1
        "#,
    )
    .bind(event_id)
    .fetch_one(e.as_mut())
    .await?;

    Ok(row)
}
