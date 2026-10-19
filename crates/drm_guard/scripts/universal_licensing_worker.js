/**
 * Shambhavi AI Labs — Cloudflare Worker: Universal Edge Licensing & Metered Usage Gateway
 * 
 * Powered by:
 *   - Cloudflare D1 (Serverless Edge SQL Engine)
 *   - Cloudflare KV (Ultra-Fast Hardware Binding & Device Cache)
 *   - RFC 8410 Ed25519 Asymmetric Cryptographic Lease Signing (Web Crypto API)
 *   - 10-Minute Health Heartbeat, Surveillance Cable Fault Telemetry & WhatsApp Alerts
 *   - Universal Metered Quotas for all 8 Member Applications
 */

// Production Fallback Registry for offline/local initialization
const DEFAULT_CLIENTS = {
  "sme_client_001": {
    licenseKey: "KEY-ENTERPRISE-2026",
    active: true,
    allowedApps: [
      "cctv_guard", "nvr_core", "voice_agent", "dsp_recorder",
      "video_studio", "financial_engine", "local_rag_engine",
      "manifestation_app", "content_engine"
    ],
    limits: {
      cctv_guard: { max_cameras: 32 },
      nvr_core: { max_cameras: 32 },
      voice_agent: { max_daily_minutes: 5000, max_calls: 2000 },
      dsp_recorder: { max_recording_hours: 500 },
      video_studio: { max_render_minutes: 1000 },
      financial_engine: { max_special_searches: 5000 },
      local_rag_engine: { max_documents: 500000, max_daily_queries: 20000 },
      manifestation_app: { max_vision_renders: 1000 },
      content_engine: { max_pipelines_daily: 200 }
    },
    contact: {
      whatsapp: null
    }
  },
  "client_demo": {
    licenseKey: "DEMO-KEY-2026",
    active: true,
    allowedApps: [
      "cctv_guard", "nvr_core", "voice_agent", "dsp_recorder",
      "video_studio", "financial_engine", "local_rag_engine",
      "manifestation_app", "content_engine"
    ],
    limits: {
      cctv_guard: { max_cameras: 4 },
      nvr_core: { max_cameras: 4 },
      voice_agent: { max_daily_minutes: 60, max_calls: 50 },
      dsp_recorder: { max_recording_hours: 5 },
      video_studio: { max_render_minutes: 30 },
      financial_engine: { max_special_searches: 25 },
      local_rag_engine: { max_documents: 500, max_daily_queries: 100 },
      manifestation_app: { max_vision_renders: 20 },
      content_engine: { max_pipelines_daily: 5 }
    },
    contact: {
      whatsapp: null
    }
  }
};

// Memory fallback if KV is not bound
const MEMORY_BINDINGS = new Map();

/**
 * Signs canonical message with Ed25519 private key seed using Web Crypto API.
 * Converts 32-byte seed into standard PKCS#8 DER format (RFC 8410).
 */
async function signEd25519(canonicalMessage, seedBytes) {
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
          "Access-Control-Allow-Methods": "POST, GET, OPTIONS",
          "Access-Control-Allow-Headers": "Content-Type, Authorization, X-Provider-Signature",
        },
      });
    }

    const url = new URL(request.url);

    // =========================================================================
    // 0. Public Health Check & Status Endpoint
    // =========================================================================
    if (request.method === "GET" && (url.pathname === "/" || url.pathname === "/health")) {
      return new Response(JSON.stringify({
        service: "Shambhavi AI Labs — Universal Edge Licensing Gateway",
        status: "operational",
        edge_runtime: "Cloudflare Workers",
        database_engine: env.DB ? "Cloudflare D1 (Active)" : "In-Memory Fallback",
        kv_cache: env.DRM_KV ? "Cloudflare KV (Active)" : "Memory Fallback",
        supported_apps: [
          "cctv_guard", "nvr_core", "voice_agent", "dsp_recorder",
          "video_studio", "financial_engine", "local_rag_engine",
          "manifestation_app", "content_engine"
        ],
        timestamp: Math.floor(Date.now() / 1000)
      }), {
        status: 200,
        headers: { "Content-Type": "application/json", "Access-Control-Allow-Origin": "*" }
      });
    }

    if (request.method !== "POST") {
      return new Response(JSON.stringify({ error: "Method not allowed. Use POST." }), {
        status: 405,
        headers: { "Content-Type": "application/json" }
      });
    }

    // =========================================================================
    // 1. 10-Minute Proactive Health Telemetry Endpoint
    // =========================================================================
    if (url.pathname === "/api/telemetry/heartbeat-10m") {
      try {
        const authHeader = request.headers.get("Authorization") || "";
        const token = authHeader.startsWith("Bearer ") ? authHeader.slice(7).trim() : null;

        const body = await request.json();
        const { client_id, machine_uuid, app_name, wire_faults, offline_cameras, storage_pct, metrics } = body;
        const nowSec = Math.floor(Date.now() / 1000);

        // Fetch client credentials from D1 or fallback
        let client = null;
        if (env.DB) {
          try {
            client = await env.DB.prepare(
              "SELECT * FROM clients WHERE client_id = ?"
            ).bind(client_id).first();
          } catch (d1Err) {
            console.error("[D1 QUERY ERROR]", d1Err);
          }
        }
        if (!client && DEFAULT_CLIENTS[client_id]) {
          client = {
            client_id,
            license_key: DEFAULT_CLIENTS[client_id].licenseKey,
            active: DEFAULT_CLIENTS[client_id].active ? 1 : 0
          };
        }

        const expectedToken = env.TELEMETRY_SHARED_TOKEN || (client ? client.license_key : null);
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
          issues.push(`⚠️ Loose wire or physical cable disconnect detected (${wire_faults} events).`);
        }
        if (offline_cameras && offline_cameras.length > 0) {
          issues.push(`🚨 Camera(s) offline: ${offline_cameras.join(", ")}`);
        }
        if (storage_pct && storage_pct >= 90) {
          issues.push(`💾 Storage critical: ${storage_pct}% full.`);
        }

        // WhatsApp / Webhook Notification
        if (issues.length > 0) {
          const recipientPhone = client?.contact_phone || env.ADMIN_WHATSAPP || null;
          if (recipientPhone && env.WHATSAPP_API_URL && env.WHATSAPP_TOKEN) {
            try {
              await fetch(env.WHATSAPP_API_URL, {
                method: "POST",
                headers: {
                  "Authorization": `Bearer ${env.WHATSAPP_TOKEN}`,
                  "Content-Type": "application/json"
                },
                body: JSON.stringify({
                  to: recipientPhone,
                  message: `[Shambhavi AI Pehredar Box Alert]\nClient: ${client_id}\nMachine: ${machine_uuid}\n${issues.join("\n")}\nPlease inspect your local appliance.`
                })
              });
              alertsDispatched++;
            } catch (notifyErr) {
              console.error("[WHATSAPP WEBHOOK ERROR]", notifyErr);
            }
          }
        }

        // Persist Telemetry Log to Cloudflare D1
        if (env.DB) {
          try {
            await env.DB.prepare(`
              INSERT INTO telemetry_logs (
                client_id, app_name, machine_uuid, timestamp, metrics_json,
                wire_faults, offline_cameras_json, storage_pct, alert_dispatched, created_at
              ) VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
            `).bind(
              client_id,
              app_name || "cctv_guard",
              machine_uuid,
              nowSec,
              JSON.stringify(metrics || {}),
              wire_faults || 0,
              JSON.stringify(offline_cameras || []),
              storage_pct || 0.0,
              alertsDispatched,
              nowSec
            ).run();
          } catch (logErr) {
            console.error("[D1 TELEMETRY LOG INSERT ERROR]", logErr);
          }
        }

        return new Response(JSON.stringify({
          status: "acknowledged",
          health_state: issues.length === 0 ? "healthy" : "attention_required",
          issues_detected: issues,
          alerts_dispatched: alertsDispatched,
          timestamp: nowSec
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
        const nowSec = Math.floor(Date.now() / 1000);

        if (!client_id || !license_key || !machine_uuid || !app_name || !nonce) {
          return new Response(JSON.stringify({
            status: "invalid",
            message: "Missing required fields (client_id, license_key, machine_uuid, app_name, nonce)."
          }), { status: 400, headers: { "Content-Type": "application/json" } });
        }

        // 1. Fetch Client from Cloudflare D1 or Fallback
        let client = null;
        if (env.DB) {
          try {
            client = await env.DB.prepare(
              "SELECT * FROM clients WHERE client_id = ? AND license_key = ?"
            ).bind(client_id, license_key).first();
          } catch (d1Err) {
            console.error("[D1 CLIENT LOOKUP ERROR]", d1Err);
          }
        }

        if (!client) {
          const fallback = DEFAULT_CLIENTS[client_id];
          if (fallback && fallback.licenseKey === license_key) {
            client = {
              client_id,
              active: fallback.active ? 1 : 0,
              plan_tier: "enterprise"
            };
          }
        }

        if (!client) {
          return new Response(JSON.stringify({
            status: "revoked",
            message: "Invalid client credentials or license key."
          }), { status: 401, headers: { "Content-Type": "application/json" } });
        }

        if (!client.active) {
          return new Response(JSON.stringify({
            status: "revoked",
            message: "Client subscription account is suspended or overdue."
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        // 2. Validate App Permissions & Metered Quotas from D1
        let appLimits = {};
        let isAppAllowed = false;

        if (env.DB) {
          try {
            const licenseRow = await env.DB.prepare(
              "SELECT allowed, limits_json FROM app_licenses WHERE client_id = ? AND app_name = ?"
            ).bind(client_id, app_name).first();

            if (licenseRow) {
              isAppAllowed = Boolean(licenseRow.allowed);
              appLimits = JSON.parse(licenseRow.limits_json || "{}");
            }
          } catch (d1AppErr) {
            console.error("[D1 APP LICENSE QUERY ERROR]", d1AppErr);
          }
        }

        // Fallback checks
        if (!isAppAllowed && DEFAULT_CLIENTS[client_id]) {
          const fb = DEFAULT_CLIENTS[client_id];
          if (fb.allowedApps.includes(app_name)) {
            isAppAllowed = true;
            appLimits = fb.limits[app_name] || {};
          }
        }

        if (!isAppAllowed) {
          return new Response(JSON.stringify({
            status: "revoked",
            message: `Application '${app_name}' is not licensed under your subscription plan.`
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        // 3. Hardware Device Binding Check (Cloudflare KV + D1)
        const bindingKey = `binding:${client_id}:${app_name}`;
        let boundMachine = null;

        // KV Fast Path
        if (env.DRM_KV) {
          boundMachine = await env.DRM_KV.get(bindingKey);
        } else {
          boundMachine = MEMORY_BINDINGS.get(bindingKey);
        }

        // D1 Persistent Path
        if (!boundMachine && env.DB) {
          try {
            const bindRow = await env.DB.prepare(
              "SELECT machine_uuid FROM device_bindings WHERE client_id = ? AND app_name = ?"
            ).bind(client_id, app_name).first();
            if (bindRow) {
              boundMachine = bindRow.machine_uuid;
              if (env.DRM_KV) await env.DRM_KV.put(bindingKey, boundMachine);
            }
          } catch (bindErr) {
            console.error("[D1 BINDING QUERY ERROR]", bindErr);
          }
        }

        if (boundMachine && boundMachine !== machine_uuid) {
          return new Response(JSON.stringify({
            status: "revoked",
            message: "Hardware machine binding mismatch. License is bound to another physical machine."
          }), { status: 403, headers: { "Content-Type": "application/json" } });
        }

        if (!boundMachine) {
          // Bind new device
          if (env.DRM_KV) await env.DRM_KV.put(bindingKey, machine_uuid);
          MEMORY_BINDINGS.set(bindingKey, machine_uuid);

          if (env.DB) {
            try {
              await env.DB.prepare(`
                INSERT OR REPLACE INTO device_bindings (client_id, app_name, machine_uuid, bound_at, last_seen_at)
                VALUES (?, ?, ?, ?, ?)
              `).bind(client_id, app_name, machine_uuid, nowSec, nowSec).run();
            } catch (insertBindErr) {
              console.error("[D1 BIND INSERT ERROR]", insertBindErr);
            }
          }
        } else if (env.DB) {
          // Update last seen
          try {
            await env.DB.prepare(
              "UPDATE device_bindings SET last_seen_at = ? WHERE client_id = ? AND app_name = ?"
            ).bind(nowSec, client_id, app_name).run();
          } catch (_) {}
        }

        // 4. Verify Metered Usage Limits Across All 8 Apps
        if (metrics) {
          // CCTV Guard & NVR Core
          if ((app_name === "cctv_guard" || app_name === "nvr_core") && appLimits.max_cameras) {
            if (metrics.active_camera_count > appLimits.max_cameras) {
              return new Response(JSON.stringify({
                status: "limit_exceeded",
                message: `Surveillance quota exceeded (${metrics.active_camera_count} cameras > ${appLimits.max_cameras} allowed).`
              }), { status: 403, headers: { "Content-Type": "application/json" } });
            }
          }

          // Voice AI Agent
          if (app_name === "voice_agent" && appLimits.max_daily_minutes) {
            if (metrics.total_call_minutes > appLimits.max_daily_minutes) {
              return new Response(JSON.stringify({
                status: "limit_exceeded",
                message: `Voice call minutes quota exceeded (${metrics.total_call_minutes} > ${appLimits.max_daily_minutes} mins allowed).`
              }), { status: 403, headers: { "Content-Type": "application/json" } });
            }
          }

          // Financial Engine
          if (app_name === "financial_engine" && appLimits.max_special_searches) {
            if (metrics.special_searches_count > appLimits.max_special_searches) {
              return new Response(JSON.stringify({
                status: "limit_exceeded",
                message: `Financial special searches quota reached (${metrics.special_searches_count} > ${appLimits.max_special_searches}).`
              }), { status: 403, headers: { "Content-Type": "application/json" } });
            }
          }

          // DSP Recorder
          if (app_name === "dsp_recorder" && appLimits.max_recording_hours) {
            if (metrics.total_recording_hours > appLimits.max_recording_hours) {
              return new Response(JSON.stringify({
                status: "limit_exceeded",
                message: `DSP audio recording quota exceeded (${metrics.total_recording_hours} > ${appLimits.max_recording_hours} hours).`
              }), { status: 403, headers: { "Content-Type": "application/json" } });
            }
          }

          // Video Studio
          if (app_name === "video_studio" && appLimits.max_render_minutes) {
            if (metrics.total_render_minutes > appLimits.max_render_minutes) {
              return new Response(JSON.stringify({
                status: "limit_exceeded",
                message: `Video studio render quota reached (${metrics.total_render_minutes} > ${appLimits.max_render_minutes} minutes).`
              }), { status: 403, headers: { "Content-Type": "application/json" } });
            }
          }

          // Local RAG Engine
          if (app_name === "local_rag_engine" && appLimits.max_documents) {
            if (metrics.indexed_document_count > appLimits.max_documents) {
              return new Response(JSON.stringify({
                status: "limit_exceeded",
                message: `Document vault quota exceeded (${metrics.indexed_document_count} > ${appLimits.max_documents} documents).`
              }), { status: 403, headers: { "Content-Type": "application/json" } });
            }
          }

          // Manifestation App
          if (app_name === "manifestation_app" && appLimits.max_vision_renders) {
            if (metrics.vision_renders_count > appLimits.max_vision_renders) {
              return new Response(JSON.stringify({
                status: "limit_exceeded",
                message: `Vision board generation limit reached (${metrics.vision_renders_count} > ${appLimits.max_vision_renders}).`
              }), { status: 403, headers: { "Content-Type": "application/json" } });
            }
          }

          // Content Engine
          if (app_name === "content_engine" && appLimits.max_pipelines_daily) {
            if (metrics.pipelines_executed > appLimits.max_pipelines_daily) {
              return new Response(JSON.stringify({
                status: "limit_exceeded",
                message: `Content engine daily pipeline quota exceeded (${metrics.pipelines_executed} > ${appLimits.max_pipelines_daily}).`
              }), { status: 403, headers: { "Content-Type": "application/json" } });
            }
          }
        }

        // 5. Generate 24-Hour Rotating Cryptographic Lease
        const expiresAt = nowSec + 86400; // 24 hours validity

        // Ephemeral 32-byte volatile session key
        const rawKey = new Uint8Array(32);
        crypto.getRandomValues(rawKey);
        const sessionKeyHex = Array.from(rawKey).map(b => b.toString(16).padStart(2, "0")).join("");

        // Canonical message format for Ed25519 lease:
        // {client_id}:{machine_uuid}:{app_name}:{expires_at}:{nonce}:{session_key_hex}
        const canonicalMsg = `${client_id}:${machine_uuid}:${app_name}:${expiresAt}:${nonce}:${sessionKeyHex}`;

        // Root signing seed (RFC 8410 Web Crypto Ed25519)
        const seedStr = env.DRM_SIGNING_SEED || "SHAMBHAVI_AI_LABS_ROOT_SEED_2026";
        const seedBytes = new TextEncoder().encode(seedStr).slice(0, 32);

        const signatureHex = await signEd25519(canonicalMsg, seedBytes);

        // 6. Check for OTA Updates in D1 or default catalog
        let updateMetadata = null;
        if (env.DB && current_version) {
          try {
            const otaRow = await env.DB.prepare(
              "SELECT * FROM ota_releases WHERE app_name = ? AND version != ? ORDER BY id DESC LIMIT 1"
            ).bind(app_name, current_version).first();
            if (otaRow) {
              updateMetadata = {
                version_available: otaRow.version,
                download_url: otaRow.download_url,
                sha256: otaRow.sha256,
                signature_hex: otaRow.signature_hex,
                mandatory: Boolean(otaRow.mandatory)
              };
            }
          } catch (_) {}
        }

        return new Response(JSON.stringify({
          status: "granted",
          expires_at: expiresAt,
          session_key_hex: sessionKeyHex,
          signature_hex: signatureHex,
          update: updateMetadata,
          message: "24-hour cryptographic Ed25519 lease granted by Cloudflare D1 Edge."
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

    return new Response(JSON.stringify({ error: "Endpoint not found" }), {
      status: 404,
      headers: { "Content-Type": "application/json" }
    });
  }
};
