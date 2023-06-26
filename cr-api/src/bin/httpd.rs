// main.rs

// dependencies, internal and external
use cr_api::configuration::get_configuration;
use cr_api::startup::Application;
use cr_api::telemetry::{get_subscriber, init_subscriber};

// main function
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // initialize tracing
    let subscriber = get_subscriber("cr-api".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // read configuration file, panic if we can't get a configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    // return an instance of the application
    let application = Application::build(configuration).await?;
    application.run_until_stopped().await?;
    Ok(())
}