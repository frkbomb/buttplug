// Buttplug Rust Source Code File - See https://buttplug.io for more info.
//
// Copyright 2016-2022 Nonpolynomial Labs LLC. All rights reserved.
//
// Licensed under the BSD 3-Clause license. See LICENSE file in the project root
// for full license information.

use super::SerialPortDeviceImplCreator;
use crate::{
  core::ButtplugResultFuture,
  server::device::communication::{
    DeviceCommunicationEvent,
    DeviceCommunicationManager,
    DeviceCommunicationManagerBuilder,
  },
};
use futures::future;
use serialport::available_ports;
use tokio::sync::mpsc::Sender;
use tracing_futures::Instrument;

#[derive(Default, Clone)]
pub struct SerialPortCommunicationManagerBuilder {}

impl DeviceCommunicationManagerBuilder for SerialPortCommunicationManagerBuilder {
  fn finish(&self, sender: Sender<DeviceCommunicationEvent>) -> Box<dyn DeviceCommunicationManager> {
    Box::new(SerialPortCommunicationManager::new(sender))
  }
}

pub struct SerialPortCommunicationManager {
  sender: Sender<DeviceCommunicationEvent>,
}

impl SerialPortCommunicationManager {
  fn new(sender: Sender<DeviceCommunicationEvent>) -> Self {
    trace!("Serial port created.");
    Self { sender }
  }
}

impl DeviceCommunicationManager for SerialPortCommunicationManager {
  fn name(&self) -> &'static str {
    "SerialPortCommunicationManager"
  }

  fn start_scanning(&self) -> ButtplugResultFuture {
    debug!("Serial port manager scanning for devices.");
    // TODO Does this block? Should it run in one of our threads?
    let sender = self.sender.clone();
    Box::pin(
      async move {
        match available_ports() {
          Ok(ports) => {
            debug!("Got {} serial ports back", ports.len());
            for p in ports {
              trace!(
                "Sending serial port {:?} for possible device connection.",
                p
              );
              if sender
                .send(DeviceCommunicationEvent::DeviceFound {
                  name: format!("Serial Port Device {}", p.port_name),
                  address: p.port_name.clone(),
                  creator: Box::new(SerialPortDeviceImplCreator::new(&p)),
                })
                .await
                .is_err()
              {
                debug!("Device manager disappeared, exiting.");
                break;
              }
            }
          }
          Err(_) => {
            debug!("No serial ports found");
          }
        }
        if sender
          .send(DeviceCommunicationEvent::ScanningFinished)
          .await
          .is_err()
        {
          error!("Error sending scanning finished.");
        }
        Ok(())
      }
      .instrument(tracing::info_span!(
        "Serial Port Device Comm Manager Scanning."
      )),
    )
  }

  fn stop_scanning(&self) -> ButtplugResultFuture {
    Box::pin(future::ready(Ok(())))
  }

  // We should always be able to at least look at serial ports.
  fn can_scan(&self) -> bool {
    true
  }
}
