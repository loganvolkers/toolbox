/** The hosted app supplies a same-origin request adapter. Offline reports use local previews. */
export type ActionRequest = (path: string, options: RequestInit) => Promise<{
  status: number;
  body: unknown;
}>;
