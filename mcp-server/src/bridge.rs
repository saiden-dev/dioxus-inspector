//! HTTP client for communicating with the Dioxus inspector bridge.

use anyhow::Result;
use serde::{Deserialize, Serialize};

#[derive(Clone)]
pub struct BridgeClient {
    client: reqwest::Client,
    base_url: String,
}

#[derive(Debug, Serialize)]
pub struct EvalRequest {
    pub script: String,
}

#[derive(Debug, Serialize)]
pub struct QueryRequest {
    pub selector: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub property: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct InspectRequest {
    pub selector: String,
}

#[derive(Debug, Serialize)]
pub struct ScreenshotRequest {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
}

#[derive(Debug, Serialize)]
pub struct ResizeRequest {
    pub width: u32,
    pub height: u32,
}

#[derive(Debug, Deserialize)]
pub struct StatusResponse {
    pub status: String,
    pub app: String,
}

#[derive(Debug, Deserialize)]
pub struct EvalResponse {
    pub success: bool,
    pub result: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ScreenshotResponse {
    pub success: bool,
    pub path: Option<String>,
    pub error: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct ResizeResponse {
    pub success: bool,
    pub width: u32,
    pub height: u32,
    pub error: Option<String>,
}

impl BridgeClient {
    pub fn new(base_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            base_url: base_url.to_string(),
        }
    }

    pub async fn status(&self) -> Result<StatusResponse> {
        let resp = self
            .client
            .get(format!("{}/status", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn eval(&self, script: &str) -> Result<EvalResponse> {
        let resp = self
            .client
            .post(format!("{}/eval", self.base_url))
            .json(&EvalRequest {
                script: script.to_string(),
            })
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn query(&self, selector: &str, property: Option<&str>) -> Result<EvalResponse> {
        let resp = self
            .client
            .post(format!("{}/query", self.base_url))
            .json(&QueryRequest {
                selector: selector.to_string(),
                property: property.map(String::from),
            })
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn dom(&self) -> Result<EvalResponse> {
        let resp = self
            .client
            .get(format!("{}/dom", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn inspect(&self, selector: &str) -> Result<EvalResponse> {
        let resp = self
            .client
            .post(format!("{}/inspect", self.base_url))
            .json(&InspectRequest {
                selector: selector.to_string(),
            })
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn diagnose(&self) -> Result<EvalResponse> {
        let resp = self
            .client
            .get(format!("{}/diagnose", self.base_url))
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn screenshot(&self, path: Option<&str>) -> Result<ScreenshotResponse> {
        let resp = self
            .client
            .post(format!("{}/screenshot", self.base_url))
            .json(&ScreenshotRequest {
                path: path.map(String::from),
            })
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }

    pub async fn resize(&self, width: u32, height: u32) -> Result<ResizeResponse> {
        let resp = self
            .client
            .post(format!("{}/resize", self.base_url))
            .json(&ResizeRequest { width, height })
            .send()
            .await?
            .json()
            .await?;
        Ok(resp)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bridge_client_new() {
        let client = BridgeClient::new("http://localhost:9999");
        assert_eq!(client.base_url, "http://localhost:9999");
    }

    #[test]
    fn test_eval_request_serialize() {
        let req = EvalRequest {
            script: "return 1".to_string(),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("return 1"));
    }

    #[test]
    fn test_query_request_serialize() {
        let req = QueryRequest {
            selector: ".btn".to_string(),
            property: Some("text".to_string()),
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains(".btn"));
        assert!(json.contains("text"));
    }

    #[test]
    fn test_query_request_without_property() {
        let req = QueryRequest {
            selector: "#id".to_string(),
            property: None,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(!json.contains("property"));
    }

    #[test]
    fn test_resize_request_serialize() {
        let req = ResizeRequest {
            width: 800,
            height: 600,
        };
        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("800"));
        assert!(json.contains("600"));
    }
}
