import { expect, type Locator } from '@playwright/test'

interface Box {
  x: number
  y: number
  width: number
  height: number
}

function describeBox(box: Box | null): string {
  if (!box) return 'no rendered box'
  const { x, y, width, height } = box
  return `x=${Math.round(x)} y=${Math.round(y)} w=${Math.round(width)} h=${Math.round(height)} right=${Math.round(x + width)} bottom=${Math.round(y + height)}`
}

function intersects(first: Box, second: Box): boolean {
  return (
    first.x < second.x + second.width &&
    second.x < first.x + first.width &&
    first.y < second.y + second.height &&
    second.y < first.y + first.height
  )
}

function contains(outer: Box, inner: Box): boolean {
  return (
    inner.x >= outer.x &&
    inner.y >= outer.y &&
    inner.x + inner.width <= outer.x + outer.width &&
    inner.y + inner.height <= outer.y + outer.height
  )
}

/**
 * A locator that resolves to nothing is a selector mistake, not a cleared obstruction, so
 * every geometry assertion demands the element exists before it reads a rectangle. A present
 * element with no rendered box (zero height, `display: none`) has no area and cannot overlap.
 */
async function boxOf(locator: Locator, label: string): Promise<Box | null> {
  await expect(locator, `${label} must exist before its geometry is compared`).not.toHaveCount(0)
  return locator.first().boundingBox()
}

export async function expectNoOverlap(
  subject: Locator,
  subjectLabel: string,
  other: Locator,
  otherLabel: string,
): Promise<void> {
  const subjectBox = await boxOf(subject, subjectLabel)
  const otherBox = await boxOf(other, otherLabel)
  if (!subjectBox || !otherBox) return
  expect(
    intersects(subjectBox, otherBox),
    `${subjectLabel} [${describeBox(subjectBox)}] overlaps ${otherLabel} [${describeBox(otherBox)}]`,
  ).toBe(false)
}

/**
 * `insetTop` trims the subject's rectangle from the top before comparing. Artwork bleeds a
 * declared distance up behind the content above it inside a fade where it is a trace, so the
 * rectangle that has to stay clear is its legible remainder, not its whole box.
 */
export async function expectClearOfAll(
  subject: Locator,
  subjectLabel: string,
  group: Locator,
  groupLabel: string,
  { insetTop = 0 }: { insetTop?: number } = {},
): Promise<void> {
  const rawBox = await boxOf(subject, subjectLabel)
  const subjectBox = rawBox && {
    ...rawBox,
    y: rawBox.y + insetTop,
    height: rawBox.height - insetTop,
  }
  if (!subjectBox || subjectBox.height <= 0) return
  const members = await group.all()
  expect(
    members.length,
    `${groupLabel} must exist before its geometry is compared`,
  ).toBeGreaterThan(0)
  const overlapping: string[] = []
  for (const [index, member] of members.entries()) {
    const memberBox = await member.boundingBox()
    if (memberBox && intersects(subjectBox, memberBox)) {
      overlapping.push(`${groupLabel}[${index}] [${describeBox(memberBox)}]`)
    }
  }
  expect(
    overlapping,
    `${subjectLabel} [${describeBox(subjectBox)}] overlaps ${overlapping.length} of ${members.length} ${groupLabel} elements`,
  ).toEqual([])
}

export async function expectContained(
  inner: Locator,
  innerLabel: string,
  outer: Locator,
  outerLabel: string,
): Promise<void> {
  const innerBox = await boxOf(inner, innerLabel)
  const outerBox = await boxOf(outer, outerLabel)
  expect(innerBox, `${innerLabel} must render a box inside ${outerLabel}`).not.toBeNull()
  expect(outerBox, `${outerLabel} must render a box`).not.toBeNull()
  if (!innerBox || !outerBox) return
  expect(
    contains(outerBox, innerBox),
    `${innerLabel} [${describeBox(innerBox)}] is not inside ${outerLabel} [${describeBox(outerBox)}]`,
  ).toBe(true)
}
