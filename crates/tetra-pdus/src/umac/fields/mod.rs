// PDU fields
pub mod channel_allocation;
pub mod sysinfo_ext_services;
pub mod sysinfo_default_def_for_access_code_a;
pub mod ts_common_frames;
pub mod basic_slotgrant;

// MAC-governed prim fields (found across SAPs but referring to MAC)
pub mod endpoint_id;

pub type EventLabel = u16;
