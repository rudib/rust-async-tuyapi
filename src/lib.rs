//! # Rust Tuyapi
//! This library can be used to interact with Tuya/Smart Home devices. It utilizes the Tuya
//! protocol version 3.1 and 3.3 to send and receive messages from the devices.
//!
//! ## Example
//! This shows how to turn on a wall socket.
//! ```no_run
//! # extern crate rust_async_tuyapi;
//! # use rust_async_tuyapi::{Payload, Result, PayloadStruct,tuyadevice::TuyaDevice};
//! # use std::net::IpAddr;
//! # use std::str::FromStr;
//! # use std::collections::HashMap;
//! # use std::time::SystemTime;
//! # use serde_json::json;
//! # async fn set_device() -> Result<()> {
//! // The dps value is device specific, this socket turns on with key "1"
//! let mut dps = HashMap::new();
//! dps.insert("1".to_string(), json!(true));
//! let current_time = SystemTime::now()
//!     .duration_since(SystemTime::UNIX_EPOCH).unwrap().as_secs() as u32;
//!
//! // Create the payload to be sent, this will be serialized to the JSON format
//! let payload = Payload::Struct(PayloadStruct{
//!        dev_id: "123456789abcdef".to_string(),
//!        gw_id: Some("123456789abcdef".to_string()),
//!        uid: None,
//!        t: Some(current_time),
//!        dp_id: None,
//!        dps: Some(dps),
//!        });
//! // Create a TuyaDevice, this is the type used to set/get status to/from a Tuya compatible
//! // device.
//! let tuya_device = TuyaDevice::create("ver3.3", Some("fedcba987654321"),
//!     IpAddr::from_str("192.168.0.123").unwrap())?;
//!
//! // Set the payload state on the Tuya device, an error here will contain
//! // the error message received from the device.
//! tuya_device.set(payload, 0).await?;
//! # Ok(())
//! # }
//! ```
mod cipher;
mod crc;
pub mod error;
pub mod mesparse;
pub mod tuyadevice;

extern crate num;
extern crate num_derive;
#[macro_use]
extern crate lazy_static;

use serde::{Deserialize, Serialize};

use std::collections::HashMap;
use std::convert::TryFrom;
use std::fmt::Display;

use crate::error::ErrorKind;
use std::convert::TryInto;

pub type Result<T> = std::result::Result<T, ErrorKind>;
/// The Payload enum represents a payload sent to, and recevied from the Tuya devices. It might be
/// a struct (ser/de from json) or a plain string.
#[derive(Debug, Clone, PartialEq)]
pub enum Payload {
    Struct(PayloadStruct),
    String(String),
}

#[derive(Debug, Clone, PartialEq)]
pub enum DpId {
    Lower,
    Higher,
}

impl DpId {
    fn get_ids(self) -> Vec<u8> {
        match self {
            DpId::Lower => vec![4, 5, 6],
            DpId::Higher => vec![18, 19, 20],
        }
    }
}

impl Payload {
    pub fn new(
        dev_id: String,
        gw_id: Option<String>,
        uid: Option<String>,
        t: Option<u32>,
        dp_id: Option<DpId>,
        dps: Option<HashMap<String, serde_json::Value>>,
    ) -> Payload {
        Payload::Struct(PayloadStruct {
            dev_id,
            gw_id,
            uid,
            t,
            dp_id: dp_id.map(DpId::get_ids),
            dps,
        })
    }
}

impl Display for Payload {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Payload::Struct(s) => write!(f, "{}", s),
            Payload::String(s) => write!(f, "{}", s),
        }
    }
}

/// The PayloadStruct is Serialized to json and sent to the device. The dps field contains the
/// actual commands to set and are device specific.
#[derive(Serialize, Deserialize, Debug, PartialEq, Clone)]
pub struct PayloadStruct {
    #[serde(rename = "devId")]
    pub dev_id: String,
    #[serde(rename = "gwId", skip_serializing_if = "Option::is_none")]
    pub gw_id: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub uid: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub t: Option<u32>,
    #[serde(rename = "dpId", skip_serializing_if = "Option::is_none")]
    pub dp_id: Option<Vec<u8>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub dps: Option<HashMap<String, serde_json::Value>>,
}

/// This trait is implemented to allow truncated logging of secret data.
pub trait Truncate {
    fn truncate(&self) -> Self;

    /// Take the last 5 characters
    fn truncate_str(text: &str) -> &str {
        if let Some((i, _)) = text.char_indices().rev().nth(5) {
            return &text[i..];
        }
        text
    }
}

impl TryFrom<Vec<u8>> for Payload {
    type Error = ErrorKind;

    fn try_from(vec: Vec<u8>) -> Result<Self> {
        match serde_json::from_slice(&vec)? {
            serde_json::Value::String(s) => Ok(Payload::String(s)),
            value => Ok(Payload::Struct(serde_json::from_value(value)?)),
        }
    }
}
impl TryInto<Vec<u8>> for Payload {
    type Error = ErrorKind;

    fn try_into(self) -> Result<Vec<u8>> {
        match self {
            Payload::Struct(s) => Ok(serde_json::to_vec(&s)?),
            Payload::String(s) => Ok(s.as_bytes().to_vec()),
        }
    }
}

impl Truncate for PayloadStruct {
    fn truncate(&self) -> PayloadStruct {
        PayloadStruct {
            dev_id: String::from("...") + Self::truncate_str(&self.dev_id),
            gw_id: self
                .gw_id
                .as_ref()
                .map(|gwid| String::from("...") + Self::truncate_str(gwid)),
            t: self.t,
            dp_id: self.dp_id.clone(),
            uid: self.uid.clone(),
            dps: self.dps.clone(),
        }
    }
}

impl Display for PayloadStruct {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let full_display = std::env::var("TUYA_FULL_DISPLAY").map_or_else(|_| false, |_| true);
        if full_display {
            write!(f, "{}", serde_json::to_string(self).unwrap())
        } else {
            write!(f, "{}", serde_json::to_string(&self.truncate()).unwrap())
        }
    }
}
