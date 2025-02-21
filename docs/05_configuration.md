## Configuration

Next we need to do some configuration. Most importantly we need to specify the domain where the federated instance is running. It should be at the domain root and available over HTTPS for production. See the documentation for a list of config options. The parameter `user_data` is for anything that your application requires in handler functions, such as database connection handle, configuration etc.

```
# use activitypub_federation::config::FederationConfig;
# let db_connection = ();
# let _ = actix_rt::System::new();
let config = FederationConfig::builder()
    .domain("example.com")
    .app_data(db_connection)
    .build()?;
# Ok::<(), anyhow::Error>(())
```

`debug` is necessary to test federation with http and localhost URLs, but it should never be used in production. The `worker_count` value can be adjusted depending on the instance size. A lower value saves resources on a small instance, while a higher value is necessary on larger instances to keep up with send jobs. `url_verifier` can be used to implement a domain blacklist.