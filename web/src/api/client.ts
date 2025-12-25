const base = (import.meta.env.VITE_BACKEND_URL as string | undefined) ?? "";

function resolveUrl(path: string): string {
  return base ? new URL(path, base).toString() : path;
}

async function readErrorBody(res: Response): Promise<string> {
  const ct = res.headers.get("content-type") ?? "";
  if (ct.includes("application/json")) {
    try {
      const j = (await res.json()) as any;
      if (j && typeof j === "object" && typeof j.error === "string")
        return j.error;
      return JSON.stringify(j);
    } catch {
      // fallthrough
    }
  }
  return await res.text().catch(() => "");
}

export async function apiGet<T>(path: string): Promise<T> {
  const res = await fetch(resolveUrl(path));
  if (!res.ok) {
    const body = await readErrorBody(res);
    throw new Error(`GET ${path} failed: ${res.status} ${body}`);
  }
  return (await res.json()) as T;
}

export async function apiPost<TReq, TRes>(
  path: string,
  body: TReq
): Promise<TRes> {
  const res = await fetch(resolveUrl(path), {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await readErrorBody(res);
    throw new Error(`POST ${path} failed: ${res.status} ${err}`);
  }
  return (await res.json()) as TRes;
}

export async function apiPut<TReq, TRes>(
  path: string,
  body: TReq
): Promise<TRes> {
  const res = await fetch(resolveUrl(path), {
    method: "PUT",
    headers: { "content-type": "application/json" },
    body: JSON.stringify(body),
  });
  if (!res.ok) {
    const err = await readErrorBody(res);
    throw new Error(`PUT ${path} failed: ${res.status} ${err}`);
  }
  return (await res.json()) as TRes;
}

export async function apiDelete<TRes>(path: string): Promise<TRes> {
  const res = await fetch(resolveUrl(path), { method: "DELETE" });
  if (!res.ok) {
    const err = await readErrorBody(res);
    throw new Error(`DELETE ${path} failed: ${res.status} ${err}`);
  }
  return (await res.json()) as TRes;
}
