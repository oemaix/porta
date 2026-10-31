// Copyright 2026 oemaix
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! Values baked in from the gitignored repo-root `.env`.

#[cfg(target_os = "espidf")]
const ENV_TEXT: &str = include_str!("../../.env");

pub struct BenchEnv<'a> {
    pub client_id: &'a str,
    pub client_secret: &'a str,
    pub device_name: &'a str,
    pub wifi: [(&'a str, &'a str); 3],
}

#[cfg(target_os = "espidf")]
pub fn load() -> BenchEnv<'static> {
    parse(ENV_TEXT)
}

pub fn parse(text: &str) -> BenchEnv<'_> {
    let mut client_id = "";
    let mut client_secret = "";
    let mut device_name = "porta";
    let mut wifi = [("", ""); 3];
    let text = text.strip_prefix('\u{feff}').unwrap_or(text);
    for line in text.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let Some((key, value)) = line.split_once('=') else {
            continue;
        };
        let mut key = key.trim();
        if let Some(rest) = key.strip_prefix("export ") {
            key = rest.trim();
        }
        let value = unquote(value);
        match key {
            "SPOTIFY_CLIENT_ID" => client_id = value,
            "SPOTIFY_CLIENT_SECRET" => client_secret = value,
            "PORTA_DEVICE_NAME" => {
                if !value.is_empty() {
                    device_name = value;
                }
            }
            "WIFI_SSID" => wifi[0].0 = value,
            "WIFI_PASSPHRASE" => wifi[0].1 = value,
            "WIFI_SSID_2" => wifi[1].0 = value,
            "WIFI_PASSPHRASE_2" => wifi[1].1 = value,
            "WIFI_SSID_3" => wifi[2].0 = value,
            "WIFI_PASSPHRASE_3" => wifi[2].1 = value,
            _ => {}
        }
    }
    BenchEnv {
        client_id,
        client_secret,
        device_name,
        wifi,
    }
}

/// DNS label for the station and for mDNS. The Spotify name may contain
/// spaces. A label is lower case, letters, digits, and hyphens, at most 31
/// bytes. An empty result stays `porta`.
pub fn station_hostname(name: &str) -> HostLabel {
    let mut buf = [0u8; 32];
    let mut len = 0usize;
    let mut hyphen = true;
    for byte in name.bytes() {
        let mapped = match byte {
            b'A'..=b'Z' => byte + 32,
            b'a'..=b'z' | b'0'..=b'9' => byte,
            _ => b'-',
        };
        if mapped == b'-' {
            if hyphen {
                continue;
            }
            hyphen = true;
        } else {
            hyphen = false;
        }
        if len == 31 {
            break;
        }
        buf[len] = mapped;
        len += 1;
    }
    while len > 0 && buf[len - 1] == b'-' {
        len -= 1;
    }
    if len == 0 {
        buf[..5].copy_from_slice(b"porta");
        len = 5;
    }
    HostLabel {
        buf,
        len: len as u8,
    }
}

pub struct HostLabel {
    buf: [u8; 32],
    len: u8,
}

impl HostLabel {
    pub fn as_str(&self) -> &str {
        core::str::from_utf8(&self.buf[..self.len as usize]).unwrap()
    }
}

fn unquote(value: &str) -> &str {
    let value = value.trim();
    let bytes = value.as_bytes();
    if bytes.len() >= 2
        && ((bytes[0] == b'"' && bytes[bytes.len() - 1] == b'"')
            || (bytes[0] == b'\'' && bytes[bytes.len() - 1] == b'\''))
    {
        return &value[1..value.len() - 1];
    }
    value
}

#[cfg(test)]
mod tests {
    use super::{parse, station_hostname};

    #[test]
    fn quotes_export_and_bom() {
        let text = "\u{feff}export WIFI_SSID=\"net one\"\nWIFI_PASSPHRASE='secret'\n# skip\nPORTA_DEVICE_NAME=desk\nWIFI_SSID_2=two\n";
        let env = parse(text);
        assert_eq!(env.device_name, "desk");
        assert_eq!(env.wifi[0].0, "net one");
        assert_eq!(env.wifi[0].1, "secret");
        assert_eq!(env.wifi[1].0, "two");
        assert_eq!(env.wifi[1].1, "");
        assert_eq!(parse("PORTA_DEVICE_NAME=\n").device_name, "porta");
    }

    #[test]
    fn hostname_is_a_dns_label() {
        assert_eq!(station_hostname("porta").as_str(), "porta");
        assert_eq!(station_hostname("Living Room").as_str(), "living-room");
        assert_eq!(station_hostname("A").as_str(), "a");
        assert_eq!(station_hostname("").as_str(), "porta");
        assert_eq!(station_hostname("---").as_str(), "porta");
        assert_eq!(station_hostname("Room  1").as_str(), "room-1");
        assert!(station_hostname(&"x".repeat(40)).as_str().len() <= 31);
    }
}
