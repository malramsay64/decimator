use anyhow::Result;

use adw::prelude::*;
use adw::Application;
use gio::resources_register_include;
use gtk::{gio, glib};

use window::Window;

mod picture_object;
mod telemetry;
mod thumbnail_image;
mod window;

use telemetry::{get_subscriber, init_subscriber};

const APP_ID: &str = "com.malramsay.Decimator";

fn main() -> Result<()> {
    let subscriber = get_subscriber(APP_ID.into(), "trace".into(), std::io::stdout);
    init_subscriber(subscriber);

    resources_register_include!("decimator.gresource").expect("Failed to register resources.");
    let app = Application::builder().application_id(APP_ID).build();

    app.connect_activate(build_ui);

    app.run();
    Ok(())
}

fn build_ui(app: &Application) {
    let window = Window::new(app);

    let path = String::from("/home/malcolm/Pictures/2022");
    window.set_path(path);

    window.present();
}
