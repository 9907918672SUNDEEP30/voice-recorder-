/**
 * Cloudflare Worker: Universal Shared DRM & Metered Usage Licensing Server
 * 
 * Enforces hardware machine binding, client license validation, app-specific
 * metered usage constraints, and 24-hour cryptographic lease token issuance.
 *
 * Deploy with Wrangler:
 *   npx wrangler deploy universal_licensing_worker.js --name "universal-drm-guard"
 */

// Multi-app client licensing tier registry (or KV/D1 backed in production)
const AUTHORIZED_CLIENTS = {
  "sme_client_001": {
    licenseKey: "KEY-ENTERPRISE-2026",
    active: true,
    boundMachineUuid: null, // Auto-binds on initial ping
    allowedApps: ["cctv_guard", "voice_agent", "local_rag_engine", "financial_engine"],
    limits: {
      cctv_guard: { max_cameras: 16 },
      voice_agent: { max_daily_minutes: 500 },
      local_rag_engine: { max_documents: 10000 },
      financial_engine: { max_daily_scans: 10000 }
    }
  },
  "client_demo": {
    licenseKey: "DEMO-KEY-2026",
    active: true,
    boundMachineUuid: null,
    allowedApps: ["cctv_guard", "voice_agent", "local_rag_engine"],
    limits: {
      cctv_guard: { max_cameras: 4 },
      voice_agent: { max_daily_minutes: 60 },
      local_rag_engine: { max_documents: 500 }
    }
  }
};

export default {
  async fetch(request, env, ctx) {
    if (request.method === "OPTIONS") {
      return new Response(null, {
        headers: {
          "Access-Control-Allow-Origin": "*",
          "Access-Control-Allow-Methods": "POST, OPTIONS",
          "Access-Control-Allow-Headers": "Content-Type",
        },
      });
    }

    if (request.method !== "POST") {
      return new Response(JSON.stringify({ error: "Method not allowed" }), {
        status: 405,
        headers: { "Content-Type": "application/json" }
      });
    }

    const url = new URL(request.url);

    if (url.pathname === "/api/heartbeat" || url.pathname === "/") {
      try {
        const body = await request.json();
        const { client_id, license_key, machine_uuid, app_name, timestamp, nonce, metrics } = body;

        if (!client_id || !license_key || !machine_uuid || !app_name) {
          return new Response(JSON.stringify({
            status: "invalid",
            message: "Missing required fields (client_id, license_key, machine_uuid, app_name)."
          }), { status: 400, headers: { "Content-Type": "application/json" } });
        }

        const client = AUTHORIZED_CLIENTS[client_id];
        if (!client || client.licenseKey !== license_key) {
          return new Response(JSON.stringify({
            status: "revoked",
            message: "Invalid client credentials or license key."
          }), { status: 401, headers: { "Content-Type": "application/json" } });
        }

        if (!client.active) {
          return new Response(JSON.stringify({
            status: "revoked",
            message: "Client account is suspended or subscription overdue."
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        if (!client.allowedApps.includes(app_name)) {
          return new Response(JSON.stringify({
            status: "revoked",
            message: `Application '${app_name}' is not licensed under client plan.`
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        // Hardware Machine Binding Check
        if (client.boundMachineUuid && client.boundMachineUuid !== machine_uuid) {
          return new Response(JSON.stringify({
            status: "revoked",
            message: "Hardware machine binding mismatch. License cannot be transferred without administrator unbind."
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        // Auto-bind machine UUID on initial heartbeat
        if (!client.boundMachineUuid) {
          client.boundMachineUuid = machine_uuid;
        }

        // Check Metered Usage Limits
        const appLimits = client.limits[app_name] || {};
        if (app_name === "cctv_guard" && appLimits.max_cameras && metrics && metrics.camera_count > appLimits.max_cameras) {
          return new Response(JSON.stringify({
            status: "limit_exceeded",
            message: `Camera quota exceeded (${metrics.camera_count} active > ${appLimits.max_cameras} allowed).`
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        // Calculate 24-Hour Expiry Window
        const nowSec = Math.floor(Date.now() / 1000);
        const expiresAt = nowSec + 86400; // 24 hours

        // Generate 32-byte ephemeral session key
        const rawKey = new Uint8Array(32);
        crypto.getRandomValues(rawKey);
        const sessionKeyHex = Array.from(rawKey).map(b => b.toString(16).padStart(2, '0')).join('');

        // Compute HMAC signature over lease payload
        const secret = env.DRM_SIGNING_SECRET || "UNIVERSAL_DRM_GUARD_SHARED_SECRET_2026";
        const encoder = new TextEncoder();
        const key = await crypto.subtle.importKey(
          "raw",
          encoder.encode(secret),
          { name: "HMAC", hash: "SHA-256" },
          false,
          ["sign"]
        );

        const payloadStr = `${client_id}:${machine_uuid}:${app_name}:${expiresAt}:${sessionKeyHex}`;
        const signatureBuffer = await crypto.subtle.sign("HMAC", key, encoder.encode(payloadStr));
        const signatureHex = Array.from(new Uint8Array(signatureBuffer)).map(b => b.toString(16).padStart(2, '0')).join('');

        return new Response(JSON.stringify({
          status: "granted",
          expires_at: expiresAt,
          session_key_hex: sessionKeyHex,
          signature_hex: signatureHex,
          message: "24-hour lease granted."
        }), {
          status: 200,
          headers: {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": "*"
          }
        });

      } catch (err) {
        return new Response(JSON.stringify({
          status: "error",
          message: err.message
        }), { status: 500, headers: { "Content-Type": "application/json" } });
      }
    }

    return new Response("Not found", { status: 404 });
  }
};
