use crate::{
    manager::{Manager, ManagerHandle, ManagerMessage, PlayingHandle},
    note::Note,
    sampler::PreparedSampler,
};
use cpal::traits::{DeviceTrait, HostTrait};
use crossbeam::channel::bounded;

#[derive(thiserror::Error, Debug)]
pub enum HardwareError {
    #[error("no default output device found")]
    NoDefaultOutputDevice,
    #[error("Error getting devices {0}")]
    DevicesError(#[from] cpal::DevicesError),
    #[error("Error getting device name {0}")]
    DeviceNameError(#[from] cpal::DeviceNameError),
    // #[error("Error getting supported formats {0}")]
    // SupportedFormatsError(#[from] cpal::SupportedFormatsError),
    // #[error("Error getting default format {0}")]
    // DefaultFormatError(#[from] cpal::DefaultFormatError),
}

pub struct Device {
    manager: ManagerHandle,
}

impl Device {
    pub fn default_output() -> Result<Self, anyhow::Error> {
        let host = cpal::default_host();
        if let Some(cpal_device) = host.default_output_device() {
            let format = cpal_device.default_output_config()?;
            let manager = Manager::open_output_device(cpal_device, format.into())?;
            Ok(Self { manager })
        } else {
            Err(anyhow::Error::from(HardwareError::NoDefaultOutputDevice))
        }
    }

    pub fn play(
        &self,
        sampler: PreparedSampler,
        note: Note,
    ) -> Result<PlayingHandle, anyhow::Error> {
        let (callback, handle) = bounded(1);
        {
            let manager = self.manager.read().expect("Error reading manager");
            manager.sender.send(ManagerMessage::Append {
                note,
                sampler,
                callback,
            })?;
        }

        Ok(handle.recv()?)
    }
}
