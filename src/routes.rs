use crate::{
    domain::{
        error::{ErrorBody, ApiError},
        http::FlightItineraryRequest
    },
    properties::FlightItineraryServiceProperties,
    service::flight_itinerary_service::FlightItineraryService,
};

use axum::{
    Router,
    extract::{
        Json as ExtractJson,
        rejection::JsonRejection,
    },
    http::StatusCode,
    response::{IntoResponse, Json, Response},
    routing::post,
};
use eyre::Result;
use log::{error, info};
use std::{any::Any, net::SocketAddr, sync::Arc};
use tower_http::catch_panic::CatchPanicLayer;

fn route(flight_itinerary_service: Arc<FlightItineraryService>) -> Router {
    Router::new().route(
        "/compute",
        post(|request: Result<ExtractJson<FlightItineraryRequest>, JsonRejection>| async move {
            let payload = match request {
                Ok(payload) => payload.0,
                Err(err) => { 
                    error!("Failed to parse payload with error: {err}");
                    return (StatusCode::BAD_REQUEST, err.to_string()).into_response();
                }
            };

            match flight_itinerary_service.calculate(payload).await {
                Ok(itinerary) => Json(itinerary).into_response(),
                Err(ApiError::EmptyFlightPaths(err))
                | Err(ApiError::InvalidFlightPath(err))
                | Err(ApiError::NoEndingAirportDiscovered(err))
                | Err(ApiError::NoStartingAirportDiscovered(err)) => (StatusCode::BAD_REQUEST, Json(err)).into_response(),
            }
        }),
    )
}

fn handle_panic(_: Box<dyn Any + Send + 'static>) -> Response {
    let error_message = "Something went wrong. Please try again later.".to_string();
    error!("{error_message}");
    (
        StatusCode::INTERNAL_SERVER_ERROR,
        Json(ErrorBody {
            error_message
        })
    ).into_response()
}

pub async fn start_server(
    config: FlightItineraryServiceProperties,
    flight_itinerary_service: Arc<FlightItineraryService>,
) -> Result<()> {
    info!(
        "Serving {} with axum on {}::{}",
        config.server.name, config.server.host, config.server.port
    );
    let routes = route(flight_itinerary_service);
    let address = SocketAddr::new(config.server.host.parse()?, config.server.port);
    let middleware = tower::ServiceBuilder::new()
        .layer(CatchPanicLayer::custom(handle_panic));

    let app = Router::new()
        .merge(routes)
        .layer(middleware);

    axum::Server::bind(&address)
        .serve(app.into_make_service())
        .await?;

    Ok(())
}
