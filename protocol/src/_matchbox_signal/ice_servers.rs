use matchbox_socket::{RtcIceServerConfig, RtcIceServerConfigs};

use crate::_const::get_relay_domain;

pub fn ice_servers() -> RtcIceServerConfigs {
    RtcIceServerConfigs {
        configs: vec![
            RtcIceServerConfig {
                urls: vec![
                    // "stun:stun.l.google.com:19302".to_string(),
                    // "stun:stun1.l.google.com:19302".to_string(),
                    // "stun:freestun.net:3478".to_string(),
                    format!("stun:{}:31232", get_relay_domain()).to_string(),
                    // 
                ],
                username: Default::default(),
                credential: Default::default(),
            },
            // RtcIceServerConfig {
            //     urls: vec![
            //         "turn:freestun.net:3478".to_string(),
            //     ],
            //     username: Some("free".to_string()),
            //     credential: Some("free".to_string()),
            // },
            // RtcIceServerConfig {
            //     urls: vec![
            //         format!("turn:{}:31234", IROH_RELAY_DOMAIN).to_string(),
            //         format!("stun:{}:31233", IROH_RELAY_DOMAIN).to_string(),
            //     ],
            //     username: Some("free3".to_string()),
            //     credential: Some("free4".to_string()),
            // }
        ],
    }
}
