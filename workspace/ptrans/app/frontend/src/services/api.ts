const API_URL = import.meta.env.VITE_API_URL || "http://localhost:4001";

export interface WithdrawRequest {
  witness: string;
}

export interface WithdrawResponse {
  proof: string;
  public_witness: string;
}

export interface CommitmentEntry {
  leaf_index: number;
  commitment: string; // hex
}

export interface CommitmentsResponse {
  commitments: CommitmentEntry[];
}

export interface RootResponse {
  root: string; // hex
}

/**
 * Отправить witness на proof generation
 */
export async function withdraw(request: WithdrawRequest): Promise<WithdrawResponse> {
  const response = await fetch(`${API_URL}/api/withdraw`, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: "Unknown error" }));
    throw new Error(error.error || "Withdraw failed");
  }

  return response.json();
}

/**
 * Получить все commitments из БД
 */
export async function getCommitments(poolAddress: string): Promise<CommitmentsResponse> {
  const response = await fetch(
    `${API_URL}/api/commitments?pool_address=${encodeURIComponent(poolAddress)}`,
  );

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: "Unknown error" }));
    throw new Error(error.error || "Failed to fetch commitments");
  }

  return response.json();
}

/**
 * Получить последний root из БД
 */
export async function getLatestRoot(poolAddress: string): Promise<RootResponse> {
  const response = await fetch(
    `${API_URL}/api/root?pool_address=${encodeURIComponent(poolAddress)}`,
  );

  if (!response.ok) {
    const error = await response.json().catch(() => ({ error: "Unknown error" }));
    throw new Error(error.error || "Failed to fetch root");
  }

  return response.json();
}

/**
 * Проверить здоровье бэкенда
 */
export async function healthCheck(): Promise<{ status: string; db: string }> {
  const response = await fetch(`${API_URL}/api/health`);

  if (!response.ok) {
    throw new Error("Health check failed");
  }

  return response.json();
}
