use std::sync::Arc;
use tokio::sync::RwLock;

use crate::domain::data_stores::BannedTokenStore as BannedTokensStoreTrait;
use crate::domain::data_stores::TwoFaCodeStore as TwoFaCodeStoreTrait;
use crate::domain::data_stores::UserStore as UserStoreTrait;
use crate::domain::email_client::EmailClient as EmailClientTrait;
use crate::utils::config::Config;

type UserStore = Arc<RwLock<dyn UserStoreTrait>>;
type BannedTokenStore = Arc<RwLock<dyn BannedTokensStoreTrait>>;
type TwoFaCodeStore = Arc<RwLock<dyn TwoFaCodeStoreTrait>>;
type EmailClient = Arc<RwLock<dyn EmailClientTrait>>;

#[derive(Clone)]
pub struct AppState {
    pub user_store: UserStore,
    pub banned_token_store: BannedTokenStore,
    pub two_fa_code_store: TwoFaCodeStore,
    pub email_client: EmailClient,
    pub config: Arc<RwLock<Config>>,
}

impl AppState {
    pub fn new(
        user_store: UserStore,
        banned_token_store: BannedTokenStore,
        two_fa_code_store: TwoFaCodeStore,
        email_client: EmailClient,
    ) -> Self {
        let config = Config::load();

        AppState {
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
            config: Arc::new(RwLock::new(config)),
        }
    }
}
