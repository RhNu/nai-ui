import type { Center } from "@/api/types";

export const POSITION_OPTIONS = (() => {
  const opts: string[] = [];
  for (let li = 0; li < 5; li++) {
    const letter = String.fromCharCode(65 + li);
    for (let ni = 1; ni <= 5; ni++) {
      opts.push(`${letter}${ni}`);
    }
  }
  return opts;
})();

export function positionToCenter(position: string): Center {
  const offset = 0.1;
  const letter = position[0] ?? "C";
  const number = position[1] ?? "3";
  const li = Math.min(4, Math.max(0, letter.toUpperCase().charCodeAt(0) - 65));
  const ni = Math.min(4, Math.max(0, parseInt(number, 10) - 1));
  return {
    x: Math.round((li * 0.2 + offset) * 10) / 10,
    y: Math.round((ni * 0.2 + offset) * 10) / 10,
  };
}

export function centerToPosition(center: Center): string {
  const offset = 0.1;
  const grid = [0, 1, 2, 3, 4].map(
    (i) => Math.round((i * 0.2 + offset) * 10) / 10
  );
  const nearestIndex = (v: number) => {
    let bestIdx = 0;
    let bestDist = Number.POSITIVE_INFINITY;
    for (let i = 0; i < grid.length; i++) {
      const d = Math.abs(grid[i] - v);
      if (d < bestDist) {
        bestDist = d;
        bestIdx = i;
      }
    }
    return bestIdx;
  };
  const li = nearestIndex(center.x);
  const ni = nearestIndex(center.y);
  const letter = String.fromCharCode(65 + li);
  const number = String(ni + 1);
  return `${letter}${number}`;
}
