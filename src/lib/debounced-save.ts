export interface DebouncedSaver<T> {
  schedule(value: T): void;
  flush(): Promise<void>;
  cancel(): void;
}

/** Coalesces edit drafts without losing the latest value during an in-flight save. */
export function createDebouncedSaver<T>(
  delayMs: number,
  save: (value: T) => Promise<void>
): DebouncedSaver<T> {
  let timer: ReturnType<typeof setTimeout> | null = null;
  let pending: T | undefined;
  let hasPending = false;
  let saveChain = Promise.resolve();

  function clearTimer() {
    if (timer) clearTimeout(timer);
    timer = null;
  }

  async function flush(): Promise<void> {
    clearTimer();
    if (!hasPending) return saveChain;

    saveChain = saveChain.catch(() => undefined).then(async () => {
      while (hasPending) {
        const value = pending as T;
        pending = undefined;
        hasPending = false;
        try {
          await save(value);
        } catch (error) {
          if (!hasPending) {
            pending = value;
            hasPending = true;
          }
          throw error;
        }
      }
    });
    return saveChain;
  }

  return {
    schedule(value) {
      pending = value;
      hasPending = true;
      clearTimer();
      timer = setTimeout(() => void flush().catch(() => undefined), delayMs);
    },
    flush,
    cancel() {
      clearTimer();
      pending = undefined;
      hasPending = false;
    },
  };
}
