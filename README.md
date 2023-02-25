# flight-itinerary
A http server that calculates the eulerian flight path when given a list of flight tickets indicating origin and destination airports.

## Input assumptions
1. The input flight paths must not form a Eulerian cycle, i.e. origin = destination, because in that case every airport can be the starting or ending airport.
2. The input flight paths should only contain flight paths taken by a single person, i.e. forming a Eulerian path where each edge is traversed only once.
3. The input flight paths can contain cycle where an airport is visited more than once, e.g. `[["MYS", "SGP"], ["MYS", "BKK"], ["SGP", "BKK"], ["BKK", "MYS"]]`, where `"MYS"` and `"BKK"` are visited more than once.

## Implementation rationale
1. In order to cater for the scenario where cycle(s) is present in the flight path, yet still being a valid Eulerian path, a directed graph structure is used to represent the flight paths.
2. This allows us to easily identify the starting airport and ending airport as the starting node and ending node of a valid Eulerian path obey the following rules ([ref](https://en.wikipedia.org/wiki/Eulerian_path#Properties)).
- The origin is the only node with (outdegree - indegree) = 1
- The destination is the only node with (outdegree - indegree) = -1
- The other nodes all have (outdegree - indegree) = 0
3. This implementation will introduce the worst time complexity of O(N * P) where N is the number of airports and P is the biggest number of outgoing + incoming flight path from/to an airport. 
4. `Arc` smart pointer is used to ensure concurrency safety should tokio runs multiple threads.

## Running the server
1. Git clone this repository.
2. Run the following at the top project directory.
```bash
cargo run
```
3. The server should be running and one should see a log message like `Serving flight-itinerary-service with axum on 0.0.0.0::8080`.
4. This [config file](./src/config/config.yaml) contains configurable settings, like host and port to run the server on, and the logging level.
5. If one wishes to use a config file from a different location, one can run the following command to override the default config file location.
```bash
cargo run -- --config-file <path-to-new-config-file>
```

## Calling the API
Curl command:
```bash
curl --location --request POST 'http://localhost:8080/compute' \
--header 'Content-Type: application/json' \
```
Request body:
```json
{
    "flightPaths": [["MYS", "SGP"], ["VXC", "BKK"], ["GBF", "MYS"], ["BKK", "GBF"]]
}
```
Response body:
```json
{
    "itinerary": ["VXC", "SGP"]
}
```

## API specifications
Please open the [OpenAPI document](openapi.yaml) in https://editor.swagger.io/.

## Test
Run the following at the top project directory.
```bash
cargo test
```
