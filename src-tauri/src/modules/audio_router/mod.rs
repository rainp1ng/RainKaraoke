pub mod recorder;
pub mod live_router;

use cpal::traits::{DeviceTrait, HostTrait};
use serde::{Deserialize, Serialize};

pub use recorder::{AudioRecorder, DualTrackRecorder, RecordingState};
pub use live_router::{LiveAudioManager, LiveAudioConfig, LiveAudioState, EffectInput, GlobalAudioState, AudioStreams, DeviceInfo};

/// 音频设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AudioDeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub is_default: bool,
    pub channels: u16,
}

/// 音频设备管理器
pub struct AudioManager {
    host: cpal::Host,
}

impl AudioManager {
    pub fn new() -> Self {
        Self {
            host: cpal::default_host(),
        }
    }

    /// 获取所有音频设备
    pub fn list_devices(&self) -> Vec<AudioDeviceInfo> {
        let mut devices = Vec::new();

        let default_input_name = self.host.default_input_device()
            .and_then(|d| d.name().ok());
        let default_output_name = self.host.default_output_device()
            .and_then(|d| d.name().ok());

        // 输入设备
        if let Ok(input_devices) = self.host.input_devices() {
            for device in input_devices {
                if let Ok(name) = device.name() {
                    let is_default = default_input_name.as_ref()
                        .map(|n| n == &name)
                        .unwrap_or(false);

                    let channels = device.default_input_config()
                        .map(|c| c.channels())
                        .unwrap_or(0);

                    devices.push(AudioDeviceInfo {
                        id: name.clone(),
                        name,
                        device_type: "input".to_string(),
                        is_default,
                        channels,
                    });
                }
            }
        }

        // 输出设备
        if let Ok(output_devices) = self.host.output_devices() {
            for device in output_devices {
                if let Ok(name) = device.name() {
                    let is_default = default_output_name.as_ref()
                        .map(|n| n == &name)
                        .unwrap_or(false);

                    let channels = device.default_output_config()
                        .map(|c| c.channels())
                        .unwrap_or(0);

                    devices.push(AudioDeviceInfo {
                        id: name.clone(),
                        name,
                        device_type: "output".to_string(),
                        is_default,
                        channels,
                    });
                }
            }
        }

        devices
    }

    /// 获取默认输入设备
    pub fn default_input_device(&self) -> Option<AudioDeviceInfo> {
        let device = self.host.default_input_device()?;
        let name = device.name().ok()?;
        let channels = device.default_input_config().ok()?.channels();

        Some(AudioDeviceInfo {
            id: name.clone(),
            name,
            device_type: "input".to_string(),
            is_default: true,
            channels,
        })
    }

    /// 获取默认输出设备
    pub fn default_output_device(&self) -> Option<AudioDeviceInfo> {
        let device = self.host.default_output_device()?;
        let name = device.name().ok()?;
        let channels = device.default_output_config().ok()?.channels();

        Some(AudioDeviceInfo {
            id: name.clone(),
            name,
            device_type: "output".to_string(),
            is_default: true,
            channels,
        })
    }
}

impl Default for AudioManager {
    fn default() -> Self {
        Self::new()
    }
}
