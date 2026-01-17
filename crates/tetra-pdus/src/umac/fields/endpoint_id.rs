
/// The endpoint identifiers between the MLE and LLC, and between the LLC and MAC, refer to the MAC resource that is
/// currently used for that service. These identifiers may be local. There shall be a unique correspondence between the
/// endpoint identifier and the physical allocation (timeslot or timeslots) used in the MAC. (This correspondence is known
/// only within the MAC.) More than one advanced link may use one MAC resource.
/// In the current implementation, the endpoint_id is just the timeslot number used by the MAC. 
pub type EndpointId = u8;