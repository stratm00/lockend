use crate::db::insert_locker_room_command_and_notify;
use crate::state::{LockerID, LockerState, UserID};
use crate::{LockerRoomCommand, LockerRoomState};
use axum::Json;
use axum::extract::{Path, State};
use sqlx::*;
use tracing::info;
//GET
use crate::db::{get_locker_room_commands_up_to_id, get_locker_room_commands_up_to_ts};
pub async fn time_travel_id<const N: usize>(
    Extension(pool): Extension<PgPool>,
    Path(to): Path<u64>,
) -> Json<Vec<LockerState>> {
    //Get Commands from DB
    let commands = get_locker_room_commands_up_to_id(&pool, to.try_into().unwrap()).await;
    let mut history = LockerRoomState::<N>::new();
    history.apply_in_series(&commands).await;

    Json(Vec::from(history.grab_copy().await))
}
use time::{OffsetDateTime, PlainDateTime};
pub async fn time_travel_ts<const N: usize>(
    Extension(pool): Extension<PgPool>,
    Path(to): Path<i64>,
) -> Json<Vec<LockerState>> {
    //Get Commands from DB
    let odt = OffsetDateTime::from_unix_timestamp(to).unwrap();
    let pdt = PlainDateTime::new(odt.date(), odt.time());
    let commands = get_locker_room_commands_up_to_ts(&pool, pdt).await;
    let mut history = LockerRoomState::<N>::new();
    history.apply_in_series(&commands).await;

    Json(Vec::from(history.grab_copy().await))
}

pub async fn current_state<const N: usize>(
    State(mut state): State<LockerRoomState<{ N }>>,
) -> Json<Vec<LockerState>> {
    Json(Vec::from(state.grab_copy().await))
}

pub async fn get_single_locker<const N: usize>(
    Path(number): Path<usize>,
    State(mut state): State<LockerRoomState<{ N }>>,
) -> Json<LockerState> {
    Json(state.grab_copy().await[number].clone())
}
use serde::{Deserialize, Serialize};
#[derive(Copy, Clone, Serialize, Deserialize)]
pub struct MakeCommandParams {
    id: LockerID,
    user: UserID,
}
use axum::extract::Extension;
use tracing::debug;
#[axum::debug_handler]
pub async fn make_command(
    Extension(pool): Extension<PgPool>,
    Json(params): Json<MakeCommandParams>,
) -> &'static str {
    //Insert Command into DB, using NOTIFY to kickoff program-internal state update
    let new_command = LockerRoomCommand::Actuate(params.user, params.id);
    insert_locker_room_command_and_notify(&pool, new_command).await;
    "OK"
}
