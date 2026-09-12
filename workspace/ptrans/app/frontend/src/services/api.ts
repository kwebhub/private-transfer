const API_URL = import.meta.env.VITE_API_URL || "http://localhost:4001";

export interface WithdrawRequest {
  witness: string; // base64 (gzip witness от noir_js)
}

export interface WithdrawResponse {
  proof: string; // base64 — 324 байта
  public_witness: string; // base64 — 140 байт
}

/**
 * Отправить witness на proof generation
 */
export async function withdraw(request: WithdrawRequest): Promise<WithdrawResponse> {
  const response = await fetch(`${API_URL}/api/withdraw`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: "Unknown error" }));
    throw new Error(error.error || "Withdraw failed");
  }

  return response.json();
}
