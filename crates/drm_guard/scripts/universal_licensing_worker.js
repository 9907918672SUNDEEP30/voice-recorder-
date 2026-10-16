/**
 * Cloudflare Worker: Universal Shared DRM & Metered Usage Licensing Server
 * 
 * Enforces hardware machine binding, client license validation, app-specific
 * metered usage constraints, 24-hour cryptographic lease token issuance,
 * and 10-minute proactive health telemetry (Loose Wire & Camera Down WhatsApp alerts).
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
      phone: "+919907918672",
      whatsapp: "+919907918672"
    }
  },
  "client_demo": {
    licenseKey: "DEMO-KEY-2026",
    active: true,
    boundMachineUuid: null,
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
      phone: "+919907918672",
      whatsapp: "+919907918672"
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

    // =========================================================================
    // 1. 10-Minute Proactive Health Telemetry Endpoint (CCTV Black-Box Wire & Camera Faults)
    // =========================================================================
    if (url.pathname === "/api/telemetry/heartbeat-10m") {
      try {
        const body = await request.json();
        const { client_id, machine_uuid, wire_faults, offline_cameras, storage_pct } = body;

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
          const client = AUTHORIZED_CLIENTS[client_id];
          const recipientPhone = client?.contact?.whatsapp || env.ADMIN_WHATSAPP || "+919907918672";
          
          console.log(`[10-MIN HEALTH ALERT] Dispatching to ${recipientPhone}:`, issues.join(" | "));
          
          // Optional Meta/Twilio WhatsApp webhook call
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

        return new Response(JSON.stringify({
          status: "acknowledged",
          health_state: issues.length === 0 ? "healthy" : "attention_required",
          issues_detected: issues,
          alerts_dispatched: alertsDispatched,
          timestamp: Math.floor(Date.now() / 1000)
        }), {
          status: 200,
          headers: { "Content-Type": "application/json", "Access-Control-Allow-Origin": "*" }
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
          updateMetadata = {
            version_available: latestVersion,
            download_url: `https://updates.shambhaviailabs.com/binaries/${app_name}_v${latestVersion}.exe`,
            sha256: "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855",
            mandatory: false
          };
        }

        return new Response(JSON.stringify({
          status: "granted",
          expires_at: expiresAt,
          session_key_hex: sessionKeyHex,
          signature_hex: signatureHex,
          update: updateMetadata,
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
