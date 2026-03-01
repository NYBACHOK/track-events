#[derive(sqlx::FromRow)]
pub struct RawEvent {
    pub id: i32,
    pub name: String,
    pub svg_icon: String,
    pub created_at: String,
    pub edited_at: String,
    pub sub_event_id: i32,
}

pub async fn events(
    mut e: impl AsMut<sqlx::SqliteConnection>,
    offset: i32,
    limit: i32,
) -> Result<Vec<RawEvent>, sqlx::Error> {
    let query = format!(
        r#"SELECT id, name, svg_icon, created_at, edited_at, sub_event_id FROM events e ORDER BY e.name LIMIT {} OFFSET {}"#,
        limit, offset
    );

    let events = sqlx::query_as(&query).fetch_all(e.as_mut()).await?;

    Ok(events)
}
