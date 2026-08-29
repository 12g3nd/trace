export type PendingEditFlusher = () => Promise<void>;

const pendingEditFlushers = new Set<PendingEditFlusher>();

export function registerPendingTaskEdit(flush: PendingEditFlusher): () => void {
  pendingEditFlushers.add(flush);
  return () => pendingEditFlushers.delete(flush);
}

export async function flushPendingTaskEdits(): Promise<void> {
  await Promise.all([...pendingEditFlushers].map((flush) => flush()));
}
