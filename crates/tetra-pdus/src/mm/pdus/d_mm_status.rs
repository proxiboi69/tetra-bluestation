use core::fmt;

use tetra_core::typed_pdu_fields::delimiters;
use tetra_core::{BitBuffer, pdu_parse_error::PduParseErr};

use crate::mm::enums::mm_pdu_type_dl::MmPduTypeDl;
use crate::mm::enums::status_downlink::StatusDownlink;

/// Representation of the D-MM STATUS PDU (Clause 16.9.2.5.1).
/// The infrastructure sends this message to the MS to request or indicate/reject a change of an operation mode.
/// Response expected: -/U-MM STATUS
/// Response to: -/U-MM STATUS
///
/// Gateway sub-PDUs (EN 300 396-5, Annex B) are encoded as variants of this PDU with
/// status_downlink values 16..24.
#[derive(Debug)]
pub struct DMmStatus {
    /// 6 bits — identifies the sub-PDU type
    pub status_downlink: StatusDownlink,
    /// For AcceptanceToContinueDmGatewayOperation: whether the SwMI still has the DM-MS address set (1 bit)
    pub retained_dm_ms_address_set: Option<bool>,
    /// For AcceptanceToStart / AcceptanceOfDmMsAddresses: rejected DM-MS SSIs (0 = all accepted)
    pub rejected_dm_ms_addresses: Vec<u32>,
}

impl DMmStatus {
    /// Create a simple gateway response (no address list, no retained flag).
    /// Suitable for: AcceptanceToStop, RejectionToStart, RejectionToContinue, etc.
    pub fn new_simple(status_downlink: StatusDownlink) -> Self {
        DMmStatus {
            status_downlink,
            retained_dm_ms_address_set: None,
            rejected_dm_ms_addresses: Vec::new(),
        }
    }

    /// Create AcceptanceToContinueDmGatewayOperation with retained flag.
    pub fn new_acceptance_continue(retained: bool) -> Self {
        DMmStatus {
            status_downlink: StatusDownlink::AcceptanceToContinueDmGatewayOperation,
            retained_dm_ms_address_set: Some(retained),
            rejected_dm_ms_addresses: Vec::new(),
        }
    }

    /// Create AcceptanceToStartDmGatewayOperation or AcceptanceOfDmMsAddresses.
    /// Empty rejected list means all addresses accepted.
    pub fn new_acceptance_with_addresses(status_downlink: StatusDownlink, rejected: Vec<u32>) -> Self {
        DMmStatus {
            status_downlink,
            retained_dm_ms_address_set: None,
            rejected_dm_ms_addresses: rejected,
        }
    }

    /// Serialize this PDU into the given BitBuffer.
    pub fn to_bitbuf(&self, buffer: &mut BitBuffer) -> Result<(), PduParseErr> {
        // PDU Type (4 bits)
        buffer.write_bits(MmPduTypeDl::DMmStatus.into_raw(), 4);
        // Status downlink (6 bits)
        buffer.write_bits(self.status_downlink.into_raw(), 6);

        match self.status_downlink {
            StatusDownlink::AcceptanceToContinueDmGatewayOperation => {
                // Retained DM-MS address set (1 bit) + reserved (7 bits)
                let retained = self.retained_dm_ms_address_set.unwrap_or(false);
                buffer.write_bits(if retained { 1 } else { 0 }, 1);
                buffer.write_bits(0, 7); // reserved
            }
            StatusDownlink::AcceptanceToStartDmGatewayOperation | StatusDownlink::AcceptanceOfDmMsAddresses => {
                // Reserved (8 bits)
                buffer.write_bits(0, 8);
                // Number of rejected DM-MS addresses (4 bits)
                let count = self.rejected_dm_ms_addresses.len() as u64;
                buffer.write_bits(count, 4);
                // Each rejected DM-MS address: address_type (2 bits, 0=SSI) + SSI (24 bits)
                for &ssi in &self.rejected_dm_ms_addresses {
                    buffer.write_bits(0, 2); // address_type = SSI
                    buffer.write_bits(ssi as u64, 24);
                }
            }
            _ => {
                // RejectionToStart, RejectionToContinue, AcceptanceToStop,
                // CommandToRemove, CommandToChangeRegistrationLabel, CommandToStop, etc.
                // All have: reserved (8 bits) only
                buffer.write_bits(0, 8);
            }
        }

        // Terminating o-bit = 0 (no optional Type 3/4 fields follow)
        delimiters::write_obit(buffer, 0);

        Ok(())
    }

    /// Parse from BitBuffer. Not fully implemented — the BTS sends D-MM STATUS but does not receive it.
    pub fn from_bitbuf(_buffer: &mut BitBuffer) -> Result<Self, PduParseErr> {
        Err(PduParseErr::NotImplemented {
            field: Some("D-MM STATUS from_bitbuf (BTS-side only)"),
        })
    }
}

impl fmt::Display for DMmStatus {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "DMmStatus {{ status_downlink: {} ", self.status_downlink)?;
        if let Some(retained) = self.retained_dm_ms_address_set {
            write!(f, "retained_dm_ms_address_set: {} ", retained)?;
        }
        if !self.rejected_dm_ms_addresses.is_empty() {
            write!(f, "rejected_dm_ms_addresses: {:?} ", self.rejected_dm_ms_addresses)?;
        }
        write!(f, "}}")
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_acceptance_to_start_no_rejections() {
        let pdu = DMmStatus::new_acceptance_with_addresses(StatusDownlink::AcceptanceToStartDmGatewayOperation, Vec::new());
        let mut buf = BitBuffer::new_autoexpand(32);
        pdu.to_bitbuf(&mut buf).unwrap();
        buf.seek(0);
        // 4 (pdu_type) + 6 (status_downlink) + 8 (reserved) + 4 (count=0) + 1 (o-bit) = 23 bits
        assert_eq!(buf.get_len(), 23);
        let pdu_type = buf.read_field(4, "pdu_type").unwrap();
        assert_eq!(pdu_type, MmPduTypeDl::DMmStatus.into_raw());
        let status = buf.read_field(6, "status_downlink").unwrap();
        assert_eq!(status, 16); // AcceptanceToStartDmGatewayOperation
        let reserved = buf.read_field(8, "reserved").unwrap();
        assert_eq!(reserved, 0);
        let count = buf.read_field(4, "count").unwrap();
        assert_eq!(count, 0);
    }

    #[test]
    fn test_acceptance_to_continue_retained() {
        let pdu = DMmStatus::new_acceptance_continue(true);
        let mut buf = BitBuffer::new_autoexpand(32);
        pdu.to_bitbuf(&mut buf).unwrap();
        buf.seek(0);
        // 4 + 6 + 1 (retained=1) + 7 (reserved) + 1 (o-bit) = 19 bits
        assert_eq!(buf.get_len(), 19);
        let _ = buf.read_field(4, "pdu_type").unwrap();
        let status = buf.read_field(6, "status_downlink").unwrap();
        assert_eq!(status, 18); // AcceptanceToContinueDmGatewayOperation
        let retained = buf.read_field(1, "retained").unwrap();
        assert_eq!(retained, 1);
    }

    #[test]
    fn test_acceptance_to_stop_simple() {
        let pdu = DMmStatus::new_simple(StatusDownlink::AcceptanceToStopDmGatewayOperation);
        let mut buf = BitBuffer::new_autoexpand(32);
        pdu.to_bitbuf(&mut buf).unwrap();
        buf.seek(0);
        // 4 + 6 + 8 (reserved) + 1 (o-bit) = 19 bits
        assert_eq!(buf.get_len(), 19);
        let _ = buf.read_field(4, "pdu_type").unwrap();
        let status = buf.read_field(6, "status_downlink").unwrap();
        assert_eq!(status, 20); // AcceptanceToStopDmGatewayOperation
    }
}
