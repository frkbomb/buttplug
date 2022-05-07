// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use super::*;
use crate::device::Endpoint;
#[cfg(feature = "serialize-json")]
use serde::{Deserialize, Serialize};

#[derive(Debug, ButtplugDeviceMessage, PartialEq, Clone)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct RawReadCmd {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Id"))]
  id: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceIndex"))]
  device_index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "Endpoint"))]
  endpoint: Endpoint,
  #[cfg_attr(feature = "serialize-json", serde(rename = "ExpectedLength"))]
  expected_length: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "Timeout"))]
  timeout: u32,
}

impl RawReadCmd {
  pub fn new(device_index: u32, endpoint: Endpoint, expected_length: u32, timeout: u32) -> Self {
    Self {
      id: 1,
      device_index,
      endpoint,
      expected_length,
      timeout,
    }
  }

  pub fn endpoint(&self) -> Endpoint {
    self.endpoint
  }

  pub fn expected_length(&self) -> u32 {
    self.expected_length
  }

  pub fn timeout(&self) -> u32 {
    self.timeout
  }
}

impl ButtplugMessageValidator for RawReadCmd {
  fn is_valid(&self) -> Result<(), ButtplugMessageError> {
    self.is_not_system_id(self.id)
    // TODO Should expected_length always be > 0?
  }
}
