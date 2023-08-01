const APP_ID: &str = "com.malramsay.Decimator";

use decimator::telemetry::{get_subscriber_terminal, init_subscriber};
use decimator::App;
use iced::{Application, Settings};

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

    App::run(Settings {
        flags: database_path,
        ..Default::default()
    })
}
