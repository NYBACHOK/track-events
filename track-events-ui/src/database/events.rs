use serde::{Deserialize, Serialize};
use sqlx::sqlite::SqliteConnection;
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
    pub sub_events: Vec<RawEvent>,
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
            (SELECT COUNT(*) FROM events WHERE parent_id = e.id) AS sub_events_count
        FROM events e
        WHERE e.parent_id IS NULL
        ORDER BY sub_events_count, e.name
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
    sqlx::query("INSERT INTO event_occurs (event_id) VALUES ($1);")
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
    let RawEvent { id, name, svg_icon, created_at, edited_at, user_enabled, event_occurrence, sub_events_count : _ } = sqlx::query_as::<_, RawEvent>(
        r#"
        SELECT 
            e.id, e.name, e.svg_icon, e.created_at, e.edited_at, e.user_enabled,
            (SELECT COUNT(*) FROM event_occurs WHERE event_id = e.id AND DATE(timestamp) = DATE('now')) AS event_occurrence,
            (SELECT COUNT(*) FROM events WHERE parent_id = e.id) AS sub_events_count
        FROM events e
        WHERE e.parent_id IS NULL AND e.id = $1;
        "#,
    )
    .bind(event_id)
    .fetch_one(e.as_mut())
    .await?;

    let sub_events = sqlx::query_as::<_, RawEvent>(
        r#"
        SELECT e.id, e.name, e.svg_icon, e.created_at, e.edited_at,e.user_enabled,
            (SELECT COUNT(*) FROM event_occurs WHERE event_id = e.id AND DATE(timestamp) = DATE('now')) AS event_occurrence,
            0 as sub_events_count
        FROM events e
        WHERE e.parent_id = $1;"#,
    )
    .bind(event_id)
    .fetch_all(e.as_mut())
    .await?;

    let res = RawEventWithChildren {
        id,
        name,
        svg_icon,
        created_at,
        edited_at,
        user_enabled,
        event_occurrence,
        sub_events,
    };

    Ok(res)
}
