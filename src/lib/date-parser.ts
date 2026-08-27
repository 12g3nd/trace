/**
 * Natural language date parsing utility for task capture.
 */

export interface ParsedDateResult {
  cleanedText: string;
  dueAt: string | null; // ISO 8601 string
}

const DAY_NAMES = ['sunday', 'monday', 'tuesday', 'wednesday', 'thursday', 'friday', 'saturday'];

export function parseNaturalDate(text: string, baseDate = new Date()): ParsedDateResult {
  let cleaned = text;
  let due: Date | null = null;

  // 1. Explicit syntax: due:YYYY-MM-DD or due:YYYY/MM/DD
  const explicitMatch = cleaned.match(/\bdue:(\d{4}[-/]\d{1,2}[-/]\d{1,2})\b/i);
  if (explicitMatch) {
    const d = new Date(explicitMatch[1].replace(/\//g, '-'));
    if (!isNaN(d.getTime())) {
      due = d;
      cleaned = cleaned.replace(explicitMatch[0], '').trim();
    }
  }

  // 2. "today"
  if (!due) {
    const todayMatch = cleaned.match(/\b(today|tonight)\b/i);
    if (todayMatch) {
      const d = new Date(baseDate);
      d.setHours(23, 59, 59, 999);
      due = d;
      cleaned = cleaned.replace(todayMatch[0], '').trim();
    }
  }

  // 3. "tomorrow"
  if (!due) {
    const tomorrowMatch = cleaned.match(/\b(tomorrow|tmrw)\b/i);
    if (tomorrowMatch) {
      const d = new Date(baseDate);
      d.setDate(d.getDate() + 1);
      d.setHours(23, 59, 59, 999);
      due = d;
      cleaned = cleaned.replace(tomorrowMatch[0], '').trim();
    }
  }

  // 4. "in N days" or "in N day"
  if (!due) {
    const inDaysMatch = cleaned.match(/\bin\s+(\d+)\s+days?\b/i);
    if (inDaysMatch) {
      const days = parseInt(inDaysMatch[1], 10);
      const d = new Date(baseDate);
      d.setDate(d.getDate() + days);
      d.setHours(23, 59, 59, 999);
      due = d;
      cleaned = cleaned.replace(inDaysMatch[0], '').trim();
    }
  }

  // 5. "in N hours" or "in N hrs"
  if (!due) {
    const inHoursMatch = cleaned.match(/\bin\s+(\d+)\s+(?:hours?|hrs?)\b/i);
    if (inHoursMatch) {
      const hours = parseInt(inHoursMatch[1], 10);
      const d = new Date(baseDate);
      d.setHours(d.getHours() + hours);
      due = d;
      cleaned = cleaned.replace(inHoursMatch[0], '').trim();
    }
  }

  // 6. Day of the week: "next friday", "friday", "this monday"
  if (!due) {
    const dayMatch = cleaned.match(/\b(?:next|this)?\s*(monday|tuesday|wednesday|thursday|friday|saturday|sunday)\b/i);
    if (dayMatch) {
      const targetDay = DAY_NAMES.indexOf(dayMatch[1].toLowerCase());
      if (targetDay !== -1) {
        const currentDay = baseDate.getDay();
        let daysUntil = (targetDay - currentDay + 7) % 7;
        if (daysUntil === 0) daysUntil = 7; // If today is Monday, "monday" means next Monday

        const d = new Date(baseDate);
        d.setDate(d.getDate() + daysUntil);
        d.setHours(23, 59, 59, 999);
        due = d;
        cleaned = cleaned.replace(dayMatch[0], '').trim();
      }
    }
  }

  // Clean up extra spaces
  cleaned = cleaned.replace(/\s+/g, ' ').trim();

  return {
    cleanedText: cleaned || text,
    dueAt: due ? due.toISOString() : null,
  };
}

/**
 * Format an ISO date string into a human-readable relative badge.
 */
export function formatRelativeDue(dueAt: string | null, now = new Date()): { label: string; isOverdue: boolean } | null {
  if (!dueAt) return null;

  const dueDate = new Date(dueAt);
  if (isNaN(dueDate.getTime())) return null;

  const diffMs = dueDate.getTime() - now.getTime();
  const diffDays = Math.round(diffMs / (1000 * 60 * 60 * 24));

  if (diffMs < 0 && Math.abs(diffDays) >= 1) {
    return { label: 'overdue', isOverdue: true };
  }

  if (diffDays === 0) {
    return { label: 'today', isOverdue: false };
  } else if (diffDays === 1) {
    return { label: 'tomorrow', isOverdue: false };
  } else if (diffDays > 1 && diffDays < 7) {
    return { label: DAY_NAMES[dueDate.getDay()].slice(0, 3), isOverdue: false };
  } else if (diffDays >= 7) {
    return { label: `${diffDays}d`, isOverdue: false };
  }

  return { label: 'due', isOverdue: false };
}
