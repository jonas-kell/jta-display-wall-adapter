#!/bin/bash
trap 'docker compose -f docker-compose.run.yml down; exit' INT

# Usage: ./run.sh /path/to/compose-dir
COMPOSE_DIR="${1:-$(pwd)}"

# Define logfile
LOGFILE="log.txt"

# Clear the logfile at the start
> "$LOGFILE"
echo "$(date '+%Y-%m-%d %H:%M:%S') script started in $COMPOSE_DIR" >> "$LOGFILE"

if [[ ! -f "$COMPOSE_DIR/docker-compose.run.yml" ]]; then
  echo "Error: docker-compose.run.yml not found in $COMPOSE_DIR"
  echo "Command failed. Press Enter to close."
  echo "$(date '+%Y-%m-%d %H:%M:%S') Error: docker-compose.run.yml not found in $COMPOSE_DIR" >> "$LOGFILE"
  echo "$(date '+%Y-%m-%d %H:%M:%S') Command failed." >> "$LOGFILE"
  read
  exit 1
fi

# setting the xhost because this might reset it seems
xhost +local:docker

echo "Starting docker compose in $COMPOSE_DIR"
echo "$(date '+%Y-%m-%d %H:%M:%S') Starting docker compose in $COMPOSE_DIR" >> "$LOGFILE"
cd "$COMPOSE_DIR" || {  
  echo "Command failed. Press Enter to close."
  echo "$(date '+%Y-%m-%d %H:%M:%S') Command failed." >> "$LOGFILE"
  read
  exit 1
}
docker compose -f docker-compose.run.yml up 2>&1 | tee -a "$LOGFILE"