export function fmtBytes(n: number): string {
  if (n < 1024) return `${n} B`;
  const u = ["KB", "MB", "GB"]; let v = n / 1024, i = 0;
  while (v >= 1024 && i < u.length - 1) { v /= 1024; i++; }
  return `${v.toFixed(v >= 100 ? 0 : 1)} ${u[i]}`;
}
export function fmtEta(remaining: number, bps: number): string | null {
  if (bps <= 0) return null;
  const s = Math.round(remaining / bps);
  return s >= 60 ? `${Math.floor(s / 60)} min ${s % 60}s` : `${s}s`;
}
export const STAGE_LABEL: Record<string, string> = {
  resolving: "Consultando fonte oficial…", downloading: "Baixando", verifying: "Verificando",
  extracting: "Extraindo com segurança…", installing: "Instalando…", done: "Instalado", error: "Erro", cancelled: "Cancelado",
};
