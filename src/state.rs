use eyre::Result;

use trillian::client::TrillianClient;

#[derive(Clone)]
pub struct AppState {
    pub trillian: TrillianClient,
}

impl AppState {
    pub async fn new(host: String) -> Result<AppState> {
        let trillian = TrillianClient::new(host).await?.build();
        Ok(AppState { trillian })
    }
}
