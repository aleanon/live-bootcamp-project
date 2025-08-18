use auth_service::Application;

#[tokio::main]
async fn main() {
    Application::build("0.0.0.0:3000")
        .await
        .expect("Failed to build app")
        .run()
        .await
        .expect("Failed to run app")
}
