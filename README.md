# Fewest Moves Chain

A blockchain which uses Fewest Moves solutions to hash-based scrambles as proof of work.

## Development

### Run development server with hot reloading

```bash
./bin/run
```

### Database

```bash
# Setup
cargo install sqlx-cli --no-default-features --features sqlite
touch app.db
sqlx migrate run  # Initial migration

# Create migration
sqlx migrate add <name>

# Run migration
sqlx migrate run

# Manual commands
sqlite3 app.db
```
