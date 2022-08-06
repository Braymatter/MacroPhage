use std::{
    net::{SocketAddr, UdpSocket},
    time::{SystemTime, UNIX_EPOCH},
};

use bevy::prelude::*;
use bevy_renet::{
    renet::{ClientAuthentication, RenetClient, RenetConnectionConfig},
    RenetClientPlugin,
};
use iyes_loopless::prelude::*;

use super::{ClientChannel, ConnectRequestEvent, ServerChannel, ServerCommand, PROTOCOL_ID};

pub struct MacroClientPlugin;
impl Plugin for MacroClientPlugin {
    fn build(&self, app: &mut App) {
        info!("Building Client Plugin");
        app.add_system(connect_to_client);
        app.add_event::<ConnectRequestEvent>();
        app.add_plugin(RenetClientPlugin);
        app.insert_resource(MacroClientState { host: None });
        app.add_system(receive_message.run_if_resource_exists::<RenetClient>());
    }
}

pub struct MacroClientState {
    pub host: Option<SocketAddr>,
}

fn new_renet_client(target: SocketAddr) -> RenetClient {
    let socket = UdpSocket::bind("[::1]:5001").unwrap();
    let connection_config = client_connection_config();
    let current_time = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap();
    let client_id = current_time.as_millis() as u64;
    let authentication = ClientAuthentication::Unsecure {
        client_id,
        protocol_id: PROTOCOL_ID,
        server_addr: target,
        user_data: None,
    };

    // Warning: This will fuck up if two clients try to connect to the server in the same second potentially
    RenetClient::new(
        current_time,
        socket,
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("It went to 88")
            .as_secs(),
        connection_config,
        authentication,
    )
    .unwrap()
}

pub fn client_connection_config() -> RenetConnectionConfig {
    RenetConnectionConfig {
        send_channels_config: ClientChannel::channels_config(),
        receive_channels_config: ServerChannel::channels_config(),
        ..Default::default()
    }
}

fn connect_to_client(
    mut conn_reqs: EventReader<ConnectRequestEvent>,
    mut client: ResMut<MacroClientState>,
    mut cmds: Commands,
) {
    if conn_reqs.is_empty() {
        return;
    }

    let ev = conn_reqs.iter().last().unwrap();
    client.host = Some(ev.socket);

    info!("Inserting Renet Client resource!");
    cmds.insert_resource(new_renet_client(ev.socket));
}

fn receive_message(mut client: ResMut<RenetClient>) {
    while let Some(message) = client.receive_message(ServerChannel::ServerMessages.id()) {
        match bincode::deserialize::<ServerCommand>(&message) {
            Ok(msg) => {
                info!("Received {msg:#?}");
            }
            Err(e) => {
                error!("{}", e);
            }
        }
    }
}
