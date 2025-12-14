// Rust RTSP Server
//
// Copyright (C) 2024 Nirbheek Chauhan <nirbheek@centricular.com>
//
// This Source Code Form is subject to the terms of the Mozilla Public License, v2.0.
// If a copy of the MPL was not distributed with this file, You can obtain one at
// <https://mozilla.org/MPL/2.0/>.
//
// SPDX-License-Identifier: MPL-2.0

use super::imp::RtspError;
use super::imp::RtspProtocol;
use super::imp::CAT;
use sdp_types::Attribute;
use sdp_types::Connection;
use std::collections::BTreeSet;
use std::net::IpAddr;
use url::Url;

macro_rules! init_payload_info {
    ($($t:expr),*,) => {
        [
            $(
                {
                    let (pt, media, encoding_name, clock_rate, encoding_params) = $t;
                    PayloadInfo {
                        pt,
                        media,
                        encoding_name,
                        clock_rate,
                        encoding_params,
                    }
                }
            ),*
        ]
    };
}

struct PayloadInfo<'a> {
    pt: u8,
    media: &'a str,
    encoding_name: &'a str,
    clock_rate: u32,
    encoding_params: Option<&'a str>,
}

// Copied from gst-plugins-base/gst-libs/gst/rtp/gstrtppayloads.h
const STATIC_PAYLOAD_INFO: &[PayloadInfo] = &init_payload_info!(
    // static audio
    (0, "audio", "PCMU", 8000, Some("1")),
    // (1, "audio", "reserved", 0, None),
    // (2, "audio", "reserved", 0, None),
    (3, "audio", "GSM", 8000, Some("1")),
    (4, "audio", "G723", 8000, Some("1")),
    (5, "audio", "DVI4", 8000, Some("1")),
    (6, "audio", "DVI4", 16000, Some("1")),
    (7, "audio", "LPC", 8000, Some("1")),
    (8, "audio", "PCMA", 8000, Some("1")),
    (9, "audio", "G722", 8000, Some("1")),
    (10, "audio", "L16", 44100, Some("2")),
    (11, "audio", "L16", 44100, Some("1")),
    (12, "audio", "QCELP", 8000, Some("1")),
    (13, "audio", "CN", 8000, Some("1")),
    (14, "audio", "MPA", 90000, None),
    (15, "audio", "G728", 8000, Some("1")),
    (16, "audio", "DVI4", 11025, Some("1")),
    (17, "audio", "DVI4", 22050, Some("1")),
    (18, "audio", "G729", 8000, Some("1")),
    // (19, "audio", "reserved", 0, None),
    // (20, "audio", "unassigned", 0, None),
    // (21, "audio", "unassigned", 0, None),
    // (22, "audio", "unassigned", 0, None),
    // (23, "audio", "unassigned", 0, None),

    // video and video/audio
    // (24, "video", "unassigned", 0, None),
    (25, "video", "CelB", 90000, None),
    (26, "video", "JPEG", 90000, None),
    // (27, "video", "unassigned", 0, None),
    (28, "video", "nv", 90000, None),
    // (29, "video", "unassigned", 0, None),
    // (30, "video", "unassigned", 0, None),
    (31, "video", "H261", 90000, None),
    (32, "video", "MPV", 90000, None),
    (33, "video", "MP2T", 90000, None),
    (34, "video", "H263", 90000, None),
    // (35-71, "unassigned", 0, 0, None),
    // (72-76, "reserved", 0, 0, None),
    // (77-95, "unassigned", 0, 0, None),
    // (96-127, "dynamic", 0, 0, None),
);

// Known media types with dynamic payloads, can only be matched via name
const DYNAMIC_PAYLOAD_INFO: &[PayloadInfo] = &init_payload_info!(
    (0, "application", "parityfec", 0, None), // [RFC3009]
    (0, "application", "rtx", 0, None),       // [RFC4588]
    (0, "audio", "AMR", 8000, None),          // [RFC4867][RFC3267]
    (0, "audio", "AMR-WB", 16000, None),      // [RFC4867][RFC3267]
    (0, "audio", "DAT12", 0, None),           // [RFC3190]
    (0, "audio", "dsr-es201108", 0, None),    // [RFC3557]
    (0, "audio", "EVRC", 8000, Some("1")),    // [RFC4788]
    (0, "audio", "EVRC0", 8000, Some("1")),   // [RFC4788]
    (0, "audio", "EVRC1", 8000, Some("1")),   // [RFC4788]
    (0, "audio", "EVRCB", 8000, Some("1")),   // [RFC4788]
    (0, "audio", "EVRCB0", 8000, Some("1")),  // [RFC4788]
    (0, "audio", "EVRCB1", 8000, Some("1")),  // [RFC4788]
    (0, "audio", "G7221", 16000, Some("1")),  // [RFC3047]
    (0, "audio", "G726-16", 8000, Some("1")), // [RFC3551][RFC4856]
    (0, "audio", "G726-24", 8000, Some("1")), // [RFC3551][RFC4856]
    (0, "audio", "G726-32", 8000, Some("1")), // [RFC3551][RFC4856]
    (0, "audio", "G726-40", 8000, Some("1")), // [RFC3551][RFC4856]
    (0, "audio", "G729D", 8000, Some("1")),   // [RFC3551][RFC4856]
    (0, "audio", "G729E", 8000, Some("1")),   // [RFC3551][RFC4856]
    (0, "audio", "GSM-EFR", 8000, Some("1")), // [RFC3551][RFC4856]
    (0, "audio", "L8", 0, None),              // [RFC3551][RFC4856]
    (0, "audio", "RED", 0, None),             // [RFC2198][RFC3555]
    (0, "audio", "rtx", 0, None),             // [RFC4588]
    (0, "audio", "VDVI", 0, Some("1")),       // [RFC3551][RFC4856]
    (0, "audio", "L20", 0, None),             // [RFC3190]
    (0, "audio", "L24", 0, None),             // [RFC3190]
    (0, "audio", "MP4A-LATM", 0, None),       // [RFC3016]
    (0, "audio", "mpa-robust", 90000, None),  // [RFC3119]
    (0, "audio", "parityfec", 0, None),       // [RFC3009]
    (0, "audio", "SMV", 8000, Some("1")),     // [RFC3558]
    (0, "audio", "SMV0", 8000, Some("1")),    // [RFC3558]
    (0, "audio", "t140c", 0, None),           // [RFC4351]
    (0, "audio", "t38", 0, None),             // [RFC4612]
    (0, "audio", "telephone-event", 0, None), // [RFC4733]
    (0, "audio", "tone", 0, None),            // [RFC4733]
    (0, "audio", "DVI4", 0, None),            // [RFC4856]
    (0, "audio", "G722", 0, None),            // [RFC4856]
    (0, "audio", "G723", 0, None),            // [RFC4856]
    (0, "audio", "G728", 0, None),            // [RFC4856]
    (0, "audio", "G729", 0, None),            // [RFC4856]
    (0, "audio", "GSM", 0, None),             // [RFC4856]
    (0, "audio", "L16", 0, None),             // [RFC4856]
    (0, "audio", "LPC", 0, None),             // [RFC4856]
    (0, "audio", "PCMA", 0, None),            // [RFC4856]
    (0, "audio", "PCMU", 0, None),            // [RFC4856]
    (0, "text", "parityfec", 0, None),        // [RFC3009]
    (0, "text", "red", 1000, None),           // [RFC4102]
    (0, "text", "rtx", 0, None),              // [RFC4588]
    (0, "text", "t140", 1000, None),          // [RFC4103]
    (0, "video", "BMPEG", 90000, None),       // [RFC2343][RFC3555]
    (0, "video", "BT656", 90000, None),       // [RFC2431][RFC3555]
    (0, "video", "DV", 90000, None),          // [RFC3189]
    (0, "video", "H263-1998", 90000, None),   // [RFC2429][RFC3555]
    (0, "video", "H263-2000", 90000, None),   // [RFC2429][RFC3555]
    (0, "video", "MP1S", 90000, None),        // [RFC2250][RFC3555]
    (0, "video", "MP2P", 90000, None),        // [RFC2250][RFC3555]
    (0, "video", "MP4V-ES", 90000, None),     // [RFC3016]
    (0, "video", "parityfec", 0, None),       // [RFC3009]
    (0, "video", "pointer", 90000, None),     // [RFC2862]
    (0, "video", "raw", 90000, None),         // [RFC4175]
    (0, "video", "rtx", 0, None),             // [RFC4588]
    (0, "video", "SMPTE292M", 0, None),       // [RFC3497]
    (0, "video", "vc1", 90000, None),         // [RFC4425]
    // not in http://www.iana.org/assignments/rtp-parameters
    (0, "audio", "AC3", 0, None),
    (0, "audio", "ILBC", 8000, None),
    (0, "audio", "MPEG4-GENERIC", 0, None),
    (0, "audio", "SPEEX", 0, None),
    (0, "audio", "OPUS", 48000, None),
    (0, "application", "MPEG4-GENERIC", 0, None),
    (0, "video", "H264", 90000, None),
    (0, "video", "H265", 90000, None),
    (0, "video", "MPEG4-GENERIC", 90000, None),
    (0, "video", "THEORA", 0, None),
    (0, "video", "VORBIS", 0, None),
    (0, "video", "X-SV3V-ES", 90000, None),
    (0, "video", "X-SORENSON-VIDEO", 90000, None),
    (0, "video", "VP8", 90000, None),
    (0, "video", "VP9", 90000, None),
);

// https://datatracker.ietf.org/doc/html/rfc2326#appendix-C.1.1
pub fn parse_control_path(path: &str, base: &Url) -> Option<Url> {
    match Url::parse(path) {
        Ok(v) => Some(v),
        Err(url::ParseError::RelativeUrlWithoutBase) => {
            if path == "*" {
                Some(base.clone())
            } else {
                base.join(path).ok()
            }
        }
        Err(_) => None,
    }
}

#[allow(clippy::result_large_err)]
fn parse_rtpmap(
    rtpmap: &str,
    pt: u8,
    media: &str,
    s: &mut gst::structure::Structure,
) -> Result<(), RtspError> {
    let Some((_pt, rtpmap)) = rtpmap.split_once(' ') else {
        return Err(RtspError::Fatal(format!(
            "Could not parse rtpmap: {rtpmap}"
        )));
    };

    let mut iter = rtpmap.split('/');
    let Some(encoding_name) = iter.next() else {
        return Err(RtspError::Fatal(format!(
            "Could not parse encoding-name from rtpmap: {rtpmap}"
        )));
    };
    let encoding_name = encoding_name.to_ascii_uppercase();
    s.set("encoding-name", &encoding_name);

    let Some(v) = iter.next() else {
        if pt >= 96 {
            return guess_rtpmap_from_pt(pt, media, s).map_err(|err| {
                RtspError::Fatal(format!(
                    "Could not get clock-rate from rtpmap {rtpmap}: {err}"
                ))
            });
        } else {
            return guess_rtpmap_from_encoding_name(&encoding_name, media, s).map_err(|err| {
                RtspError::Fatal(format!(
                    "Could not get clock-rate from rtpmap {rtpmap}: {err}"
                ))
            });
        }
    };

    let Ok(clock_rate) = v.parse::<i32>() else {
        return Err(RtspError::Fatal(format!(
            "Could not parse clock-rate from rtpmap: {rtpmap}"
        )));
    };
    s.set("clock-rate", clock_rate);

    if let Some(v) = iter.next() {
        s.set("encoding-params", v);
    }

    debug_assert!(iter.next().is_none());

    Ok(())
}

#[allow(clippy::result_large_err)]
fn guess_rtpmap_from_encoding_name(
    encoding_name: &str,
    media: &str,
    s: &mut gst::structure::Structure,
) -> Result<(), RtspError> {
    for info in STATIC_PAYLOAD_INFO
        .iter()
        .chain(DYNAMIC_PAYLOAD_INFO.iter())
    {
        if media == info.media && encoding_name == info.encoding_name {
            s.set("encoding-name", info.encoding_name);
            if info.clock_rate > 0 {
                s.set("clock-rate", info.clock_rate);
            }
            if let Some(v) = info.encoding_params {
                s.set("encoding-params", v);
            };
            return Ok(());
        }
    }
    Err(RtspError::Fatal(format!(
        "Cannot guess rtpmap: unknown encoding name {encoding_name}"
    )))
}

#[allow(clippy::result_large_err)]
fn guess_rtpmap_from_pt(
    pt: u8,
    media: &str,
    s: &mut gst::structure::Structure,
) -> Result<(), RtspError> {
    if pt >= 96 {
        return Err(RtspError::Fatal(format!(
            "Unknown dynamic payload type {pt}",
        )));
    }
    for info in STATIC_PAYLOAD_INFO {
        if pt == info.pt && media == info.media {
            s.set("encoding-name", info.encoding_name);
            if info.clock_rate > 0 {
                s.set("clock-rate", info.clock_rate);
            }
            if let Some(v) = info.encoding_params {
                s.set("encoding-params", v);
            };
            return Ok(());
        }
    }
    Err(RtspError::Fatal(format!(
        "Cannot guess rtpmap: unknown static payload type {pt}"
    )))
}

fn parse_fmtp(fmtp: &str, s: &mut gst::structure::Structure) {
    // Non-compliant RTSP servers will incorrectly set these here, ignore them
    let ignore_fields = [
        "media",
        "payload",
        "clock-rate",
        "encoding-name",
        "encoding-params",
    ];
    let encoding_name = s.get::<String>("encoding-name").unwrap();
    let Some((_pt, fmtp)) = fmtp.split_once(' ') else {
        gst::warning!(CAT, "Could not parse fmtp: {fmtp}");
        return;
    };
    let iter = fmtp.split(';').map_while(|x| x.split_once('='));
    for (k, v) in iter {
        let k = k.trim().to_ascii_lowercase();
        if ignore_fields.contains(&k.as_str()) {
            continue;
        }
        if encoding_name == "H264" && k == "profile-level-id" {
            let profile_idc = u8::from_str_radix(&v[0..2], 16);
            let csf_idc = u8::from_str_radix(&v[2..4], 16);
            let level_idc = u8::from_str_radix(&v[4..6], 16);
            if let (Ok(p), Ok(c), Ok(l)) = (profile_idc, csf_idc, level_idc) {
                let sps = &[p, c, l];
                let profile = gst_pbutils::codec_utils_h264_get_profile(sps);
                let level = gst_pbutils::codec_utils_h264_get_level(sps);
                if let (Ok(profile), Ok(level)) = (profile, level) {
                    s.set("profile", profile);
                    s.set("level", level);
                    continue;
                }
            }
            gst::warning!(CAT, "Failed to parse profile-level-id {v}, ignoring...");
            continue;
        }
        s.set(k, v);
    }

    // Adjust H264 caps for level asymmetry
    if encoding_name == "H264" {
        if s.get_optional("level-asymmetry-allowed") != Ok(Some("0")) && s.has_field("level") {
            s.remove_field("level");
        }
        if s.has_field("level-asymmetry-allowed") {
            s.remove_field("level-asymmetry-allowed");
        };
    }
}

fn parse_framesize(framesize: &str, s: &mut gst::structure::Structure) {
    let Some((_pt, dim)) = framesize.split_once(' ') else {
        gst::warning!(CAT, "Could not parse framesize {framesize}, ignoring");
        return;
    };

    s.set("a-framesize", dim);
}

#[allow(clippy::result_large_err)]
fn parse_extmap(extmap: &str, s: &mut gst::structure::Structure) -> Result<(), RtspError> {
    let mut parts = extmap.split_whitespace();

    // The format is: extmap-id direction extname [extensionattributes]
    // where extmap-id is the ID for this extension
    if let Some(extmap_id) = parts.next() {
        s.set("extmap-id", extmap_id);

        // Get the direction (may be present) and extension name
        if let Some(value) = parts.next() {
            // Check if the first part is a direction like sendonly, recvonly, etc
            if ["sendonly", "recvonly", "sendrecv", "inactive"].contains(&value) {
                s.set("extmap-direction", value);
                // If there's more after the direction, that should be the extension name
                if let Some(extname) = parts.next() {
                    s.set("extmap-extname", extname);
                }
            } else {
                // No direction, so the first part is the extension name
                s.set("extmap-extname", value);
            }
        }

        // Any remaining parts are extension attributes
        let remaining: Vec<&str> = parts.collect();
        if !remaining.is_empty() {
            s.set("extmap-attrs", remaining.join(" "));
        }
    }
    Ok(())
}

#[allow(clippy::result_large_err)]
fn parse_key_mgmt(key_mgmt: &str, s: &mut gst::structure::Structure) -> Result<(), RtspError> {
    // The format is typically: key-type key-mgmt-uri
    let parts: Vec<&str> = key_mgmt.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(());
    }

    // The first part is usually the key type (e.g., "uri:00")
    if let Some(key_type) = parts.first() {
        s.set("key-type", key_type);
    }

    // The remaining parts form the key management URI
    if parts.len() > 1 {
        let uri = parts[1..].join(" ");
        s.set("key-mgmt-uri", uri);
    }

    Ok(())
}

#[allow(clippy::result_large_err)]
fn parse_rid(rid: &str, s: &mut gst::structure::Structure) -> Result<(), RtspError> {
    // The format is: rid-id [direction] [pt=pt-list] [max-width=max-width] [max-height=max-height] [max-fps=max-fps] ...
    let mut params = rid.split_whitespace();

    if let Some(rid_id) = params.next() {
        s.set("rid-id", rid_id);

        // Parse the remaining parameters
        for param in params {
            if param.starts_with("pt=") {
                let pt_list = &param[3..]; // Remove "pt=" prefix
                s.set("rid-pt", pt_list);
            } else if param.starts_with("max-width=") {
                let width = &param[10..]; // Remove "max-width=" prefix
                s.set("max-width", width);
            } else if param.starts_with("max-height=") {
                let height = &param[11..]; // Remove "max-height=" prefix
                s.set("max-height", height);
            } else if param.starts_with("max-fps=") {
                let fps = &param[8..]; // Remove "max-fps=" prefix
                s.set("max-fps", fps);
            } else if param.starts_with("depend=") {
                let depend = &param[7..]; // Remove "depend=" prefix
                s.set("rid-depend", depend);
            } else {
                // Try to parse as direction if it's one of the standard values
                if ["send", "recv"].contains(&param) {
                    s.set("rid-direction", param);
                } else {
                    // Treat as additional parameter
                    s.set(format!("a-rid-param-{}", param), "");
                }
            }
        }
    }

    Ok(())
}

#[allow(clippy::result_large_err)]
fn parse_rtcp_fb(rtcp_fb: &str, s: &mut gst::structure::Structure) -> Result<(), RtspError> {
    // The format is: [payload type] feedback_type [feedback_parameter]
    let parts: Vec<&str> = rtcp_fb.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(());
    }

    let mut idx = 0;
    // Check if first part is a payload type (numeric)
    if let Ok(_) = parts[0].parse::<u8>() {
        s.set("rtcp-fb-pt", parts[0]);
        idx = 1;
    }

    if idx < parts.len() {
        s.set("rtcp-fb-type", parts[idx]);
        idx += 1;
    }

    if idx < parts.len() {
        let fb_params: Vec<&str> = parts[idx..].to_vec();
        s.set("rtcp-fb-params", fb_params.join(" "));
    }

    Ok(())
}

#[allow(clippy::result_large_err)]
fn parse_source_filter(source_filter: &str, s: &mut gst::structure::Structure) -> Result<(), RtspError> {
    // The format is: filter-mode nettype addrtype src-list dst-addr
    let parts: Vec<&str> = source_filter.split_whitespace().collect();

    if parts.len() >= 4 {
        s.set("filter-mode", parts[0]);
        s.set("nettype", parts[1]);
        s.set("addrtype", parts[2]);
        s.set("src-list", parts[3]);

        // Destination address might be present
        if parts.len() >= 5 {
            s.set("dst-addr", parts[4]);
        }
    }

    Ok(())
}

#[allow(clippy::result_large_err)]
fn parse_ssrc(ssrc: &str, s: &mut gst::structure::Structure) -> Result<(), RtspError> {
    // The format is: ssrc-id [attribute] or ssrc-id [cname: value | tool: value | ...]
    let parts: Vec<&str> = ssrc.split_whitespace().collect();

    if parts.is_empty() {
        return Ok(());
    }

    // Parse the SSRC ID
    if let Some(ssrc_id) = parts.first() {
        // Split by colon to separate the SSRC ID from attributes
        let ssrc_parts: Vec<&str> = ssrc_id.split(':').collect();
        if !ssrc_parts.is_empty() {
            s.set("ssrc-id", ssrc_parts[0]);

            // If ssrc attribute has format like "ssrc-id:attrname=value", parse that too
            if ssrc_parts.len() > 1 {
                s.set(format!("ssrc-{}", ssrc_parts[0]), ssrc_parts[1..].join(":"));
            }
        }
    }

    // Process any additional attributes
    if parts.len() > 1 {
        for attr in &parts[1..] {
            if let Some((attr_name, attr_value)) = attr.split_once('=') {
                s.set(format!("ssrc-{}", attr_name), attr_value);
            } else {
                // Attribute without value, set to empty string
                s.set(format!("ssrc-{}", attr), "");
            }
        }
    }

    Ok(())
}

#[allow(clippy::result_large_err)]
pub fn parse_media_attributes(
    attrs: &Vec<Attribute>,
    pt: u8,
    media: &str,
    s: &mut gst::structure::Structure,
) -> Result<(), RtspError> {
    let mut skip_attrs = vec!["control", "range"]; // Remove "ssrc" from hardcoded skip since we now parse it

    for Attribute { attribute, value } in attrs {
        let attr = attribute.as_str();
        if skip_attrs.contains(&attr) {
            continue;
        }

        let Some(value) = value else {
            continue;
        };

        if attr.starts_with("x-") {
            s.set(attr, value);
            continue;
        }

        match attr {
            "rtpmap" => parse_rtpmap(value, pt, media, s)?,
            "fmtp" => parse_fmtp(value, s),
            "framesize" => parse_framesize(value, s),
            "extmap" => parse_extmap(value, s)?,
            "key-mgmt" => parse_key_mgmt(value, s)?,
            "rid" => parse_rid(value, s)?,
            "rtcp-fb" => parse_rtcp_fb(value, s)?,
            "source-filter" => parse_source_filter(value, s)?,
            "ssrc" => parse_ssrc(value, s)?,
            _ => s.set(format!("a-{attribute}"), value),
        };
        skip_attrs.push(attr);
    }

    if !skip_attrs.contains(&"rtpmap") {
        guess_rtpmap_from_pt(pt, media, s)?;
    }

    Ok(())
}

pub fn parse_connections(conns: &Vec<Connection>) -> (BTreeSet<RtspProtocol>, bool) {
    let mut is_ipv4 = true;
    let mut conn_protocols = BTreeSet::new();
    for conn in conns {
        if conn.nettype != "IN" {
            continue;
        }
        // XXX: For now, assume that all connections use the same addrtype
        match conn.addrtype.as_str() {
            "IP4" => is_ipv4 = true,
            "IP6" => is_ipv4 = false,
            _ => continue,
        };
        // Strip subnet mask, if any
        let addr = if let Some((first, _)) = conn.connection_address.split_once('/') {
            first
        } else {
            conn.connection_address.as_str()
        };
        let Ok(addr) = addr.parse::<IpAddr>() else {
            continue;
        };
        // If this is an instance of gst-rtsp-server that only supports
        // udp-multicast, it will put the multicast address in the media
        // connections field.
        if addr.is_multicast() {
            conn_protocols.insert(RtspProtocol::UdpMulticast);
        } else {
            conn_protocols.insert(RtspProtocol::Tcp);
            conn_protocols.insert(RtspProtocol::Udp);
        }
    }
    (conn_protocols, is_ipv4)
}
