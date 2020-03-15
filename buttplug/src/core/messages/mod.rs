// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2020 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

//! Structs representing low level [Buttplug
//! Protocol](https://buttplug-spec.docs.buttplug.io) messages

mod device_added;
mod device_removed;
mod device_list;
mod device_message_info;
mod error;
mod fleshlight_launch_fw12_cmd;
mod kiiroo_cmd;
mod linear_cmd;
mod log;
mod log_level;
mod lovense_cmd;
mod message_attributes;
mod ok;
mod ping;
mod raw_read_cmd;
mod raw_reading;
mod raw_write_cmd;
mod request_device_list;
mod request_log;
mod request_server_info;
mod rotate_cmd;
mod scanning_finished;
mod server_info;
mod single_motor_vibrate_cmd;
mod start_scanning;
mod stop_scanning;
mod stop_all_devices;
mod stop_device_cmd;
mod subscribe_cmd;
mod test;
mod unsubscribe_cmd;
mod vibrate_cmd;
mod vorze_a10_cyclone_cmd;

pub use device_added::DeviceAdded;
pub use device_removed::DeviceRemoved;
pub use device_list::DeviceList;
pub use device_message_info::{DeviceMessageInfo, MessageAttributesMap};
pub use error::{Error, ErrorCode};
pub use fleshlight_launch_fw12_cmd::FleshlightLaunchFW12Cmd;
pub use kiiroo_cmd::KiirooCmd;
pub use linear_cmd::{LinearCmd, VectorSubcommand};
pub use self::log::Log;
pub use log_level::LogLevel;
pub use lovense_cmd::LovenseCmd;
pub use message_attributes::MessageAttributes;
pub use ok::Ok;
pub use ping::Ping;
pub use raw_read_cmd::RawReadCmd;
pub use raw_write_cmd::RawWriteCmd;
pub use raw_reading::RawReading;
pub use request_device_list::RequestDeviceList;
pub use request_log::RequestLog;
pub use request_server_info::RequestServerInfo;
pub use rotate_cmd::{RotateCmd, RotationSubcommand};
pub use scanning_finished::ScanningFinished;
pub use server_info::ServerInfo;
pub use single_motor_vibrate_cmd::SingleMotorVibrateCmd;
pub use start_scanning::StartScanning;
pub use stop_all_devices::StopAllDevices;
pub use stop_device_cmd::StopDeviceCmd;
pub use stop_scanning::StopScanning;
pub use subscribe_cmd::SubscribeCmd;
pub use test::Test;
pub use unsubscribe_cmd::UnsubscribeCmd;
pub use vibrate_cmd::{VibrateCmd, VibrateSubcommand};
pub use vorze_a10_cyclone_cmd::VorzeA10CycloneCmd;

use super::errors::*;
use std::convert::TryFrom;
#[cfg(feature = "serialize_json")]
use serde::{Deserialize, Serialize};

pub enum ButtplugMessageSpecVersion {
    Version0 = 0,
    Version1 = 1,
    Version2 = 2,
}

/// Base trait for all Buttplug Protocol Message Structs. Handles management of
/// message ids, as well as implementing conveinence functions for converting
/// between message structs and [ButtplugMessageUnion] enums, serialization, etc...
pub trait ButtplugMessage: Send + Sync + Clone {
    /// Returns the id number of the message
    fn get_id(&self) -> u32;
    /// Sets the id number of the message.
    fn set_id(&mut self, id: u32);
    /// Returns the message as a string in Buttplug JSON Protocol format.
    #[cfg(feature = "serialize_json")]
    fn as_protocol_json(self) -> String
    where
        Self: ButtplugMessage + Serialize + Deserialize<'static>,
    {
        serde_json::to_string(&[&self]).unwrap()
    }
}

pub trait ButtplugDeviceMessage: ButtplugMessage {
    fn get_device_index(&self) -> u32;
    fn set_device_index(&mut self, id: u32);
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize_json", derive(Serialize, Deserialize))]
pub enum ButtplugMessageType {
    // Status messages
    Ok,
    Error,
    Ping,
    Test,
    RequestLog,
    Log,
    // Handshake messages
    RequestServerInfo,
    ServerInfo,
    // Device enumeration messages
    DeviceList,
    DeviceAdded,
    DeviceRemoved,
    StartScanning,
    StopScanning,
    ScanningFinished,
    RequestDeviceList,
    // Generic commands
    StopAllDevices,
    VibrateCmd,
    LinearCmd,
    RotateCmd,
    RawWriteCmd,
    RawReadCmd,
    StopDeviceCmd,
    RawReading,
    SubscribeCmd,
    UnsubscribeCmd,
    // Deprecated generic commands
    SingleMotorVibrateCmd,
    // Deprecated device specific commands
    FleshlightLaunchFW12Cmd,
    LovenseCmd,
    KiirooCmd,
    VorzeA10CycloneCmd,
    // To Add:
    // PatternCmd
    // BatteryLevelCmd
    // BatteryLevelReading
    // RSSILevelCmd
    // RSSILevelReading
    // ShockCmd?
    // ToneEmitterCmd?
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serialize_json", derive(Serialize, Deserialize))]
pub enum ButtplugDeviceMessageType {
    // Generic commands
    VibrateCmd,
    LinearCmd,
    RotateCmd,
    RawWriteCmd,
    RawReadCmd,
    StopDeviceCmd,
    SubscribeCmd,
    UnsubscribeCmd,
    // Deprecated generic commands
    SingleMotorVibrateCmd,
    // Deprecated device specific commands
    FleshlightLaunchFW12Cmd,
    LovenseCmd,
    KiirooCmd,
    VorzeA10CycloneCmd,
    // To Add:
    // PatternCmd
    // BatteryLevelCmd
    // BatteryLevelReading
    // RSSILevelCmd
    // RSSILevelReading
    // ShockCmd?
    // ToneEmitterCmd?
}

#[derive(Debug, Clone, PartialEq, ButtplugMessage, FromSpecificButtplugMessage)]
#[cfg_attr(feature = "serialize_json", derive(Serialize, Deserialize))]
pub enum ButtplugMessageUnion {
    // Status messages
    Ok(Ok),
    Error(Error),
    Ping(Ping),
    Test(Test),
    RequestLog(RequestLog),
    Log(Log),
    // Handshake messages
    RequestServerInfo(RequestServerInfo),
    ServerInfo(ServerInfo),
    // Device enumeration messages
    DeviceList(DeviceList),
    DeviceAdded(DeviceAdded),
    DeviceRemoved(DeviceRemoved),
    StartScanning(StartScanning),
    StopScanning(StopScanning),
    ScanningFinished(ScanningFinished),
    RequestDeviceList(RequestDeviceList),
    // Generic commands
    StopAllDevices(StopAllDevices),
    VibrateCmd(VibrateCmd),
    LinearCmd(LinearCmd),
    RotateCmd(RotateCmd),
    RawWriteCmd(RawWriteCmd),
    RawReadCmd(RawReadCmd),
    StopDeviceCmd(StopDeviceCmd),
    RawReading(RawReading),
    SubscribeCmd(SubscribeCmd),
    UnsubscribeCmd(UnsubscribeCmd),
    // Deprecated generic commands
    SingleMotorVibrateCmd(SingleMotorVibrateCmd),
    // Deprecated device specific commands
    FleshlightLaunchFW12Cmd(FleshlightLaunchFW12Cmd),
    LovenseCmd(LovenseCmd),
    KiirooCmd(KiirooCmd),
    VorzeA10CycloneCmd(VorzeA10CycloneCmd),
    // To Add:
    // PatternCmd
    // BatteryLevelCmd
    // BatteryLevelReading
    // RSSILevelCmd
    // RSSILevelReading
    // ShockCmd?
    // ToneEmitterCmd?
}

#[cfg(feature = "serialize_json")]
impl ButtplugMessageUnion {
    pub fn try_deserialize(msg_str: &str) -> Result<Self, ButtplugError> {
        serde_json::from_str::<Vec<ButtplugMessageUnion>>(&msg_str)
            .and_then(|msg_vec| Ok(msg_vec[0].clone()))
            .map_err(|e| ButtplugMessageError::new(&e.to_string()).into())
    }
}

#[derive(Debug, Clone, PartialEq, ButtplugMessage, FromSpecificButtplugMessage)]
#[cfg_attr(feature = "serialize_json", derive(Serialize, Deserialize))]
pub enum ButtplugMessageUnionVersion2 {
    Ok(Ok),
    Error(Error),
    Ping(Ping),
    Test(Test),
    RequestLog(RequestLog),
    Log(Log),
    // Handshake messages
    RequestServerInfo(RequestServerInfo),
    ServerInfo(ServerInfo),
    // Device enumeration messages
    DeviceList(DeviceList),
    DeviceAdded(DeviceAdded),
    DeviceRemoved(DeviceRemoved),
    StartScanning(StartScanning),
    StopScanning(StopScanning),
    ScanningFinished(ScanningFinished),
    RequestDeviceList(RequestDeviceList),
    // Generic commands
    StopAllDevices(StopAllDevices),
    VibrateCmd(VibrateCmd),
    LinearCmd(LinearCmd),
    RotateCmd(RotateCmd),
    RawWriteCmd(RawWriteCmd),
    RawReadCmd(RawReadCmd),
    StopDeviceCmd(StopDeviceCmd),
    RawReading(RawReading),
    SubscribeCmd(SubscribeCmd),
    UnsubscribeCmd(UnsubscribeCmd),
    // PatternCmd
    // BatteryLevelCmd
    // BatteryLevelReading
    // RSSILevelCmd
    // RSSILevelReading
    // ShockCmd?
    // ToneEmitterCmd?
}

#[derive(Debug, Clone, PartialEq, ButtplugMessage, FromSpecificButtplugMessage)]
#[cfg_attr(feature = "serialize_json", derive(Serialize, Deserialize))]
enum ButtplugMessageUnionVersion1 {
    Ok(Ok),
    Error(Error),
    Ping(Ping),
    Test(Test),
    RequestLog(RequestLog),
    Log(Log),
    RequestServerInfo(RequestServerInfo),
    ServerInfo(ServerInfo),
    DeviceList(DeviceList),
    DeviceAdded(DeviceAdded),
    DeviceRemoved(DeviceRemoved),
    StartScanning(StartScanning),
    StopScanning(StopScanning),
    ScanningFinished(ScanningFinished),
    RequestDeviceList(RequestDeviceList),
    StopAllDevices(StopAllDevices),
    VibrateCmd(VibrateCmd),
    LinearCmd(LinearCmd),
    RotateCmd(RotateCmd),
    FleshlightLaunchFW12Cmd(FleshlightLaunchFW12Cmd),
    LovenseCmd(LovenseCmd),
    KiirooCmd(KiirooCmd),
    VorzeA10CycloneCmd(VorzeA10CycloneCmd),
    SingleMotorVibrateCmd(SingleMotorVibrateCmd),
    StopDeviceCmd(StopDeviceCmd),
}

#[derive(Debug, Clone, PartialEq, ButtplugMessage, FromSpecificButtplugMessage)]
#[cfg_attr(feature = "serialize_json", derive(Serialize, Deserialize))]
enum ButtplugMessageSpecVersion0 {
    Ok(Ok),
    Error(Error),
    Log(Log),
    RequestLog(RequestLog),
    Ping(Ping),
    Test(Test),
    RequestServerInfo(RequestServerInfo),
    ServerInfo(ServerInfo),
    RequestDeviceList(RequestDeviceList),
    DeviceList(DeviceList),
    DeviceAdded(DeviceAdded),
    DeviceRemoved(DeviceRemoved),
    StartScanning(StartScanning),
    StopScanning(StopScanning),
    ScanningFinished(ScanningFinished),
    SingleMotorVibrateCmd(SingleMotorVibrateCmd),
    FleshlightLaunchFW12Cmd(FleshlightLaunchFW12Cmd),
    LovenseCmd(LovenseCmd),
    KiirooCmd(KiirooCmd),
    VorzeA10CycloneCmd(VorzeA10CycloneCmd),
    StopDeviceCmd(StopDeviceCmd),
    StopAllDevices(StopAllDevices),
}

/// Messages that should never be received from the client.
#[derive(
    Debug, Clone, PartialEq, ButtplugMessage, TryFromButtplugMessageUnion, FromSpecificButtplugMessage,
)]
pub enum ButtplugSystemMessageUnion {
    Ok(Ok),
    Error(Error),
    Log(Log),
    ServerInfo(ServerInfo),
    DeviceList(DeviceList),
    DeviceAdded(DeviceAdded),
    DeviceRemoved(DeviceRemoved),
    ScanningFinished(ScanningFinished),
    RawReading(RawReading),
}

/// Messages that should never be received from the client.
#[derive(
    Debug, 
    Clone, 
    PartialEq, 
    ButtplugMessage, 
    TryFromButtplugMessageUnion, 
    FromSpecificButtplugMessage,
)]
pub enum ButtplugDeviceManagerMessageUnion {
    RequestDeviceList(RequestDeviceList),
    StopAllDevices(StopAllDevices),
    StartScanning(StartScanning),
    StopScanning(StopScanning),
}

/// Messages that should be routed to device instances.
#[derive(
    Debug,
    Clone,
    PartialEq,
    ButtplugDeviceMessage,
    TryFromButtplugMessageUnion,
    FromSpecificButtplugMessage,
)]
pub enum ButtplugDeviceCommandMessageUnion {
    FleshlightLaunchFW12Cmd(FleshlightLaunchFW12Cmd),
    SingleMotorVibrateCmd(SingleMotorVibrateCmd),
    VorzeA10CycloneCmd(VorzeA10CycloneCmd),
    KiirooCmd(KiirooCmd),
    // No LovenseCmd, it was never implemented anywhere.
    VibrateCmd(VibrateCmd),
    LinearCmd(LinearCmd),
    RotateCmd(RotateCmd),
    RawWriteCmd(RawWriteCmd),
    RawReadCmd(RawReadCmd),
    StopDeviceCmd(StopDeviceCmd),
    SubscribeCmd(SubscribeCmd),
    UnsubscribeCmd(UnsubscribeCmd),
}
