use matchbox_socket::{RtcIceServerConfig, RtcIceServerConfigs};

pub fn ice_servers() -> RtcIceServerConfigs {
    RtcIceServerConfigs {
        configs: vec![
            RtcIceServerConfig {
                urls: vec![
                    "stun:stun.l.google.com:19302".to_string(),
                    "stun:stun1.l.google.com:19302".to_string(),
                    // "stun:freestun.net:3478".to_string(),
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
            // }
        ]
    }
}