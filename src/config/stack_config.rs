use std::sync::{Arc, RwLock};
use serde::Deserialize;

use crate::{common::freqs::FreqInfo, config::stack_config_soapy::CfgSoapySdr, entities::lmac::components::scramble::scrambler};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum StackMode {
    Bs,
    Ms,
    Mon,
}

/// The PHY layer backend type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Deserialize)]
#[serde(rename_all = "PascalCase")]
pub enum PhyBackend {
    Undefined,
    None,
    SoapySdr
}

/// PHY layer I/O configuration
#[derive(Debug, Clone, Deserialize)]
pub struct CfgPhyIo {
    /// Backend type: Soapysdr, File, or None
    pub backend: PhyBackend,
    
    /// For File backend: path to input file
    pub input_file: Option<String>,
    
    /// For Soapysdr backend: SoapySDR configuration
    pub soapysdr: Option<CfgSoapySdr>,
}

impl Default for CfgPhyIo {
    fn default() -> Self {
        Self {
            backend: PhyBackend::Undefined,
            input_file: None,
            soapysdr: None,
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgNetInfo {
    /// 10 bits, from 18.4.2.1 D-MLE-SYNC
    pub mcc: u16,
    /// 14 bits, from 18.4.2.1 D-MLE-SYNC
    pub mnc: u16,
}

#[derive(Debug, Clone, Deserialize)]
pub struct CfgCellInfo {
    // 2 bits, from 18.4.2.1 D-MLE-SYNC
    #[serde(default)]
    pub neighbor_cell_broadcast: u8,
    // 2 bits, from 18.4.2.1 D-MLE-SYNC
    #[serde(default)]
    pub cell_load_ca: u8,
    // 1 bit, from 18.4.2.1 D-MLE-SYNC
    #[serde(default)]
    pub late_entry_supported: bool,

    /// 12 bits, from MAC SYSINFO
    #[serde(default = "default_main_carrier")]
    pub main_carrier: u16,
    /// 4 bits, from MAC SYSINFO
    #[serde(default = "default_freq_band")]
    pub freq_band: u8,
    /// 2 bits, from MAC SYSINFO
    #[serde(default)]
    pub freq_offset: u8,
    /// 3 bits, from MAC SYSINFO
    #[serde(default)]
    pub duplex_spacing_setting: u8,
    /// 1 bits, from MAC SYSINFO
    #[serde(default)]
    pub reverse_operation: bool,

    // 14 bits, from 18.4.2.2 D-MLE-SYSINFO
    #[serde(default)]
    pub location_area: u16,
    // 16 bits, from 18.4.2.2 D-MLE-SYSINFO
    #[serde(default)]
    pub subscriber_class: u16,

    // 1-bit service flags
    #[serde(default)]
    pub registration: bool,
    #[serde(default)]
    pub deregistration: bool,
    #[serde(default)]
    pub priority_cell: bool,
    #[serde(default)]
    pub no_minimum_mode: bool,
    #[serde(default)]
    pub migration: bool,
    #[serde(default)]
    pub system_wide_services: bool,
    #[serde(default)]
    pub voice_service: bool,
    #[serde(default)]
    pub circuit_mode_data_service: bool,
    #[serde(default)]
    pub sndcp_service: bool,
    #[serde(default)]
    pub aie_service: bool,
    #[serde(default)]
    pub advanced_link: bool,

    // From SYNC
    #[serde(default)]
    pub system_code: u8,
    #[serde(default)]
    pub colour_code: u8,
    #[serde(default)]
    pub sharing_mode: u8,
    #[serde(default)]
    pub ts_reserved_frames: u8,
    #[serde(default)]
    pub u_plane_dtx: bool,
    #[serde(default)]
    pub frame_18_ext: bool,
}

impl Default for CfgCellInfo {
    fn default() -> Self {
        Self {
            freq_band: default_freq_band(),
            main_carrier: default_main_carrier(),
            freq_offset: 0,
            duplex_spacing_setting: 0,
            reverse_operation: false,

            neighbor_cell_broadcast: 0,
            cell_load_ca: 0,
            late_entry_supported: false,
            location_area: 0,
            subscriber_class: 0,
            registration: true,
            deregistration: true,
            priority_cell: false,
            no_minimum_mode: false,
            migration: false,
            system_wide_services: false,
            voice_service: false,
            circuit_mode_data_service: false,
            sndcp_service: false,
            aie_service: false,
            advanced_link: false,

            system_code: 0,
            colour_code: 0,
            sharing_mode: 0,
            ts_reserved_frames: 0,
            u_plane_dtx: false,
            frame_18_ext: false,
        }
    }
}

#[inline]
fn default_freq_band() -> u8 {
    4
}

#[inline]
fn default_main_carrier() -> u16 {
    1521
}

#[derive(Debug, Clone, Deserialize)]
pub struct StackConfig {
    #[serde(default = "default_stack_mode")]
    pub stack_mode: StackMode,

    #[serde(default)]
    pub phy_io: CfgPhyIo,

    /// Network info is REQUIRED - no default provided
    pub net: CfgNetInfo,

    #[serde(default)]
    pub cell: CfgCellInfo,
}

fn default_stack_mode() -> StackMode {
    StackMode::Bs
}

impl StackConfig {
    
    pub fn new(mode: StackMode, mcc: u16, mnc: u16) -> Self {
        StackConfig {
            stack_mode: mode,
            phy_io: CfgPhyIo::default(),
            net: CfgNetInfo { mcc, mnc },
            cell: CfgCellInfo::default(),
        }
    }

    /// Validate that all required configuration fields are properly set.
    pub fn validate(&self) -> Result<(), &str> {

        // Check input device settings
        match self.phy_io.backend {

            PhyBackend::SoapySdr => {
                let Some(ref soapy_cfg) = self.phy_io.soapysdr else {
                    return Err("soapysdr configuration must be provided for Soapysdr backend");
                };
                
                // Validate that exactly one hardware configuration is present
                let config_count = [
                    soapy_cfg.io_cfg.iocfg_usrpb2xx.is_some(),
                    soapy_cfg.io_cfg.iocfg_limesdr.is_some(),
                    soapy_cfg.io_cfg.iocfg_sxceiver.is_some(),
                ].iter().filter(|&&x| x).count();
                if config_count != 1 {
                    return Err("soapysdr backend requires exactly one hardware configuration (iocfg_usrpb2xx, iocfg_limesdr, or iocfg_sxceiver)");
                }
            },
            PhyBackend::None => {}, // For testing
            PhyBackend::Undefined => {
                return Err("phy_io backend must be defined");
            },
        };

        // Sanity check on main carrier property fields in SYSINFO
        if self.phy_io.backend == PhyBackend::SoapySdr {
            let soapy_cfg = self.phy_io.soapysdr.as_ref().expect("SoapySdr config must be set for SoapySdr PhyIo");

            // Check consistency of RF frequency settings with TETRA stack settings
            // let Some(ul_freq) = soapy_cfg.ul_freq else {
            //     return Err("PhyIo SoapySdr UL frequency must be set for BS stack mode");
            // };
            // let Some(dl_freq) = soapy_cfg.dl_freq else {
            //     return Err("PhyIo SoapySdr DL frequency must be set for BS stack mode");
            // };

            let Ok(f1) = FreqInfo::from_dlul_freqs(soapy_cfg.dl_freq as u32, soapy_cfg.ul_freq as u32) else {
                return Err("Invalid PhyIo DL/UL frequencies");
            };
            let     Ok(f2) = FreqInfo::from_sysinfo_settings(
                    self.cell.freq_band, 
                    self.cell.main_carrier, 
                    self.cell.freq_offset, 
                    self.cell.duplex_spacing_setting,
                    self.cell.reverse_operation) else {
                return Err("Invalid cell info frequency settings");
            };

            if f1.band != f2.band {
                return Err("PhyIo Tx frequency band does not match cell info band");
            };
            if f1.carrier != f2.carrier {
                return Err("PhyIo Tx frequency carrier does not match cell info carrier");
            };
            if f1.freq_offset != f2.freq_offset {
                return Err("PhyIo Tx frequency offset does not match cell info offset");
            };
            if f1.reverse_operation != f2.reverse_operation {
                return Err("PhyIo Tx frequency reverse operation does not match cell info reverse operation");
            };
            if f1.duplex_spacing != f2.duplex_spacing {
                return Err("PhyIo Tx frequency duplex spacing does not match cell info duplex spacing");
            }
        }

        Ok(())
    }

    /// Useful shorthand to get scrambling code for the current configuration.
    pub fn scrambling_code(&self) -> u32 {
        scrambler::tetra_scramb_get_init(self.net.mcc, self.net.mnc, self.cell.colour_code)
    }
}

/// Mutable, stack-editable state (mutex-protected).
#[derive(Debug, Clone)]
#[derive(Default)]
pub struct StackState {
    pub cell_load_ca: u8,
}


/// Global shared configuration: immutable config + mutable state.
#[derive(Clone)]
pub struct SharedConfig {
    /// Read-only configuration (immutable after construction).
    cfg: Arc<StackConfig>,
    /// Mutable state guarded with RwLock (write by the stack, read by others).
    state: Arc<RwLock<StackState>>,
}

impl SharedConfig {
    pub fn new(mode: StackMode, mcc: u16, mnc: u16) -> Self {
        Self::from_config(StackConfig::new(mode, mcc, mnc))
    }

    pub fn from_config(cfg: StackConfig) -> Self {
        Self::from_parts(cfg, StackState::default())
    }

    pub fn from_parts(cfg: StackConfig, state: StackState) -> Self {
        
        // Check config for validity before returning the SharedConfig object
        match cfg.validate() {
            Ok(_) => {}
            Err(e) => panic!("Invalid stack configuration: {}", e),
        }

        Self {
            cfg: Arc::new(cfg),
            state: Arc::new(RwLock::new(state)),
        }
    }

    /// Access immutable config.
    pub fn config(&self) -> Arc<StackConfig> {
        Arc::clone(&self.cfg)
    }

    /// Read guard for mutable state.
    pub fn state_read(&self) -> std::sync::RwLockReadGuard<'_, StackState> {
        self.state.read().expect("StackState RwLock blocked")
    }

    /// Write guard for mutable state.
    pub fn state_write(&self) -> std::sync::RwLockWriteGuard<'_, StackState> {
        self.state.write().expect("StackState RwLock blocked")
    }
}
