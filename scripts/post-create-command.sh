#!/bin/bash
set -e

MARKER_FILE="/var/lib/influxdb/.restore_complete"
BACKUP_FILE="/var/lib/influxdb/wattle-i-do.zip"
RESTORE_SOURCE_DIR="/tmp/influxdb_restore_source"

# Check if backup file exists before proceeding
if [ ! -f "$BACKUP_FILE" ]; then
    echo "Backup file $BACKUP_FILE not found. Skipping restore."
    exit 0
fi

# Check if restore already done
if [ -f "$MARKER_FILE" ]; then
    echo "InfluxDB restore already completed (marker file found), skipping."
else
    echo "Starting InfluxDB restore..."

    # Explicitly start InfluxDB service
    echo "Starting InfluxDB service..."
    sudo service influxdb start

    # Wait for a while to allow the service to start
    echo "Waiting for InfluxDB service to start..."
    sleep 15

    # Check the status of the InfluxDB service
    echo "Checking InfluxDB service status..."
    sudo service influxdb status
    SERVICE_STATUS=$?

    if [ "$SERVICE_STATUS" -ne 0 ]; then
        echo "InfluxDB service failed to start. Aborting restore."
        exit 1
    fi

    # 1. Unzip backup
    echo "Unzipping backup from $BACKUP_FILE..."
    sudo mkdir -p "$RESTORE_SOURCE_DIR"
    sudo unzip "$BACKUP_FILE" -d "$RESTORE_SOURCE_DIR"

    # 2. Restore (Corrected command)
    echo "Restoring database from $RESTORE_SOURCE_DIR..."
    sudo influxd restore -portable "$RESTORE_SOURCE_DIR"

    RESTORE_EXIT_CODE=$?
    if [ $RESTORE_EXIT_CODE -ne 0 ]; then
        echo "Error during restore. Exit code: $RESTORE_EXIT_CODE"
        exit 1 # Exit with an error code to signal failure
    fi

    # 3. Fix permissions (important after restore)
    echo "Setting permissions for InfluxDB directories..."
    sudo chown -R influxdb:influxdb /var/lib/influxdb/meta /var/lib/influxdb/data /var/lib/influxdb/wal

    # 4. Clean up temp unzip dir
    echo "Cleaning up temporary restore source directory..."
    sudo rm -rf "$RESTORE_SOURCE_DIR"

    # 5. Create marker file to prevent re-running
    echo "Creating marker file $MARKER_FILE..."
    sudo touch "$MARKER_FILE"
    sudo chown influxdb:influxdb "$MARKER_FILE"

    echo "InfluxDB restore completed successfully."
fi