const decimal = new Intl.NumberFormat(undefined, { maximumFractionDigits: 1 });
const integer = new Intl.NumberFormat(undefined, { maximumFractionDigits: 0 });

export function formatPercent(value: number | null | undefined) {
  return value == null ? "Collecting…" : `${decimal.format(value)}%`;
}

export function formatBytes(value: number | null | undefined) {
  if (value == null) return "Unavailable";
  const units = ["B", "KiB", "MiB", "GiB", "TiB", "PiB"];
  let amount = value;
  let unit = 0;
  while (amount >= 1024 && unit < units.length - 1) {
    amount /= 1024;
    unit += 1;
  }
  return `${decimal.format(amount)} ${units[unit] ?? "PiB"}`;
}

export function formatRate(value: number | null | undefined) {
  return value == null ? "Collecting…" : `${formatBytes(value)}/s`;
}

export function formatCount(value: number) {
  return integer.format(value);
}

export function formatDuration(seconds: number) {
  const days = Math.floor(seconds / 86_400);
  const hours = Math.floor((seconds % 86_400) / 3_600);
  const minutes = Math.floor((seconds % 3_600) / 60);
  return [
    days ? `${String(days)}d` : "",
    hours ? `${String(hours)}h` : "",
    `${String(minutes)}m`,
  ]
    .filter(Boolean)
    .join(" ");
}

export function formatState(kind: string) {
  return kind.replaceAll("_", " ");
}
