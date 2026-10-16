// ==============================================================================
// Shambhavi AI Labs — 24-Hour Rotating Cryptographic Lease Worker
// Cloudflare Worker + Supabase REST Backend
// ==============================================================================

export default {
  async fetch(request, env) {
    if (request.method !== "POST") {
      return new Response(JSON.stringify({ error: "Method not allowed" }), { status: 405 });
    }

    try {
      const body = await request.json();
      const { box_id, machine_hash } = body;

      if (!box_id || !machine_hash) {
        return new Response(JSON.stringify({ error: "Missing box_id or machine_hash" }), { status: 400 });
      }

      // 1. Query Supabase for active subscription
      // const supabaseUrl = env.SUPABASE_URL;
      // const supabaseKey = env.SUPABASE_SERVICE_KEY;
      // const subRes = await fetch(`${supabaseUrl}/rest/v1/subscriptions?box_id=eq.${box_id}&select=*`, {
      //   headers: { "apikey": supabaseKey, "Authorization": `Bearer ${supabaseKey}` }
      // });
      // const subData = await subRes.json();

      // Simulated active check (replace with real Supabase record in production)
      const isSubscriptionPaid = true;

      if (!isSubscriptionPaid) {
        return new Response(JSON.stringify({
          status: "LOCKED",
          reason: "Monthly subscription payment pending. Please recharge at billing.shambhavi-ai-labs.com",
          renewal_url: "https://billing.shambhavi-ai-labs.com/renew"
        }), { status: 402 });
      }

      // 2. Generate 24-Hour Cryptographic Lease Token
      const now = new Date();
      const expiresAt = new Date(now.getTime() + 24 * 60 * 60 * 1000); // +24 Hours

      const payload = {
        box_id,
        machine_hash,
        tenant_id: "tenant_shambhavi_active",
        issued_at: now.toISOString(),
        expires_at: expiresAt.toISOString(),
        cameras_entitled: 8,
        signature_hex: "ed25519_signed_signature_shambhavi_v1"
      };

      return new Response(JSON.stringify({
        status: "ACTIVE",
        lease: payload
      }), {
        headers: { "Content-Type": "application/json" }
      });

    } catch (err) {
      return new Response(JSON.stringify({ error: err.message }), { status: 500 });
    }
  }
};
