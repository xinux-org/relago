use core::{fmt, str::FromStr};

/// A 128-bit journal `MESSAGE_ID`.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MessageId(u128);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ParseError {
    Length(usize),
    Digit(usize),
}

impl fmt::Display for ParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Length(n) => writeln!(f, "MESSAGE_ID must be 32 hex digits, got {n}"),
            Self::Digit(i) => writeln!(f, "MESSAGE_ID must be lowercase hex, bad character at {i}"),
        }
    }
}

impl core::error::Error for ParseError {}

impl MessageId {
    pub const fn parse(s: &str) -> Result<Self, ParseError> {
        let mut bytes = s.as_bytes();
        if bytes.len() != 32 {
            return Err(ParseError::Length(bytes.len()));
        }
        let mut value: u128 = 0;
        let mut i = 0;
        while let [byte, rest @ ..] = bytes {
            let digit = match *byte {
                b @ b'0'..=b'9' => b - b'0',
                b @ b'a'..=b'f' => b - b'a' + 10,
                _ => return Err(ParseError::Digit(i)),
            };
            value = (value << 4) | digit as u128;
            bytes = rest;
            i += 1;
        }
        Ok(Self(value))
    }

    #[must_use]
    pub const fn from_hex(s: &str) -> Self {
        assert!(Self::parse(s).is_ok(), "invalid MESSAGE_ID literal");
        match Self::parse(s) {
            Ok(id) => id,
            Err(_) => Self(0), // unreachable:
        }
    }

    // The journal match string e.g `MESSAGE_ID=fc2e22bc6ee647b6b90729ab34a250b1`.
    #[must_use]
    pub fn journal_match(&self) -> String {
        format!("MESSAGE_ID={:032x}", self.0)
    }
}

impl FromStr for MessageId {
    type Err = ParseError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::parse(s)
    }
}
macro_rules! message_ids {
    ($($name:ident = $hex:literal,)*) => {
        $(pub const $name: MessageId = MessageId::from_hex($hex);)*
        pub const ALL: &[(&str, MessageId)] = &[$((stringify!($name), $name),)*];
    };
}

message_ids! {
    SD_MESSAGE_COREDUMP = "fc2e22bc6ee647b6b90729ab34a250b1",
    SD_MESSAGE_UNIT_OUT_OF_MEMORY = "fe6faa94e7774663a0da52717891d8ef",
    SD_MESSAGE_UNIT_FAILURE_RESULT = "d9b373ed55a64feb8242e02dbe79a49c",
    SD_MESSAGE_UNIT_FAILED = "be02cf6855d2428ba40df7e9d022f03d",
    SD_MESSAGE_UNIT_STARTED = "39f53479d3a045ac8e11786248231fbf",
    SD_MESSAGE_UNIT_RESTART_SCHEDULED = "5eb03494b6584870a536b337290809b3",
    SD_MESSAGE_UNIT_PROCESS_EXIT = "98e322203f7a4ed290d09fe03c09fe15",
    SD_MESSAGE_UNIT_OOMD_KILL = "d989611b15e44c9dbf31e3c81256e4ed",
    SD_MESSAGE_SESSION_START = "8d45620c1a4348dbb17410da57c60c66",
    SD_MESSAGE_SESSION_STOP = "3354939424b4456d9802ca8333ed424a",
    SD_MESSAGE_SLEEP_START = "6bbd95ee977941e497c48be27c254128",
    SD_MESSAGE_SLEEP_STOP = "8811e6df2a8e40f58a94cea26f8ebf14",
    SD_MESSAGE_SHUTDOWN = "98268866d1d54a499c4e98921d93bc40",
}

#[test]
fn runtime_parse_reports_errors() {
    assert_eq!(
        "fc2e22bc6ee647b6b90729ab34a250b1".parse::<MessageId>(),
        Ok(SD_MESSAGE_COREDUMP)
    );
    assert_eq!(
        MessageId::parse("fc2e22bc6ee647b6b90729ab34a250b"),
        Err(ParseError::Length(31))
    );
    assert_eq!(
        MessageId::parse("fc2e22bc6ee647b6b90729ab34a250g1"),
        Err(ParseError::Digit(30))
    );
    assert_eq!(
        MessageId::parse("FC2E22BC6EE647B6B90729AB34A250B1"),
        Err(ParseError::Digit(0))
    );
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashMap;

    fn header_ids(header: &str) -> Result<HashMap<&str, MessageId>, ParseError> {
        header
            .lines()
            .filter_map(|line| {
                let mut words = line.split_whitespace();
                let (Some("#define"), Some(name), Some(value)) =
                    (words.next(), words.next(), words.next())
                else {
                    return None;
                };
                let bytes = value.strip_prefix("SD_ID128_MAKE(")?.strip_suffix(')')?;
                Some(MessageId::parse(&bytes.replace(',', "")).map(|id| (name, id)))
            })
            .collect()
    }

    #[test]
    fn ids_match_systemd_header() -> Result<(), Box<dyn std::error::Error>> {
        if let Some(path) = std::env::var_os("SD_MESSAGES_H") {
            let header = std::fs::read_to_string(path)?;
            let systemd = header_ids(&header)?;
            for (name, id) in ALL {
                assert_eq!(
                    systemd.get(name),
                    Some(id),
                    "{name} differs from systemd's sd-messages.h"
                );
            }
        } else {
            assert!(
                std::env::var_os("RELAGO_REQUIRE_SD_MESSAGES").is_none(),
                "SD_MESSAGES_H must be set when RELAGO_REQUIRE_SD_MESSAGES is set"
            );
        }
        Ok(())
    }
}
