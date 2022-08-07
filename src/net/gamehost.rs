use crate::net::{RequestProfileCmd, ServerChannel, ClientChannel};
use bevy::prelude::*;
use bevy_renet::{
    renet::{
        RenetConnectionConfig, RenetError, RenetServer, ServerAuthentication, ServerConfig,
        ServerEvent,
    },
    RenetServerPlugin,
};
use bincode::Options;
use iyes_loopless::prelude::*;
use socket2::{Domain, Protocol, Socket, Type};
use std::{net::SocketAddr, time::SystemTime};

pub struct GameHostPlugin;
impl Plugin for GameHostPlugin {
    fn build(&self, app: &mut App) {
        info!("Building Game Host Plugin");
        app.insert_resource(HostNetworkResource {});
        app.add_loopless_state(HostState::Inactive);
        app.add_plugin(RenetServerPlugin);
        app.insert_resource(build_host_server());
        app.add_system(panic_on_error_system);
        app.add_system(renet_event_logger.run_if_resource_exists::<RenetServer>());
    }
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum HostState {
    Inactive,
    Pregame,
    Playing,
    PostGame,
}

#[derive(Debug, Clone)]
pub struct HostNetworkResource {}

// If any error is found we just panic
// This should probably change the game state to the main screen
// and open a pop up with a network error
fn panic_on_error_system(mut renet_error: EventReader<RenetError>) {
    for e in renet_error.iter() {
        error!("{}", e);
    }
}

fn build_host_server() -> RenetServer {
    let socket = Socket::new(Domain::IPV6, Type::DGRAM, Some(Protocol::UDP)).unwrap();
    socket
        .set_only_v6(false)
        .expect("Could not set only v6 in build_host_server");

    let sock2: SocketAddr = "[::1]:5000".parse().unwrap();

    socket
        .bind(&sock2.into())
        .expect("Could not bind server socket");

    //socket.bind(&sock2_v4.into()).expect("Could not bind ipv4 localhost to socket");

    let connection_config = RenetConnectionConfig {
        send_channels_config: ServerChannel::channels_config(),
        receive_channels_config: ClientChannel::channels_config(),
        ..Default::default()
    };

    let server_config = ServerConfig::new(
        64,
        super::PROTOCOL_ID,
        sock2,
        ServerAuthentication::Unsecure,
    );

    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();

    info!("Instantiating RenetServer on {}", sock2.to_string());
    RenetServer::new(
        current_time,
        server_config,
        connection_config,
        socket.into(),
    )
    .unwrap()
}

fn renet_event_logger(mut server: ResMut<RenetServer>, mut server_evs: EventReader<ServerEvent>) {
    for event in server_evs.iter() {
        match event {
            ServerEvent::ClientConnected(id, _userdata) => {
                info!("Client Connected! Assigned id: {}", id);

                let message = serde_json::to_vec(&RequestProfileCmd { id: *id }).unwrap();
                server.send_message(
                    *id,
                    ServerChannel::ServerMessages.id(),
                    message,
                );
            }
            ServerEvent::ClientDisconnected(id) => {
                warn!("Client Disconnected: {}", id);
            }
        }
    }
}
