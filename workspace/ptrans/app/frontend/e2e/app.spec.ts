import { test, expect } from "@playwright/test";

/**
 * E2E-тесты для zk-pool frontend.
 *
 * Тестируют только UI (рендеринг, наличие элементов).
 * Реальный deposit/withdraw не тестируется — требует Phantom + devnet + ZK-proof.
 */

test.describe("zk-pool app", () => {
  test.beforeEach(async ({ page }) => {
    await page.goto("/");
  });

  test("renders main page with header", async ({ page }) => {
    await expect(page.locator("h1")).toContainText("Private Transfers on Solana");
  });

  test("shows tagline", async ({ page }) => {
    await expect(page.getByText("Zero-knowledge private pool")).toBeVisible();
  });

  test("shows wallet connect section", async ({ page }) => {
    // Phantom/Solflare не установлены в Playwright — показывается "Connect" или "Install wallet"
    const walletSection = page.locator("text=/Wallet|Connect/i").first();
    await expect(walletSection).toBeVisible();
  });

  test("renders Deposit form", async ({ page }) => {
    await expect(page.getByText("Deposit SOL")).toBeVisible();
  });

  test("renders Withdraw form", async ({ page }) => {
    await expect(page.getByText("Withdraw SOL")).toBeVisible();
  });

  test("renders footer with tech stack", async ({ page }) => {
    await expect(page.getByText(/Noir/)).toBeVisible();
    await expect(page.getByText(/Sunspot/)).toBeVisible();
  });
});
