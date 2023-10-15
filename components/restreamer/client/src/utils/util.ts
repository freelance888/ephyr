import clipboardCopy from 'clipboard-copy';
import UIkit from 'uikit';
import { SubscriptionClient } from 'subscriptions-transport-ws';
import { ApolloClient } from '@apollo/client/core';
import { WebSocketLink } from '@apollo/client/link/ws';
import { InMemoryCache } from '@apollo/client/cache';
import isEqual from 'lodash/isEqual';
import take from 'lodash/take';
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

const MixPage = 'mix';
export function isMixPage(): boolean {
  return window.location.pathname === `/${MixPage}`;
}

export const getMixPageUrl = (restreamId: string, outputId: string) => {
  return `/${MixPage}?id=${restreamId}&output=${outputId}`;
};

const FullStreamPage = 'full-stream';
export function isFullStreamPage(): boolean {
  return window.location.pathname === `/${FullStreamPage}`;
}

export const getFullStreamUrl = (restreamId: string) => {
  return `/${FullStreamPage}?restream-id=${restreamId}`;
};

/**
 * Creates graphQL client for specified apiUrl
 **/
export function createGraphQlClient(
  apiUrl: string,
  onConnect: Function,
  onDisconnect: Function
): ApolloClient<unknown> {
  const protocol = window.location.protocol === 'https:' ? 'wss' : 'ws';
  const host = window.location.hostname;
  let port: string = process.env.EPHYR_DEV_HOST_PORT || window.location.port;
  let url = `${protocol}://${host}:${port}${apiUrl}`;
  console.log(`Connecting to \`${url}\` backend...`);
  const wsClient = new SubscriptionClient(url, { reconnect: true });

  wsClient.onConnected(() => onConnect());
  wsClient.onReconnected(() => onConnect());
  wsClient.onDisconnected(() => onDisconnect());

  return new ApolloClient({
    link: new WebSocketLink(wsClient),
    cache: new InMemoryCache(),
  });
}

const YT_VIDEO_REGEX =
  /(?:youtube(?:-nocookie)?\.com\/(?:[^\/\n\s]+\/\S+\/|(?:v|e(?:mbed)?)\/|\S*?[?&]v=)|youtu\.be\/)([a-zA-Z0-9_-]{11})/;

export const isYoutubeVideo = (url: string): boolean => {
  return YT_VIDEO_REGEX.test(url);
};

export const getYoutubeVideoID = (url: string): string | undefined => {
  const result = url.match(YT_VIDEO_REGEX);
  return result && result.length ? result[1] : undefined;
};

export const isNumber = (value: unknown): boolean => {
  return typeof value == 'number';
};

export const isFailoverInput = (input: any) => {
  return input?.src?.__typename === 'FailoverInputSrc';
};

export const escapeRegExp = (str: string) => {
  return str.replace(/[.*+?^${}()|[\]\\]/g, '\\$&'); // $& means the whole matched string
};

export const sanitizeTooltip = (message) => message.replaceAll(':', ' - ');

export const isFullGDrivePath = (id: string): boolean => {
  return id.startsWith('https://drive.google.com');
};

/**
 * Extracts the file ID from a Google Drive URL or returns the given ID if it's not a full path.
 *
 * @param id - The input string, which can be a Google Drive URL or a file ID.
 * @returns The extracted file ID if the input is a Google Drive URL, otherwise returns the original ID.
 */
export const getFileIdFromGDrive = (id) => {
  if (isFullGDrivePath(id)) {
    const result = id.match(/file\/d\/([^\/]+)/);
    if (result) {
      return result[1];
    }
  }

  return id;
};

/**
 * Extracts the folder ID from a Google Drive URL or returns the given ID if it's not a full path.
 *
 * @param id - The input string, which can be a Google Drive URL or a folder ID.
 * @returns The extracted folder ID if the input is a Google Drive URL, otherwise returns the original ID.
 */
export const getFolderIdFromGDrive = (id) => {
  if (isFullGDrivePath(id)) {
    const result = id.match(/folders\/([a-zA-Z0-9-_]+)/);
    if (result) {
      return result[1];
    }
  }

  return id;
};

export const isArrayStartWithAnother = (arr1: [], arr2: []) => {
  return isEqual(arr1, take(arr2, arr1.length));
};
