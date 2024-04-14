Simple logging GQL requests with duration in [async-graphql](https://github.com/async-graphql/async-graphql)

Run example
```bash
RUST_LOG=debug cargo run --example axum

[2024-04-14T11:00:31Z INFO  gql_logger] [QueryID: 547177987] query { healthCheck(input: 1) }
[2024-04-14T11:00:31Z DEBUG gql_logger] [QueryID: 547177987] Response: {healthCheck: true}
[2024-04-14T11:00:31Z INFO  gql_logger] [QueryID: 547177987] Duration: 102ms
```
