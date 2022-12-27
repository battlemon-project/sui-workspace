use battlemon_backend::{config, startup::App};
use reqwest::{Client, Response};

pub struct TestApp {
    pub addr: String,
}

impl TestApp {
    pub async fn get(&self, path: &str, query: &str) -> Response {
        Client::new()
            .get(&format!("http://{}/{path}?{query}", self.addr))
            .send()
            .await
            .unwrap_or_else(|e| panic!("Failed to execute request {:#?}", e))
    }
}

pub async fn spawn_app() -> TestApp {
    let config = config::load_config().expect("Failed to load config");
    let app = App::build(config).await.expect("Failed to build app");
    let addr = format!("127.0.0.1:{}", app.port());
    tokio::spawn(app.run_until_stopped());

    TestApp { addr }
}
