# Altenia Deployment

Docker Compose setup for running the complete Altenia stack.

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                    Docker Network                        │
│                                                          │
│  ┌──────────────┐    ┌──────────────┐    ┌───────────┐  │
│  │   Frontend   │───▶│   Backend    │───▶│ PostgreSQL│  │
│  │   (nginx)    │    │   (Rust)     │    │           │  │
│  │   :80        │    │   :3000      │    │   :5432   │  │
│  └──────────────┘    └──────────────┘    └───────────┘  │
│         │                                               │
└─────────┼───────────────────────────────────────────────┘
          │
          ▼
    localhost:80 (or FRONTEND_PORT)
```

## Quick Start

```bash
# 1. Navigate to deploy folder
cd deploy

# 2. (Optional) Create .env file from template
cp .env.example .env
# Edit .env to customize settings

# 3. Build and start all services
docker-compose up -d --build

# 4. Open browser
open http://localhost
```

## Services

| Service | Description | Internal Port | External Port |
|---------|-------------|---------------|---------------|
| `postgres` | PostgreSQL 16 database | 5432 | - |
| `backend` | Rust API server | 3000 | - |
| `frontend` | Nginx serving static files + API proxy | 80 | 80 (configurable) |

## Commands

```bash
# Build and start all services
docker-compose up -d --build

# View logs (all services)
docker-compose logs -f

# View logs (specific service)
docker-compose logs -f backend

# Rebuild a specific service
docker-compose up -d --build backend

# Stop all services
docker-compose down

# Stop and remove volumes (WARNING: deletes all data)
docker-compose down -v

# Check service status
docker-compose ps

# Execute command in container
docker-compose exec backend sh
docker-compose exec postgres psql -U altenia -d altenia
```

## Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `POSTGRES_USER` | `altenia` | PostgreSQL username |
| `POSTGRES_PASSWORD` | `altenia_dev_password` | PostgreSQL password |
| `POSTGRES_DB` | `altenia` | Database name |
| `JWT_ACCESS_SECRET` | `dev-access-secret...` | JWT access token secret |
| `JWT_REFRESH_SECRET` | `dev-refresh-secret...` | JWT refresh token secret |
| `REFRESH_TOKEN_DURATION_DAYS` | `7` | Refresh token lifetime |
| `FRONTEND_PORT` | `80` | Host port for frontend |
| `RUST_LOG` | `info,sqlx=warn` | Rust logging level |

## Development vs Production

### Development (default)
- Uses default secrets (not secure)
- PostgreSQL data persisted in Docker volume
- All services run locally

### Production Checklist
1. **Change JWT secrets** - Use strong, random 32+ character strings
2. **Change database password** - Use a secure password
3. **Use HTTPS** - Add TLS termination (nginx config or reverse proxy)
4. **Secure PostgreSQL** - Don't expose port externally
5. **Set up backups** - Regular database backups
6. **Use proper logging** - Configure log aggregation
7. **Add health monitoring** - Set up uptime monitoring

## Troubleshooting

### Backend won't start
```bash
# Check logs
docker-compose logs backend

# Common issues:
# - Database not ready: Wait for postgres healthcheck
# - Migration errors: Check migration SQL files
```

### Database connection failed
```bash
# Check postgres is running
docker-compose ps postgres

# Check postgres logs
docker-compose logs postgres

# Connect manually
docker-compose exec postgres psql -U altenia -d altenia
```

### Frontend not loading
```bash
# Check nginx logs
docker-compose logs frontend

# Verify backend is accessible from frontend container
docker-compose exec frontend wget -qO- http://backend:3000/api/auth/me || echo "Backend not reachable"
```

### Rebuild everything fresh
```bash
docker-compose down -v
docker-compose up -d --build
```
