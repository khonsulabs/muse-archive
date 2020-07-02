#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Message {
    Start,
    TimingClock,
    Continue,
    Stop,
    ActiveSensing,
    SystemReset,

    Vendor {
        vendor_id: u8,
        payload: Vec<u8>,
    },

    Channel {
        channel: u8,
        message: ChannelMessage,
    },

    Unsupported(Vec<u8>),
}

impl From<&[u8]> for Message {
    fn from(bytes: &[u8]) -> Self {
        if bytes.is_empty() || bytes[0] < 0x80 {
            return Message::Unsupported(bytes.into());
        }

        if bytes[0] < 0xF0 {
            // Channel message
            let status = bytes[0] & 0xF0;
            let channel = bytes[0] & 0x0F;
            Message::Channel {
                channel,
                message: match status {
                    0x80 => ChannelMessage::NoteOff {
                        key: bytes[1],
                        velocity: bytes[2],
                    },
                    0x90 => ChannelMessage::NoteOn {
                        key: bytes[1],
                        velocity: bytes[2],
                    },
                    0xA0 => ChannelMessage::PolyphonicKeyPressure {
                        key: bytes[1],
                        pressure: bytes[2],
                    },
                    0xB0 => ChannelMessage::ControlChange {
                        controller: bytes[1].into(),
                        value: bytes[2],
                    },
                    0xC0 => ChannelMessage::ProgramChange { program: bytes[1] },
                    0xD0 => ChannelMessage::ChannelPressure { pressure: bytes[1] },
                    0xE0 => ChannelMessage::PitchBend {
                        // Byte 1 is the least significant byte (7 bits though)
                        // Byte 2 is the most significant (also 7 bits)
                        // Value of 8192 (64 << 7) is no bend
                        amount: ((bytes[2] as i16) << 7 | bytes[1] as i16) - 8192,
                    },
                    _ => unreachable!(),
                },
            }
        } else {
            match bytes[0] {
                0xF0 => Message::Vendor {
                    vendor_id: bytes[1],
                    payload: bytes.iter().skip(2).cloned().collect(),
                },
                _ => Message::Unsupported(bytes.into()),
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ChannelMessage {
    AllSoundOff,
    ResetAllControllers,
    LocalControlOff,
    LocalControlOn,
    AllNotesOff,
    NoteOff { key: u8, velocity: u8 },
    NoteOn { key: u8, velocity: u8 },
    ProgramChange { program: u8 },
    ControlChange { controller: Controller, value: u8 },
    PolyphonicKeyPressure { key: u8, pressure: u8 },
    ChannelPressure { pressure: u8 },
    PitchBend { amount: i16 },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Controller {
    Undefined(u8),
    ModulationWheel,
    BreathController,
    FootController,
    PortamentoTime,
    DataEntrySlider,
    MainVolume,
    Balance,
    Pan,
    ExpressionController,
    GeneralPurpose1,
    GeneralPurpose2,
    GeneralPurpose3,
    GeneralPurpose4,
    Damper,
    Portamento,
    Sostenuto,
    SoftPedal,
}

impl From<u8> for Controller {
    fn from(value: u8) -> Self {
        match value {
            1 => Controller::ModulationWheel,
            2 => Controller::BreathController,
            4 => Controller::FootController,
            5 => Controller::PortamentoTime,
            6 => Controller::DataEntrySlider,
            7 => Controller::MainVolume,
            8 => Controller::Balance,
            10 => Controller::Pan,
            11 => Controller::ExpressionController,
            16 => Controller::GeneralPurpose1,
            17 => Controller::GeneralPurpose2,
            18 => Controller::GeneralPurpose3,
            19 => Controller::GeneralPurpose4,
            64 => Controller::Damper,
            65 => Controller::Portamento,
            66 => Controller::Sostenuto,
            67 => Controller::SoftPedal,
            undefined => Controller::Undefined(undefined),
        }
    }
}
