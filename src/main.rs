// #![allow(unused_imports)]
// #![allow(unused_variables)]
#![allow(dead_code)]
// #![allow(unused_mut)]

#[cfg(test)]
mod testing;
mod config;
mod common;
mod entities;
mod saps;
 
use clap::Parser;

use common::debug::setup_logging_default;
use common::tdma_time::TdmaTime;
use common::messagerouter::MessageRouter;
use config::stack_config::*;
use config::toml_config;
use crate::entities::cmce::cmce_bs::CmceBs;
use crate::entities::mle::mle_bs_ms::Mle;
use crate::entities::phy::components::rxtxdev_soapysdr::RxTxDevSoapySdr;
use crate::entities::sndcp::sndcp_bs::Sndcp;
use crate::entities::lmac::lmac_bs::LmacBs;
use crate::entities::mm::mm_bs::MmBs;
use crate::entities::phy::phy_bs::PhyBs;
use crate::entities::llc::llc_bs_ms::Llc;
use crate::entities::umac::umac_bs::UmacBs;


/// Runs the full stack either forever or for a specified number of ticks.
fn run_stack(router: &mut MessageRouter, num_ticks: Option<usize>) {
    
    let mut ticks: usize = 0;

    loop {
        // Send tick_start event
        router.tick_all();
        
        // Deliver messages until queue empty
        while router.get_msgqueue_len() > 0{
            router.deliver_all_messages();
        }

        // Send tick_end event and process final messages
        router.tick_end();
        
        // Check if we should stop
        ticks += 1;
        if let Some(num_ticks) = num_ticks {
            if ticks >= num_ticks {
                break;
            }
        }
    }
}

/// Load configuration file
fn load_config_from_toml(cfg_path: &str) -> SharedConfig {
    match toml_config::from_file(cfg_path) {
        Ok(c) => c,
        Err(e) => {
            tracing::error!("Failed to load configuration from {}: {}", cfg_path, e);
            std::process::exit(1);
        }
    }
}

/// Start base station stack
fn build_bs_stack(cfg: &mut SharedConfig) -> MessageRouter {

    let mut router = MessageRouter::new(cfg.clone());

    // Add suitable Phy component based on PhyIo type
    match cfg.config().phy_io.backend {
        PhyBackend::SoapySdr => {
            let rxdev = RxTxDevSoapySdr::new(cfg);
            let phy = PhyBs::new(cfg.clone(), rxdev);
            router.register_entity(Box::new(phy));
        } 
        _ => {
            panic!("Unsupported PhyIo type: {:?}", cfg.config().phy_io.backend);
        }
    }
    
    // Add remaining components
    let lmac = LmacBs::new(cfg.clone());
    let umac = UmacBs::new(cfg.clone());
    let llc = Llc::new(cfg.clone());
    let mle = Mle::new(cfg.clone());
    let mm = MmBs::new(cfg.clone());
    let sndcp = Sndcp::new(cfg.clone());
    let cmce = CmceBs::new(cfg.clone());
    router.register_entity(Box::new(lmac));
    router.register_entity(Box::new(umac));
    router.register_entity(Box::new(llc));
    router.register_entity(Box::new(mle));
    router.register_entity(Box::new(mm));
    router.register_entity(Box::new(sndcp));
    router.register_entity(Box::new(cmce));
    
    // Init network time
    router.set_dl_time(TdmaTime::default());

    router
}


#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "TETRA BlueStation Stack",
    long_about = "Runs the TETRA BlueStation stack using the provided TOML configuration files"
)]


struct Args {
    /// Config file (required)
    #[arg(
        help = "TOML config with network/cell parameters",
    )]
    config: String,
}

fn main() {

    let args = Args::parse();

    setup_logging_default();
    let mut cfg = load_config_from_toml(&args.config);
    let mut router = match cfg.config().stack_mode {

        StackMode::Mon => {
            unimplemented!("Monitor mode is not implemented");
        },
        StackMode::Ms => {
            unimplemented!("MS mode is not implemented");
        },
        StackMode::Bs => {
            build_bs_stack(&mut cfg)
        }
    };

    run_stack(&mut router, None);
}
