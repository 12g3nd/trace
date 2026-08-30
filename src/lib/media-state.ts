export interface MediaState {
  available: boolean;
  title: string;
  artist: string;
  source: string;
  artworkKey: string | null;
  artwork: string | null;
  playing: boolean;
  canToggle: boolean;
  canPrevious: boolean;
  canNext: boolean;
}

export const EMPTY_MEDIA: MediaState = {
  available: false,
  title: '',
  artist: '',
  source: '',
  artworkKey: null,
  artwork: null,
  playing: false,
  canToggle: false,
  canPrevious: false,
  canNext: false,
};

export const MEDIA_STATE_EVENT = 'orbit-media-state';
export const MEDIA_POPOVER_HOVER_EVENT = 'orbit-media-popover-hover';
export const MEDIA_REFRESH_EVENT = 'orbit-media-refresh';
