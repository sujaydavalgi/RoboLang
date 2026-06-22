//! One peer message delivered over the fleet mesh or remote relay path.
//!
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PeerDelivery {
    pub from_robot: String,
    pub to_robot: String,
    pub topic: String,
    pub step: String,
    pub delivered: bool,
}
