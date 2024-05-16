const APP_ID: &str = "com.malramsay.Decimator";

use decimator::telemetry::{get_subscriber_terminal, init_subscriber};
use decimator::App;
use iced::{Application, Settings};
use sea_orm::{ConnectOptions, Database};

fn main() -> Result<(), iced::Error> {
    // Configure tracing information
    let subscriber = get_subscriber_terminal(APP_ID.into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Set up the database we are running from
    let mut path = dirs::data_local_dir().expect("Unable to find local data dir");
    path.push(crate::APP_ID);
    std::fs::create_dir_all(&path).expect("Could not create directory.");
    let database_path = format!("sqlite://{}/database.db?mode=rwc", path.display());
    dbg!(&database_path);

    let connection = tokio::task::spawn(async {
        let mut connection_options = ConnectOptions::new(database_path);
        // The minimum number of connections is rather important. There are cases within the application where
        // we have multiple connections open simultaneously to handle the streaming of data from the database
        // while performing operations on the data. This doesn't work if we don't increase the minimum number
        // of connections resulting in a lock on the connections.
        connection_options.max_connections(20).min_connections(4);
        tracing::debug!("Connection Options: {:?}", connection_options);
        Database::connect(connection_options)
            .await
            .expect("Unable to initialise sqlite database")
    });

    App::run(Settings {
        flags: connection,
        ..Default::default()
    })
}
