# Fewest Moves Chain

A blockchain which uses Fewest Moves solutions to hash-based scrambles as proof of work.

## Development

### Run development server with hot reloading

```bash
./bin/run
```


### Generate a context for an LLM

```bash
./bin/generate_context
```


### Backups

```bash
./bin/backup
```


### Database

```bash
# Setup
cargo install sqlx-cli --no-default-features --features sqlite
cp .env.example .env
touch app.db

# Run migration
sqlx migrate run

# Create migration
sqlx migrate add <name>

# Manual commands
sqlite3 app.db

# Prepare for build
cargo sqlx prepare
```


### Deployment

```bash
./bin/deploy
```
