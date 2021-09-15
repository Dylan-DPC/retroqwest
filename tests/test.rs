use reqwest::ClientBuilder;
use retroqwest::RetroqwestError;

use serde::{Deserialize, Serialize};
use wiremock::{
    matchers::{method, path},
    Mock, ResponseTemplate,
};

#[derive(Deserialize, Serialize)]
pub struct HttpBinResponse {
    pub url: String,
}

#[retroqwest::retroqwest]
pub trait HttpBin {
    #[get::json("/anything")]
    async fn get_anything(&self) -> Result<HttpBinResponse, RetroqwestError>;

    #[get::json("/anything/{name}")]
    async fn get_by_name(&self, name: String) -> Result<HttpBinResponse, RetroqwestError>;
}

impl HttpBinClient {
    pub fn new(base_uri: String) -> Result<Self, RetroqwestError> {
        Self::from_builder(base_uri, ClientBuilder::default())
    }
}

// This method allows for better code completion
// since `impl HttpBin` is better than the generated struct...
fn build_client(uri: String) -> Result<impl HttpBin, retroqwest::RetroqwestError> {
    Ok(HttpBinClient::new(uri)?)
}

#[tokio::test]
async fn test_simple_gets() -> Result<(), Box<dyn std::error::Error>> {
    let server = wiremock::MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/anything"))
        .respond_with(ResponseTemplate::new(200).set_body_json(HttpBinResponse {
            url: "test".to_string(),
        }))
        .mount(&server)
        .await;

    let client = build_client(server.uri())?;
    let result: HttpBinResponse = client.get_anything().await?;

    assert_eq!(result.url, "test".to_string());
    Ok(())
}

#[tokio::test]
async fn test_gets_with_vars_in_path() -> Result<(), Box<dyn std::error::Error>> {
    let server = wiremock::MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/anything/test"))
        .respond_with(ResponseTemplate::new(200).set_body_json(HttpBinResponse {
            url: "test".to_string(),
        }))
        .mount(&server)
        .await;

    let client = build_client(server.uri())?;

    let result: HttpBinResponse = client.get_by_name("test".to_string()).await?;

    assert_eq!(result.url, "test".to_string());

    Ok(())
}

#[tokio::test]
async fn test_get_errors() -> Result<(), Box<dyn std::error::Error>> {
    let server = wiremock::MockServer::start().await;

    Mock::given(method("GET"))
        .and(path("/anything/test"))
        .respond_with(ResponseTemplate::new(400))
        .mount(&server)
        .await;

    let client = build_client(server.uri())?;

    let result: String = client
        .get_by_name("test".to_string())
        .await
        .err()
        .unwrap()
        .to_string();

    assert!(result.starts_with(
        "Response status code indicates error: HTTP status client error (400 Bad Request)"
    ));
    Ok(())
}
