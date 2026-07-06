export const apiBase: string = import.meta.env.VITE_API_URL ?? "";

export async function request<T>(path: string, init?: RequestInit): Promise<T> {
  const response = await fetch(`${apiBase}${path}`, init);
  if (!response.ok) {
    const body = await response.json().catch(() => null);
    throw new Error(
      body?.description ?? `request failed with status ${response.status}`,
    );
  }
  return response.json();
}
