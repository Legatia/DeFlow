use crate::types::ConfigValue;
use ic_cdk::api::management_canister::http_request::{
    http_request, CanisterHttpRequestArgument, HttpHeader, HttpMethod, HttpResponse, TransformArgs, TransformContext
};
use serde_json::Value;
use std::collections::HashMap;

pub struct HttpClient;

#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<HttpHeader>,
    pub body: Option<Vec<u8>>,
    pub max_response_bytes: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct HttpClientResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body: String,
}

impl HttpClient {
    pub async fn request(request: HttpRequest) -> Result<HttpClientResponse, String> {
        let http_req = CanisterHttpRequestArgument {
            url: request.url.clone(),
            method: request.method,
            body: request.body,
            max_response_bytes: request.max_response_bytes,
            transform: Some(TransformContext::from_name("transform_http_response".to_string(), vec![])),
            headers: request.headers,
        };

        match http_request(http_req, 10_000_000_000).await {
            Ok((response,)) => {
                let headers = response.headers.iter()
                    .map(|header| (header.name.clone(), header.value.clone()))
                    .collect();

                Ok(HttpClientResponse {
                    status: 200u16, // Default to 200, TODO: parse actual status
                    headers,
                    body: String::from_utf8_lossy(&response.body).to_string(),
                })
            }
            Err((rejection_code, msg)) => {
                Err(format!("HTTP request failed: {:?} - {}", rejection_code, msg))
            }
        }
    }

    pub async fn get(url: &str, headers: Option<HashMap<String, String>>) -> Result<HttpClientResponse, String> {
        let http_headers = headers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, value)| HttpHeader { name, value })
            .collect();

        let request = HttpRequest {
            url: url.to_string(),
            method: HttpMethod::GET,
            headers: http_headers,
            body: None,
            max_response_bytes: Some(1024 * 1024), // 1MB max
        };

        Self::request(request).await
    }

    pub async fn post(
        url: &str,
        body: Option<String>,
        headers: Option<HashMap<String, String>>
    ) -> Result<HttpClientResponse, String> {
        let mut http_headers = headers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, value)| HttpHeader { name, value })
            .collect::<Vec<_>>();

        // Add Content-Type if not present and body exists
        if body.is_some() && !http_headers.iter().any(|h| h.name.to_lowercase() == "content-type") {
            http_headers.push(HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            });
        }

        let request = HttpRequest {
            url: url.to_string(),
            method: HttpMethod::POST,
            headers: http_headers,
            body: body.map(|b| b.into_bytes()),
            max_response_bytes: Some(1024 * 1024), // 1MB max
        };

        Self::request(request).await
    }

    pub async fn put(
        url: &str,
        body: Option<String>,
        headers: Option<HashMap<String, String>>
    ) -> Result<HttpClientResponse, String> {
        let mut http_headers = headers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, value)| HttpHeader { name, value })
            .collect::<Vec<_>>();

        if body.is_some() && !http_headers.iter().any(|h| h.name.to_lowercase() == "content-type") {
            http_headers.push(HttpHeader {
                name: "Content-Type".to_string(),
                value: "application/json".to_string(),
            });
        }

        let request = HttpRequest {
            url: url.to_string(),
            method: HttpMethod::POST, // ICP doesn't support PUT, use POST
            headers: http_headers,
            body: body.map(|b| b.into_bytes()),
            max_response_bytes: Some(1024 * 1024),
        };

        Self::request(request).await
    }

    pub async fn delete(url: &str, headers: Option<HashMap<String, String>>) -> Result<HttpClientResponse, String> {
        let http_headers = headers
            .unwrap_or_default()
            .into_iter()
            .map(|(name, value)| HttpHeader { name, value })
            .collect();

        let request = HttpRequest {
            url: url.to_string(),
            method: HttpMethod::GET, // ICP doesn't support DELETE, use GET
            headers: http_headers,
            body: None,
            max_response_bytes: Some(1024 * 1024),
        };

        Self::request(request).await
    }
}

// Transform function required for HTTP outcalls
#[ic_cdk::query]
fn transform_http_response(args: TransformArgs) -> HttpResponse {
    let mut response = args.response;
    
    // Remove sensitive headers
    response.headers.retain(|header| {
        !matches!(header.name.to_lowercase().as_str(), 
            "authorization" | "cookie" | "set-cookie" | "x-api-key"
        )
    });
    
    response
}

// Helper functions for working with API responses
pub fn parse_json_response(response: &HttpClientResponse) -> Result<Value, String> {
    serde_json::from_str(&response.body)
        .map_err(|e| format!("Failed to parse JSON: {}", e))
}

pub fn json_to_config_value(value: &Value) -> ConfigValue {
    match value {
        Value::Null => ConfigValue::String("null".to_string()),
        Value::Bool(b) => ConfigValue::Boolean(*b),
        Value::Number(n) => ConfigValue::Number(n.as_f64().unwrap_or(0.0)),
        Value::String(s) => ConfigValue::String(s.clone()),
        Value::Array(arr) => {
            let config_array: Vec<ConfigValue> = arr.iter()
                .map(json_to_config_value)
                .collect();
            ConfigValue::Array(config_array)
        }
        Value::Object(obj) => {
            let config_object: HashMap<String, ConfigValue> = obj.iter()
                .map(|(k, v)| (k.clone(), json_to_config_value(v)))
                .collect();
            ConfigValue::Object(config_object)
        }
    }
}

pub fn config_value_to_json(value: &ConfigValue) -> Value {
    match value {
        ConfigValue::String(s) => Value::String(s.clone()),
        ConfigValue::Number(n) => serde_json::Number::from_f64(*n)
            .map(Value::Number)
            .unwrap_or(Value::Null),
        ConfigValue::Boolean(b) => Value::Bool(*b),
        ConfigValue::Array(arr) => {
            let json_array: Vec<Value> = arr.iter()
                .map(config_value_to_json)
                .collect();
            Value::Array(json_array)
        }
        ConfigValue::Object(obj) => {
            let json_object: serde_json::Map<String, Value> = obj.iter()
                .map(|(k, v)| (k.clone(), config_value_to_json(v)))
                .collect();
            Value::Object(json_object)
        }
    }
}

// Utility function to merge headers
pub fn merge_headers(
    base_headers: Option<HashMap<String, String>>,
    additional_headers: HashMap<String, String>
) -> HashMap<String, String> {
    let mut merged = base_headers.unwrap_or_default();
    for (key, value) in additional_headers {
        merged.insert(key, value);
    }
    merged
}

// Common API patterns
pub struct ApiClient {
    pub base_url: String,
    pub default_headers: HashMap<String, String>,
}

impl ApiClient {
    pub fn new(base_url: String) -> Self {
        Self {
            base_url,
            default_headers: HashMap::new(),
        }
    }

    pub fn with_auth_header(mut self, token: String) -> Self {
        self.default_headers.insert("Authorization".to_string(), format!("Bearer {}", token));
        self
    }

    pub fn with_api_key(mut self, key: String) -> Self {
        self.default_headers.insert("X-API-Key".to_string(), key);
        self
    }

    pub async fn get(&self, endpoint: &str) -> Result<HttpClientResponse, String> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        HttpClient::get(&url, Some(self.default_headers.clone())).await
    }

    pub async fn post(&self, endpoint: &str, body: Option<String>) -> Result<HttpClientResponse, String> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        HttpClient::post(&url, body, Some(self.default_headers.clone())).await
    }

    pub async fn put(&self, endpoint: &str, body: Option<String>) -> Result<HttpClientResponse, String> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        HttpClient::put(&url, body, Some(self.default_headers.clone())).await
    }

    pub async fn delete(&self, endpoint: &str) -> Result<HttpClientResponse, String> {
        let url = format!("{}/{}", self.base_url.trim_end_matches('/'), endpoint.trim_start_matches('/'));
        HttpClient::delete(&url, Some(self.default_headers.clone())).await
    }
}