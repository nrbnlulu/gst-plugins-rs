// GStreamer RTSP Source 2 - Secure Connection Support
//
// Copyright (C) 2023-2024 GStreamer contributors
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// <https://mozilla.org/MPL/2.0/>.
//
// SPDX-License-Identifier: MPL-2.0

use std::io;
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::net::TcpStream;
use native_tls::TlsConnector;
use tokio_tls::TlsStream;

pub enum SecureTcpStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl AsyncRead for SecureTcpStream {
    fn poll_read(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &mut tokio::io::ReadBuf<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        match &mut *self {
            SecureTcpStream::Plain(ref mut s) => std::pin::Pin::new(s).poll_read(cx, buf),
            SecureTcpStream::Tls(ref mut s) => std::pin::Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for SecureTcpStream {
    fn poll_write(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
        buf: &[u8],
    ) -> std::task::Poll<io::Result<usize>> {
        match &mut *self {
            SecureTcpStream::Plain(ref mut s) => std::pin::Pin::new(s).poll_write(cx, buf),
            SecureTcpStream::Tls(ref mut s) => std::pin::Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        match &mut *self {
            SecureTcpStream::Plain(ref mut s) => std::pin::Pin::new(s).poll_flush(cx),
            SecureTcpStream::Tls(ref mut s) => std::pin::Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<io::Result<()>> {
        match &mut *self {
            SecureTcpStream::Plain(ref mut s) => std::pin::Pin::new(s).poll_shutdown(cx),
            SecureTcpStream::Tls(ref mut s) => std::pin::Pin::new(s).poll_shutdown(cx),
        }
    }
}

pub async fn connect_secure(hostname_port: &str, is_tls: bool) -> Result<SecureTcpStream, Box<dyn std::error::Error>> {
    let stream = TcpStream::connect(hostname_port).await?;
    
    if is_tls {
        let connector = TlsConnector::builder().build()?;
        let tls_stream = tokio_tls::TlsConnector::from(connector)
            .connect(&hostname_port.split(':').next().unwrap_or(""), stream)
            .await?;
        Ok(SecureTcpStream::Tls(tls_stream))
    } else {
        Ok(SecureTcpStream::Plain(stream))
    }
}