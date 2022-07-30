// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use super::*;
use getset::{CopyGetters, Getters};
#[cfg(feature = "serialize-json")]
use serde::{Deserialize, Serialize};

// This message can have an Id of 0, as it can be emitted as part of a
// subscription and won't have a matching task Id in that case.
#[derive(Debug, ButtplugDeviceMessage, ButtplugMessageValidator, Clone, Getters, CopyGetters)]
#[cfg_attr(feature = "serialize-json", derive(Serialize, Deserialize))]
pub struct SensorReading {
  #[cfg_attr(feature = "serialize-json", serde(rename = "Id"))]
  id: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "DeviceIndex"))]
  device_index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "FeatureIndex"))]
  #[getset[get_copy="pub"]]
  feature_index: u32,
  #[cfg_attr(feature = "serialize-json", serde(rename = "SensorType"))]
  #[getset[get_copy="pub"]]
  sensor_type: SensorType,
  #[cfg_attr(feature = "serialize-json", serde(rename = "Data"))]
  #[getset[get="pub"]]
  data: Vec<f64>,
}

impl SensorReading {
  pub fn new(
    device_index: u32,
    feature_index: u32,
    sensor_type: SensorType,
    data: Vec<f64>,
  ) -> Self {
    Self {
      id: 1,
      device_index,
      feature_index,
      sensor_type,
      data,
    }
  }
}
