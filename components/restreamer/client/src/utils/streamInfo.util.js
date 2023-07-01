import { isFailoverInput } from './util';
import omit from 'lodash/omit';
import isEqual from 'lodash/isEqual';

export const hasEndpointsWithStreamsErrors = (input) => {
  return !!getEndpointsWithStreamsErrors(input)?.length;
};

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

export const getEndpointsWithDiffStreams = (input) => {
  if (isFailoverInput(input)) {
    const endpoints = input.src.inputs
      .map((i) => [i.key, i.endpoints.filter((e) => e.streamStat)[0]])
      .filter(([_, endpoint] = x) => endpoint);

    if (!endpoints?.length) {
      return false;
    }

    const excludedProps = ['videoRFrameRate', 'bitRate'];
    const [[firstEndpointKey, { streamStat: firstStreamStat }], _] = endpoints;
    const endpointsWithDiffStreams = endpoints
      .slice(1)
      .reduce(
        (
          diffKeys,
          [currentKey, { streamStat: currentStreamStat }] = current
        ) => {
          if (
            !isEqual(
              omit(currentStreamStat, excludedProps),
              omit(firstStreamStat, excludedProps)
            )
          ) {
            diffKeys = [...diffKeys, currentKey];
          }

          return diffKeys;
        },
        []
      );

    return { firstEndpointKey, endpointsWithDiffStreams };
  }

  return false;
};

export function getPlaylistItemsWithDiffStreams(queue) {
  const filesWithStreamInfo = queue.filter(x => Boolean(x.file?.streamStat));
  if (Array.isArray(filesWithStreamInfo) && filesWithStreamInfo.length > 0) {
    const { name: firstFileNameWithStreamInfo, file: { streamStat: firstStreamStat } } = filesWithStreamInfo[0];

    const filesWithDiffStreams = filesWithStreamInfo.slice(1)
      .reduce(
        (diffFiles, { name: currentFileName,  file: { streamStat:currentStreamStat } } = current) => {
          if (
            !isEqual(
              omit(currentStreamStat, excludedProps),
              omit(firstStreamStat, excludedProps)
            )
          ) {
            diffFiles = [...diffFiles, currentFileName];
          }

          return diffFiles;
        },
        []
      );

    return filesWithDiffStreams.length > 0
      ? [ firstFileNameWithStreamInfo, ...filesWithDiffStreams ]
      : false;
  }

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
