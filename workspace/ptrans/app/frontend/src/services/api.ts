const API_URL = import.meta.env.VITE_API_URL || "http://localhost:4001";

export interface WithdrawRequest {
  nullifierSecret: string; // hex
  secret: string; // hex
  amount: number; // lamports
  recipient: string; // base58 pubkey
}

export interface WithdrawResponse {
  txSignature: string;
  nullifierHash: string;
  proof: string; // base64
  root: string; // hex
  proofSize: number;
}

export interface PoolInfoResponse {
  address: string;
  vaultAddress: string;
  nextLeafIndex: number;
  totalDeposits: number;
  currentRoot: string;
  vaultBalance: number;
  vaultBalanceSol: number;
  nullifiersCount: number;
}

export interface DepositResponse {
  txSignature: string;
  commitment: string;
  leafIndex: number;
}

export interface DepositRequest {
  commitment: string; // hex
  amount: number; // lamports
}

/**
 * Отправить запрос на депозит (только commitment)
 */
export async function deposit(request: DepositRequest): Promise<DepositResponse> {
  const response = await fetch(`${API_URL}/api/deposit`, {
    method: "POST",
    headers: {
      "Content-Type": "application/json",
    },
    body: JSON.stringify(request),
  });

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || "Deposit failed");
  }

  return response.json();
}

/**
 * Отправить запрос на вывод (секреты для генерации proof)
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
    const error = await response.json();
    throw new Error(error.error || "Withdraw failed");
  }

  return response.json();
}

/**
 * Получить информацию о пуле
 */
export async function getPoolInfo(): Promise<PoolInfoResponse> {
  const response = await fetch(`${API_URL}/api/pool`);

  if (!response.ok) {
    const error = await response.json();
    throw new Error(error.error || "Failed to get pool info");
  }

  return response.json();
}

/**
 * Проверить здоровье бэкенда
 */
export async function healthCheck(): Promise<{ status: string; version: string; network: string }> {
  const response = await fetch(`${API_URL}/health`);

  if (!response.ok) {
    throw new Error("Health check failed");
  }

  return response.json();
}
