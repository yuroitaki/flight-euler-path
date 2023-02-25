mod domain;
mod properties;
mod routes;
mod service;
mod util;

use domain::cli::CliFields;
use properties::FlightItineraryServiceProperties;
use routes::start_server;
use service::flight_itinerary_service::FlightItineraryService;
use util::parse_config_file;

use eyre::Result;
use log::Level;
use std::{sync::Arc, str::FromStr};
use structopt::StructOpt;

#[tokio::main]
async fn main() -> Result<()> {
    // load command line argument which is config file location
    let cli_fields: CliFields = CliFields::from_args();
    let config: FlightItineraryServiceProperties = parse_config_file(&cli_fields.config_file)?;

    let logging_level = &&config.logging.default_level[..];
    simple_logger::init_with_level(Level::from_str(logging_level)?)?;

    let flight_itinerary_service = Arc::new(FlightItineraryService::new());
    start_server(config, flight_itinerary_service).await?;
    
    Ok(())
}
