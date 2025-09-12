# how to

```bash
cargo install sqlx-cli --no-default-features -F rustls,postgres
sqlx database create
sqlx migrate add users_table
sqlx migrate add files_table
sqlx migrate add shared_links_table
sqlx migrate run
```

please refer to https://www.youtube.com/watch?v=t5w2dauFmhM
