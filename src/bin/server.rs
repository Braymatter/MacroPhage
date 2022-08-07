use std::net::IpAddr;

use bevy::{log::LogPlugin, prelude::*};
use bevy_renet::RenetServerPlugin;
use macrophage::net::{gamehost::GameHostPlugin, IpRes};



#[tokio::main]
async fn main() {
    let mut app = App::new();

    if let Some(ip) = public_ip::addr().await{
        app.insert_resource(IpRes{public_ip: ip});
        info!("Found Public Ip: {}", ip);
    }else{
        panic!("Could not fetch public ip, cannot mount server");
    }


    app.add_plugins(MinimalPlugins);
    app.add_plugin(LogPlugin);
    app.add_plugin(RenetServerPlugin);
    app.add_plugin(GameHostPlugin);

    app.run();
}
