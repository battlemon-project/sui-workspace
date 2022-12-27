use crate::helpers::spawn_app;
use color_eyre::eyre::Result;

mod helpers;

#[tokio::test]
async fn healthcheck_success() -> Result<()> {
    let app = spawn_app().await;
    let resp = app.get("healthcheck", "").await;
    assert!(resp.status().is_success());

    Ok(())
}
