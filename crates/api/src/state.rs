use std::env;
use std::str::FromStr;

use bb8::Pool;
use bb8_postgres::PostgresConnectionManager;
use eyre::{Error, Report, Result};
use openssl::error::ErrorStack;
use openssl::ssl::{SslConnector, SslMethod};
use postgres_openssl::MakeTlsConnector;
use tokio_postgres::Config;
use tracing::{debug, error, instrument, trace};

use trillian::client::{TrillianClient, TrillianClientApiMethods};

pub type ConnectionPool = Pool<PostgresConnectionManager<MakeTlsConnector>>;
pub type TrillianState = Box<dyn TrillianClientApiMethods + Send + Sync>;

#[allow(dead_code)]
#[derive(Builder, Clone)]
#[builder(build_fn(private, name = "fallible_build"))]
pub struct AppState {
    #[builder(try_setter, setter(into, name = "trillian_tree"))]
    pub trillian_tree: i64,

    pub trillian: TrillianState,

    trillian_host: String,

    #[builder(setter(custom))]
    pub db_pool: ConnectionPool,
    #[builder(setter(custom))]
    db_config: Config,
}

impl AppStateBuilder {
    #[instrument(skip(self))]
    pub fn create_trillian_client(&mut self, host: &str) -> &mut Self {
        let new = self;
        new.trillian_host = Some(host.to_string());
        new
    }

    #[instrument(skip(self))]
    pub fn create_postgres_client(&mut self, host: &str) -> &mut Self {
        let mut config = Config::from_str(host).expect("valid db url");
        config.application_name("image-veracity-api");
        if let Ok(pwd) = env::var("DATABASE_PASSWORD") {
            debug!("Setting DB password from environment variable");
            config.password(pwd);
        }
        self.db_config = Some(config);
        self
    }

    fn ssl_config() -> Result<MakeTlsConnector, ErrorStack> {
        let mut builder = SslConnector::builder(SslMethod::tls())?;
        if let Ok(root_cert_path) = env::var("DATABASE_ROOT_CERT_PATH") {
            debug!("Setting CA to path {}", root_cert_path);
            builder.set_ca_file(root_cert_path)?;
        }
        Ok(MakeTlsConnector::new(builder.build()))
    }

    #[instrument(skip(self))]
    pub async fn build(&mut self) -> Result<AppState> {
        let connector = match AppStateBuilder::ssl_config() {
            Ok(x) => x,
            Err(err) => return Err(Report::from(err)),
        };

        let config = match self.db_config.as_ref() {
            None => return Err(Error::msg("expected database configuration")),
            Some(x) => x.clone(),
        };

        // set up connection pool
        let pg_mgr = PostgresConnectionManager::new(config, connector);
        let pool = match Pool::builder().max_size(15).build(pg_mgr).await {
            Ok(pool) => pool,
            Err(e) => {
                error!("{}", e);
                panic!("connection pool error: {e:?}")
            }
        };
        debug!("Created DB connection pool");
        self.db_pool = Some(pool);

        // When we need to make out client
        if self.trillian.is_none() {
            let host = self
                .trillian_host
                .replace("".to_string())
                .expect("Trillian host address was supplied");

            let trillian = TrillianClient::new(host)
                .await
                .expect("created trillian client")
                .build();

            debug!("Connected Trillian client");
            self.trillian = Some(Box::from(trillian));
        }

        debug!("Created application state");
        match self.fallible_build() {
            Ok(state) => Ok(state),
            Err(err) => Err(Error::from(err)),
        }
    }
}
