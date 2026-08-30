export interface MediaPopoverIntentOptions {
  show: () => void;
  hide: () => void;
  showDelayMs?: number;
  hideDelayMs?: number;
}

export class MediaPopoverIntent {
  private readonly show: () => void;
  private readonly hide: () => void;
  private readonly showDelayMs: number;
  private readonly hideDelayMs: number;
  private showTimer: ReturnType<typeof setTimeout> | undefined;
  private hideTimer: ReturnType<typeof setTimeout> | undefined;
  private mediaActive = false;
  private mediaAvailable = false;
  private compactHovered = false;
  private popoverHovered = false;
  private open = false;

  constructor(options: MediaPopoverIntentOptions) {
    this.show = options.show;
    this.hide = options.hide;
    this.showDelayMs = options.showDelayMs ?? 220;
    this.hideDelayMs = options.hideDelayMs ?? 220;
  }

  setContext(mediaActive: boolean, mediaAvailable: boolean): void {
    const wasEligible = this.eligible();
    this.mediaActive = mediaActive;
    this.mediaAvailable = mediaAvailable;

    if (!this.eligible()) {
      this.cancelShow();
      this.cancelHide();
      if (wasEligible || this.open) this.close();
      return;
    }

    if (this.compactHovered && !this.open) this.scheduleShow();
  }

  setCompactHovered(hovered: boolean): void {
    this.compactHovered = hovered;
    if (hovered) {
      this.cancelHide();
      if (this.eligible() && !this.open) this.scheduleShow();
      return;
    }

    this.cancelShow();
    if (!this.popoverHovered) this.scheduleHide();
  }

  setPopoverHovered(hovered: boolean): void {
    this.popoverHovered = hovered;
    if (hovered) {
      this.cancelHide();
    } else if (!this.compactHovered) {
      this.scheduleHide();
    }
  }

  hideImmediately(): void {
    this.cancelShow();
    this.cancelHide();
    this.close();
  }

  destroy(): void {
    this.hideImmediately();
  }

  private eligible(): boolean {
    return this.mediaActive && this.mediaAvailable;
  }

  private scheduleShow(): void {
    if (this.showTimer || this.open) return;
    this.showTimer = setTimeout(() => {
      this.showTimer = undefined;
      if (!this.eligible() || !this.compactHovered || this.open) return;
      this.open = true;
      this.show();
    }, this.showDelayMs);
  }

  private scheduleHide(): void {
    if (!this.open || this.hideTimer) return;
    this.hideTimer = setTimeout(() => {
      this.hideTimer = undefined;
      if (this.compactHovered || this.popoverHovered) return;
      this.close();
    }, this.hideDelayMs);
  }

  private close(): void {
    if (!this.open) return;
    this.open = false;
    this.hide();
  }

  private cancelShow(): void {
    if (!this.showTimer) return;
    clearTimeout(this.showTimer);
    this.showTimer = undefined;
  }

  private cancelHide(): void {
    if (!this.hideTimer) return;
    clearTimeout(this.hideTimer);
    this.hideTimer = undefined;
  }
}
