/**
 * Cloudflare Worker: 24-Hour DRM Heartbeat & Licensing Server
 * 
 * Enforces hardware machine binding, client license validation,
 * and 24-hour lease token generation for Local RAG Engine nodes.
 *
 * Deploy with Wrangler:
 *   npx wrangler deploy
 */

// In-Memory or KV-backed authorized SME client registry
const AUTHORIZED_CLIENTS = {
  "sme_client_001": {
    licenseKey: "DEMO-SME-2026-KEY",
    active: true,
    boundMachineUuid: null, // null = auto-bind on first activation
    tier: "Enterprise_SME",
    expiresOn: "2027-12-31"
  },
  "sme_client_002": {
    licenseKey: "CLIENT-002-PRO-KEY",
    active: true,
    boundMachineUuid: null,
    tier: "Standard_SME",
    expiresOn: "2027-12-31"
  }
};

export default {
  async fetch(request, env, ctx) {
    // 1. CORS Preflight
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

    // Route: /api/drm/heartbeat
    if (url.pathname === "/api/drm/heartbeat" || url.pathname === "/") {
      try {
        const body = await request.json();
        const { client_id, license_key, machine_uuid, timestamp, nonce } = body;

        if (!client_id || !license_key || !machine_uuid) {
          return new Response(JSON.stringify({
            status: "invalid",
            message: "Missing required fields (client_id, license_key, machine_uuid)."
          }), { status: 400, headers: { "Content-Type": "application/json" } });
        }

        // Validate Client & Key against Registry (or Cloudflare KV / D1)
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
            message: "Client account is suspended or overdue."
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        // Hardware Machine Binding Check
        if (client.boundMachineUuid && client.boundMachineUuid !== machine_uuid) {
          return new Response(JSON.stringify({
            status: "revoked",
            message: "Hardware machine binding mismatch. License cannot be transferred without admin unbind."
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        // Auto-bind machine UUID on initial heartbeat
        if (!client.boundMachineUuid) {
          client.boundMachineUuid = machine_uuid;
        }

        // Calculate 24-Hour Expiry Window
        const nowSec = Math.floor(Date.now() / 1000);
        const expiresAt = nowSec + 86400; // 24 hours lease

        // Generate 32-byte session key
        const rawKey = new Uint8Array(32);
        crypto.getRandomValues(rawKey);
        const sessionKeyHex = Array.from(rawKey).map(b => b.toString(16).padStart(2, '0')).join('');

        // Compute HMAC signature over lease payload
        const secret = env.DRM_SIGNING_SECRET || "INSTITUTIONAL_SME_DRM_SECRET_2026";
        const encoder = new TextEncoder();
        const key = await crypto.subtle.importKey(
          "raw",
          encoder.encode(secret),
          { name: "HMAC", hash: "SHA-256" },
          false,
          ["sign"]
        );

        const payloadStr = `${client_id}:${machine_uuid}:${expiresAt}:${sessionKeyHex}`;
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
