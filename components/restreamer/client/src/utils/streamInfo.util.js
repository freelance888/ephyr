import { isFailoverInput } from './util';
import omit from 'lodash/omit';
import isEqual from 'lodash/isEqual';

export const hasEndpointsWithStreamsErrors = (input) => {
  return !!getEndpointsWithStreamsErrors(input)?.length;
};

/**
 * Retrieves input keys with endpoints that have stream errors from a given input.
 *
 * @param {Object} input - The input object.
 * @returns {Array|string} - An array of input keys with endpoints having stream errors,
 *                           or `false` if there are no endpoints with stream errors.
 */
export const getEndpointsWithStreamsErrors = (input) => {
  if (isFailoverInput(input)) {
    const endpoints = input.src.inputs
      .map((i) => [i.key, i.endpoints.filter((e) => e.streamStat)[0]])
      .filter(([_, endpoint] = x) => endpoint?.streamStat?.error);

    return endpoints.map(([inputKey, _] = x) => inputKey);
  }

  return false;
};

export const hasEndpointsWithDiffStreams = (input) => {
  return !!getEndpointsWithDiffStreams(input)?.endpointsWithDiffStreams?.length;
};

const excludedProps = ['videoRFrameRate', 'bitRate'];

/**
 * Retrieves endpoints with different streams from a given input.
 *
 * @param {Object} input - The input object.
 * @param {Object} currentlyPlayingFile - The currently playing file from the playlist.
 * @returns {Object|boolean} - An object with the key of the first endpoint and an array of endpoints with different streams,
 *                            or `false` if there are no endpoints with stream statistics.
 */
export const getEndpointsWithDiffStreams = (input, currentlyPlayingFile) => {
  // Check if the input is considered a failover input
  if (isFailoverInput(input)) {
    // Extract endpoints from the input and filter out endpoints without stream statistics
    const endpoints = input.src.inputs
      .map((i) => [i.key, i.endpoints.filter((e) => e.streamStat)[0]])
      .filter(([_, endpoint] = x) => endpoint);

    // If there are no endpoints with stream statistics, return false
    if (!endpoints?.length) {
      return false;
    }

    // Properties to be excluded during comparison
    const excludedProps = ['videoRFrameRate', 'bitRate'];

    // Get the key and stream statistics of the first endpoint
    const [[firstEndpointKey, { streamStat: firstStreamStat }], _] = endpoints;

    // Find endpoints with different streams compared to the first endpoint
    let endpointsWithDiffStreams = endpoints
      .slice(1)
      .reduce(
        (
          diffKeys,
          [currentKey, { streamStat: currentStreamStat }] = current,
        ) => {
          // Compare the stream statistics of current endpoint with the first endpoint
          if (
            !isEqual(
              omit(currentStreamStat, excludedProps),
              omit(firstStreamStat, excludedProps),
            )
          ) {
            // If there are differences, add the current key to the result array
            diffKeys = [...diffKeys, currentKey];
          }

          return diffKeys;
        },
        [],
      );

    // For currently playing file from playlist compare its stream info with
    // first endpoint stream info
    if (currentlyPlayingFile?.streamStat) {
      const { name, streamStat: playlistFileSteamStat } = currentlyPlayingFile;
      if (
        !isEqual(
          omit(playlistFileSteamStat, excludedProps),
          omit(firstStreamStat, excludedProps),
        )
      ) {
        endpointsWithDiffStreams = [...endpointsWithDiffStreams, name];
      }
    }

    // Return an object containing the key of the first endpoint and endpoints with different streams
    return { firstEndpointKey, endpointsWithDiffStreams };
  }

  // If it's not a failover input, return false
  return false;
};

/**
 * Retrieves playlist items with different stream information.
 *
 * @param {Array} queue - The playlist queue
 * @returns {Array|boolean} - An array of playlist items with different stream information or false if none found
 */
export function getPlaylistItemsWithDiffStreams(queue) {
  const filesWithStreamInfo = queue.filter((x) => Boolean(x.file?.streamStat));

  // Check if there are playlist items with stream information
  if (Array.isArray(filesWithStreamInfo) && filesWithStreamInfo.length > 0) {
    const {
      name: firstFileNameWithStreamInfo,
      file: { streamStat: firstStreamStat },
    } = filesWithStreamInfo[0];

    // Reduce the list of playlist items to find those with different stream information
    const filesWithDiffStreams = filesWithStreamInfo
      .slice(1)
      .reduce(
        (
          diffFiles,
          {
            name: currentFileName,
            file: { streamStat: currentStreamStat },
          } = current,
        ) => {
          // Compare the streamStat objects of the current playlist item and the first playlist item
          if (
            !isEqual(
              omit(currentStreamStat, excludedProps),
              omit(firstStreamStat, excludedProps),
            )
          ) {
            diffFiles = [...diffFiles, currentFileName];
          }

          return diffFiles;
        },
        [],
      );

    // Return the list of playlist items with different stream information, or false if none found
    return filesWithDiffStreams.length > 0
      ? [firstFileNameWithStreamInfo, ...filesWithDiffStreams]
      : false;
  }

  // Return false if no playlist items with stream information found
  return false;
}

export const formatStreamInfo = (streamStat, title = '') => {
  if (streamStat) {
    return streamStat.error
      ? streamStat.error
      : `<span><strong>${title}</strong></span>
          <br/>
          <span><strong>video</strong>&#58; ${
            streamStat.videoCodecName
          }, </span>
          <span>${streamStat.videoWidth}x${streamStat.videoHeight},</span>
          <span>${streamStat.videoRFrameRate?.replace('/1', '')} FPS</span>
          <br/>
          <span><strong>audio</strong>&#58; ${streamStat.audioCodecName},</span>
          <span>${streamStat.audioSampleRate},</span>
          <span>${streamStat.audioChannelLayout},</span>
          <span>channels&#58; ${streamStat.audioChannels}</span>`;
  }

  return '';
};
