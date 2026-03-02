use time::OffsetDateTime;

#[derive(sqlx::FromRow)]
pub struct RawEvent {
    pub id: i32,
    pub name: String,
    pub svg_icon: String,
    pub created_at: OffsetDateTime,
    pub edited_at: OffsetDateTime,
    pub sub_event_id: i32,
    pub event_occurrence: i32,
}

pub async fn events(
    mut e: impl AsMut<sqlx::SqliteConnection>,
    offset: u32,
    limit: usize,
) -> Result<Vec<RawEvent>, sqlx::Error> {
    let query = format!(
        r#"SELECT e.id, e.name, e.svg_icon, e.created_at, e.edited_at, e.sub_event_id, COALESCE(v.count, 0) 
        AS event_occurrence FROM events e LEFT JOIN (SELECT event_id, COUNT(*) 
        as count FROM event_occurs WHERE DATE(timestamp) = CURRENT_DATE GROUP BY event_id) v ON e.id = v.event_id ORDER BY e.name LIMIT {} OFFSET {}"#,
        limit, offset
    );

    let events = sqlx::query_as(&query).fetch_all(e.as_mut()).await?;

    Ok(events)
}

pub async fn event_occurrence_create(
    mut e: impl AsMut<sqlx::SqliteConnection>,
    event_id: u32,
) -> Result<(), sqlx::Error> {
    sqlx::query("INSERT INTO event_occurs (event_id, timestamp) VALUES ($1, $2);")
        .bind(event_id)
        .bind(OffsetDateTime::now_utc())
        .execute(e.as_mut())
        .await?;

    Ok(())
}
