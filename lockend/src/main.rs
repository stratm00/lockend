use sqlx::postgres::PgListener;
use sqlx::*;
mod consts {
    pub const LOCKERS_N: usize = 30;
}
mod db;
mod endpoints;
mod state;
use axum::Router;
use axum::extract::Extension;
use axum::routing::{get, post};
use state::{LockerRoomCommand, LockerRoomState};
use tracing::info;
#[tokio::main]
async fn main() -> std::result::Result<(), Box<dyn std::error::Error>> {
    //dotenvy::dotenv()?;
    tracing_subscriber::fmt::init();

    info!("tracing_subscriber::fmt::init");
    let conn_str = std::env::var("PG_CONNECTION").expect("Need PG CONNECTION");
    let pool = sqlx::PgPool::connect(&conn_str).await.expect("CANNOT CONNECT TO PG");
    let do_setup_tbl = std::env::var("SETUP_TABLE")
        .map(|s| s == "1")
        .unwrap_or(false);

    if do_setup_tbl {
        setup_tbl(&pool).await;
    }

    use consts::LOCKERS_N;
    //Create state
    let mut locker_room_state = initialize_locker_state::<{ LOCKERS_N }>(&pool).await;

    //Load Commands, insert into state

    let app = Router::new()
        .route("/all", get(endpoints::current_state::<{ LOCKERS_N }>))
        .route(
            "/id/{id}",
            get(endpoints::get_single_locker::<{ LOCKERS_N }>),
        )
        .route(
            "/timestamp_travel/{ts}",
            get(endpoints::time_travel_ts::<{ LOCKERS_N }>),
        )
        .route(
            "/time_travel_id/{ts}",
            get(endpoints::time_travel_id::<{ LOCKERS_N }>),
        )
        .route("/", post(endpoints::make_command))
        .with_state(locker_room_state.clone())
        .layer(Extension(pool.clone()));
    let mut listener = PgListener::connect_with(&pool).await?;

    info!("Entering LISTEN");

    listener.listen("commands_notify").await?;
    let cloned_pool = pool.clone();
    let _listener = tokio::spawn(async move {
        loop {
            let not = listener.recv().await;
            info!("RCVD: {not:?}");
            //Grab the latest Command, update inner state
            match db::get_latest_locker_room_command(&cloned_pool).await {
                Ok(cmd) => {
                    info!("gotlatestlockerroomcoommand!!!! {cmd:?}");
                    let _ = locker_room_state.apply(cmd).await;
                }
                Err(error) => {
                    info!("err!!!! {error:?}");
                }
            }
        }
    });

    let tcp_lstnr = tokio::net::TcpListener::bind("0.0.0.0:8088").await.unwrap();
    let _ = axum::serve(tcp_lstnr, app).await;
    pool.close().await;
    Ok(())
}
async fn initialize_locker_state<const N: usize>(pool: &PgPool) -> LockerRoomState<N> {
    let commands = db::get_all_locker_room_commands(pool).await;
    info!("Retrieve Locker Commands: {commands:?}");
    let mut st = LockerRoomState::<N>::new();

    let errs: Vec<_> = st
        .apply_in_series(&commands)
        .await
        .into_iter()
        .map(|e| e.format())
        .collect();
    info!("Errs: {errs:?}");
    info!("Initialized Locker State!!");
    st
}
async fn setup_tbl(pool: &PgPool) -> () {
    let res = sqlx::query(
        r#"CREATE TABLE IF NOT EXISTS commands(
            cmdid serial unique PRIMARY KEY,
            ts timestamp DEFAULT current_timestamp,
            data json
        );"#,
    )
    .execute(pool)
    .await;
    info!("tbl setup: {res:?}");
}
