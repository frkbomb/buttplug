// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.


//! Device identification and configuration, and protocol definitions
//!
//! The device module is used by the Buttplug Server to identify devices that buttplug can connect
//! to, and match them to supported protocols in order to establish communication. 

pub mod configuration_manager;
pub mod protocol;
use serde::{
  de::{self, Visitor},
  Deserialize,
  Deserializer,
  Serialize,
  Serializer,
};
use std::{
  fmt::{self, Debug},
  str::FromStr,
  string::ToString,
  sync::Arc,
};

use crate::{
  core::{
    errors::ButtplugError,
    messages::{
      self,
      ButtplugDeviceCommandMessageUnion,
      ButtplugServerMessage,
      DeviceMessageAttributesMap,
      RawReadCmd,
      RawReading,
      RawSubscribeCmd,
      RawUnsubscribeCmd,
      RawWriteCmd,
    },
    ButtplugResultFuture,
  },
  device::{
    configuration_manager::{DeviceConfigurationManager, ProtocolCommunicationSpecifier, ProtocolDeviceConfiguration, ProtocolAttributesIdentifier},
    protocol::ButtplugProtocol,
  },
};
use async_trait::async_trait;
use core::hash::{Hash, Hasher};
use futures::future::BoxFuture;
use tokio::sync::broadcast;


// We need this array to be exposed in our WASM FFI, but the only way to do that
// is to expose it at the declaration level. Therefore, we use the WASM feature
// to assume we're building for WASM and attach our bindgen. The serde
// de/serialization is taken care of at the FFI level.
#[derive(EnumString, Clone, Debug, PartialEq, Eq, Hash, Display, Copy)]
#[strum(serialize_all = "lowercase")]
pub enum Endpoint {
  Command,
  Firmware,
  Rx,
  RxAccel,
  RxBLEBattery,
  RxBLEModel,
  RxPressure,
  RxTouch,
  Tx,
  TxMode,
  TxShock,
  TxVibrate,
  TxVendorControl,
  Whitelist,
  Generic0,
  Generic1,
  Generic2,
  Generic3,
  Generic4,
  Generic5,
  Generic6,
  Generic7,
  Generic8,
  Generic9,
  Generic10,
  Generic11,
  Generic12,
  Generic13,
  Generic14,
  Generic15,
  Generic16,
  Generic17,
  Generic18,
  Generic19,
  Generic20,
  Generic21,
  Generic22,
  Generic23,
  Generic24,
  Generic25,
  Generic26,
  Generic27,
  Generic28,
  Generic29,
  Generic30,
  Generic31,
}

impl Serialize for Endpoint {
  fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
  where
    S: Serializer,
  {
    serializer.serialize_str(&self.to_string())
  }
}

struct EndpointVisitor;

impl<'de> Visitor<'de> for EndpointVisitor {
  type Value = Endpoint;

  fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
    formatter.write_str("a string representing an endpoint")
  }

  fn visit_str<E>(self, value: &str) -> Result<Self::Value, E>
  where
    E: de::Error,
  {
    Endpoint::from_str(value).map_err(|e| E::custom(format!("{}", e)))
  }
}

impl<'de> Deserialize<'de> for Endpoint {
  fn deserialize<D>(deserializer: D) -> Result<Endpoint, D::Error>
  where
    D: Deserializer<'de>,
  {
    deserializer.deserialize_str(EndpointVisitor)
  }
}

pub type ButtplugDeviceResultFuture =
  BoxFuture<'static, Result<ButtplugServerMessage, ButtplugError>>;

#[derive(PartialEq, Debug)]
pub struct DeviceReadCmd {
  pub endpoint: Endpoint,
  pub length: u32,
  pub timeout_ms: u32,
}

impl DeviceReadCmd {
  pub fn new(endpoint: Endpoint, length: u32, timeout_ms: u32) -> Self {
    Self {
      endpoint,
      length,
      timeout_ms,
    }
  }
}

impl From<RawReadCmd> for DeviceReadCmd {
  fn from(msg: RawReadCmd) -> Self {
    Self {
      endpoint: msg.endpoint(),
      length: msg.expected_length(),
      timeout_ms: msg.timeout(),
    }
  }
}

#[derive(PartialEq, Debug)]
pub struct DeviceWriteCmd {
  pub endpoint: Endpoint,
  pub data: Vec<u8>,
  pub write_with_response: bool,
}

impl DeviceWriteCmd {
  pub fn new(endpoint: Endpoint, data: Vec<u8>, write_with_response: bool) -> Self {
    Self {
      endpoint,
      data,
      write_with_response,
    }
  }
}

impl From<RawWriteCmd> for DeviceWriteCmd {
  fn from(msg: RawWriteCmd) -> Self {
    Self {
      endpoint: msg.endpoint(),
      data: msg.data().clone(),
      write_with_response: msg.write_with_response(),
    }
  }
}

#[derive(PartialEq, Debug)]
pub struct DeviceSubscribeCmd {
  pub endpoint: Endpoint,
}

impl DeviceSubscribeCmd {
  pub fn new(endpoint: Endpoint) -> Self {
    Self { endpoint }
  }
}

impl From<RawSubscribeCmd> for DeviceSubscribeCmd {
  fn from(msg: RawSubscribeCmd) -> Self {
    Self {
      endpoint: msg.endpoint(),
    }
  }
}

#[derive(PartialEq, Debug)]
pub struct DeviceUnsubscribeCmd {
  pub endpoint: Endpoint,
}

impl DeviceUnsubscribeCmd {
  pub fn new(endpoint: Endpoint) -> Self {
    Self { endpoint }
  }
}

impl From<RawUnsubscribeCmd> for DeviceUnsubscribeCmd {
  fn from(msg: RawUnsubscribeCmd) -> Self {
    Self {
      endpoint: msg.endpoint(),
    }
  }
}

#[derive(PartialEq, Debug)]
pub enum DeviceImplCommand {
  // Endpoint, data, write with response
  Write(DeviceWriteCmd),
  // Endpoint, length, timeout in ms
  Read(DeviceReadCmd),
  Subscribe(DeviceSubscribeCmd),
  Unsubscribe(DeviceUnsubscribeCmd),
}

impl From<RawWriteCmd> for DeviceImplCommand {
  fn from(msg: RawWriteCmd) -> Self {
    DeviceImplCommand::Write(msg.into())
  }
}

impl From<RawSubscribeCmd> for DeviceImplCommand {
  fn from(msg: RawSubscribeCmd) -> Self {
    DeviceImplCommand::Subscribe(msg.into())
  }
}

impl From<RawUnsubscribeCmd> for DeviceImplCommand {
  fn from(msg: RawUnsubscribeCmd) -> Self {
    DeviceImplCommand::Unsubscribe(msg.into())
  }
}

impl From<DeviceReadCmd> for DeviceImplCommand {
  fn from(msg: DeviceReadCmd) -> Self {
    DeviceImplCommand::Read(msg)
  }
}

impl From<DeviceWriteCmd> for DeviceImplCommand {
  fn from(msg: DeviceWriteCmd) -> Self {
    DeviceImplCommand::Write(msg)
  }
}

impl From<DeviceSubscribeCmd> for DeviceImplCommand {
  fn from(msg: DeviceSubscribeCmd) -> Self {
    DeviceImplCommand::Subscribe(msg)
  }
}

impl From<DeviceUnsubscribeCmd> for DeviceImplCommand {
  fn from(msg: DeviceUnsubscribeCmd) -> Self {
    DeviceImplCommand::Unsubscribe(msg)
  }
}

#[derive(Debug)]
pub struct ButtplugDeviceImplInfo {
  pub endpoints: Vec<Endpoint>,
  pub manufacturer_name: Option<String>,
  pub product_name: Option<String>,
  pub serial_number: Option<String>,
}

#[derive(Debug)]
pub enum ButtplugDeviceCommand {
  Connect,
  Message(DeviceImplCommand),
  Disconnect,
}

// TODO Split this down into connections and other returns.
#[derive(Debug)]
pub enum ButtplugDeviceReturn {
  Connected(ButtplugDeviceImplInfo),
  Ok(messages::Ok),
  RawReading(messages::RawReading),
  Error(ButtplugError),
}

#[derive(Debug, Clone)]
pub enum ButtplugDeviceEvent {
  Connected(Arc<ButtplugDevice>),
  Notification(String, Endpoint, Vec<u8>),
  Removed(String),
}
pub struct DeviceImpl {
  name: String,
  address: String,
  endpoints: Vec<Endpoint>,
  internal_impl: Box<dyn DeviceImplInternal>,
}

impl DeviceImpl {
  pub fn new(
    name: &str,
    address: &str,
    endpoints: &[Endpoint],
    internal_impl: Box<dyn DeviceImplInternal>,
  ) -> Self {
    Self {
      name: name.to_owned(),
      address: address.to_owned(),
      endpoints: endpoints.into(),
      internal_impl,
    }
  }

  pub fn name(&self) -> &str {
    &self.name
  }

  pub fn address(&self) -> &str {
    &self.address
  }

  pub fn connected(&self) -> bool {
    self.internal_impl.connected()
  }

  pub fn event_stream(&self) -> broadcast::Receiver<ButtplugDeviceEvent> {
    self.internal_impl.event_stream()
  }

  pub fn endpoints(&self) -> Vec<Endpoint> {
    self.endpoints.clone()
  }

  pub fn disconnect(&self) -> ButtplugResultFuture {
    self.internal_impl.disconnect()
  }

  pub fn read_value(
    &self,
    msg: DeviceReadCmd,
  ) -> BoxFuture<'static, Result<RawReading, ButtplugError>> {
    self.internal_impl.read_value(msg)
  }

  pub fn write_value(&self, msg: DeviceWriteCmd) -> ButtplugResultFuture {
    self.internal_impl.write_value(msg)
  }

  pub fn subscribe(&self, msg: DeviceSubscribeCmd) -> ButtplugResultFuture {
    self.internal_impl.subscribe(msg)
  }

  pub fn unsubscribe(&self, msg: DeviceUnsubscribeCmd) -> ButtplugResultFuture {
    self.internal_impl.unsubscribe(msg)
  }
}

pub trait DeviceImplInternal: Sync + Send {
  fn connected(&self) -> bool;
  fn disconnect(&self) -> ButtplugResultFuture;
  // Ugh. Don't want to have to pass these around internally, but don't have a
  // better solution yet.
  fn event_stream(&self) -> broadcast::Receiver<ButtplugDeviceEvent>;
  fn read_value(&self, msg: DeviceReadCmd)
    -> BoxFuture<'static, Result<RawReading, ButtplugError>>;
  fn write_value(&self, msg: DeviceWriteCmd) -> ButtplugResultFuture;
  fn subscribe(&self, msg: DeviceSubscribeCmd) -> ButtplugResultFuture;
  fn unsubscribe(&self, msg: DeviceUnsubscribeCmd) -> ButtplugResultFuture;
}

#[async_trait]
pub trait ButtplugDeviceImplCreator: Sync + Send + Debug {
  fn specifier(&self) -> ProtocolCommunicationSpecifier;
  async fn try_create_device_impl(
    &mut self,
    protocol: ProtocolDeviceConfiguration,
  ) -> Result<DeviceImpl, ButtplugError>;
}

pub struct ButtplugDevice {
  protocol: Box<dyn ButtplugProtocol>,
  device: Arc<DeviceImpl>,
  display_name: Option<String>,
  device_identifier: String
}

impl Debug for ButtplugDevice {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    f.debug_struct("ButtplugDevice")
      .field("name", &self.name())
      .field("identifier", &self.device_identifier())
      .finish()
  }
}

impl Hash for ButtplugDevice {
  fn hash<H: Hasher>(&self, state: &mut H) {
    self.device_identifier().hash(state);
  }
}

impl Eq for ButtplugDevice {
}

impl PartialEq for ButtplugDevice {
  fn eq(&self, other: &Self) -> bool {
    self.device_identifier() == other.device_identifier()
  }
}

pub fn form_device_identifier(protocol_identifier: &str, protocol_attributes_identifier: &ProtocolAttributesIdentifier, device_address: &str ) -> String {
  format!("{}|{:?}|{}", protocol_identifier, protocol_attributes_identifier, device_address)
}

impl ButtplugDevice {
  pub fn new(protocol: Box<dyn ButtplugProtocol>, device: Arc<DeviceImpl>) -> Self {
    Self {
      device_identifier: form_device_identifier(protocol.protocol_identifier(), protocol.protocol_attributes_identifier(), device.address()),
      protocol,
      device,
      display_name: None,
    }
  }

  pub fn device_identifier(&self) -> &str {
    &self.device_identifier
  }

  pub fn device_impl_address(&self) -> &str {
    self.device.address()
  }

  pub fn protocol_identifier(&self) -> &str {
    self.protocol.protocol_identifier()
  }

  pub fn protocol_attributes_identifier(&self) -> &ProtocolAttributesIdentifier {
    self.protocol.protocol_attributes_identifier()
  }

  pub async fn try_create_device(
    device_config_mgr: Arc<DeviceConfigurationManager>,
    mut device_creator: Box<dyn ButtplugDeviceImplCreator>,
  ) -> Result<Option<ButtplugDevice>, ButtplugError> {
    // First off, we need to see if we even have a configuration available
    // for the device we're trying to create. If we don't, return Ok(None),
    // because this isn't actually an error. However, if we *do* have a
    // configuration but something goes wrong after this, then it's an
    // error.
    let protocol_builder = match device_config_mgr.protocol_builder(&device_creator.specifier()) {
      Some(builder) => builder,
      None => return Ok(None)
    };
      

    // Now that we have both a possible device implementation and a
    // configuration for that device, try to initialize the implementation.
    // This usually means trying to connect to whatever the device is,
    // finding endpoints, etc.
    let device_impl = device_creator.try_create_device_impl(protocol_builder.configuration().clone()).await?;
    info!(
      address = tracing::field::display(device_impl.address()),
      "Found Buttplug Device {}",
      device_impl.name()
    );

    // If we've made it this far, we now have a connected device
    // implementation with endpoints set up. We now need to run whatever
    // protocol initialization might need to happen. We'll fetch a protocol
    // creator, pass the device implementation to it, then let it do
    // whatever it needs. For most protocols, this is a no-op. However, for
    // devices like Lovense, some Kiiroo, etc, this can get fairly
    // complicated.
    let sharable_device_impl = Arc::new(device_impl);
    let protocol_impl =
      protocol_builder.create(sharable_device_impl.clone()).await?;
    Ok(Some(ButtplugDevice::new(
      protocol_impl,
      sharable_device_impl,
    )))
  }

  pub fn set_display_name(&mut self, name: &str) {
    info!(
      "Adding display name {} to device {} ({})",
      name,
      self.name(),
      self.device_identifier()
    );
    self.display_name = Some(name.to_owned());
  }

  pub fn display_name(&self) -> Option<String> {
    self.display_name.clone()
  }

  pub fn name(&self) -> String {
    // Instead of checking for raw messages at the protocol level, add the raw
    // call here, since this is the only way to access devices in the library
    // anyways.
    //
    // Having raw turned on means it'll work for read/write/sub/unsub on any
    // endpoint so just use an arbitrary message here to check.
    if self
      .protocol
      .supports_message(&ButtplugDeviceCommandMessageUnion::RawSubscribeCmd(
        RawSubscribeCmd::new(1, Endpoint::Tx),
      ))
      .is_ok()
    {
      format!("{} (Raw Messages Allowed)", self.protocol.name())
    } else {
      self.protocol.name().to_owned()
    }
  }

  pub fn disconnect(&self) -> ButtplugResultFuture {
    self.device.disconnect()
  }

  pub fn message_attributes(&self) -> DeviceMessageAttributesMap {
    self.protocol.device_attributes().message_attributes_map()
  }

  pub fn parse_message(
    &self,
    message: ButtplugDeviceCommandMessageUnion,
  ) -> ButtplugDeviceResultFuture {
    self.protocol.handle_command(self.device.clone(), message)
  }

  pub fn event_stream(&self) -> broadcast::Receiver<ButtplugDeviceEvent> {
    self.device.event_stream()
  }

  // TODO Handle raw messages here.
}
