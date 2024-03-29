// cr-api/src/lib/state.rs

// everything related to state, including associated types and implementations

// dependencies
use crate::email_client::EmailClient;
use axum::extract::FromRef;
use axum_flash::Key;
use secrecy::{ExposeSecret, Secret};
use sqlx::PgPool;

// struct for the ApplicationBaseUrl type
#[derive(Debug, Clone)]
pub struct ApplicationBaseUrl(pub String);

// strut for the Hmac Secret
#[derive(Debug, Clone)]
pub struct HmacSecret(pub Secret<String>);

// struct for the AppState type, derives Clone as well as FromRef
#[derive(Clone, Debug, FromRef)]
pub struct AppState {
    pub db_pool: PgPool,
    pub em_client: EmailClient,
    pub bs_url: ApplicationBaseUrl,
    pub flash_config: axum_flash::Config,
}

// implementation block for AppState, create a state using a database pool, email client, application base url, and flash message config
impl AppState {
    pub fn create_state(
        pool: PgPool,
        client: EmailClient,
        url: ApplicationBaseUrl,
        hmac_secret: HmacSecret,
    ) -> Self {
        Self {
            db_pool: pool,
            em_client: client,
            bs_url: url,
            flash_config: axum_flash::Config::new(Key::from(
                hmac_secret.0.expose_secret().as_bytes(),
            )),
        }
    }
}
