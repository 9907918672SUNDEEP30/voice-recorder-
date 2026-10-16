#!/usr/bin/env bash
# ==============================================================================
# Shambhavi AI Labs — Pehredar Box Appliance Installer
# Automated Turnkey Setup for Headless Ubuntu / Debian Linux Appliances
# ==============================================================================

set -euo pipefail

echo "============================================================"
echo "🛡️  SHAMBHAVI AI LABS — PEHREDAR BOX INSTALLER (LINUX)   🛡️"
echo "============================================================"

INSTALL_DIR="/opt/shambhavi/nvr"
MODELS_DIR="/opt/shambhavi/models"
DATA_DIR="/data/recordings"
EVIDENCE_DIR="/data/evidence"

echo "[1/5] Creating application and storage directory structures..."
mkdir -p "${INSTALL_DIR}"
mkdir -p "${MODELS_DIR}"
mkdir -p "${DATA_DIR}"
mkdir -p "${EVIDENCE_DIR}"

echo "[2/5] Copying binaries and configuration..."
if [ -f "./target/release/nvr_core" ]; then
    cp ./target/release/nvr_core "${INSTALL_DIR}/"
elif [ -f "./target/debug/nvr_core" ]; then
    cp ./target/debug/nvr_core "${INSTALL_DIR}/"
fi

if [ -f "./nvr_config.json" ]; then
    cp ./nvr_config.json "${INSTALL_DIR}/"
fi

echo "[3/5] Setting up AI models..."
if [ -f "C:/AI_Models/yolov8m.onnx" ]; then
    cp "C:/AI_Models/yolov8m.onnx" "${MODELS_DIR}/yolov8m.onnx"
elif [ -f "./models/yolov8m.onnx" ]; then
    cp "./models/yolov8m.onnx" "${MODELS_DIR}/yolov8m.onnx"
fi

echo "[4/5] Installing systemd auto-start service..."
cp ./deploy/pehredar.service /etc/systemd/system/
systemctl daemon-reload
systemctl enable pehredar.service

echo "[5/5] Hardening permissions..."
chmod +x "${INSTALL_DIR}/nvr_core"

echo "============================================================"
echo "✅ SHAMBHAVI PEHREDAR BOX APPLIANCE IS 100% READY!"
echo "Start engine:  sudo systemctl start pehredar.service"
echo "Check status:  sudo systemctl status pehredar.service"
echo "View logs:     sudo journalctl -u pehredar.service -f"
echo "============================================================"
