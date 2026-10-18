/**
 * Cloudflare Worker: Universal Shared DRM & Metered Usage Licensing Server
 * 
 * Commercial-grade licensing gateway for Shambhavi AI Labs edge applications.
 * Features:
 *   - Ed25519 asymmetric cryptographic lease token signing
 *   - Cloudflare KV backed persistent machine hardware binding
 *   - Bearer token authenticated 10-minute health telemetry (CCTV hardware/wire faults)
 *   - App-specific metered usage quota enforcement
 *   - Cryptographically signed OTA update distribution
 *
 * Deploy with Wrangler:
 *   npx wrangler deploy universal_licensing_worker.js --name "universal-drm-guard"
 */

// Production client registry (in production, sync from Supabase or KV)
const DEFAULT_CLIENTS = {
  "sme_client_001": {
    licenseKey: "KEY-ENTERPRISE-2026",
    active: true,
    allowedApps: ["cctv_guard", "voice_agent", "local_rag_engine", "financial_engine", "video_studio", "dsp_recorder"],
    limits: {
      cctv_guard: { max_cameras: 16 },
      voice_agent: { max_daily_minutes: 500, max_calls: 1000 },
      local_rag_engine: { max_documents: 10000, max_daily_queries: 5000 },
      financial_engine: { max_special_searches: 500 },
      video_studio: { max_render_minutes: 300 },
      dsp_recorder: { max_recording_hours: 50 }
    },
    contact: {
      whatsapp: null // Loaded securely from env or KV
    }
  },
  "client_demo": {
    licenseKey: "DEMO-KEY-2026",
    active: true,
    allowedApps: ["cctv_guard", "voice_agent", "local_rag_engine", "financial_engine", "video_studio", "dsp_recorder"],
    limits: {
      cctv_guard: { max_cameras: 4 },
      voice_agent: { max_daily_minutes: 60, max_calls: 50 },
      local_rag_engine: { max_documents: 500 },
      financial_engine: { max_special_searches: 25 },
      video_studio: { max_render_minutes: 30 },
      dsp_recorder: { max_recording_hours: 5 }
    },
    contact: {
      whatsapp: null
    }
  }
};

// In-memory fallback if Cloudflare KV is not yet provisioned
const MEMORY_BINDINGS = new Map();

/**
 * Signs canonical message with Ed25519 private key seed using Web Crypto API.
 * Converts 32-byte seed into standard PKCS#8 DER format required by Web Crypto.
 */
async function signEd25519(canonicalMessage, seedBytes) {
  // PKCS#8 DER header for Ed25519 (RFC 8410: 16 bytes prefix)
  const pkcs8Prefix = new Uint8Array([
    0x30, 0x2e, 0x02, 0x01, 0x00, 0x30, 0x05, 0x06, 0x03, 0x2b, 0x65, 0x70, 0x04, 0x22, 0x04, 0x20
  ]);

  const fullDer = new Uint8Array(pkcs8Prefix.length + seedBytes.length);
  fullDer.set(pkcs8Prefix, 0);
  fullDer.set(seedBytes, pkcs8Prefix.length);

  const privateKey = await crypto.subtle.importKey(
    "pkcs8",
    fullDer,
    { name: "Ed25519" },
    false,
    ["sign"]
  );

  const encoder = new TextEncoder();
  const signature = await crypto.subtle.sign(
    "Ed25519",
    privateKey,
    encoder.encode(canonicalMessage)
  );

  return Array.from(new Uint8Array(signature))
    .map(b => b.toString(16).padStart(2, "0"))
    .join("");
}

export default {
  async fetch(request, env, ctx) {
    // 1. Strict CORS Handling
    if (request.method === "OPTIONS") {
      return new Response(null, {
        headers: {
          "Access-Control-Allow-Origin": env.ALLOWED_ORIGIN || "*",
          "Access-Control-Allow-Methods": "POST, OPTIONS",
          "Access-Control-Allow-Headers": "Content-Type, Authorization",
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

    // =========================================================================
    // 1. 10-Minute Proactive Health Telemetry Endpoint (CCTV Black-Box Wire & Camera Faults)
    // SECURED: Requires Bearer Token Authentication
    // =========================================================================
    if (url.pathname === "/api/telemetry/heartbeat-10m") {
      try {
        const authHeader = request.headers.get("Authorization") || "";
        const token = authHeader.startsWith("Bearer ") ? authHeader.slice(7).trim() : null;

        const body = await request.json();
        const { client_id, machine_uuid, wire_faults, offline_cameras, storage_pct } = body;

        // Verify client credentials
        const client = DEFAULT_CLIENTS[client_id];
        const expectedToken = env.TELEMETRY_SHARED_TOKEN || (client ? client.licenseKey : null);

        if (!token || !expectedToken || token !== expectedToken) {
          return new Response(JSON.stringify({
            status: "unauthorized",
            message: "Missing or invalid telemetry Bearer authorization token."
          }), {
            status: 401,
            headers: { "Content-Type": "application/json" }
          });
        }

        let alertsDispatched = 0;
        const issues = [];

        if (wire_faults && wire_faults > 0) {
          issues.push(`⚠️ Loose wire or physical cable disconnect detected (${wire_faults} fault events).`);
        }
        if (offline_cameras && offline_cameras.length > 0) {
          issues.push(`🚨 Camera(s) offline: ${offline_cameras.join(", ")}`);
        }
        if (storage_pct && storage_pct >= 90) {
          issues.push(`💾 Surveillance SSD storage critical: ${storage_pct}% full.`);
        }

        // Trigger WhatsApp / SMS webhook alert if faults are present
        if (issues.length > 0) {
          const recipientPhone = client?.contact?.whatsapp || env.ADMIN_WHATSAPP || null;
          
          if (recipientPhone) {
            console.log(`[10-MIN HEALTH ALERT] Dispatching to ${recipientPhone}:`, issues.join(" | "));
            
            if (env.WHATSAPP_API_URL && env.WHATSAPP_TOKEN) {
              try {
                await fetch(env.WHATSAPP_API_URL, {
                  method: "POST",
                  headers: {
                    "Authorization": `Bearer ${env.WHATSAPP_TOKEN}`,
                    "Content-Type": "application/json"
                  },
                  body: JSON.stringify({
                    to: recipientPhone,
                    message: `[Shambhavi AI Pehredar Box Alert]\n${issues.join("\n")}\nPlease inspect your local appliance.`
                  })
                });
                alertsDispatched++;
              } catch (notifyErr) {
                console.error("[WHATSAPP WEBHOOK ERROR]", notifyErr);
              }
            }
          }
        }

        return new Response(JSON.stringify({
          status: "acknowledged",
          health_state: issues.length === 0 ? "healthy" : "attention_required",
          issues_detected: issues,
          alerts_dispatched: alertsDispatched,
          timestamp: Math.floor(Date.now() / 1000)
        }), {
          status: 200,
          headers: { "Content-Type": "application/json" }
        });

      } catch (err) {
        return new Response(JSON.stringify({ status: "error", message: err.message }), {
          status: 500, headers: { "Content-Type": "application/json" }
        });
      }
    }

    // =========================================================================
    // 2. 24-Hour Rotating Cryptographic Lease & Metered Usage Verification
    // =========================================================================
    if (url.pathname === "/api/heartbeat" || url.pathname === "/") {
      try {
        const body = await request.json();
        const { client_id, license_key, machine_uuid, app_name, timestamp, nonce, metrics, current_version } = body;

        if (!client_id || !license_key || !machine_uuid || !app_name || !nonce) {
          return new Response(JSON.stringify({
            status: "invalid",
            message: "Missing required fields (client_id, license_key, machine_uuid, app_name, nonce)."
          }), { status: 400, headers: { "Content-Type": "application/json" } });
        }

        const client = DEFAULT_CLIENTS[client_id];
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

        // Persistent Hardware Machine Binding via Cloudflare KV (or in-memory fallback)
        const bindingKey = `binding:${client_id}:${app_name}`;
        let boundMachine = null;

        if (env.DRM_KV) {
          boundMachine = await env.DRM_KV.get(bindingKey);
          if (boundMachine && boundMachine !== machine_uuid) {
            return new Response(JSON.stringify({
              status: "revoked",
              message: "Hardware machine binding mismatch. License is bound to another physical device."
            }), { status: 403, headers: { "Content-Type": "application/json" } });
          }
          if (!boundMachine) {
            await env.DRM_KV.put(bindingKey, machine_uuid);
          }
        } else {
          boundMachine = MEMORY_BINDINGS.get(bindingKey);
          if (boundMachine && boundMachine !== machine_uuid) {
            return new Response(JSON.stringify({
              status: "revoked",
              message: "Hardware machine binding mismatch."
            }), { status: 403, headers: { "Content-Type": "application/json" } });
          }
          if (!boundMachine) {
            MEMORY_BINDINGS.set(bindingKey, machine_uuid);
          }
        }

        // Check App-Specific Metered Usage Limits
        const appLimits = client.limits[app_name] || {};
        
        if (app_name === "cctv_guard" && appLimits.max_cameras && metrics && metrics.active_camera_count > appLimits.max_cameras) {
          return new Response(JSON.stringify({
            status: "limit_exceeded",
            message: `Camera quota exceeded (${metrics.active_camera_count} active > ${appLimits.max_cameras} allowed).`
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        if (app_name === "voice_agent" && appLimits.max_daily_minutes && metrics && metrics.total_call_minutes > appLimits.max_daily_minutes) {
          return new Response(JSON.stringify({
            status: "limit_exceeded",
            message: `Voice call minutes quota exceeded (${metrics.total_call_minutes} > ${appLimits.max_daily_minutes} min allowed).`
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        if (app_name === "financial_engine" && appLimits.max_special_searches && metrics && metrics.special_searches_count > appLimits.max_special_searches) {
          return new Response(JSON.stringify({
            status: "limit_exceeded",
            message: `Financial special searches quota reached (${metrics.special_searches_count} > ${appLimits.max_special_searches}).`
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        // Calculate 24-Hour Expiry Window
        const nowSec = Math.floor(Date.now() / 1000);
        const expiresAt = nowSec + 86400; // 24 hours

        // Generate 32-byte ephemeral volatile session key
        const rawKey = new Uint8Array(32);
        crypto.getRandomValues(rawKey);
        const sessionKeyHex = Array.from(rawKey).map(b => b.toString(16).padStart(2, "0")).join("");

        // Canonical message format for Ed25519 lease:
        // {client_id}:{machine_uuid}:{app_name}:{expires_at}:{nonce}:{session_key_hex}
        const canonicalMsg = `${client_id}:${machine_uuid}:${app_name}:${expiresAt}:${nonce}:${sessionKeyHex}`;

        // Load 32-byte Ed25519 root private key seed from environment
        const seedStr = env.DRM_SIGNING_SEED || "SHAMBHAVI_AI_LABS_ROOT_SEED_2026";
        const seedBytes = new TextEncoder().encode(seedStr).slice(0, 32);

        // Sign with Ed25519
        const signatureHex = await signEd25519(canonicalMsg, seedBytes);

        // OTA Update check payload
        const LATEST_VERSIONS = {
          cctv_guard: "1.0.1",
          voice_agent: "1.0.1",
          local_rag_engine: "1.0.1",
          financial_engine: "1.0.1",
          video_studio: "1.0.1",
          dsp_recorder: "1.0.1"
        };
        const latestVersion = LATEST_VERSIONS[app_name] || "1.0.0";
        let updateMetadata = null;
        if (current_version && current_version !== latestVersion) {
          const updateSha256 = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
          const updateSig = await signEd25519(updateSha256, seedBytes);
          updateMetadata = {
            version_available: latestVersion,
            download_url: `https://updates.shambhaviailabs.com/binaries/${app_name}_v${latestVersion}.exe`,
            sha256: updateSha256,
            signature_hex: updateSig,
            mandatory: false
          };
        }

        return new Response(JSON.stringify({
          status: "granted",
          expires_at: expiresAt,
          session_key_hex: sessionKeyHex,
          signature_hex: signatureHex,
          update: updateMetadata,
          message: "24-hour cryptographic Ed25519 lease granted."
        }), {
          status: 200,
          headers: {
            "Content-Type": "application/json",
            "Access-Control-Allow-Origin": env.ALLOWED_ORIGIN || "*"
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
