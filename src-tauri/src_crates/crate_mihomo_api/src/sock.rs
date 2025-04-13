use crate::model::E;
use async_trait::async_trait;
use http_body_util::{BodyExt, Full};
use hyper::{
    Method, Request,
    body::Bytes,
    header::{HeaderName, HeaderValue},
};
use hyper_util::client::legacy::Client;
use hyperlocal::{UnixClientExt, Uri};
use serde_json::Value;
use std::sync::Arc;
use tokio::sync::Mutex;

use crate::{
    MihomoData,
    model::{MihomoClient, MihomoManager},
};

impl MihomoManager {
    pub fn new(socket_path: String) -> Self {
        let client: Client<_, Full<Bytes>> = Client::unix();
        Self {
            socket_path,
            client: Arc::new(Mutex::new(client)),
            data: Arc::new(Mutex::new(MihomoData::default())),
        }
    }
}

#[async_trait]
impl MihomoClient for MihomoManager {
    async fn set_data_proxies(&self, data: Value) {
        self.data.lock().await.proxies = data;
    }

    async fn set_data_providers_proxies(&self, data: Value) {
        self.data.lock().await.providers_proxies = data;
    }

    async fn get_data_proxies(&self) -> Value {
        self.data.lock().await.proxies.clone()
    }

    async fn get_data_providers_proxies(&self) -> Value {
        self.data.lock().await.providers_proxies.clone()
    }

    async fn generate_unix_path(&self, path: &str) -> Uri {
        Uri::new(self.socket_path.clone(), path).into()
    }

    async fn send_request(
        &self,
        path: &str,
        method: Method,
        body: Option<Value>,
    ) -> Result<Value, E> {
        let uri = self.generate_unix_path(path).await;

        let mut request_builder = Request::builder().method(method).uri(uri);

        let body_bytes = if let Some(body) = body {
            request_builder = request_builder.header(
                HeaderName::from_static("Content-Type"),
                HeaderValue::from_static("application/json"),
            );
            Bytes::from(serde_json::to_vec(&body)?)
        } else {
            Bytes::new()
        };

        let request = request_builder.body(Full::new(body_bytes))?;

        let response = self.client.lock().await.request(request).await?;
        let body_bytes = response.into_body().collect().await?.to_bytes();
        let json_value = serde_json::from_slice(&body_bytes)?;

        Ok(json_value)
    }
    async fn is_mihomo_running(&self) -> Result<(), E> {
        let _ = self.send_request("/version", Method::GET, None).await?;
        Ok(())
    }

    async fn put_configs_force(&self, clash_config_path: &str) -> Result<(), E> {
        let body = serde_json::json!({
            "path": clash_config_path
        });
        let _ = self
            .send_request("/configs?force=true", Method::PUT, Some(body))
            .await?;
        Ok(())
    }

    async fn patch_configs(&self, config: Value) -> Result<(), E> {
        let _ = self
            .send_request("/configs", Method::PATCH, Some(config))
            .await?;
        Ok(())
    }

    async fn refresh_proxies(&self) -> Result<&Self, E> {
        let data = self.send_request("/proxies", Method::GET, None).await?;
        self.set_data_proxies(data).await;
        Ok(self)
    }

    async fn refresh_providers_proxies(&self) -> Result<&Self, E> {
        let data = self
            .send_request("/providers/proxies", Method::GET, None)
            .await?;
        self.set_data_providers_proxies(data).await;
        Ok(self)
    }

    async fn get_connections(&self) -> Result<Value, E> {
        let data = self.send_request("/connections", Method::GET, None).await?;
        Ok(data)
    }

    async fn delete_connections(&self, id: &str) -> Result<(), E> {
        let _ = self
            .send_request(&format!("/connections/{}", id), Method::DELETE, None)
            .await?;
        Ok(())
    }

    async fn test_proxy_delay(
        &self,
        name: &str,
        test_url: Option<String>,
        timeout: i32,
    ) -> Result<Value, E> {
        let test_url = test_url.unwrap_or("http://cp.cloudflare.com/generate_204".to_string());
        let data = self
            .send_request(
                &format!(
                    "/proxies/{}/delay?url={}&timeout={}",
                    name, test_url, timeout
                ),
                Method::GET,
                None,
            )
            .await?;
        Ok(data)
    }
}
