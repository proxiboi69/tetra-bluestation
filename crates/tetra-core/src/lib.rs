//! Core utilities for TETRA BlueStation
//!
//! This crate provides fundamental types and utilities used across the TETRA stack:
//! - BitBuffer for bit-level PDU manipulation
//! - TdmaTime for TDMA frame timing
//! - Address types (ISSI, GSSI, etc.)
//! - PHY types (PhyBlockNum, BurstType, etc.)
//! - Common macros and debug utilities

pub mod address;
pub mod bitbuffer;
pub mod debug;
pub mod freqs;
pub mod pdu_parse_error;
pub mod phy_types;
pub mod tdma_time;
pub mod tetra_common;
pub mod tetra_entities;
pub mod typed_pdu_fields;

// Re-export commonly used items
pub use address::*;
pub use bitbuffer::BitBuffer;
pub use pdu_parse_error::PduParseErr;
pub use phy_types::*;
pub use tdma_time::TdmaTime;
pub use tetra_common::*;
