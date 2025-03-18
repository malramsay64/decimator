const APP_ID: &str = "com.malramsay.Decimator";

use anyhow::Error;
use camino::Utf8PathBuf;
use decimator::directory::{DirectoryData, DirectoryMessage};
use decimator::telemetry::{get_subscriber_terminal, init_subscriber};
use decimator::{App, Message};
use entity::directory;
use entity::picture;
use futures::{StreamExt, TryStreamExt};
use iced::Task;
use sea_orm::entity::*;
use sea_orm::prelude::*;
use sea_orm::query::*;
use sea_orm::{ConnectOptions, Database, DatabaseConnection, EntityOrSelect, EntityTrait};
use tracing::instrument::WithSubscriber;
use tracing::Instrument;
use uuid::Uuid;

fn main() -> Result<(), iced::Error> {
    // Configure tracing information
    let subscriber = get_subscriber_terminal(APP_ID.into(), "trace".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Set up the database we are running from
    let mut path = dirs::data_local_dir().expect("Unable to find local data dir");
    path.push(crate::APP_ID);
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    let database_path = format!("sqlite://{}/database.db?mode=rwc", path.display());
    dbg!(&database_path);

    let mut connection_options = ConnectOptions::new(database_path);
    // The minimum number of connections is rather important. There are cases within the application where
    // we have multiple connections open simultaneously to handle the streaming of data from the database
    // while performing operations on the data. This doesn't work if we don't increase the minimum number
    // of connections resulting in a lock on the connections.
    connection_options.max_connections(20).min_connections(4);
    tracing::debug!("Connection Options: {:?}", connection_options);
    let handle = Database::connect(connection_options);

    let connection = {
        let runtime = tokio::runtime::Builder::new_multi_thread()
            .worker_threads(1)
            .enable_all()
            .build()
            .unwrap();
        runtime
            .block_on(handle)
            .expect("Unable to initialise sqlite database")
    };

    let app = App::new(connection);

    iced::application("Decimator", App::update, App::view)
        .subscription(App::subscription)
        .run_with(|| {
            (
                app,
                Task::done(DirectoryMessage::QueryDirectories).map(Message::Directory),
            )
        })
}
