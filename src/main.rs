mod controller;
mod modbus_client;

use controller::Controller;
use jarvis_lib::spot_prices_state_client::{SpotPricesStateClient, SpotPricesStateClientConfig};
use modbus_client::{ModbusClient, ModbusClientConfig};

#[tokio::main]
pub async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .json()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();

    let spot_prices_state_client =
        SpotPricesStateClient::new(SpotPricesStateClientConfig::from_env().await?);
    let modbus_client = ModbusClient::new(ModbusClientConfig::from_env()?);

    let controller = Controller::new(spot_prices_state_client, modbus_client);

    controller.run().await?;

    Ok(())
}

#[cfg(test)]
#[ctor::ctor]
fn init() {
    tracing_subscriber::fmt()
        .with_env_filter(tracing_subscriber::EnvFilter::from_default_env())
        .init();
}
