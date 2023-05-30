use byteorder::BigEndian;
use byteorder::ByteOrder;
use modbus::tcp;
use modbus::Client;
use std::env;
use std::error::Error;
use std::time::Duration;
use tracing::debug;

pub const INVERTER_STATE_REGISTER: u16 = 40018;
pub const INVERTER_STATE_REGISTER_COUNT: u16 = 2;
// pub const INVERTER_STATE_STANDBY: u32 = 311_u32;
pub const INVERTER_STATE_START: u32 = 1467_u32;
pub const INVERTER_STATE_STOP: u32 = 1749_u32;

pub struct ModbusClientConfig {
    host: String,
    port: u16,
    unit_id: u8,
}

impl ModbusClientConfig {
    pub fn new(host: String, port: u16, unit_id: u8) -> Result<Self, Box<dyn Error>> {
        debug!(
            "ModbusClientConfig::new(host: {}, port: {}, unit_id: {})",
            host, port, unit_id
        );

        if host.is_empty() {
            return Err(Box::<dyn Error>::from(
                "Please set the ip address of your modbus device on your local network",
            ));
        }
        if port != 502 && (port < 49152) {
            return Err(Box::<dyn Error>::from("Please set the modbus port of your modbus device on your local network to its default 502, or anywhere between 49152 and 65535 if changed in the installer menu"));
        }

        Ok(Self {
            host,
            port,
            unit_id,
        })
    }

    pub fn from_env() -> Result<Self, Box<dyn Error>> {
        let host = env::var("MODBUS_HOST_IP").unwrap_or_else(|_| "127.0.0.1".to_string());
        let port: u16 = env::var("MODBUS_HOST_PORT")
            .unwrap_or_else(|_| "502".to_string())
            .parse()
            .unwrap_or(502);
        let unit_id: u8 = env::var("MODBUS_UNIT_ID")
            .unwrap_or_else(|_| "3".to_string())
            .parse()
            .unwrap_or(3);

        Self::new(host, port, unit_id)
    }
}

pub struct ModbusClient {
    config: ModbusClientConfig,
}

impl ModbusClient {
    pub fn new(config: ModbusClientConfig) -> ModbusClient {
        ModbusClient { config }
    }

    fn init_modbus_cfg(&self) -> modbus::Config {
        modbus::Config {
            tcp_port: self.config.port,
            modbus_uid: self.config.unit_id,
            tcp_connect_timeout: Some(Duration::new(20, 0)),
            ..Default::default()
        }
    }

    pub fn init_modbus_client(&self) -> std::io::Result<modbus::Transport> {
        let cfg = self.init_modbus_cfg();

        tcp::Transport::new_with_cfg(&self.config.host, cfg)
    }

    pub fn get_inverter_state(
        &self,
        modbus_client: &mut modbus::Transport,
    ) -> Result<u32, Box<dyn Error>> {
        let value_bytes = modbus_client
            .read_input_registers(INVERTER_STATE_REGISTER, INVERTER_STATE_REGISTER_COUNT)?;

        let value_bytes = value_bytes
            .iter()
            .flat_map(|b| b.to_be_bytes())
            .collect::<Vec<_>>();

        let value = BigEndian::read_u32(&value_bytes);

        Ok(value)
    }

    pub fn stop_inverter(
        &self,
        modbus_client: &mut modbus::Transport,
    ) -> Result<(), Box<dyn Error>> {
        let mut command_bytes: [u16; INVERTER_STATE_REGISTER_COUNT as usize] =
            [0; INVERTER_STATE_REGISTER_COUNT as usize];
        BigEndian::read_u16_into(&INVERTER_STATE_STOP.to_be_bytes(), &mut command_bytes);

        modbus_client.write_multiple_registers(INVERTER_STATE_REGISTER, &command_bytes)?;

        Ok(())
    }

    pub fn start_inverter(
        &self,
        modbus_client: &mut modbus::Transport,
    ) -> Result<(), Box<dyn Error>> {
        let mut command_bytes: [u16; INVERTER_STATE_REGISTER_COUNT as usize] =
            [0; INVERTER_STATE_REGISTER_COUNT as usize];
        BigEndian::read_u16_into(&INVERTER_STATE_START.to_be_bytes(), &mut command_bytes);

        modbus_client.write_multiple_registers(INVERTER_STATE_REGISTER, &command_bytes)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    #[ignore]
    fn get_sma_inverter_state() {
        let modbus_client = ModbusClient::new(
            ModbusClientConfig::new("192.168.195.3".to_string(), 502, 3).unwrap(),
        );
        let mut modbus_transport = modbus_client
            .init_modbus_client()
            .expect("Failed initializing transport");

        let inverter_state = modbus_client
            .get_inverter_state(&mut modbus_transport)
            .expect("Failed sending stop command");

        modbus_transport.close().expect("Failed closing transport");

        // • 311 = Standby
        // • 1467 = start
        // • 1749 = full stop (AC and DC sides)

        assert_eq!(inverter_state, INVERTER_STATE_START);
    }

    #[test]
    #[ignore]
    fn stop_sma_inverter() {
        let modbus_client = ModbusClient::new(
            ModbusClientConfig::new("192.168.195.3".to_string(), 502, 3).unwrap(),
        );
        let mut modbus_transport = modbus_client
            .init_modbus_client()
            .expect("Failed initializing transport");

        modbus_client
            .stop_inverter(&mut modbus_transport)
            .expect("Failed sending stop command");

        modbus_transport.close().expect("Failed closing transport");
    }

    #[test]
    #[ignore]
    fn start_sma_inverter() {
        let modbus_client = ModbusClient::new(
            ModbusClientConfig::new("192.168.195.3".to_string(), 502, 3).unwrap(),
        );
        let mut modbus_transport = modbus_client
            .init_modbus_client()
            .expect("Failed initializing transport");

        modbus_client
            .start_inverter(&mut modbus_transport)
            .expect("Failed sending stop command");

        modbus_transport.close().expect("Failed closing transport");
    }
}
