use auth_service::Application;

pub struct TestApp {
    pub address: String,
    pub http_client: reqwest::Client,
}

impl TestApp {
    pub async fn new() -> Self {
        let app = Application::build("127.0.0.0:0")
            .await
            .expect("Failed to build app");

        let address = format!("http://{}", app.address.clone());

        let _ = tokio::spawn(app.run());

        let http_client = reqwest::Client::new();

        TestApp {
            address,
            http_client,
        }
    }

    pub async fn get_root(&self) -> reqwest::Response {
        self.http_client
            .get(&format!("{}/", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn sign_up(&self, email: String, password: String) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/signup", &self.address))
            .json(&format!("email:{},password:{}", email, password))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn login(&self, email: String, password: String) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/login", &self.address))
            .json(&format!("email:{},password:{}", email, password))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn logout(&self) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/logout", &self.address))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_2fa(&self, code: String) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify_2fa", &self.address))
            .json(&format!("code:{}", code))
            .send()
            .await
            .expect("Failed to execute request")
    }

    pub async fn verify_token(&self, token: String) -> reqwest::Response {
        self.http_client
            .post(&format!("{}/verify_token", &self.address))
            .json(&format!("token:{}", token))
            .send()
            .await
            .expect("Failed to execute request")
    }
}
