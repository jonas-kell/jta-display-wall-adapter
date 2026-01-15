#!/bin/bash
trap 'docker compose -f docker-compose.run.yml down; exit' INT

# Usage: ./run.sh < --native>
COMPOSE_DIR="${1:-$(pwd)}"

# arguments
native=false
for arg in "$@"; do
  if [[ "$arg" == "--native" ]]; then
    native=true
    break
  fi
done

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

# Disable X11 screen blank -> only on X11
# xset s off      || echo "Blanking Screen disable not supported"     # Disable screen saver
# xset -dpms      || echo "Blanking Screen disable not supported"     # Disable DPMS (Energy Star) features
# xset s noblank  || echo "Blanking Screen disable not supported"     # Disable screen blanking

# setting the xhost because this might reset it seems
xhost +local:docker

# Background loop to move the window on wayland sway
(
  while true; do
    if [[ -f "move_container/coords.txt" ]]; then
      CONTENT=$(<move_container/coords.txt)
      if swaymsg "[title=\"JTA Display Window\"]" move position "$CONTENT"; then
        rm -f move_container/coords.txt
      else
        echo "Failed to move window — keeping coords.txt for retry."
      fi
    fi
    sleep 5
  done
) &

echo "Starting client in $COMPOSE_DIR"
echo "$(date '+%Y-%m-%d %H:%M:%S') Starting client in $COMPOSE_DIR" >> "$LOGFILE"
cd "$COMPOSE_DIR" || {  
  echo "Command failed. Press Enter to close."
  echo "$(date '+%Y-%m-%d %H:%M:%S') Command failed." >> "$LOGFILE"
  read
  exit 1
}
(docker compose -f docker-compose.run.yml pull || echo "⚠️ Skipping pull (offline or failed)") 2>&1 | tee -a "$LOGFILE"

if $native; then
  echo "Running client natively"
  echo "$(date '+%Y-%m-%d %H:%M:%S') Running client natively" >> "$LOGFILE"
  echo "Copy over executable"
  # must match values in docker-compose.yml !
  docker create --name tmp_copy_container kellehorreur/jta-display-wall-adapter:latest
  docker cp tmp_copy_container:/app/client .
  docker rm tmp_copy_container
  echo "Executable has been prepared"
  echo "$(date '+%Y-%m-%d %H:%M:%S') Executable has been prepared" >> "$LOGFILE"
  # Run natively
  ./client client --display-client-communication-port 5678 --wait-ms-before-testing-for-shutdown=5000 --emit-file-on-location-update 2>&1 | tee -a "$LOGFILE"
else
  echo "Running client inside docker"
  echo "$(date '+%Y-%m-%d %H:%M:%S') Running client inside docker" >> "$LOGFILE"
  docker compose -f docker-compose.run.yml up 2>&1 | tee -a "$LOGFILE"
fi

echo "This should not be reached"
echo "$(date '+%Y-%m-%d %H:%M:%S') This should not be reached." >> "$LOGFILE"
{  
  echo "Command failed. Press Enter to close."
  read
  exit 1
}