use clap::ValueEnum;
use std::sync::RwLock;

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum DebugType {
    All,
    Decoder,
    Header,
    SideInfo,
    ScaleFactor,
}

#[derive(Debug, Default)]
pub struct DebugConfig {
    pub enabled_types: Vec<DebugType>,
}

pub static DEBUG_CONFIG: RwLock<Option<DebugConfig>> = RwLock::new(None);

impl DebugConfig {
    pub fn new(debug_types: &[DebugType]) -> Self {
        let mut config = DebugConfig::default();
        if debug_types.iter().any(|&dt| matches!(dt, DebugType::All)) {
            config.enabled_types = vec![DebugType::Decoder, DebugType::Header, DebugType::SideInfo, DebugType::ScaleFactor];
        } else {
            config.enabled_types = debug_types.to_vec();
        }
        config
    }

    pub fn init(debug_types: &[DebugType]) {
        let config = DebugConfig::new(debug_types);
        *DEBUG_CONFIG.write().unwrap() = Some(config);
    }
    pub fn is_enabled(&self, debug_type: DebugType) -> bool {
        self.enabled_types.iter().any(|&dt| std::mem::discriminant(&dt) == std::mem::discriminant(&debug_type))
    }
}

pub fn debug_print(debug_type: DebugType, message: &str) {
    if let Some(config) = DEBUG_CONFIG.read().unwrap().as_ref() {
        if config.is_enabled(debug_type) {
            println!("[{:?}] {}", debug_type, message);
        }
    }
}

#[macro_export]
macro_rules! dbg_println {
    ($debug_type:expr, $($arg:tt)*) => {
        $crate::debug::debug_print($debug_type, &format!($($arg)*));
    }
}