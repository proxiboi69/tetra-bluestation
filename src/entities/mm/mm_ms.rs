use crate::config::stack_config::SharedConfig;
use crate::common::messagerouter::MessageQueue;
use crate::saps::sapmsg::{SapMsg, SapMsgInner};
use crate::entities::mm::enums::mm_pdu_type_dl::MmPduTypeDl;
use crate::entities::TetraEntityTrait;
use crate::common::tetra_common::{Sap};
use crate::common::tetra_entities::TetraEntity;
use crate::unimplemented_log;

pub struct MmMs {
    // config: Option<SharedConfig>,
    config: SharedConfig,
}

impl MmMs {
    pub fn new(config: SharedConfig) -> Self {
        Self { config }
    }

    fn rx_lmm_mle_unitdata_ind(&mut self, _queue: &mut MessageQueue, mut message: SapMsg) {

        // unimplemented_log!("rx_lmm_mle_unitdata_ind not implemented for MM component");
        let SapMsgInner::LmmMleUnitdataInd(prim) = &mut message.msg else {panic!()};

        let Some(bits) = prim.sdu.peek_bits(4) else {
            tracing::warn!("insufficient bits: {}", prim.sdu.dump_bin());
            return;
        };

        let Ok(pdu_type) = MmPduTypeDl::try_from(bits) else {
            tracing::warn!("invalid pdu type: {} in {}", bits, prim.sdu.dump_bin());
            return;
        };


        match pdu_type {
            MmPduTypeDl::DOtar => 
                unimplemented_log!("DOtar not implemented"),
            MmPduTypeDl::DAuthentication => 
                unimplemented_log!("DAuthentication not implemented"),
            MmPduTypeDl::DCkChangeDemand => 
                unimplemented_log!("DCkChangeDemand not implemented"),
            MmPduTypeDl::DDisable => 
                unimplemented_log!("DDisable not implemented"),
            MmPduTypeDl::DEnable => 
                unimplemented_log!("DEnable not implemented"),
            MmPduTypeDl::DLocationUpdateAccept => 
                unimplemented_log!("DLocationUpdateAccept not implemented"),
            MmPduTypeDl::DLocationUpdateCommand => 
                unimplemented_log!("DLocationUpdateCommand not implemented"),
            MmPduTypeDl::DLocationUpdateReject => 
                unimplemented_log!("DLocationUpdateReject not implemented"),
            MmPduTypeDl::DLocationUpdateProceeding => 
                unimplemented_log!("DLocationUpdateProceeding not implemented"),
            MmPduTypeDl::DAttachDetachGroupIdentity => 
                unimplemented_log!("DAttachDetachGroupIdentity not implemented"),
            MmPduTypeDl::DAttachDetachGroupIdentityAcknowledgement => 
                unimplemented_log!("DAttachDetachGroupIdentityAcknowledgement not implemented"),
            MmPduTypeDl::DMmStatus => 
                unimplemented_log!("DMmStatus not implemented"),
            MmPduTypeDl::MmPduFunctionNotSupported => 
                unimplemented_log!("MmPduFunctionNotSupported not implemented"),
        };
    }
}

impl TetraEntityTrait for MmMs {

    fn entity(&self) -> TetraEntity {
        TetraEntity::Mm
    }

    fn set_config(&mut self, config: SharedConfig) {
        self.config = config;
    }

    fn rx_prim(&mut self, queue: &mut MessageQueue, message: SapMsg) {
        
        tracing::debug!("rx_prim: {:?}", message);

        // There is only one SAP for MM
        assert!(message.sap == Sap::LmmSap);
        
        match message.msg {
            SapMsgInner::LmmMleUnitdataInd(_) => {
                self.rx_lmm_mle_unitdata_ind(queue, message);
            }
            _ => { panic!(); }
        }
    }
}
