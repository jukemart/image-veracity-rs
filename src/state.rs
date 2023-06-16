use eyre::Result;
use tracing::{instrument, trace};

use trillian::client::TrillianClient;

#[derive(Builder, Clone)]
#[builder(
    pattern = "immutable",
    custom_constructor,
    create_empty = "empty",
    build_fn(private, name = "fallible_build", validate = "Self::validate")
)]
pub struct AppState {
    pub trillian_tree: i64,

    #[builder(setter(custom))]
    pub trillian: TrillianClient,

    // TODO replace with actual
    #[builder(default)]
    pub db_connection: Option<i64>,
}

impl AppStateBuilder {
    pub async fn create_trillian_client(host: String) -> Result<AppStateBuilder> {
        let trillian: TrillianClient = TrillianClient::new(host).await?.build();
        let mut state_builder = AppStateBuilder::empty();
        state_builder.trillian = Some(trillian);
        Ok(state_builder)
    }

    #[instrument(skip(self))]
    pub fn build(&self) -> AppState {
        trace!("Created application state");
        self.fallible_build()
            .expect("All required fields were initialized")
    }
    fn validate(&self) -> Result<(), String> {
        if self.trillian.is_none() {
            return Err("Trillian client is required".to_string());
        }
        if self.trillian_tree.is_none() {
            return Err("Trillian tree target is required".to_string());
        }

        Ok(())
    }
}
