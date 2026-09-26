use crate::state::LockerRoomCommand;
use sqlx::*;
#[derive(Debug, FromRow)]
struct LockerRoomCommandRow {
    cmdid: i32,
    data: serde_json::Value,
    ts: Option<time::PlainDateTime>,
}

impl From<LockerRoomCommandRow> for LockerRoomCommand {
    fn from(lrcr: LockerRoomCommandRow) -> Self {
        Self::Actuate(
            lrcr.data["Actuate"][0].as_u64().expect("AAA"),
            lrcr.data["Actuate"][1].as_u64().expect("AAA") as usize,
        )
    }
}
pub async fn get_latest_locker_room_command(pool: &PgPool) -> Result<LockerRoomCommand, ()> {
    sqlx::query_as!(
        LockerRoomCommandRow,
        r#"SELECT * FROM commands ORDER BY cmdid DESC LIMIT 1;"#
    )
    .fetch_one(pool)
    .await
    .map_err(|_e| ())
    .map(|row| row.into())
}
pub async fn get_all_locker_room_commands(pool: &PgPool) -> Vec<LockerRoomCommand> {
    let _tx = pool.begin().await.ok().unwrap();
    let commands: Vec<_> = sqlx::query_as!(LockerRoomCommandRow, r#"SELECT * FROM commands"#)
        .fetch_all(pool)
        .await
        .ok()
        .unwrap();

    commands.into_iter().map(|r| r.into()).collect()
}
pub async fn get_locker_room_commands_up_to_ts(
    pool: &PgPool,
    ts: time::PlainDateTime,
) -> Vec<LockerRoomCommand> {
    let commands = sqlx::query_as!(
        LockerRoomCommandRow,
        "SELECT * FROM commands WHERE ts <= $1",
        ts
    )
    .fetch_all(pool)
    .await
    .ok()
    .unwrap();

    commands.into_iter().map(|r| r.into()).collect()
}
pub async fn get_locker_room_commands_up_to_id(pool: &PgPool, id: i32) -> Vec<LockerRoomCommand> {
    let commands = sqlx::query_as!(
        LockerRoomCommandRow,
        "SELECT * FROM commands WHERE cmdid <= $1",
        id
    )
    .fetch_all(pool)
    .await
    .ok()
    .unwrap();

    commands.into_iter().map(|row| row.into()).collect()
}
pub async fn insert_locker_room_command_and_notify(pool: &PgPool, lrc: LockerRoomCommand) {
    let _command = sqlx::query!(
        "INSERT INTO commands (data) VALUES ($1);",
        serde_json::to_value(lrc).expect("AA")
    )
    .fetch_all(pool)
    .await
    .ok()
    .unwrap();
    sqlx::query!("NOTIFY commands_notify;")
        .fetch_all(pool)
        .await
        .ok()
        .unwrap();
}
