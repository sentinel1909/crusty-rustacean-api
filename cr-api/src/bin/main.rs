// main.rs

use cr_api_docker::configuration::get_configuration;
use cr_api_docker::startup::run;
use cr_api_docker::telemetry::{get_subscriber, init_subscriber};
use sqlx::postgres::PgPoolOptions;
use std::net::TcpListener;

#[tokio::main]
async fn main() -> hyper::Result<()> {
    // initialize tracing
    let subscriber = get_subscriber("cr-api".into(), "info".into(), std::io::stdout);
    init_subscriber(subscriber);

    // Panic if we can't read configuration
    let configuration = get_configuration().expect("Failed to read configuration.");

    // setup the database connection pool;
    let connection_pool = PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(2))
        .connect_lazy_with(configuration.database.with_db());    

    // Construct an address and port, get the port from the configuration settings
    // pass the listener to the run function, which spins up the API
    let address = format!(
        "{}:{}",
        configuration.application.host, configuration.application.port
    );
    let listener = match TcpListener::bind(address) {
        Ok(listen) => listen,
        Err(err) => panic!("Could not get a valid address and port. {:?}", err),
    };
    run(listener,connection_pool)?.await
}
