import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest';
import { MediaPopoverIntent } from './media-popover-intent';

describe('MediaPopoverIntent', () => {
  beforeEach(() => vi.useFakeTimers());
  afterEach(() => vi.useRealTimers());

  function setup() {
    const show = vi.fn();
    const hide = vi.fn();
    const intent = new MediaPopoverIntent({ show, hide });
    intent.setContext(true, true);
    return { intent, show, hide };
  }

  it('requires an intentional compact-media hover before showing', () => {
    const { intent, show } = setup();
    intent.setCompactHovered(true);

    vi.advanceTimersByTime(219);
    expect(show).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(show).toHaveBeenCalledOnce();
  });

  it('keeps the popover open while crossing the physical gap', () => {
    const { intent, show, hide } = setup();
    intent.setCompactHovered(true);
    vi.advanceTimersByTime(220);
    expect(show).toHaveBeenCalledOnce();

    intent.setCompactHovered(false);
    vi.advanceTimersByTime(140);
    intent.setPopoverHovered(true);
    vi.advanceTimersByTime(220);
    expect(hide).not.toHaveBeenCalled();
  });

  it('hides after both media surfaces remain unhovered for the grace period', () => {
    const { intent, hide } = setup();
    intent.setCompactHovered(true);
    vi.advanceTimersByTime(220);
    intent.setCompactHovered(false);
    intent.setPopoverHovered(true);
    intent.setPopoverHovered(false);

    vi.advanceTimersByTime(219);
    expect(hide).not.toHaveBeenCalled();
    vi.advanceTimersByTime(1);
    expect(hide).toHaveBeenCalledOnce();
  });

  it('closes immediately when MEDIA is no longer selected', () => {
    const { intent, hide } = setup();
    intent.setCompactHovered(true);
    vi.advanceTimersByTime(220);

    intent.setContext(false, true);
    expect(hide).toHaveBeenCalledOnce();
  });

  it('does not show without an active media session', () => {
    const show = vi.fn();
    const intent = new MediaPopoverIntent({ show, hide: vi.fn() });
    intent.setContext(true, false);
    intent.setCompactHovered(true);
    vi.advanceTimersByTime(500);
    expect(show).not.toHaveBeenCalled();

    intent.setContext(true, true);
    vi.advanceTimersByTime(220);
    expect(show).toHaveBeenCalledOnce();
  });
});
