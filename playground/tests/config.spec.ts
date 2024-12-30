import { test, expect } from "@playwright/test";

test("home page opens", async ({ page }) => {
  // Navigate to the home page
  await page.goto("http://localhost:5173");

  // Check if the main heading or any expected element is visible
  // For a React app created by Vite + React template, the initial content often includes <h1>Vite + React</h1>
  await expect(page.getByRole("link", { name: "quary Quary" })).toBeVisible();

  // Click on format and check if the format page opens and shows sql
  await page.getByLabel("Format").click();

  // Replace the main with 'select foo.bar from table1 foo' and check if the secondary panel shows 'select foo.bar from table1 foo'
  await page
    .locator("#main")
    .getByRole("code")
    .locator("div")
    .filter({ hasText: "SELECT name from USERS" })
    .nth(3)
    .click();
  // Type the new value
  await page.keyboard.press("Control+A");
  await page.keyboard.press("Backspace");
  await page.keyboard.type("select foo.bar from table1 foo");
  await expect(page.locator("#secondary-panel")).toContainText(
    "select foo.bar from table1 foo",
  );

  // Click on the config
  await page.getByLabel("Settings").click();
  await expect(page.locator("#main")).toContainText("dialect = ansi");
  await expect(page.locator("#main")).toContainText("rules = core");

  // Change the rules to all
  await page
    .locator("#main")
    .getByRole("code")
    .locator("div")
    .filter({ hasText: "rules" })
    .nth(3)
    .click();
  // Type the new value
  await page.keyboard.press("Control+A");
  await page.keyboard.press("Backspace");
  await page.keyboard.type("[sqruff]\n" + "dialect = ansi\n" + "rules = all\n");
  await expect(page.locator("#secondary-panel")).toContainText(
    "select foo.bar from table1 as foo",
  );

  // Change the rule to AL01
  await page
    .locator("#main")
    .getByRole("code")
    .locator("div")
    .filter({ hasText: "rules" })
    .nth(3)
    .click();
  // Type the new value
  await page.keyboard.press("Control+A");
  await page.keyboard.press("Backspace");
  await page.keyboard.type(
    "[sqruff]\n" + "dialect = ansi\n" + "rules = AL01, CP01\n",
  );
  await expect(page.locator("#secondary-panel")).toContainText(
    "select foo.bar from table1 as foo",
  );
});