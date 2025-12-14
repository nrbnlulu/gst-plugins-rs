// GStreamer RTSP Source 2 - Authentication
//
// Copyright (C) 2023-2024 GStreamer contributors
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// <https://mozilla.org/MPL/2.0/>.
//
// SPDX-License-Identifier: MPL-2.0

use base64::{engine::general_purpose::STANDARD as BASE64_STANDARD, Engine as _};

use rtsp_types::headers::{AUTHORIZATION, WWW_AUTHENTICATE};
use rtsp_types::{Message, Request, Response, StatusCode, Version};

#[derive(Debug)]
pub enum AuthenticationType {
    Basic,
    Digest,
}

#[derive(Debug)]
pub struct RtspAuthenticator {
    auth_type: Option<AuthenticationType>,
    username: String,
    password: String,
    // Digest auth specific fields
    nonce: Option<String>,
    realm: Option<String>,
    opaque: Option<String>,
    algorithm: Option<String>,
    qop: Option<String>,
}

impl RtspAuthenticator {
    pub fn new(username: String, password: String) -> Self {
        RtspAuthenticator {
            auth_type: None,
            username,
            password,
            nonce: None,
            realm: None,
            opaque: None,
            algorithm: None,
            qop: None,
        }
    }

    pub fn needs_authentication(&self) -> bool {
        self.auth_type.is_some()
    }

    pub fn handle_401_challenge(
        &mut self,
        response: &Response<Vec<u8>>,
    ) -> Result<(), crate::rtspsrc::imp::RtspError> {
        if response.status() != StatusCode::Unauthorized {
            return Ok(());
        }

        // Check WWW-Authenticate header to determine auth type
        if let Some(auth_header) = response.header(&WWW_AUTHENTICATE) {
            let header_str = std::str::from_utf8(auth_header).unwrap_or("");
            if header_str.trim_start().to_lowercase().starts_with("basic") {
                self.auth_type = Some(AuthenticationType::Basic);
                // Parse basic auth params if needed
            } else if header_str.trim_start().to_lowercase().starts_with("digest") {
                self.auth_type = Some(AuthenticationType::Digest);
                // Parse digest params
                self.parse_digest_params(header_str)?;
            }
        }

        Ok(())
    }

    fn parse_digest_params(&mut self, header: &str) -> Result<(), crate::rtspsrc::imp::RtspError> {
        // Parse the digest parameters from the WWW-Authenticate header
        let auth_line = &header[6..].trim(); // Remove "Digest" prefix

        // Split the parameters by comma
        for param in auth_line.split(',') {
            let parts: Vec<&str> = param.splitn(2, '=').map(|s| s.trim()).collect();
            if parts.len() != 2 {
                continue;
            }

            let key = parts[0].trim_start_matches('"').trim_end_matches('"');
            let value = parts[1]
                .trim()
                .trim_start_matches('"')
                .trim_end_matches('"');

            match key.to_lowercase().as_str() {
                "realm" => self.realm = Some(value.to_string()),
                "nonce" => self.nonce = Some(value.to_string()),
                "opaque" => self.opaque = Some(value.to_string()),
                "algorithm" => self.algorithm = Some(value.to_string()),
                "qop" => self.qop = Some(value.to_string()),
                _ => {}
            }
        }

        if self.realm.is_none() || self.nonce.is_none() {
            return Err(crate::rtspsrc::imp::RtspError::Fatal(
                "Digest auth missing required parameters".to_string(),
            ));
        }

        Ok(())
    }

    pub fn add_auth_header(
        &self,
        request: Request<Vec<u8>>,
        cseq: u32,
        method: &str,
        uri: &str,
    ) -> Result<Request<Vec<u8>>, crate::rtspsrc::imp::RtspError> {
        match self.auth_type {
            Some(AuthenticationType::Basic) => self.add_basic_auth(request, cseq),
            Some(AuthenticationType::Digest) => self.add_digest_auth(request, cseq, method, uri),
            None => Ok(request), // No authentication needed
        }
    }

    fn add_basic_auth(
        &self,
        request: Request<Vec<u8>>,
        cseq: u32,
    ) -> Result<Request<Vec<u8>>, crate::rtspsrc::imp::RtspError> {
        let credentials = format!("{}:{}", self.username, self.password);
        let encoded = BASE64_STANDARD.encode(credentials);
        let auth_value = format!("Basic {}", encoded);

        // Rebuild the request with the authorization header
        let mut builder = Request::builder(request.method().clone(), request.version());

        // Copy existing headers
        for (name, value) in request.headers() {
            if name != &AUTHORIZATION {
                builder = builder.header(name.clone(), value);
            }
        }

        // Add authorization header
        builder = builder.header(AUTHORIZATION, auth_value.as_bytes());

        // Build with the same URI and body
        Ok(builder
            .request_uri(request.request_uri().clone())
            .build(request.into_body()))
    }

    fn add_digest_auth(
        &self,
        request: Request<Vec<u8>>,
        cseq: u32,
        method: &str,
        uri: &str,
    ) -> Result<Request<Vec<u8>>, crate::rtspsrc::imp::RtspError> {
        if self.realm.is_none() || self.nonce.is_none() {
            return Err(crate::rtspsrc::imp::RtspError::Fatal(
                "Digest auth parameters not set".to_string(),
            ));
        }

        let realm = self.realm.as_ref().unwrap();
        let nonce = self.nonce.as_ref().unwrap();

        // Calculate HA1: MD5(username:realm:password)
        let ha1_input = format!("{}:{}:{}", self.username, realm, self.password);
        let ha1 = format!("{:x}", md5::compute(ha1_input.as_bytes()));

        // Calculate HA2: MD5(method:digestURI)
        let ha2_input = format!("{}:{}", method, uri);
        let ha2 = format!("{:x}", md5::compute(ha2_input.as_bytes()));

        // Calculate response: MD5(HA1:nonce:HA2)
        let response_input = if self.qop.as_deref() == Some("auth") {
            // Quality of Protection is used, include nc and cnonce
            // For simplicity, we'll use a default nc (nonce count) and a simple cnonce
            let nc = "00000001";
            let cnonce = "1234567890123456"; // Should be random in real implementation
            format!("{}:{}:{}:{}:{}:{}", ha1, nonce, nc, cnonce, "auth", ha2)
        } else {
            format!("{}:{}:{}", ha1, nonce, ha2)
        };

        let response = format!("{:x}", md5::compute(response_input.as_bytes()));

        let mut auth_parts = vec![
            format!("username=\"{}\"", self.username),
            format!("realm=\"{}\"", realm),
            format!("nonce=\"{}\"", nonce),
            format!("uri=\"{}\"", uri),
            format!("response=\"{}\"", response),
        ];

        if let Some(opaque) = &self.opaque {
            auth_parts.push(format!("opaque=\"{}\"", opaque));
        }

        if let Some(algorithm) = &self.algorithm {
            auth_parts.push(format!("algorithm={}", algorithm));
        }

        if let Some(qop) = &self.qop {
            // Include nc and cnonce if qop is specified
            if qop == "auth" {
                auth_parts.push(format!("qop={}", qop));
                auth_parts.push("nc=00000001".to_string());
                auth_parts.push("cnonce=\"1234567890123456\"".to_string());
            }
        }

        let auth_value = format!("Digest {}", auth_parts.join(", "));

        // Rebuild the request with the authorization header
        let mut builder = Request::builder(request.method().clone(), request.version());

        // Copy existing headers
        for (name, value) in request.headers() {
            if name != &AUTHORIZATION {
                builder = builder.header(name.clone(), value);
            }
        }

        // Add authorization header
        builder = builder.header(AUTHORIZATION, auth_value.as_bytes());

        // Build with the same URI and body
        Ok(builder
            .request_uri(request.request_uri().clone())
            .build(request.into_body()))
    }
}
