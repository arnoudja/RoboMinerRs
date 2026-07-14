use robominer_db::{AppShellHudRecord, MySqlPool};

use crate::DomainError;

pub async fn load_app_shell_hud(
    pool: &MySqlPool,
    user_id: i64,
) -> Result<AppShellHudRecord, DomainError> {
    robominer_db::load_app_shell_hud(pool, user_id)
        .await
        .map_err(DomainError::from)
}
