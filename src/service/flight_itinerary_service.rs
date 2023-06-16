use crate::domain::{
    error::{ApiError, ErrorBody},
    http::{FlightItineraryRequest, FlightItineraryResponse},
};

use eyre::Result;
use log::{error, info};
use petgraph::{prelude::DiGraphMap, Direction};

pub struct FlightItineraryService {}

impl FlightItineraryService {
    pub fn new() -> Self {
        FlightItineraryService {}
    }

    // validate payload to ensure it is valid to be processed
    fn check_payload(&self, payload: &FlightItineraryRequest) -> Result<(), ApiError> {
        if payload.flight_paths.is_empty() {
            let error_message =
                "Flight paths given is empty, please provide one with valid values.".to_string();
            error!("{error_message}");
            return Err(ApiError::EmptyFlightPaths(ErrorBody { error_message }));
        }
        if payload
            .flight_paths
            .iter()
            .any(|[origin, desti]| origin.to_lowercase() == desti.to_lowercase())
        {
            let error_message = "Some of the flight paths given has the same origin and destination airports, which is not valid.".to_string();
            error!("{error_message}");
            return Err(ApiError::InvalidFlightPath(ErrorBody { error_message }));
        }

        Ok(())
    }

    pub async fn calculate(
        &self,
        payload: FlightItineraryRequest,
    ) -> Result<FlightItineraryResponse, ApiError> {
        info!(
            "Received request to calculate flight itinerary: {:?}",
            payload
        );

        self.check_payload(&payload)?;

        // convert payload into a format that can be used to build a graph
        let flight_paths: Vec<(&str, &str)> = payload
            .flight_paths
            .iter()
            .map(|[origin, desti]| (&origin[..], &desti[..]))
            .collect();

        // build a directed graph
        let flight_graph = DiGraphMap::<&str, ()>::from_edges(&flight_paths);

        // placeholder for the output we are looking for
        let mut origin: Option<&str> = None;
        let mut desti: Option<&str> = None;

        // iterate through each node and calculate its in-degree and out-degree
        // given that we assume the input will form a graph with Eulerian path
        // the origin is the only node with (out-degree - in-degree) = 1
        // the destination is the only node with (out-degree - in-degree) = -1
        for node in flight_graph.nodes() {
            let outgoing_edge_count = flight_graph
                .edges_directed(node, Direction::Outgoing)
                .count();
            let incoming_edge_count = flight_graph
                .edges_directed(node, Direction::Incoming)
                .count();
            let diff = outgoing_edge_count as i32 - incoming_edge_count as i32;
            match diff {
                1 => {
                    if origin.is_none() {
                        origin = Some(node);
                    } else {
                        let error_message = "Failed to calculate the starting airport — more than 1 potential starting airport found, possibly because the flight paths don't form a single connected path.".to_string();
                        error!("{error_message}");
                        return Err(ApiError::NoStartingAirportDiscovered(ErrorBody {
                            error_message,
                        }));
                    }
                }
                -1 => {
                    if desti.is_none() {
                        desti = Some(node);
                    } else {
                        let error_message = "Failed to calculate the ending airport — more than 1 potential ending airport found, possibly because the flight paths don't form a single connected path.".to_string();
                        error!("{error_message}");
                        return Err(ApiError::NoEndingAirportDiscovered(ErrorBody {
                            error_message,
                        }));
                    }
                }
                0 => (),
                _ => {
                    let error_message = "Failed to calculate the starting/ending airport — some non starting/ending airport has invalid paths from them".to_string();
                    error!("{error_message}");
                    return Err(ApiError::InvalidFlightPath(ErrorBody { error_message }));
                }
            }
        }
        let starting_airport = match origin {
            Some(airport) => airport,
            None => {
                let error_message = "Failed to calculate the starting airport — possibly due to the starting and ending airport being the same, or the flight paths don't form a single connected path.".to_string();
                error!("{error_message}");
                return Err(ApiError::NoStartingAirportDiscovered(ErrorBody {
                    error_message,
                }));
            }
        };
        let ending_airport = match desti {
            Some(airport) => airport,
            None => {
                let error_message = "Failed to calculate the ending airport — possibly due to the starting and ending airport being the same, or the flight paths don't form a single connected path.".to_string();
                error!("{error_message}");
                return Err(ApiError::NoEndingAirportDiscovered(ErrorBody {
                    error_message,
                }));
            }
        };

        info!(
            "Successfully discovered starting airport: {} and ending airport: {}!",
            starting_airport, ending_airport
        );
        let itinerary = vec![starting_airport.to_string(), ending_airport.to_string()];
        Ok(FlightItineraryResponse { itinerary })
    }
}

#[cfg(test)]
mod test {
    use crate::domain::error::ApiError;

    use super::{FlightItineraryRequest, FlightItineraryService, Result};

    #[test]
    fn test_check_payload_with_valid_input() {
        let service = FlightItineraryService::new();
        let payload = FlightItineraryRequest {
            flight_paths: vec![
                ["A".to_string(), "B".to_string()],
                ["B".to_string(), "C".to_string()],
            ],
        };
        assert!(
            service.check_payload(&payload).is_ok(),
            "Failed to check valid payload"
        );
    }

    #[test]
    fn test_check_payload_with_empty_input() {
        let service = FlightItineraryService::new();
        let payload = FlightItineraryRequest {
            flight_paths: vec![],
        };
        let result = service.check_payload(&payload);
        assert!(result.is_err(), "Failed to catch empty input.");
    }

    #[test]
    fn test_check_payload_with_duplicate_origin_and_desti() {
        let service = FlightItineraryService::new();
        let payload = FlightItineraryRequest {
            flight_paths: vec![
                ["A".to_string(), "A".to_string()],
                ["B".to_string(), "C".to_string()],
            ],
        };
        let result = service.check_payload(&payload);
        assert!(result.is_err(), "Failed to catch duplicated input.");
    }

    #[tokio::test]
    async fn test_calculate_with_simple_payload() -> Result<(), ApiError> {
        let service = FlightItineraryService::new();
        let payload = FlightItineraryRequest {
            flight_paths: vec![["GBB".to_string(), "SGP".to_string()]],
        };
        let result = service.calculate(payload).await?;
        assert_eq!(result.itinerary, vec!["GBB".to_string(), "SGP".to_string()]);
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_with_payload() -> Result<(), ApiError> {
        let service = FlightItineraryService::new();
        let payload = FlightItineraryRequest {
            flight_paths: vec![
                ["MYS".to_string(), "SGP".to_string()],
                ["GBB".to_string(), "BKK".to_string()],
                ["GSO".to_string(), "MYS".to_string()],
                ["BKK".to_string(), "GSO".to_string()],
            ],
        };
        let result = service.calculate(payload).await?;
        assert_eq!(result.itinerary, vec!["GBB".to_string(), "SGP".to_string()]);
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_with_payload_with_cycle() -> Result<(), ApiError> {
        let service = FlightItineraryService::new();
        let payload = FlightItineraryRequest {
            flight_paths: vec![
                ["MYS".to_string(), "SGP".to_string()],
                ["MYS".to_string(), "BKK".to_string()],
                ["SGP".to_string(), "BKK".to_string()],
                ["BKK".to_string(), "MYS".to_string()],
            ],
        };
        let result = service.calculate(payload).await?;
        assert_eq!(result.itinerary, vec!["MYS".to_string(), "BKK".to_string()]);
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_with_more_than_one_origin() -> Result<(), ApiError> {
        let service = FlightItineraryService::new();
        let payload = FlightItineraryRequest {
            flight_paths: vec![
                ["MYS".to_string(), "SGP".to_string()],
                ["MYS".to_string(), "BKK".to_string()],
                ["SGP".to_string(), "BKK".to_string()],
                ["BKK".to_string(), "MYS".to_string()],
                ["SGP".to_string(), "MYS".to_string()],
                ["REN".to_string(), "BKK".to_string()],
            ],
        };
        let result = service.calculate(payload).await;
        println!("{:?}", result);
        assert!(
            result.is_err(),
            "Failed to catch invalid input with more than one origin."
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_with_more_than_one_desti() -> Result<(), ApiError> {
        let service = FlightItineraryService::new();
        let payload = FlightItineraryRequest {
            flight_paths: vec![
                ["MYS".to_string(), "SGP".to_string()],
                ["MYS".to_string(), "BKK".to_string()],
                ["SGP".to_string(), "BKK".to_string()],
                ["BKK".to_string(), "MYS".to_string()],
                ["SGP".to_string(), "MYS".to_string()],
                ["REN".to_string(), "BEN".to_string()],
            ],
        };
        let result = service.calculate(payload).await;
        println!("{:?}", result);
        assert!(
            result.is_err(),
            "Failed to catch invalid input with more than one desti."
        );
        Ok(())
    }

    #[tokio::test]
    async fn test_calculate_with_eulerian_cycle() -> Result<(), ApiError> {
        let service = FlightItineraryService::new();
        let payload = FlightItineraryRequest {
            flight_paths: vec![
                ["MYS".to_string(), "SGP".to_string()],
                ["SGP".to_string(), "BKK".to_string()],
                ["BKK".to_string(), "MYS".to_string()],
            ],
        };
        let result = service.calculate(payload).await;
        println!("{:?}", result);
        assert!(
            result.is_err(),
            "Failed to catch invalid input with Eulerian cycle."
        );
        Ok(())
    }
}
