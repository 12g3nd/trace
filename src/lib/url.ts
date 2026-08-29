export interface TaskLinkValidation {
  value: string | null;
  error: string | null;
}

const BARE_DOMAIN = /^[a-z0-9](?:[a-z0-9-]*[a-z0-9])?(?:\.[a-z0-9](?:[a-z0-9-]*[a-z0-9])?)+(?:\:\d+)?(?:[/?#]\S*)?$/i;

export function normalizeTaskLink(input: string | null | undefined): TaskLinkValidation {
  const trimmed = input?.trim() ?? '';
  if (!trimmed) return { value: null, error: null };

  const candidate = /^https?:\/\//i.test(trimmed)
    ? trimmed
    : BARE_DOMAIN.test(trimmed)
      ? `https://${trimmed}`
      : null;

  if (!candidate) {
    return { value: null, error: 'Use an http:// or https:// URL.' };
  }

  try {
    const parsed = new URL(candidate);
    if (!parsed.hostname || (parsed.protocol !== 'http:' && parsed.protocol !== 'https:')) {
      throw new Error('unsupported URL');
    }
    return { value: candidate, error: null };
  } catch {
    return { value: null, error: 'Enter a valid web address.' };
  }
}
