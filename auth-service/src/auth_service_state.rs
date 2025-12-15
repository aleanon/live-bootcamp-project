use crate::domain::data_stores::BannedTokenStore as BannedTokensStoreTrait;
use crate::domain::data_stores::TwoFaCodeStore as TwoFaCodeStoreTrait;
use crate::domain::data_stores::UserStore as UserStoreTrait;
use crate::domain::email_client::EmailClient as EmailClientTrait;
use std::sync::Arc;
use tokio::sync::RwLock;

pub struct AuthServiceState<UserStore, BannedTokenStore, TwoFaCodeStore, EmailClient>
where
    UserStore: UserStoreTrait,
    BannedTokenStore: BannedTokensStoreTrait,
    TwoFaCodeStore: TwoFaCodeStoreTrait,
    EmailClient: EmailClientTrait,
{
    pub user_store: Arc<RwLock<UserStore>>,
    pub banned_token_store: Arc<RwLock<BannedTokenStore>>,
    pub two_fa_code_store: Arc<RwLock<TwoFaCodeStore>>,
    pub email_client: Arc<EmailClient>,
}

impl<UserStore, BannedTokenStore, TwoFaCodeStore, EmailClient>
    AuthServiceState<UserStore, BannedTokenStore, TwoFaCodeStore, EmailClient>
where
    UserStore: UserStoreTrait,
    BannedTokenStore: BannedTokensStoreTrait,
    TwoFaCodeStore: TwoFaCodeStoreTrait,
    EmailClient: EmailClientTrait,
{
    pub fn new(
        user_store: Arc<RwLock<UserStore>>,
        banned_token_store: Arc<RwLock<BannedTokenStore>>,
        two_fa_code_store: Arc<RwLock<TwoFaCodeStore>>,
        email_client: Arc<EmailClient>,
    ) -> Self {
        AuthServiceState {
            user_store,
            banned_token_store,
            two_fa_code_store,
            email_client,
        }
    }
}

impl<UserStore, BannedTokenStore, TwoFaCodeStore, EmailClient> Clone
    for AuthServiceState<UserStore, BannedTokenStore, TwoFaCodeStore, EmailClient>
where
    UserStore: UserStoreTrait,
    BannedTokenStore: BannedTokensStoreTrait,
    TwoFaCodeStore: TwoFaCodeStoreTrait,
    EmailClient: EmailClientTrait,
{
    fn clone(&self) -> Self {
        Self {
            user_store: self.user_store.clone(),
            banned_token_store: self.banned_token_store.clone(),
            two_fa_code_store: self.two_fa_code_store.clone(),
            email_client: self.email_client.clone(),
        }
    }
}
