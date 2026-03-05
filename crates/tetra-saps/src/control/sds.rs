/// SDS data routing between CMCE SDS subentity and Brew entity
#[derive(Debug, Clone)]
pub struct CmceSdsData {
    /// Source ISSI (calling party)
    pub source_issi: u32,
    /// Destination ISSI (called party)
    pub dest_issi: u32,
    /// Short data type identifier (0-3)
    pub short_data_type_identifier: u8,
    /// SDS payload data (raw bytes from user_defined_data_1/2/3/4)
    pub data: Vec<u8>,
    /// Length in bits (for type 4, from length_indicator)
    pub length_bits: u16,
}
