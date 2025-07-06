# LLM Performance Benchmark - VM Setup Guide

This guide walks through setting up the LLM Performance Benchmark system on a fresh VM.

## Prerequisites

- Ubuntu 22.04 LTS or similar Linux distribution
- At least 4GB RAM
- 20GB+ disk space
- sudo access

## 1. System Dependencies

```bash
# Update system
sudo apt update && sudo apt upgrade -y

# Install build essentials
sudo apt install -y build-essential curl git pkg-config libssl-dev

# Install PostgreSQL
sudo apt install -y postgresql postgresql-contrib

# Install Node.js 20.x
curl -fsSL https://deb.nodesource.com/setup_20.x | sudo -E bash -
sudo apt install -y nodejs
```

## 2. Install Rust

```bash
# Install Rust
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source "$HOME/.cargo/env"

# Verify installation
rustc --version
cargo --version
```

## 3. PostgreSQL Setup

```bash
# Switch to postgres user
sudo -u postgres psql

# In PostgreSQL prompt, run:
CREATE USER benchmark_user WITH PASSWORD 'your_secure_password';
CREATE DATABASE llm_benchmarks OWNER benchmark_user;
GRANT ALL PRIVILEGES ON DATABASE llm_benchmarks TO benchmark_user;
\q

# Test connection
psql -h localhost -U benchmark_user -d llm_benchmarks
```

## 4. Clone and Setup Project

```bash
# Clone the repository
git clone <your-repo-url> llm-performance
cd llm-performance

# Create .env file for backend
cd backend
cat > .env << EOF
DATABASE_URL=postgres://benchmark_user:your_secure_password@localhost/llm_benchmarks
EOF

# Install sqlx-cli for migrations
cargo install sqlx-cli --no-default-features --features postgres

# Run database migrations
sqlx migrate run

# Build backend
cargo build --release
cd ..
```

## 5. Build Frontend

```bash
cd frontend
npm install
npm run build
cd ..
```

## 6. Build Uploader Tool

```bash
cd uploader
cargo build --release
cd ..
```

## 7. Running the System

### Option A: Direct Execution

```bash
# Terminal 1: Run backend
cd backend
./target/release/llm-benchmark-backend

# The server will be available at http://<your-vm-ip>:3000
```

### Option B: Systemd Service (Recommended)

Create a systemd service file:

```bash
sudo nano /etc/systemd/system/llm-benchmark.service
```

Add the following content:

```ini
[Unit]
Description=LLM Benchmark Backend
After=network.target postgresql.service

[Service]
Type=simple
User=your_username
WorkingDirectory=/home/your_username/llm-performance/backend
Environment="DATABASE_URL=postgres://benchmark_user:your_secure_password@localhost/llm_benchmarks"
ExecStart=/home/your_username/llm-performance/backend/target/release/llm-benchmark-backend
Restart=on-failure
RestartSec=5

[Install]
WantedBy=multi-user.target
```

Enable and start the service:

```bash
sudo systemctl daemon-reload
sudo systemctl enable llm-benchmark
sudo systemctl start llm-benchmark
sudo systemctl status llm-benchmark
```

## 8. Firewall Configuration

```bash
# If using ufw
sudo ufw allow 3000/tcp
sudo ufw reload

# If using iptables
sudo iptables -A INPUT -p tcp --dport 3000 -j ACCEPT
sudo netfilter-persistent save
```

## 9. Upload Data

Use the uploader tool to import benchmark data:

```bash
cd uploader

# Upload llama-bench results
./target/release/benchmark-uploader llama-bench \
  --file /path/to/llama-bench.json \
  --server http://localhost:3000

# Upload MMLU-Pro results
./target/release/benchmark-uploader mmlu-pro \
  --file /path/to/report.txt \
  --model "model-name/version" \
  --quantization "Q4_K_M" \
  --server http://localhost:3000
```

## 10. Nginx Reverse Proxy (Optional)

For production deployment with SSL:

```bash
sudo apt install -y nginx certbot python3-certbot-nginx

# Create nginx config
sudo nano /etc/nginx/sites-available/llm-benchmark
```

Add:

```nginx
server {
    listen 80;
    server_name your-domain.com;

    location / {
        proxy_pass http://localhost:3000;
        proxy_http_version 1.1;
        proxy_set_header Upgrade $http_upgrade;
        proxy_set_header Connection 'upgrade';
        proxy_set_header Host $host;
        proxy_cache_bypass $http_upgrade;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
    }
}
```

Enable the site:

```bash
sudo ln -s /etc/nginx/sites-available/llm-benchmark /etc/nginx/sites-enabled/
sudo nginx -t
sudo systemctl reload nginx

# Get SSL certificate
sudo certbot --nginx -d your-domain.com
```

## Troubleshooting

### Check logs
```bash
# System logs
sudo journalctl -u llm-benchmark -f

# PostgreSQL logs
sudo tail -f /var/log/postgresql/postgresql-*.log
```

### Test database connection
```bash
cd backend
DATABASE_URL=postgres://benchmark_user:your_secure_password@localhost/llm_benchmarks cargo run
```

### Verify ports are open
```bash
sudo netstat -tlnp | grep 3000
```

### Reset database (if needed)
```bash
cd backend
sqlx database drop
sqlx database create
sqlx migrate run
```

## Security Considerations

1. **Change default passwords**: Use strong passwords for PostgreSQL
2. **Firewall**: Only open necessary ports
3. **HTTPS**: Use SSL certificates for production
4. **Updates**: Keep system and dependencies updated
5. **Backups**: Regular PostgreSQL backups:
   ```bash
   pg_dump -U benchmark_user llm_benchmarks > backup_$(date +%Y%m%d).sql
   ```

## Monitoring

Consider setting up:
- PostgreSQL connection monitoring
- Disk space monitoring
- Service health checks
- Log rotation

## Next Steps

1. Access the web interface at `http://<your-vm-ip>:3000`
2. Upload benchmark data using the uploader tool
3. Configure automated benchmark runs if needed