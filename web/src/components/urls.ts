const base = (import.meta.env.VITE_BACKEND_URL as string | undefined) ?? "";

export function assetUrl(path: string): string {
  if (!path) return path;
  if (!base) return path;
  return new URL(path, base).toString();
}

export function outputsUrl(relPath: string): string {
  const normalized = relPath.replaceAll("\\\\", "/").replaceAll("\\", "/");
  return assetUrl(`/outputs/${normalized}`);
}
