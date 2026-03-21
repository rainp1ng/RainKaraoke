use std::sync::{Arc, Mutex};
use midir::{MidiInput, MidiInputConnection};
use serde::{Deserialize, Serialize};
use tauri::{AppHandle, Emitter};

/// MIDI 设备信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiDevice {
    pub id: String,
    pub name: String,
}

/// MIDI 消息类型
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "UPPERCASE")]
pub enum MidiMessageType {
    Note,
    CC,
    PC,
}

/// MIDI 事件
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct MidiEvent {
    pub message_type: MidiMessageType,
    pub channel: u8,
    pub data1: u8,   // note for NOTE, controller for CC, program for PC
    pub data2: u8,   // velocity for NOTE, value for CC
    pub is_on: bool, // for NOTE: true = NoteOn, false = NoteOff
}

/// MIDI 处理器
pub struct MidiHandler {
    input: Option<MidiInput>,
    connection: Option<MidiInputConnection<()>>,
    connected_port: Option<String>,
    app_handle: Option<AppHandle>,
}

impl MidiHandler {
    pub fn new() -> Self {
        Self {
            input: None,
            connection: None,
            connected_port: None,
            app_handle: None,
        }
    }

    pub fn set_app_handle(&mut self, handle: AppHandle) {
        self.app_handle = Some(handle);
    }

    /// 获取可用的 MIDI 输入设备列表
    pub fn list_devices(&mut self) -> Result<Vec<MidiDevice>, String> {
        if self.input.is_none() {
            self.input = Some(MidiInput::new("RainKaraoke MIDI Input")
                .map_err(|e| format!("无法创建 MIDI 输入: {}", e))?);
        }

        let input = self.input.as_ref().unwrap();
        let ports = input.ports();

        Ok(ports
            .iter()
            .enumerate()
            .filter_map(|(i, port)| {
                input.port_name(port).ok().map(|name| MidiDevice {
                    id: format!("midi_{}", i),
                    name,
                })
            })
            .collect())
    }

    /// 连接 MIDI 设备
    pub fn connect(&mut self, port_name: &str) -> Result<(), String> {
        if self.connection.is_some() {
            self.disconnect();
        }

        let input = MidiInput::new("RainKaraoke MIDI Input")
            .map_err(|e| format!("无法创建 MIDI 输入: {}", e))?;

        // 查找指定端口
        let port = input
            .ports()
            .iter()
            .find(|p| {
                input.port_name(p)
                    .map(|name| name == port_name)
                    .unwrap_or(false)
            })
            .ok_or("找不到指定的 MIDI 设备")?
            .clone();

        let app_handle = self.app_handle.clone();

        // 创建连接
        let connection = input
            .connect(
                &port,
                "RainKaraoke",
                move |_timestamp, message, _| {
                    // 解析 MIDI 消息
                    if message.is_empty() {
                        return;
                    }

                    let status = message[0];
                    let channel = status & 0x0F;
                    let status_byte = status & 0xF0;

                    // NOTE ON (0x90) or NOTE OFF (0x80)
                    if (status_byte == 0x90 || status_byte == 0x80) && message.len() >= 3 {
                        let note = message[1];
                        let velocity = message[2];
                        let is_note_on = status_byte == 0x90 && velocity > 0;

                        let event = MidiEvent {
                            message_type: MidiMessageType::Note,
                            channel,
                            data1: note,
                            data2: velocity,
                            is_on: is_note_on,
                        };

                        if let Some(ref handle) = app_handle {
                            let _ = handle.emit("midi:event", &event);
                        }
                    }
                    // CONTROL CHANGE (0xB0)
                    else if status_byte == 0xB0 && message.len() >= 3 {
                        let controller = message[1];
                        let value = message[2];

                        let event = MidiEvent {
                            message_type: MidiMessageType::CC,
                            channel,
                            data1: controller,
                            data2: value,
                            is_on: true,
                        };

                        if let Some(ref handle) = app_handle {
                            let _ = handle.emit("midi:event", &event);
                        }
                    }
                    // PROGRAM CHANGE (0xC0)
                    else if status_byte == 0xC0 && message.len() >= 2 {
                        let program = message[1];

                        let event = MidiEvent {
                            message_type: MidiMessageType::PC,
                            channel,
                            data1: program,
                            data2: 0,
                            is_on: true,
                        };

                        if let Some(ref handle) = app_handle {
                            let _ = handle.emit("midi:event", &event);
                        }
                    }
                },
                (),
            )
            .map_err(|e| format!("无法连接 MIDI 设备: {}", e))?;

        self.connection = Some(connection);
        self.connected_port = Some(port_name.to_string());

        Ok(())
    }

    /// 断开连接
    pub fn disconnect(&mut self) {
        if let Some(conn) = self.connection.take() {
            conn.close();
        }
        self.connected_port = None;
    }

    /// 获取连接状态
    pub fn get_status(&self) -> MidiStatus {
        MidiStatus {
            connected: self.connection.is_some(),
            device_name: self.connected_port.clone(),
        }
    }
}

impl Default for MidiHandler {
    fn default() -> Self {
        Self::new()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MidiStatus {
    pub connected: bool,
    pub device_name: Option<String>,
}
