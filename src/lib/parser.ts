import type { ParsedInput } from './types';

/**
 * Parse raw capture input into structured task fields.
 *
 * Syntax (all parts optional except task text):
 *   "Do the thing ~ context ***"
 *
 * - Text after the last " ~ " becomes the context.
 * - Trailing asterisks set priority (capped at 5).
 * - If parsing extracts nothing useful, the full input becomes task text.
 */
export function parseInput(raw: string): ParsedInput {
  let text = raw.trim();
  let context: string | null = null;
  let priority = 0;

  if (!text) {
    return { text: '', context: null, priority: 0 };
  }

  // Extract context: split on the last " ~ " occurrence.
  const tildeIndex = text.lastIndexOf(' ~ ');
  if (tildeIndex !== -1) {
    const afterTilde = text.slice(tildeIndex + 3).trim();
    const beforeTilde = text.slice(0, tildeIndex).trim();

    // Check if the part after ~ is purely asterisks
    const pureStars = afterTilde.match(/^(\*+)$/);
    if (pureStars) {
      priority = Math.min(pureStars[1].length, 5);
      text = beforeTilde;
    } else {
      // Check if afterTilde has trailing asterisks after non-asterisk text
      const contextStarMatch = afterTilde.match(/^([^*]+?)\s*(\*+)$/);
      if (contextStarMatch) {
        context = contextStarMatch[1].trim().toLowerCase() || null;
        priority = Math.min(contextStarMatch[2].length, 5);
        text = beforeTilde;
      } else if (afterTilde) {
        context = afterTilde.toLowerCase();
        text = beforeTilde;
      }
    }
  }

  // If no priority was extracted from after the tilde, check for trailing stars on the text itself.
  if (priority === 0) {
    const starMatch = text.match(/^([^*]+?)\s*(\*+)$/);
    if (starMatch) {
      text = starMatch[1].trim();
      priority = Math.min(starMatch[2].length, 5);
    }
  }

  return { text, context, priority };
}
