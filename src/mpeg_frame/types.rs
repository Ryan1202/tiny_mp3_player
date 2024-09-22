#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MpegVersion {
    V2_5,
    Reserved,
    V2,
    V1,
}
impl MpegVersion {
    pub fn new(value: u8) -> Self {
        match value {
            0b00 => {Self::V2_5},
            0b01 => {Self::Reserved},
            0b10 => {Self::V2},
            0b11 => {Self::V1},
            _ => Self::Reserved,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::V2_5 => {"MPEG Version 2.5"},
            Self::Reserved => {"Reserved"},
            Self::V2 => {"MPEG Version 2"},
            Self::V1 => {"MPEG Version 1"},
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            Self::V2_5 => 0b00,
            Self::Reserved => 0b01,
            Self::V2 => 0b10,
            Self::V1 => 0b11,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MpegLayer {
    Reserved,
    Layer3,
    Layer2,
    Layer1,
}
impl MpegLayer {
    pub fn new(value: u8) -> Self {
        match value {
            0b00 => {Self::Reserved},
            0b01 => {Self::Layer3},
            0b10 => {Self::Layer2},
            0b11 => {Self::Layer1},
            _ => Self::Reserved,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::Reserved => {"Reserved"},
            Self::Layer3 => {"Layer III"},
            Self::Layer2 => {"Layer II"},
            Self::Layer1 => {"Layer I"},
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            Self::Reserved => 0b00,
            Self::Layer3 => 0b01,
            Self::Layer2 => 0b10,
            Self::Layer1 => 0b11,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MpegProtection {
    Protected,
    NotProtected,
}
impl MpegProtection {
    pub fn new(value: u8) -> Self {
        match value {
            0b0 => {Self::Protected},
            0b1 => {Self::NotProtected},
            _ => Self::NotProtected,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::Protected => {"Protected"},
            Self::NotProtected => {"NotProtected"},
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            Self::Protected => 0b0,
            Self::NotProtected => 0b1,
        }
    }
}

pub struct MpegBitRate {
    value: usize,
    rate: usize,
}
impl MpegBitRate {
    pub fn new(value: u8, ver: MpegVersion, layer: MpegLayer) -> Self {
        let rate = match value {
            0b0001 => {
                if ver == MpegVersion::V2 && (layer == MpegLayer::Layer2 || layer == MpegLayer::Layer3) {8} else {32}
            },
            0b0010 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {64}
                    else if layer == MpegLayer::Layer2 {48}
                    else {40}
                } else {
                    if layer == MpegLayer::Layer1 {48}
                    else {16}
                }
            },
            0b0011 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {96}
                    else if layer == MpegLayer::Layer2 {56}
                    else {48}
                } else {
                    if layer == MpegLayer::Layer1 {56}
                    else {24}
                }
            },
            0b0100 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {128}
                    else if layer == MpegLayer::Layer2 {64}
                    else {56}
                } else {
                    if layer == MpegLayer::Layer1 {64}
                    else {32}
                }
            },
            0b0101 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {160}
                    else if layer == MpegLayer::Layer2 {80}
                    else {64}
                } else {
                    if layer == MpegLayer::Layer1 {80}
                    else {40}
                }
            },
            0b0110 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {192}
                    else if layer == MpegLayer::Layer2 {96}
                    else {80}
                } else {
                    if layer == MpegLayer::Layer1 {96}
                    else {48}
                }
            },
            0b0111 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {224}
                    else if layer == MpegLayer::Layer2 {112}
                    else {96}
                } else {
                    if layer == MpegLayer::Layer1 {112}
                    else {56}
                }
            },
            0b1000 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {256}
                    else if layer == MpegLayer::Layer2 {128}
                    else {112}
                } else {
                    if layer == MpegLayer::Layer1 {128}
                    else {64}
                }
            },
            0b1001 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {288}
                    else if layer == MpegLayer::Layer2 {160}
                    else {128}
                } else {
                    if layer == MpegLayer::Layer1 {144}
                    else {80}
                }
            },
            0b1010 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {320}
                    else if layer == MpegLayer::Layer2 {192}
                    else {160}
                } else {
                    if layer == MpegLayer::Layer1 {160}
                    else {96}
                }
            },
            0b1011 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {352}
                    else if layer == MpegLayer::Layer2 {224}
                    else {192}
                } else {
                    if layer == MpegLayer::Layer1 {176}
                    else {112}
                }
            },
            0b1100 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {384}
                    else if layer == MpegLayer::Layer2 {256}
                    else {224}
                } else {
                    if layer == MpegLayer::Layer1 {192}
                    else {128}
                }
            },
            0b1101 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {416}
                    else if layer == MpegLayer::Layer2 {320}
                    else {256}
                } else {
                    if layer == MpegLayer::Layer1 {224}
                    else {144}
                }
            },
            0b1110 => {
                if ver == MpegVersion::V1 {
                    if layer == MpegLayer::Layer1 {448}
                    else if layer == MpegLayer::Layer2 {384}
                    else {320}
                } else {
                    if layer == MpegLayer::Layer1 {256}
                    else {160}
                }
            },
            _ => 0,
        };
        Self { value: value as usize, rate: rate }
    }

    pub fn get_value(&self) -> usize {
        self.value
    }

    pub fn get_rate(&self) -> usize {
        self.rate
    }

    pub fn to_string(&self) -> String {
        format!("{}Kbps", self.rate)
    }
}

pub struct MpegSampleRate {
    value: usize,
    rate: usize,
}
impl MpegSampleRate {
    pub fn new(value: u8, ver: MpegVersion) -> Self {
        let rate = match value {
            0b00 => {
                match ver {
                    MpegVersion::V1 => 44100,
                    MpegVersion::V2 => 22050,
                    MpegVersion::V2_5 => 11025,
                    _ => 0,
                }
            },
            0b01 => {
                match ver {
                    MpegVersion::V1 => 48000,
                    MpegVersion::V2 => 24000,
                    MpegVersion::V2_5 => 12000,
                    _ => 0,
                }
            }
            0b10 => {
                match ver {
                    MpegVersion::V1 => 32000,
                    MpegVersion::V2 => 16000,
                    MpegVersion::V2_5 => 8000,
                    _ => 0,
                }
            }
            _ => 0,
        };
        Self { value: value as usize, rate: rate }
    }

    pub fn get_value(&self) -> usize {
        self.value
    }

    pub fn get_rate(&self) -> usize {
        self.rate
    }

    pub fn to_string(&self) -> String {
        format!("{}Hz", self.rate)
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub enum MpegChannelMode {
    Stereo,
    JointStereo,
    DualChannel,
    SingleChannel,
}
impl MpegChannelMode {
    pub fn new(value: u8) -> Self {
        match value {
            0b00 => {Self::Stereo},
            0b01 => {Self::JointStereo},
            0b10 => {Self::DualChannel},
            0b11 => {Self::SingleChannel},
            _ => Self::Stereo,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::Stereo => "Stereo",
            Self::JointStereo => "Joint Stereo",
            Self::DualChannel => "Dual Channel",
            Self::SingleChannel => "Single Channel",
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            Self::Stereo => 0b00,
            Self::JointStereo => 0b01,
            Self::DualChannel => 0b10,
            Self::SingleChannel => 0b11,
        }
    }

    pub fn to_channel_num(&self) -> usize {
        match self {
            Self::Stereo => 2,
            Self::JointStereo => 2,
            Self::DualChannel => 2,
            Self::SingleChannel => 1,
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct MpegModeExtension {
    pub ms_stereo: bool,
    pub intensity_stereo: bool,
}
impl MpegModeExtension {
    pub fn new(value: u8) -> Self {
        Self { ms_stereo: (value >> 1) != 0, intensity_stereo: (value & 1) != 0 }
    }
    pub fn to_string(&self) -> String {
        format!("MS Stereo: {}, Intensity Stereo: {}", self.ms_stereo, self.intensity_stereo)
    }
    pub fn to_value(&self) -> u8 {
        (self.ms_stereo as u8) << 1 | (self.intensity_stereo as u8)
    }
}

pub enum MpegCopyright {
    NotCopyrighted,
    Copyrighted,
}
impl MpegCopyright {
    pub fn new(value: u8) -> Self {
        match value {
            0b0 => Self::NotCopyrighted,
            0b1 => Self::Copyrighted,
            _ => Self::NotCopyrighted,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::NotCopyrighted => "Not Copyrighted",
            Self::Copyrighted => "Copyrighted",
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            Self::NotCopyrighted => 0b0,
            Self::Copyrighted => 0b1,
        }
    }
}

pub enum MpegOringinal {
    CopyOfOriginal,
    Original,
}
impl MpegOringinal {
    pub fn new(value: u8) -> Self {
        match value {
            0b0 => Self::CopyOfOriginal,
            0b1 => Self::Original,
            _ => Self::CopyOfOriginal,
        }
    }

    pub fn to_string(&self) -> &str {
        match self {
            Self::CopyOfOriginal => "Copy of Original Media",
            Self::Original => "Original Media",
        }
    }

    pub fn to_value(&self) -> u8 {
        match self {
            Self::CopyOfOriginal => 0b0,
            Self::Original => 0b1,
        }
    }
}

pub struct MpegSideInfo {
    pub main_data_end: usize,
    pub private_bits: usize,
    pub scfsi: [[usize; 4]; 2],
    pub granule: [Granule; 2],
}
impl MpegSideInfo {
    pub fn new() -> Self {
        Self {
            main_data_end: 0,
            private_bits: 0,
            scfsi: [[0usize; 4]; 2],
            granule: [Granule::new(); 2],
        }
    }
}

#[derive(Clone, Copy)]
pub struct Granule {
    pub channel: [Channel; 2],
}
impl Granule {
    pub fn new() -> Self {
        Self {
            channel: [Channel::new(), Channel::new()],
        }
    }
    
}

#[derive(Clone, Copy)]
pub struct Channel {
    pub part2_3_length: usize,
    pub big_values: usize,
    pub global_gain: usize,
    pub scalefac_compress: usize,
    pub blocksplit_flag: usize,
    pub block_type: usize,
    pub switch_point: usize,
    pub table_select: [usize; 3],
    pub subblock_gain: [usize; 3],
    pub region_address1: usize,
    pub region_address2: usize,
    pub preflag: usize,
    pub scalefac_scale: usize,
    pub count1table_select: usize,
}
impl Channel {
    pub fn new() -> Self {
        Self {
            part2_3_length: 0,
            big_values: 0,
            global_gain: 0,
            scalefac_compress: 0,
            blocksplit_flag: 0,
            block_type: 0,
            switch_point: 0,
            table_select: [0usize;3],
            subblock_gain: [0usize;3],
            region_address1: 0,
            region_address2: 0,
            preflag: 0,
            scalefac_scale: 0,
            count1table_select: 0,
        }
    }
}

#[derive(Clone, Copy)]
pub struct ScaleFactor {
    pub sf_l: [usize; 23],
    pub sf_s: [[usize;3]; 13],
}
impl ScaleFactor {
    pub fn new() -> Self {
        Self { sf_l: [0usize; 23], sf_s: [[0usize; 3]; 13] }
    }
}