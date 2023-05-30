use std::error::Error;

use chrono::Utc;
use jarvis_lib::spot_prices_state_client::SpotPricesStateClient;
use tracing::info;

use crate::modbus_client::{ModbusClient, INVERTER_STATE_START, INVERTER_STATE_STOP};

pub struct Controller {
    spot_prices_state_client: SpotPricesStateClient,
    modbus_client: ModbusClient,
}

impl Controller {
    pub fn new(
        spot_prices_state_client: SpotPricesStateClient,
        modbus_client: ModbusClient,
    ) -> Self {
        Self {
            spot_prices_state_client,
            modbus_client,
        }
    }

    pub async fn run(&self) -> Result<(), Box<dyn Error>> {
        let mut modbus_client = self.modbus_client.init_modbus_client()?;
        let inverter_state = self.modbus_client.get_inverter_state(&mut modbus_client)?;

        let now = Utc::now();

        let spot_price_state = self.spot_prices_state_client.read_state()?;
        if let Some(spot_price_state) = spot_price_state {
            if let Some(current_spot_price) = spot_price_state
                .future_spot_prices
                .iter()
                .find(|sp| sp.from <= now && now < sp.till)
            {
                let current_spot_price_total_price = current_spot_price.total_price();
                if current_spot_price_total_price < 0. {
                    // stop inverter
                    if inverter_state != INVERTER_STATE_STOP {
                        info!(
                            "Stopping inverter due to negative price - {} - and current inverter state {}",
                            current_spot_price_total_price,
                            inverter_state
                        );
                        self.modbus_client.stop_inverter(&mut modbus_client)?;
                    } else {
                        info!(
                            "Inverter already stopped, keeping it due to negative price - {}",
                            current_spot_price_total_price
                        );
                    }
                } else {
                    // start inverter
                    if inverter_state != INVERTER_STATE_START {
                        info!(
                        "Starting inverter due to positive price - {} - and current inverter state {}",
                        current_spot_price_total_price,
                        inverter_state
                    );
                        self.modbus_client.start_inverter(&mut modbus_client)?;
                    } else {
                        info!(
                            "Inverter already started, keeping it due to positive price - {}",
                            current_spot_price_total_price
                        );
                    }
                }
            } else {
                info!(
                    "No spot price valid for {}, won't adjust inverter state",
                    now
                );
            }
        } else {
            info!("No spot prices, won't adjust inverter state");
        }

        Ok(())
    }
}
