use std::net::{IpAddr, Ipv4Addr, Ipv6Addr, SocketAddr};
use std::str::FromStr;

use bevy::prelude::*;
use bevy_punchthrough::client::{PunchthroughClientPlugin, PunchthroughEvent, RequestSwap};
use bevy_punchthrough::renet_plugin::renet::RenetError;
use dns_lookup::lookup_host;
use leafwing_input_manager::prelude::ActionState;

use crate::game::controller::PlayerAction;

pub struct PunchthroughPlugin;

impl Plugin for PunchthroughPlugin {
    fn build(&self, app: &mut App) {
        app.add_system(renet_test_controls);
        app.add_system(panic_on_error_system);

        build_punchthrough_plugin(app);
    }
}

pub fn monitor_punchthrough_events(mut pt_events: EventReader<PunchthroughEvent>) {
    for ev in pt_events.iter() {
        info!("Received PT Event {ev:#?}");
    }
}

// If any error is found we just panic
// This should probably change the game state to the main screen
// and open a pop up with a network error
pub fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        panic!("{}", e);
    }
}

pub fn renet_test_controls(
    actions: Query<&ActionState<PlayerAction>>,
    mut request_host_ev: EventWriter<RequestSwap>,
) {
    let actions = actions.single();
    if actions.just_pressed(PlayerAction::HotKey1) {
        println!("Requesting New Host Lobby");
        request_host_ev.send(RequestSwap::HostLobby);
    }
}

fn build_punchthrough_plugin(app: &mut App) {
    let local_ip = Ipv6Addr::new(0, 0, 0, 0, 0, 0, 0, 1);
    let local_socket: SocketAddr = SocketAddr::new(IpAddr::from(local_ip), 5001);
    let pt_server_ip: IpAddr = match lookup_host("matchmaking.gimgam.games") {
        Ok(ips) => {
            if ips.is_empty() {
                error!("No IP's Found from DNS Lookup to gimgam.games!! Aborting Punchthrough initialization");
            }

            let ip = ips[0];

            if ip.is_ipv4() {
                let ipv6 = Ipv4Addr::from_str(ip.to_string().as_str())
                    .expect("Could not convert ip to ipv4 from string")
                    .to_ipv6_compatible();

                info!(
                    "Converted DNS Lookup IP {} to ipv6 {}",
                    ip.to_string(),
                    ipv6.to_string()
                );
                IpAddr::V6(ipv6)
            } else {
                ip
            }
        }
        Err(e) => {
            error!("Could not lookup dns entry for gimgam.games!! Aborting Punchthrough initialization. Error: {e:#?}");
            return;
        }
    };
    info!("Discovered matchmaking.gimgam.games ip: {pt_server_ip:#?}");
    app.add_plugin(PunchthroughClientPlugin {
        local_socket,
        punchthrough_server: SocketAddr::new(pt_server_ip, 5000),
    });
}
