#!/bin/bash

# SecureGuard Service Control Script
# Usage: ./myservice.sh [start|stop] [dev|prod]

ACTION=$1
ENVIRONMENT=${2:-dev}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
MAGENTA='\033[0;35m'
CYAN='\033[0;36m'
WHITE='\033[1;37m'
GRAY='\033[0;37m'
NC='\033[0m' # No Color

show_usage() {
    echo -e "${YELLOW}Usage: ./myservice.sh [start|stop] [dev|prod]${NC}"
    echo -e "${CYAN}Examples:${NC}"
    echo -e "${WHITE}  ./myservice.sh start dev    - Start development environment${NC}"
    echo -e "${WHITE}  ./myservice.sh start prod   - Start production environment${NC}"
    echo -e "${WHITE}  ./myservice.sh stop dev     - Stop development environment${NC}"
    echo -e "${WHITE}  ./myservice.sh stop prod    - Stop production environment${NC}"
}

start_dev_environment() {
    echo -e "${GREEN}[DEV] Starting SecureGuard Development Environment...${NC}"
    
    export DATABASE_URL="postgresql://secureguard:password@localhost:5432/secureguard_dev"
    export RUST_LOG="secureguard_api=debug,tower_http=debug,axum=debug"
    export NODE_ENV="development"

    echo -e "${YELLOW}[1/3] Starting PostgreSQL Database (Development)...${NC}"
    if ! docker-compose up -d db; then
        echo -e "${RED}ERROR: Failed to start database${NC}"
        exit 1
    fi

    echo -e "${YELLOW}[2/3] Starting Rust Backend Server (Debug Mode)...${NC}"
    cd crates/secureguard-api
    cargo run > ../../backend-dev.log 2>&1 &
    BACKEND_PID=$!
    cd ../..
    echo $BACKEND_PID > backend-dev.pid

    echo -e "${YELLOW}[3/3] Starting React Dashboard (Development)...${NC}"
    sleep 5
    cd dashboard
    PORT=3002 npm start > ../dashboard-dev.log 2>&1 &
    DASHBOARD_PID=$!
    cd ..
    echo $DASHBOARD_PID > dashboard-dev.pid

    echo ""
    echo -e "${GREEN}✅ Development Environment Started${NC}"
    echo -e "${CYAN}🔗 Dashboard: http://localhost:3002${NC}"
    echo -e "${CYAN}🔗 API: http://localhost:3000/api${NC}"
    echo -e "${CYAN}📊 Database: localhost:5432 (secureguard_dev)${NC}"
    echo ""
    echo -e "${GRAY}Logs:${NC}"
    echo -e "${GRAY}  Backend: backend-dev.log${NC}"
    echo -e "${GRAY}  Dashboard: dashboard-dev.log${NC}"
}

start_prod_environment() {
    echo -e "${MAGENTA}[PROD] Starting SecureGuard Production Environment...${NC}"
    
    export DATABASE_URL="postgresql://secureguard:password@localhost:5432/secureguard_prod"
    export RUST_LOG="secureguard_api=info"
    export NODE_ENV="production"

    echo -e "${YELLOW}[1/3] Starting PostgreSQL Database (Production)...${NC}"
    # Try production compose file first, fallback to dev
    if ! docker-compose -f docker-compose.prod.yml up -d db 2>/dev/null; then
        if ! docker-compose up -d db; then
            echo -e "${RED}ERROR: Failed to start database${NC}"
            exit 1
        fi
    fi

    echo -e "${YELLOW}[2/3] Building and Starting Rust Backend Server (Release Mode)...${NC}"
    cd crates/secureguard-api
    cargo run --release > ../../backend-prod.log 2>&1 &
    BACKEND_PID=$!
    cd ../..
    echo $BACKEND_PID > backend-prod.pid

    echo -e "${YELLOW}[3/3] Building and Starting React Dashboard (Production)...${NC}"
    sleep 5
    cd dashboard
    
    # Check if serve is installed, if not install it
    if ! command -v serve &> /dev/null; then
        echo -e "${YELLOW}Installing serve globally...${NC}"
        npm install -g serve
    fi
    
    # Build and serve the app
    npm run build
    serve -s build -l 3002 > ../dashboard-prod.log 2>&1 &
    DASHBOARD_PID=$!
    cd ..
    echo $DASHBOARD_PID > dashboard-prod.pid

    echo ""
    echo -e "${MAGENTA}✅ Production Environment Started${NC}"
    echo -e "${CYAN}🔗 Dashboard: http://localhost:3002${NC}"
    echo -e "${CYAN}🔗 API: http://localhost:3000/api${NC}"
    echo -e "${CYAN}📊 Database: localhost:5432 (secureguard_prod)${NC}"
    echo ""
    echo -e "${GRAY}Logs:${NC}"
    echo -e "${GRAY}  Backend: backend-prod.log${NC}"
    echo -e "${GRAY}  Dashboard: dashboard-prod.log${NC}"
}

kill_by_pid_file() {
    local pid_file=$1
    local service_name=$2
    
    if [ -f "$pid_file" ]; then
        local pid=$(cat "$pid_file")
        if kill -0 "$pid" 2>/dev/null; then
            echo -e "${GRAY}Killing $service_name (PID: $pid)${NC}"
            kill -TERM "$pid" 2>/dev/null
            sleep 2
            if kill -0 "$pid" 2>/dev/null; then
                echo -e "${GRAY}Force killing $service_name (PID: $pid)${NC}"
                kill -KILL "$pid" 2>/dev/null
            fi
        fi
        rm -f "$pid_file"
    else
        # Try to find and kill by port
        if [ "$service_name" = "Backend" ]; then
            local port_pid=$(lsof -ti:3000 2>/dev/null)
        elif [ "$service_name" = "Dashboard" ]; then
            local port_pid=$(lsof -ti:3002 2>/dev/null)
        fi
        
        if [ -n "$port_pid" ]; then
            echo -e "${GRAY}Killing $service_name by port (PID: $port_pid)${NC}"
            kill -TERM "$port_pid" 2>/dev/null
            sleep 2
            if kill -0 "$port_pid" 2>/dev/null; then
                kill -KILL "$port_pid" 2>/dev/null
            fi
        fi
    fi
}

stop_environment() {
    local env_type=$1
    echo -e "${RED}[${env_type^^}] Stopping $env_type Environment...${NC}"
    
    echo -e "${GRAY}Stopping React Dashboard (port 3002)...${NC}"
    kill_by_pid_file "dashboard-$env_type.pid" "Dashboard"
    
    echo -e "${GRAY}Stopping Rust Backend Server (port 3000)...${NC}"
    kill_by_pid_file "backend-$env_type.pid" "Backend"
    
    echo -e "${GRAY}Stopping PostgreSQL Database...${NC}"
    docker-compose down 2>/dev/null
    docker-compose -f docker-compose.prod.yml down 2>/dev/null
    
    # Clean up log files
    rm -f backend-$env_type.log dashboard-$env_type.log
    
    echo -e "${GREEN}✅ ${env_type^^} Environment Stopped${NC}"
}

# Validate arguments
if [ -z "$ACTION" ]; then
    show_usage
    exit 1
fi

if [[ "$ACTION" != "start" && "$ACTION" != "stop" ]]; then
    echo -e "${RED}ERROR: Action must be 'start' or 'stop'${NC}"
    show_usage
    exit 1
fi

if [[ "$ENVIRONMENT" != "dev" && "$ENVIRONMENT" != "prod" ]]; then
    echo -e "${RED}ERROR: Environment must be 'dev' or 'prod'${NC}"
    show_usage
    exit 1
fi

# Main script logic
echo -e "${CYAN}SecureGuard Service Control - $ACTION $ENVIRONMENT${NC}"
echo ""

case $ACTION in
    start)
        case $ENVIRONMENT in
            dev) start_dev_environment ;;
            prod) start_prod_environment ;;
        esac
        ;;
    stop)
        stop_environment $ENVIRONMENT
        ;;
esac

echo ""
echo -e "${MAGENTA}Login credentials:${NC}"
echo -e "${WHITE}  Username: admin${NC}"
echo -e "${WHITE}  Password: admin123${NC}"