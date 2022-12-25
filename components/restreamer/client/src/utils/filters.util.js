import {
  ONLINE,
  OFFLINE,
  INITIALIZING,
  UNSTABLE,
  STREAM_ERROR,
  STREAM_WARNING,
} from './constants';
import {
  hasEndpointsWithDiffStreams,
  hasEndpointsWithStreamsErrors,
} from './input.util';

export const getAggregatedStreamsData = (reStreams) =>
  reStreams.reduce(
    (acc, reStream) => {
      const streamInputStatus = reStream.input.endpoints[0].status;

      reStream.outputs.forEach((output) => {
        acc.outputsCountByStatus[output.status]++;
      });

      acc.inputsCountByStatus[streamInputStatus]++;

      if (hasEndpointsWithDiffStreams(reStream.input)) {
        acc.endpointsStreamsStatus[STREAM_WARNING].push(reStream.input.id);
      }

      if (hasEndpointsWithStreamsErrors(reStream.input)) {
        acc.endpointsStreamsStatus[STREAM_ERROR].push(reStream.input.id);
      }

      return acc;
    },
    {
      inputsCountByStatus: {
        [OFFLINE]: 0,
        [INITIALIZING]: 0,
        [ONLINE]: 0,
        [UNSTABLE]: 0,
      },
      endpointsStreamsStatus: {
        [STREAM_ERROR]: [],
        [STREAM_WARNING]: [],
      },
      outputsCountByStatus: {
        [OFFLINE]: 0,
        [INITIALIZING]: 0,
        [ONLINE]: 0,
        [UNSTABLE]: 0,
      },
    }
  );

export const getReStreamOutputsCount = (reStream) =>
  reStream.outputs.reduce(
    (acc, output) => {
      const outputStatus = output.status;

      acc[outputStatus]++;

      return acc;
    },
    {
      [OFFLINE]: 0,
      [INITIALIZING]: 0,
      [ONLINE]: 0,
      [UNSTABLE]: 0,
    }
  );

export const toggleFilterStatus = (filters, filter) => {
  const filterIndex = filters.indexOf(filter);

  return filterIndex === -1
    ? [...filters, filter]
    : filters.filter((item) => item !== filter);
};
