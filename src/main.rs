use adw::prelude::*;
use adw::Application;
use anyhow::Result;
use gio::resources_register_include;
use gtk::gio;
use window::Window;

mod data;
mod directory;
mod picture;
mod telemetry;
mod window;

use telemetry::{get_subscriber, init_subscriber};

const APP_ID: &str = "com.malramsay.Decimator";

fn main() -> Result<()> {
    let subscriber = get_subscriber(APP_ID.into(), "debug".into(), std::io::stdout);
    init_subscriber(subscriber);
    resources_register_include!("decimator.gresource").expect("Failed to register resources.");
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    let window = Window::new(app);

    window.present();
}
