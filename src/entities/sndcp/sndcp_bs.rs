use crate::common::messagerouter::MessageQueue;
use crate::common::tetra_common::Sap;
use crate::common::tetra_entities::TetraEntity;
use crate::config::stack_config::SharedConfig;
use crate::entities::TetraEntityTrait;
use crate::saps::sapmsg::SapMsg;
use crate::unimplemented_log;

pub struct Sndcp {
    // config: Option<SharedConfig>,
    config: SharedConfig,
}

impl Sndcp {
    pub fn new(config: SharedConfig) -> Self {
        Self { config }
    }
}

impl TetraEntityTrait for Sndcp {
    fn entity(&self) -> TetraEntity {
        TetraEntity::Sndcp
    }

    fn rx_prim(&mut self, _queue: &mut MessageQueue, message: SapMsg) {
        tracing::debug!("rx_prim: {:?}", message);

        // There is only one SAP for SNDCP
        // OR.. SN-SAP? TODO FIXME check docs
        assert!(message.sap == Sap::TlpdSap);
        unimplemented_log!("sndcp not implemented");
    }
}
