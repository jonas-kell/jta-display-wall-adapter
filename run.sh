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

echo "Starting docker compose in $COMPOSE_DIR"
echo "$(date '+%Y-%m-%d %H:%M:%S') Starting docker compose in $COMPOSE_DIR" >> "$LOGFILE"
cd "$COMPOSE_DIR" || {  
  echo "Command failed. Press Enter to close."
  echo "$(date '+%Y-%m-%d %H:%M:%S') Command failed." >> "$LOGFILE"
  read
  exit 1
}
(docker compose -f docker-compose.run.yml pull || echo "⚠️ Skipping pull (offline or failed)") 2>&1 | tee -a "$LOGFILE"
docker compose -f docker-compose.run.yml up 2>&1 | tee -a "$LOGFILE"

echo "This should not be reached"
echo "$(date '+%Y-%m-%d %H:%M:%S') This should not be reached." >> "$LOGFILE"
{  
  echo "Command failed. Press Enter to close."
  read
  exit 1
}