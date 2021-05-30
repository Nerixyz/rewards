export function parseDuration(d: string): number | null {
  if (!d.length) {
    return null;
  }
  if (/^\d+$/.test(d)) {
    return Number(d);
  }

  const [fullMatch, num, unit] = d.match(/^(\d+\.?\d*)([smhd])$/) ?? [];
  if (!fullMatch) return null;

  const base = Number(num);

  return base * DURATIONS[unit as keyof typeof DURATIONS];
}

export function isValidDuration(s: unknown): boolean {
  if (typeof s === 'number') return true;
  if (typeof s !== 'string') return false;

  const dur = parseDuration(s);
  return dur !== null;
}

const DURATIONS = { s: 1, m: 60, h: 60 * 60, d: 24 * 60 * 60 };

export function isValidRewardDurationExpression(expr: string): boolean {
  if (expr.startsWith('rand')) {
    return /rand\(([^;]+);([^)]+)\)/.test(expr);
  }
  return true;
}
