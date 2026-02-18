import type { Page, Locator } from "@playwright/test";
import { expect } from "@playwright/test";

interface BoundingBox {
  x: number;
  y: number;
  width: number;
  height: number;
}

function rectsOverlap(a: BoundingBox, b: BoundingBox): boolean {
  return !(
    a.x + a.width <= b.x ||
    b.x + b.width <= a.x ||
    a.y + a.height <= b.y ||
    b.y + b.height <= a.y
  );
}

export async function assertNoOverlap(page: Page, selector: string) {
  const elements = await page.locator(selector).all();
  const rects: BoundingBox[] = [];

  for (const el of elements) {
    const box = await el.boundingBox();
    if (box) rects.push(box);
  }

  for (let i = 0; i < rects.length; i++) {
    for (let j = i + 1; j < rects.length; j++) {
      expect(
        rectsOverlap(rects[i], rects[j]),
        `Elements ${i} and ${j} overlap`,
      ).toBe(false);
    }
  }
}

export async function assertSingleRow(page: Page, selector: string) {
  const elements = await page.locator(selector).all();
  const tops: number[] = [];

  for (const el of elements) {
    const box = await el.boundingBox();
    if (box) tops.push(Math.round(box.y));
  }

  const uniqueTops = new Set(tops);
  expect(
    uniqueTops.size,
    `Expected all elements in single row, got ${uniqueTops.size} rows`,
  ).toBe(1);
}

export async function assertContainedIn(
  page: Page,
  childSelector: string,
  parentSelector: string,
) {
  const childBox = await page.locator(childSelector).boundingBox();
  const parentBox = await page.locator(parentSelector).boundingBox();

  expect(childBox).not.toBeNull();
  expect(parentBox).not.toBeNull();

  if (childBox && parentBox) {
    expect(childBox.x).toBeGreaterThanOrEqual(parentBox.x);
    expect(childBox.y).toBeGreaterThanOrEqual(parentBox.y);
    expect(childBox.x + childBox.width).toBeLessThanOrEqual(
      parentBox.x + parentBox.width,
    );
    expect(childBox.y + childBox.height).toBeLessThanOrEqual(
      parentBox.y + parentBox.height,
    );
  }
}
