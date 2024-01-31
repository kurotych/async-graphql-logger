Simple logging GQL requests with duration in [async-graphql](https://github.com/async-graphql/async-graphql)

Run example
```bash
RUST_LOG=info cargo run --example axum

[2024-01-31T17:32:59Z INFO  gql_logger] [QueryID: 639312411] query { healthCheck(input: 1) }
[2024-01-31T17:32:59Z INFO  gql_logger] [QueryID: 639312411] Duration: 101ms
```
