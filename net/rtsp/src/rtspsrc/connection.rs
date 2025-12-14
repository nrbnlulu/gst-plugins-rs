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
use std::pin::Pin;
use std::task::{Context, Poll};
use tokio::io::{AsyncRead, AsyncWrite, ReadBuf};
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;

pub enum SecureTcpStream {
    Plain(TcpStream),
    Tls(TlsStream<TcpStream>),
}

impl AsyncRead for SecureTcpStream {
    fn poll_read(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        match &mut *self {
            SecureTcpStream::Plain(ref mut s) => Pin::new(s).poll_read(cx, buf),
            SecureTcpStream::Tls(ref mut s) => Pin::new(s).poll_read(cx, buf),
        }
    }
}

impl AsyncWrite for SecureTcpStream {
    fn poll_write(
        mut self: Pin<&mut Self>,
        cx: &mut Context<'_>,
        buf: &[u8],
    ) -> Poll<io::Result<usize>> {
        match &mut *self {
            SecureTcpStream::Plain(ref mut s) => Pin::new(s).poll_write(cx, buf),
            SecureTcpStream::Tls(ref mut s) => Pin::new(s).poll_write(cx, buf),
        }
    }

    fn poll_flush(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            SecureTcpStream::Plain(ref mut s) => Pin::new(s).poll_flush(cx),
            SecureTcpStream::Tls(ref mut s) => Pin::new(s).poll_flush(cx),
        }
    }

    fn poll_shutdown(mut self: Pin<&mut Self>, cx: &mut Context<'_>) -> Poll<io::Result<()>> {
        match &mut *self {
            SecureTcpStream::Plain(ref mut s) => Pin::new(s).poll_shutdown(cx),
            SecureTcpStream::Tls(ref mut s) => Pin::new(s).poll_shutdown(cx),
        }
    }
}

pub async fn connect_secure(
    hostname: &str,
    port: u16,
    is_tls: bool,
) -> Result<SecureTcpStream, Box<dyn std::error::Error>> {
    let addr = format!("{}:{}", hostname, port);
    let stream = TcpStream::connect(&addr).await?;

    if is_tls {
        let connector = native_tls::TlsConnector::builder().build()?;
        let connector = tokio_native_tls::TlsConnector::from(connector);
        let tls_stream = connector.connect(hostname, stream).await?;
        Ok(SecureTcpStream::Tls(tls_stream))
    } else {
        Ok(SecureTcpStream::Plain(stream))
    }
}
