import clipboardCopy from 'clipboard-copy';
import UIkit from 'uikit';

/**
 * Displays an error UI popup with the given error `message`.
 *
 * @param message    Error message to be displayed.
 */
export function showError(message: string) {
  // Register global 'copy to clipboard' function. It's used in onclick handler of notification message
  const win = window as any;
  if (!Boolean(win.copyToClipboard)) {
    win.copyToClipboard = async (message) => {
      await copyToClipboard(atob(message));
    };
  }

  const maxAllowedLength = 300;
  let base64Message = btoa(message);

  message =
    message.length > maxAllowedLength
      ? `${message.substring(0, maxAllowedLength)} ...`
      : message;

  const htmlMessage = `${message}<button class="uk-icon-link uk-margin-small-left" uk-icon="copy" onclick="copyToClipboard('${base64Message}');"></button>`;
  UIkit.notification(htmlMessage, {
    status: 'danger',
    pos: 'top-right',
    timeout: 300_000 /* 5 min */,
  });
}

/**
 * Copies the given `text` to clipboard and displays a success UI popup when
 * it's done.
 *
 * @param text    Text to be copied to clipboard.
 */
export async function copyToClipboard(text: string) {
  await clipboardCopy(text);
  UIkit.notification('Copied', {
    status: 'success',
    pos: 'top-center',
    timeout: 1500,
  });
}

/**
 * Sanitizes the given `label` by replacing any space-like characters sequences
 * with a single space.
 *
 * @param label    Label to be sanitized.
 *
 * @returns    Sanitized label.
 */
export function sanitizeLabel(label: string): string {
  return label.replace(/[\s]+/g, ' ').trim();
}

/**
 * Sanitizes the given `url` by removing any space-like characters from it.
 *
 * @param url    URL to be sanitized.
 *
 * @returns    Sanitized URL.
 */
export function sanitizeUrl(url: string): string {
  return url.replace(/[\s]+/g, '');
}

/**
 * Checks whether the given location page corresponds to
 * `/restream/#/id/:restream_id/output/:output_id` route.
 */
export function isOutputPage(): boolean {
  const pathname = window.location.pathname;
  const p = window.location.hash.split('/');

  return pathname === '/restream/' && p[3] === 'output';
}

const YT_VIDEO_REGEX = /(?:youtube(?:-nocookie)?\.com\/(?:[^\/\n\s]+\/\S+\/|(?:v|e(?:mbed)?)\/|\S*?[?&]v=)|youtu\.be\/)([a-zA-Z0-9_-]{11})/;

export const isYoutubeVideo = (url: string): boolean => {
  return YT_VIDEO_REGEX.test(url);
};

export const getYoutubeVideoID = (url: string): string | undefined => {
  const result = url.match(YT_VIDEO_REGEX);
  return result && result.length ? result[1] : undefined;
};
