use shipwright_db::entities::{register_token::RegisterToken, session::Session, user::User};

pub struct Account {
    pub user: User,
    pub session: Session,
    pub register_token: RegisterToken,
}
